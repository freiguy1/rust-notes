use handlebars::Handlebars;
use std::path::{ Path, PathBuf };
use std::fs::File;
use std::io::Read;

mod markdown;
mod dir;

pub trait FileType {
    fn get_url(&self, context: &::AppContext) -> String;
    fn convert(&self, context: &::AppContext);
    fn get_type_str(&self) -> &'static str;
}


pub fn initialize<P: AsRef<Path>>(source_root: P) -> Result<Handlebars, &'static str> {
    let mut handlebars = Handlebars::new();
    try!(markdown::Markdown::register_handlebars(&source_root, &mut handlebars));
    try!(dir::Dir::register_handlebars(&source_root, &mut handlebars));
    Ok(handlebars)
}

pub fn create_file_type<P: AsRef<Path>>(path: P) -> Option<Box<FileType>> {
    match &path {
        p if markdown::Markdown::is_valid_path(p) =>
            Some(Box::new(markdown::Markdown::new(p))),
        p if dir::Dir::is_valid_path(p) =>
            Some(Box::new(dir::Dir::new(p))),
        _ => None
    }
}

#[derive(RustcEncodable)]
struct Link {
    name: String,
    url: String
}

fn create_parent_links(base_url: &str, path: &Path, is_dir: bool) -> Vec<Link> {
    if is_dir && path.file_name().is_none() {
        Vec::new()
    } else {
        let mut result: Vec<Link> = vec![Link {
            name: String::from_str("root"),
            url: format!("{}", base_url)
        }];
        let mut temp = PathBuf::from(path.clone().parent().unwrap());
        while temp.file_name().is_some() {
            let file_name = String::from_str(temp.file_name().unwrap().to_str().unwrap());
            let url = format!("{}{}", &base_url, &file_name);
            result.insert(1, Link {
                name: file_name,
                url: url
            });
            temp.pop();
        }
        result
    }
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, &'static str> {
    let path: &Path = path.as_ref();
    let mut file = match File::open(&path) {
        Ok(ok_file) => ok_file,
        Err(_) => return Err("Could not open path.")
    };
    let mut contents= String::new();
    match file.read_to_string(&mut contents) {
        Err(_) => return Err("Could not read file."),
        _ => ()
    }
    Ok(contents)
}
