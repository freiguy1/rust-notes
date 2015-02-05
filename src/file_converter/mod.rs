use std::old_io::fs::PathExtensions;
use std::rc::Rc;

enum FileType {
    Dir(String),
    Markdown(String)
}

impl FileType {

    fn is_valid_path(&self, path: &Path) -> bool {
        let name = path.as_str().expect("Could not parse file type");
        match *self {
            FileType::Dir(_) if path.is_dir() => true,
            FileType::Markdown(_) => path.is_file() && (
                name.ends_with(".md") || 
                name.ends_with(".markdown") || 
                name.ends_with(".mkd")),
            _ => false
        }
    }

    fn type_str(&self) -> &'static str {        
        match *self {
            FileType::Dir(_) => "dir",
            FileType::Markdown(_) => "markdown"
        }
    }

    fn convert(&self, fc: &FileConverter, relative: &Path) {
    }

    fn converted_url(&self, fc: &FileConverter, relative: &Path) -> &str {
        "hello world"
    }

}

trait FileTypeConverter {
    /// I expect source_root to be the notes directory, so relative works for both source_root and
    /// dest_root.
    fn convert(&self,
               source_root: &Path,
               dest_root: &Path,
               relative: &Path,
               base_url: &str);

    fn converted_url(&self, base_url: &str, relative: &Path) -> &str;

    fn type_str(&self) -> &str;

    fn is_valid_path(path: &Path) -> bool;
}

struct MarkdownConverter {
    path: Path,
    handlebars: Rc<String>
}

impl MarkdownConverter {
    fn new(path: &Path, handlebars: Rc<String>) -> MarkdownConverter {
        MarkdownConverter {
            path: path.clone(),
            handlebars: handlebars
        }
    }
}

struct Link {
    name: String,
    url: String
}


pub fn create_parent_links(&self, path: &Path, is_note: bool) -> Vec<Link> {

    
    mut result = if path.is_dir() {
        Vec::new<Link>()

    }

    if !is_note && path.filename().is_none() {
        Vec::new()
    } else {
        let mut result: Vec<Link> = vec![Link {
            name: String::from_str("root"),
            url: format!("{}/", self.base_url)
        }];
        let mut temp = path.clone().dir_path();
        while temp.filename().is_some() {
            result.insert(1, Link {
                name: String::from_str(temp.filename_str().unwrap()),
                url: format!("{}/{}", self.base_url, temp.as_str().unwrap())
            });
            temp.pop();
        }
        result
    }
}


struct MarkdownModel {
    name: String,
    parents: Vec<Link>,
    content: String,
    base_url: String
}

impl FileTypeConverter for MarkdownConverter {
    fn convert(&self,
               source_root: &Path,
               dest_root: &Path,
               relative: &Path,
               base_url: &str) {
    /*
    fn convert_file(&self, source_file_path: &Path, dest_parent_dir_path: &Path, relative_path: &Path) {
        if Generator::is_markdown_file(source_file_path) {
            let source_contents = File::open(source_file_path).read_to_string().unwrap();
            let file_name = source_file_path.filestem_str().unwrap();

            // Create Model
            let content = Markdown(source_contents.as_slice());
            let parents = self.get_parent_links(relative_path, true);

            let model = NoteModel {
                name: String::from_str(file_name),
                parents : parents,
                content : format!("{}", content),
                base_url: self.base_url.clone()
            };

            match self.handlebars.render(self.note_template_name, &model) {
                Ok(rendered) => {
                    // Create File
                    let new_rendered = String::from_str(rendered.as_slice())
                        .replace("\\n", "\n")
                        .replace("\\\"", "\"");
                    let new_file_path = dest_parent_dir_path.join(format!("{}.html", file_name));
                    let mut file = File::create(&new_file_path).ok().expect("Could not create note html file");
                    fs::chmod(&new_file_path, USER_FILE).ok().expect("Couldn't chmod new file");
                    file.write_str(new_rendered.as_slice())
                        .ok().expect("Could not write html to file");
                },
                Err(why) => panic!("Error rendering note: {:?}", why)
            }
        }
    }
    */
        let source_file = source_root.clone().join(relative);
        let dest_file = dest_root.clone().join(relative);
        let file_name = relative.filestem_str().unwrap();
        let source_contents = File::open(source_file).read_to_string().unwrap();
        // Create Model
        let content = Markdown(source_contents.as_slice());
        let parents = self.get_parent_links(relative_path, true);

        let model = MarkdownModel {
            name: String::from_str(file_name),
            parents : parents,
            content : format!("{}", content),
            base_url: base_url.clone()
        };
        match self.handlebars.render(self.note_template_name, &model) {
            Ok(rendered) => {
                // Create File
                let new_rendered = String::from_str(rendered.as_slice())
                    .replace("\\n", "\n")
                    .replace("\\\"", "\"");
                let new_file_path = dest_parent_dir_path.join(format!("{}.html", file_name));
                let mut file = File::create(&dest_file.dirname_str().unwrap()).ok().expect("Could not create markdown html file");
                fs::chmod(&new_file_path, USER_FILE).ok().expect("Couldn't chmod new file");
                file.write_str(new_rendered.as_slice())
                    .ok().expect("Could not write html to file");
            },
            Err(why) => panic!("Error rendering markdown: {:?}", why)
        }
    }

    fn converted_url(&self, base_url: &str, relative: &Path) -> &str;

    fn type_str(&self) -> &str;

    fn is_valid_path(path: &Path) -> bool;
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
