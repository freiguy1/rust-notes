use handlebars::Handlebars;
use std::path::{ AsPath, Path, PathBuf };
use std::fs::File;
use std::io::Read;

mod markdown;
mod dir;

pub enum FileType {
    Dir(PathBuf),
    Markdown(PathBuf)
}

impl FileType {

    pub fn new<P: AsPath>(path: P) -> Option<FileType> {
        let path = path.as_path();
        match path {
            p if markdown::is_valid_path(p) =>
                Some(FileType::Markdown(PathBuf::new(path))),
            p if dir::is_valid_path(p) =>
                Some(FileType::Dir(PathBuf::new(path))),
            _ => None
        }
    }

    fn get_url(&self, context: &::AppContext) -> String {
        match *self {
            FileType::Dir(ref p) => dir::get_url(context, &p),
            FileType::Markdown(ref p) => markdown::get_url(context, &p)
        }
    }

    fn get_type_str(&self) -> &'static str {
        match *self {
            FileType::Dir(_) => dir::type_str(),
            FileType::Markdown(_) => markdown::type_str()
        }
    }

    pub fn convert(&self, context: &::AppContext) {
        match *self {
            FileType::Dir(ref p) => dir::convert(context, &p),
            FileType::Markdown(ref p) => markdown::convert(context, &p)
        }
    }

    pub fn register_handlebars(source_root: &Path) -> Result<Handlebars, &'static str> {
        let mut handlebars = Handlebars::new();
        try!(markdown::register_handlebars(source_root, &mut handlebars));
        try!(dir::register_handlebars(source_root, &mut handlebars));

        Ok(handlebars)
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
        let mut temp = PathBuf::new(path.clone().parent().unwrap());
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

pub fn read_file<P: AsPath>(path: P) -> Result<String, &'static str> {
    let path = path.as_path();
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

