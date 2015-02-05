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

    /*
    fn create_converter(&self, path: &Path) -> Box<FileTypeConverter> {
            match *self {
                FileType::Dir(ref hbs, ref template_name) => {
                    Box::new(markdown::MarkdownConverter::new(path, hbs.clone(), template_name.as_slice()))
                },
                FileType::Markdown(ref hbs, ref template_name) => {
                    Box::new(markdown::MarkdownConverter::new(path, hbs.clone(), template_name.as_slice()))
                }
            }
    }
    */

    fn is_valid_path(&self, path: &Path) -> bool {
        let name = path.as_str().expect("Could not parse file type");
        match *self {
            FileType::Dir(_, _) => false,
            FileType::Markdown(_, _) => markdown::MarkdownConverter::is_valid_path(path),
        }
    }

    fn type_str(&self) -> &'static str {        
        match *self {
            FileType::Dir(_, _) => "dir",
            FileType::Markdown(_, _) => "markdown"
        }
    }

    fn convert(&self,
               notes_root: &Path,
               dest_root: &Path,
               relative: &Path,
               base_url: &str) {
        match *self {
            FileType::Dir(ref hbs, ref template_name) => {
            },
            FileType::Markdown(ref hbs, ref template_name) => {
                markdown::MarkdownConverter::convert(&hbs, template_name.as_slice(), notes_root, dest_root, relative, base_url);
            }
        }
    }


    fn converted_url(&self, base_url: &str, relative: &Path) -> String {
        match *self {
            FileType::Dir(ref hbs, ref template_name) => {
                String::from_str("dir")
            },
            FileType::Markdown(ref hbs, ref template_name) => {
                markdown::MarkdownConverter::converted_url(base_url, relative)
            }
        }
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
