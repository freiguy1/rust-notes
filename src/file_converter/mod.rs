use std::old_io::fs::PathExtensions;
use std::rc::Rc;
use std::old_io::File;
use handlebars::Handlebars;

mod markdown;

enum FileType {
    Dir(Handlebars, String),
    Markdown(Handlebars, String)
}

impl FileType {

    fn register(source_root: &Path) -> Result<Vec<FileType>, &'static str> {
        // Validate generic stuff
        let header_hbs_path = source_root.clone().join("partials/header.hbs");
        if !header_hbs_path.exists() {
            return Err("Missing partials/header.hbs");
        }

        let footer_hbs_path = source_root.clone().join("partials/footer.hbs");
        if !footer_hbs_path.exists() {
            return Err("Missing partials/footer.hbs");
        }

        // Validate Dir
        let dir_hbs_path = source_root.clone().join("layouts/dir.hbs");
        if !dir_hbs_path.exists() {
            return Err("Missing /layouts/dir.hbs");
        }

        // Validate Markdown
        let note_hbs_path = source_root.clone().join("layouts/note.hbs");
        if !note_hbs_path.exists() {
            return Err("Missing /layouts/note.hbs");
        }

        // Grab generic stuff
        let header_hbs_contents = File::open(&header_hbs_path).read_to_string().unwrap();
        let footer_hbs_contents = File::open(&footer_hbs_path).read_to_string().unwrap();

        let mut result: Vec<FileType> = Vec::new();

        // Create Dir
        let dir_template_name = "dir_template";
        let dir_hbs_contents = File::open(&dir_hbs_path).read_to_string().unwrap();
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string(dir_template_name, format!("{}\n{}\n{}", header_hbs_contents, dir_hbs_contents, footer_hbs_contents))
            .ok().expect("Error registering header|dir|footer template");

        result.push(FileType::Dir(handlebars, String::from_str(dir_template_name)));

        // Create Markdown
        let note_template_name = "note_template";
        let note_hbs_contents = File::open(&note_hbs_path).read_to_string().unwrap();
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string(note_template_name, format!("{}\n{}\n{}", header_hbs_contents, note_hbs_contents, footer_hbs_contents))
            .ok().expect("Error registering header|note|footer template");

        result.push(FileType::Markdown(handlebars, String::from_str(note_template_name)));

        Ok(result)
    }

    fn create_converter(&self, path: &Path) -> FileTypeConverter {
            FileType::Dir(hbs, template_name) => {

            },
            FileType::Markdown(hbs, template_name) => {
            }
    }

    fn is_valid_path(&self, path: &Path) -> bool {
        let name = path.as_str().expect("Could not parse file type");
        match *self {
            FileType::Dir(_, _) if path.is_dir() => true,
            FileType::Markdown(_, _) => path.is_file() && (
                name.ends_with(".md") || 
                name.ends_with(".markdown") || 
                name.ends_with(".mkd")),
            _ => false
        }
    }

    fn type_str(&self) -> &'static str {        
        match *self {
            FileType::Dir(_, _) => "dir",
            FileType::Markdown(_, _) => "markdown"
        }
    }


    /*
    fn convert(&self, fc: &FileConverter, relative: &Path) {
    }

    fn converted_url(&self, fc: &FileConverter, relative: &Path) -> &str {
        "hello world"
    }
    */

}

trait FileTypeConverter {
    /// I expect source_root to be the notes directory, so relative works for both source_root and
    /// dest_root.
    fn convert(&self,
               source_root: &Path,
               dest_root: &Path,
               relative: &Path,
               base_url: &str);

    fn converted_url(&self, base_url: &str, relative: &Path) -> String;

    fn type_str(&self) -> &str;

    fn is_valid_path(path: &Path) -> bool;
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



/*
struct FileConverter<'a> {
    source_root: Path,
    dest_root: Path,
    base_url: String,
    file_types: Vec<FileType>
}

impl<'a> FileConverter<'a> {
    fn init(source_root: &Path, dest_root: &Path, base_url: &str) -> FileConverter<'a> {
        let dir_ft = FileType::Dir(String::from_str("first string"));
        let markdown_ft = FileType::Markdown(String::from_str("second string"));
        FileConverter {
            source_root: source_root.clone(),
            dest_root: dest_root.clone(),
            base_url: String::from_str(base_url),
            file_types: vec![dir_ft, markdown_ft]
        }
    }

    pub fn convert(&self, relative: &Path) {
        match self.file_type_by_path(relative) {
            Some(ft) => {
                // Valid file, can process
                println!("Can process file: {:?}", relative);
            },
            None => println!("Cannot process file, unknown type: {:?}", relative)
        }
    }

    fn converted_url(&self, relative: &Path) -> Option<&str> {
        self.file_type_by_path(relative).map(|ft| ft.converted_url(&self, relative))
    }

    fn file_type_by_path(&self, relative: &Path) -> Option<&FileType> {
        self.file_types.iter().find(|ft| ft.is_valid_path(relative))
    }
}


*/
