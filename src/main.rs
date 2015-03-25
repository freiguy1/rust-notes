#![feature(std_misc, path_relative_from, fs_walk, path_ext, core, rustdoc, collections)]

extern crate docopt;
extern crate rustdoc;
extern crate "rustc-serialize" as rustc_serialize;
extern crate handlebars;

use docopt::Docopt;
use std::fs;
use std::fs::PathExt;
use std::path::{ Path, PathBuf };
use handlebars::Handlebars;

mod file_type;

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
    for item in fs::walk_dir(source).ok().expect("Problem copying directory") {
        let item = item.ok().expect("Problem copying directory").path();
        let relative = item
            .relative_from(source)
            .unwrap();
        let dest_path = dest.clone().join(&relative);
        if item.is_file() {
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
    context: AppContext
}

impl Generator {

    fn convert(&self, path: &Path) {
        match file_type::FileType::new(path) {
            Some(ft) => ft.convert(&self.context),
            None => { println!("Couldn't handle file: {:?}", path); }
        }
    }

    pub fn new(args: Args) -> Result<Generator, &'static str> {
        let source_path = Path::new(args.arg_source.as_slice());
        let dest_path = Path::new(args.arg_dest.as_slice());

        if !source_path.is_dir() {
            return Err("Invalid source path");
        }

        if !dest_path.is_dir() {
            match fs::create_dir_all(&dest_path) {
                Err(_) => return Err("Cannot create destination directory"),
                _ => ()
            }
        }

        // Validate source
        let notes_source_path = source_path.clone().join("notes");
        if !notes_source_path.is_dir() {
            return Err("Source directory missing required files");
        }

        let base_url = match args.flag_base_url {
            Some(ref base_url) if base_url.is_empty() => None,
            Some(base_url) => {
                let mut result = String::from_str(base_url.trim_matches('/'));
                result = format!("/{}/", result);
                Some(result)
            },
            None => None
        };

        let handlebars = try!(file_type::FileType::register_handlebars(&source_path));

        // Good to go! Let's return something good

        let context = AppContext {
            root_source: PathBuf::new(source_path),
            root_dest: PathBuf::new(dest_path),
            root_notes: PathBuf::new(notes_source_path),
            handlebars: handlebars,
            base_url: base_url.clone().unwrap_or(String::from_str("/"))
        };
        
        Ok(Generator{
            context: context
        })
    }

    pub fn begin(&self) {
        self.clean_dest();
        let assets_source_path = self.context.root_source.clone().join("assets");
        if assets_source_path.is_dir() {
            let assets_dest_path = self.context.root_dest.clone().join("assets");
            cp_dir(&assets_source_path, &assets_dest_path);
        }
        self.convert(&self.context.root_notes);
        for item in fs::walk_dir(&self.context.root_notes).ok().unwrap() {
            self.convert(&item.ok().unwrap().path());
        }
    }



    fn clean_dest(&self) {

        for entry in fs::read_dir(&self.context.root_dest).ok().unwrap() {
            let entry = entry.ok().unwrap();
            if entry.path().is_file() {
                fs::remove_file(entry.path()).ok().expect("Could not remove file");
            } else {
                fs::remove_dir_all(entry.path()).ok().expect("Could not remove directory");
            }
        }
    }
}
