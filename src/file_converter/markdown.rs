
use std::old_io::fs;
use std::old_io::fs::PathExtensions;
use std::old_io::{ FileType, File, USER_DIR, USER_FILE };

use rustc_serialize::json;
use rustc_serialize::json::{ ToJson, Json };

use rustdoc::html::markdown::Markdown;

use handlebars::Handlebars;

use std::rc::Rc;

use ::file_converter::{ FileTypeConverter, create_parent_links, Link };

#[derive(RustcEncodable)]
struct MarkdownModel {
    name: String,
    parents: Vec<Link>,
    content: String,
    base_url: String
}

impl ToJson for MarkdownModel {
    fn to_json(&self) -> Json {
        Json::from_str(json::encode(&self).unwrap().as_slice()).unwrap()
    }
}


struct MarkdownConverter {
    path: Path,
    handlebars: Rc<Handlebars>,
    template_name: String
}

impl MarkdownConverter {
    fn new(path: &Path, handlebars: Rc<Handlebars>, template_name: &str) -> MarkdownConverter {
        MarkdownConverter {
            path: path.clone(),
            handlebars: handlebars,
            template_name: String::from_str(template_name)
        }
    }
}

impl FileTypeConverter for MarkdownConverter {
    fn convert(&self,
               source_root: &Path,
               dest_root: &Path,
               relative: &Path,
               base_url: &str) {
        let file_name = relative.filestem_str().unwrap();
        let source_file = source_root.clone().join(relative);
        let dest_file = dest_root.clone().join(relative.dirname_str().unwrap()).join(format!("{}.html", file_name));
        let source_contents = File::open(&source_file).read_to_string().unwrap();
        // Create Model
        let content = Markdown(source_contents.as_slice());
        let parents = create_parent_links(base_url, relative, false);

        let model = MarkdownModel {
            name: String::from_str(file_name),
            parents : parents,
            content : format!("{}", content),
            base_url: String::from_str(base_url)
        };
        match self.handlebars.render(self.template_name.as_slice(), &model) {
            Ok(rendered) => {
                // Create File
                let mut file = File::create(&dest_file).ok().expect("Could not create markdown html file");
                fs::chmod(&dest_file, USER_FILE).ok().expect("Couldn't chmod new file");
                file.write_str(rendered.as_slice())
                    .ok().expect("Could not write html to file");
            },
            Err(why) => panic!("Error rendering markdown: {:?}", why)
        }
    }

    fn converted_url(&self, base_url: &str, relative: &Path) -> String {
        let file_name = relative.filestem_str().unwrap();
        format!("{}{}{}", base_url, relative.dirname_str().unwrap(), format!("{}.html", file_name))
    }

    fn type_str(&self) -> &str {
        "markdown"
    }

    fn is_valid_path(path: &Path) -> bool {
        let name = path.filename_str().unwrap();
        path.is_file() && (
            name.ends_with(".md") || 
            name.ends_with(".markdown") || 
            name.ends_with(".mkd"))
    }
}
