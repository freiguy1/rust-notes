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

trait FileTypeFactory {
    fn try_create(&self, path: &Path) -> Option<Box<FileType>>;
    fn initialize(&self, app_context: &mut ::AppContext) -> Result<(), &'static str>;
}

pub struct FileTypeManager {
    factories: Vec<Box<FileTypeFactory>>
}

impl FileTypeManager {
    pub fn new() -> FileTypeManager {

        FileTypeManager {
            factories: vec![
                Box::new(markdown::MarkdownFactory),
                Box::new(dir::DirFactory)
            ]
        }
    }

    pub fn initialize_app_context(&self, app_context: &mut ::AppContext) -> Result<(), &'static str> {
        for factory in self.factories.iter() {
            try!(factory.initialize(app_context));
        }
        Ok(())
    }

    pub fn create_file_type<P: AsRef<Path>>(&self, path: P) -> Option<Box<FileType>> {
        for factory in self.factories.iter() {
            let result_maybe = factory.try_create(path.as_ref());
            if result_maybe.is_some() {
                return result_maybe;
            }
        }
        None
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
