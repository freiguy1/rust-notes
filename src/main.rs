#![feature(io, path, core, rustdoc, collections)]

extern crate docopt;
extern crate rustdoc;
extern crate "rustc-serialize" as rustc_serialize;
extern crate handlebars;

use docopt::Docopt;
use std::old_io::fs;
use std::old_io::fs::PathExtensions;
use std::old_io::{ FileType, File, USER_DIR, USER_FILE };
use rustdoc::html::markdown::Markdown;
use handlebars::Handlebars;
use rustc_serialize::json::{ ToJson, Json };
use std::collections::BTreeMap;

// Docopt usage string
static USAGE: &'static str = "
Usage: rust-notes <source> <dest>
";

#[derive(Show, RustcDecodable)]
struct Args {
    arg_source: String,
    arg_dest: String
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    
    //Generator(args).start();
    match Generator::new(args) {
        Ok(generator) => {
            generator.begin();
        },
        Err(message) => panic!(message)
    }
}

struct Link {
    name: String,
    url: String
}

impl ToJson for Link {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("name".to_string(), self.name.to_json());
        m.insert("url".to_string(), self.url.to_json());
        Json::Object(m)
    }
}

struct DirModel {
    name: String,
    parents: Vec<Link>,
    dirs: Vec<Link>,
    notes: Vec<Link>
}

impl ToJson for DirModel {
    fn to_json(&self) -> Json {
        let mut m = BTreeMap::new();
        m.insert("name".to_string(), self.name.to_json());
        m.insert("parents".to_string(), self.parents.to_json());
        m.insert("dirs".to_string(), self.dirs.to_json());
        m.insert("notes".to_string(), self.notes.to_json());
        Json::Object(m)
    }
}

/*
#[derive(Show, Decodable)]
struct NoteModel {
    name: String,
    parents: Vec<Link>,
    content: String,
}
*/


struct Generator {
    root_source_path: Path,
    root_dest_path: Path,
    notes_source_path: Path,
    handlebars: Handlebars,
    dir_template_name: &'static str,
    note_template_name: &'static str
}

impl Generator {

    pub fn new(args: Args) -> Result<Generator, &'static str> {
        let source_path = Path::new(args.arg_source.as_slice());
        let dest_path = Path::new(args.arg_dest.as_slice());

        if !source_path.is_dir() {
            return Err("Invalid source path");
        }

        if !dest_path.is_dir() {
            match fs::mkdir_recursive(&dest_path, USER_DIR) {
                Err(_) => return Err("Cannot create destination directory"),
                _ => ()
            }
        }

        // Validate source
        let notes_source_path = source_path.clone().join("notes");
        if !notes_source_path.is_dir() {
            return Err("Source directory missing required files");
        }

        let dir_hbs_path = source_path.clone().join("dir.hbs");
        if !dir_hbs_path.exists() {
            return Err("Source directory missing dir.hbs");
        }

        // Good to go! Let's return something good

        let dir_template_name = "dir_template";
        let note_template_name = "note_template";

        let dir_hbs_contents = File::open(&dir_hbs_path).read_to_string().unwrap();
        //println!("dir_hbs_contents: {}", dir_hbs_contents);
        let mut handlebars = Handlebars::new();
        match handlebars.register_template_string(dir_template_name, dir_hbs_contents.to_string()) {
            Err(why) => panic!("Error registering template: {:?}", why),
            _ => ()
        }
        
