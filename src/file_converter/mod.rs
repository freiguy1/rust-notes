use std::old_io::fs::PathExtensions;
use std::rc::Rc;
use std::old_io::File;
use handlebars::Handlebars;

mod markdown;

pub enum FileType {
    Dir(Handlebars, String),
    Markdown(Handlebars, String)
}

impl FileType {

    pub fn register(source_root: &Path) -> Result<Vec<FileType>, &'static str> {
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

        result.push(
            FileType::Markdown(
                try!(markdown::create_handlebars(source_root)),
                String::from_str(markdown::type_str())
            )
        );

        Ok(result)
    }

    pub fn is_valid_path(&self, path: &Path) -> bool {
        let name = path.as_str().expect("Could not parse file type");
        match *self {
            FileType::Dir(_, _) => false,
            FileType::Markdown(_, _) => markdown::is_valid_path(path),
        }
    }

    pub fn convert(&self,
               notes_root: &Path,
               dest_root: &Path,
               relative: &Path,
               base_url: &str) {
        match *self {
            FileType::Dir(ref hbs, ref template_name) => {
            },
            FileType::Markdown(ref hbs, ref template_name) => {
                markdown::convert(&hbs, template_name.as_slice(), notes_root, dest_root, relative, base_url);
            }
        }
    }


    pub fn converted_url(&self, base_url: &str, relative: &Path) -> String {
        match *self {
            FileType::Dir(ref hbs, ref template_name) => {
                String::from_str("dir")
            },
            FileType::Markdown(ref hbs, ref template_name) => {
                markdown::converted_url(base_url, relative)
            }
        }
    }
}

pub fn type_str_by_path(path: &Path) -> &'static str {
    match path {
        p if markdown::is_valid_path(p) => markdown::type_str(),
        _ => "unknown"
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


