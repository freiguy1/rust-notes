#![feature(rustdoc)]

extern crate docopt;
extern crate rustdoc;
extern crate rustc_serialize;
extern crate handlebars;

use docopt::Docopt;
use std::fs;
use std::path::{ Path, PathBuf };
use handlebars::Handlebars;
use util::RelativeFrom;

mod file_type;
mod util;

// Docopt usage string
static USAGE: &'static str = "
Usage: rust-notes [options] <source> <dest>

Options:
    -b, --base-url BASE     Base URL for site. Start with '/' but do not end with one. Should not include hostname.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_source: String,
    arg_dest: String,
    flag_base_url: Option<String>
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

fn cp_dir(source: &Path, dest: &Path) {
    fs::create_dir(dest).ok().expect("Problem copying directory");
    for item in util::walk_dir(source).ok().expect("Problem copying directory") {
        let item = item.ok().expect("Problem copying directory").path();
        let item_metadata = fs::metadata(&item).expect("Error fetching file metadata");
        let relative = item
            .my_relative_from(source)
            .unwrap();
        let dest_path = dest.clone().join(&relative);
        if item_metadata.is_file() {
            fs::copy(&item, &dest_path).ok().expect("Problem copying directory");
        } else {
            fs::create_dir(&dest_path).ok().expect("Problem copying directory");
        }
    }
}

pub struct AppContext {
    root_source: PathBuf,
    root_dest: PathBuf,
    root_notes: PathBuf,
    handlebars: Handlebars,
    base_url: String,
}

struct Generator {
    context: AppContext,
    file_type_manager: file_type::FileTypeManager
}

impl Generator {

    fn convert(&self, path: &Path) {
        self.file_type_manager.create_file_type(path).convert(&self.context);
    }

    pub fn new(args: Args) -> Result<Generator, &'static str> {
        let source_path = Path::new(&args.arg_source);
        let source_path_metadata = fs::metadata(source_path).expect("Error fetching file metadata");
        let dest_path = Path::new(&args.arg_dest);
        let dest_path_metadata = fs::metadata(dest_path).expect("Error fetching file metadata");

        if !source_path_metadata.is_dir() {
            return Err("Invalid source path");
        }

        if !dest_path_metadata.is_dir() {
            match fs::create_dir_all(&dest_path) {
                Err(_) => return Err("Cannot create destination directory"),
                _ => ()
            }
        }

        // Validate source
        let notes_source_path = source_path.join("notes");
        let notes_source_path_metadata = fs::metadata(source_path)
            .expect("Error fetching file metadata");

        if !notes_source_path_metadata.is_dir() {
            return Err("Source directory missing required files");
        }

        let base_url = match args.flag_base_url {
            Some(ref base_url) if base_url.is_empty() => None,
            Some(base_url) => {
                let mut result = String::from(base_url.trim_matches('/'));
                result = format!("/{}/", result);
                Some(result)
            },
            None => None
        };


        let mut context = AppContext {
            root_source: PathBuf::from(source_path),
            root_dest: PathBuf::from(dest_path),
            root_notes: notes_source_path,
            handlebars: Handlebars::new(),
            base_url: base_url.clone().unwrap_or(String::from("/"))
        };

        let file_type_manager = file_type::FileTypeManager::new();
        try!(file_type_manager.initialize_app_context(&mut context));

        // Good to go! Let's return something good

        Ok(Generator{
            context: context,
            file_type_manager: file_type_manager
        })
    }

    pub fn begin(&self) {
        self.clean_dest();
        let assets_source_path = self.context.root_source.join("assets");
        let assets_source_path_metadata = fs::metadata(&assets_source_path)
            .expect("Error fetching file metadata");
        if assets_source_path_metadata.is_dir() {
            let assets_dest_path = self.context.root_dest.join("assets");
            cp_dir(&assets_source_path, &assets_dest_path);
        }
        self.convert(&self.context.root_notes);
        for item in util::walk_dir(&self.context.root_notes).ok().unwrap() {
            self.convert(&item.ok().unwrap().path());
        }
    }



    fn clean_dest(&self) {
        for entry in fs::read_dir(&self.context.root_dest).ok().unwrap() {
            let entry = entry.ok().unwrap();
            let entry_metadata = fs::metadata(entry.path())
                .expect("Error fetching file metadata");
            if entry_metadata.is_file() {
                fs::remove_file(entry.path()).ok().expect("Could not remove file");
            } else {
                fs::remove_dir_all(entry.path()).ok().expect("Could not remove directory");
            }
        }
    }
}