        Ok(Generator{
            root_source_path: source_path,
            root_dest_path: dest_path,
            notes_source_path: notes_source_path,
            handlebars: handlebars,
            dir_template_name: dir_template_name,
            note_template_name: note_template_name
        })
    }

    pub fn begin(&self) {
        self.clean_dest();
        self.generate_dir(&Path::new(""));
    }

    fn generate_dir(&self, relative_path: &Path) {
        let full_dest_path = self.root_dest_path.clone()
            .join(relative_path.as_str().unwrap());
        let full_source_path = self.notes_source_path.clone()
            .join(relative_path.as_str().unwrap());

        self.create_dir_index(&full_dest_path, relative_path);

        match fs::readdir(&full_source_path) {
            Ok(items) => {
                for item in items.iter() {
                    if item.is_file() {
                        self.convert_file(item, &full_dest_path);
                    } else {
                        self.convert_dir(item, &full_dest_path, relative_path);
                    }
                }
            },
            Err(_) => ()
        }
    }

    fn convert_file(&self, source_file_path: &Path, dest_parent_dir_path: &Path) {
        if Generator::is_markdown_file(source_file_path) {
            let source_contents = File::open(source_file_path).read_to_string().unwrap();
            let new_file_path = dest_parent_dir_path.join(format!("{}.html", source_file_path.filestem_str().unwrap()));
            //println!("Creating file '{:?}'", new_file_path);
            let mut file = File::create(&new_file_path).ok().expect("Could not create html file");
            fs::chmod(&new_file_path, USER_FILE).ok().expect("Couldn't chmod new file");
            let markdown_content = Markdown(source_contents.as_slice());
            file.write_str(format!("{}", markdown_content).as_slice())
                .ok().expect("Could not write html to file");
        }
    }

    fn convert_dir(&self, source_dir_path: &Path, dest_parent_dir_path: &Path, relative_path: &Path) {
        let mut contains_note: bool = false;
        let mut dirs = fs::walk_dir(source_dir_path)
            .ok().expect("Could not walk through directories recursively");
        for item2 in dirs {
            contains_note |= Generator::is_markdown_file(&item2);
        }

        if contains_note {
            let dirname = source_dir_path.filename_str().unwrap();
            let new_dir_path = dest_parent_dir_path.clone().join(dirname);
            //println!("creating directory: {:?}", new_dir_path);
            fs::mkdir(&new_dir_path, USER_DIR).ok().expect("Cannot create destination subdir");
            self.generate_dir(&relative_path.clone().join(dirname));
        }
    }

    fn is_markdown_file(file_path: &Path) -> bool {
        let name = file_path.as_str().unwrap();
        let stat = fs::stat(file_path).ok().expect("Could not get stat for file");
        if stat.kind != FileType::RegularFile {
            return false;
        }
        if !name.ends_with(".md") && !name.ends_with(".markdown") {
            return false;
        }
        true
    }

    fn create_dir_index(&self, dir_path: &Path, relative_path: &Path) {
        let new_file_path = dir_path.join("index.html");
        let dirname = dir_path.filename_str().unwrap();
        let mut file = File::create(&new_file_path).ok().expect("Could not create html file");
        fs::chmod(&new_file_path, USER_FILE).ok().expect("Couldn't chmod new file");

        //  Parents
        let mut parents: Vec<Link> = Vec::new();

        let mut temp = relative_path.clone();
        temp.pop();
        while temp.filename_str().is_some() {
            parents.insert(0, Link {
                name: String::from_str(temp.filename_str().unwrap()),
                url: format!("/{}", temp.as_str().unwrap())
            });
            temp.pop();
        }

        // Name
        let name = if relative_path.as_str().unwrap() == "." {
            "Notes"
        } else {
            parents.insert(0, Link {
                name: String::from_str("Home"),
                url: String::from_str("/")
            });
            dirname
        };

        let mut dirs: Vec<Link> = Vec::new();
        let mut notes: Vec<Link> = Vec::new();

        let source_path = self.notes_source_path.clone().join(relative_path);
        match fs::readdir(&source_path) {
            Ok(items) => {
                for item in items.iter() {
                    if item.is_file() && Generator::is_markdown_file(item) {
                        let name = item.filestem_str().unwrap();
                        let url = relative_path.clone().join(format!("{}.html", name).as_slice());
                        notes.push(Link{
                            name: String::from_str(name),
                            url: String::from_str(format!("/{}", url.as_str().unwrap()).as_slice())
                        });
                    } else {
                        let name = item.filename_str().unwrap();
                        let url = relative_path.clone().join(name);
                        dirs.push(Link{
                            name: String::from_str(name),
                            url: String::from_str(format!("/{}", url.as_str().unwrap()).as_slice())
                        });
                    }
                }
            },
            Err(_) => ()
        }

        let model = DirModel {
            name: String::from_str(name),
            parents: parents,
            dirs: dirs,
            notes: notes
        };
        
        match self.handlebars.render(self.dir_template_name, &model) {
            Ok(rendered) => {
                file.write_str(rendered.as_slice())
                    .ok().expect("Could not write html to file");
            },
            Err(why) => panic!("Error rendering: {:?}", why)
        }
    }

    fn clean_dest(&self) {
        match fs::readdir(&self.root_dest_path) {
            Ok(items) => {
                for item in items.iter() {
                    if item.is_file() {
                        //println!("removing file: {:?}", item);
                        fs::unlink(item).ok().expect("Could not remove file");
                    } else {
                        //println!("removing directory: {:?}", item);
                        fs::rmdir_recursive(item).ok().expect("Could not remove directory");
                    }
                }
            },
            Err(_) => ()
        }
    }
}
