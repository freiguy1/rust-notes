use std::old_io::fs::PathExtensions;
use std::rc::Rc;
use std::old_io::File;
use handlebars::Handlebars;

mod markdown;
mod dir;

pub enum FileType {
    Dir(Path),
    Markdown(Path)
}

impl FileType {

    pub fn new(path: &Path) -> Option<FileType> {
        match path {
            p if markdown::is_valid_path(p) => Some(FileType::Markdown(p.clone())),
            p if p.is_dir() => Some(FileType::Dir(p.clone())),
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

pub fn create_parent_links(base_url: &str, path: &Path, is_dir: bool) -> Vec<Link> {
    if is_dir && path.filename().is_none() {
        Vec::new()
    } else {
        let mut result: Vec<Link> = vec![Link {
            name: String::from_str("root"),
            url: format!("{}", base_url)
        }];
        let mut temp = path.clone().dir_path();
        while temp.filename().is_some() {
            result.insert(1, Link {
                name: String::from_str(temp.filename_str().unwrap()),
                url: format!("{}{}", base_url, temp.as_str().unwrap())
            });
            temp.pop();
        }
        result
    }
}


