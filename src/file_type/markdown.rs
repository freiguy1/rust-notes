use std::fs::{metadata, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use pulldown_cmark::html;
use pulldown_cmark::Parser;

use serde::Serialize;

use crate::file_type::{create_parent_links, read_file, FileType, Link};
use crate::util::RelativeFrom;

static TYPE_STR: &'static str = "markdown";

pub struct MarkdownFactory;

impl crate::file_type::FileTypeFactory for MarkdownFactory {
    fn try_create(&self, path: &Path) -> Option<Box<dyn FileType>> {
        let name = path.file_name().unwrap().to_str().unwrap();
        let path_metadata = metadata(&path).ok().expect("Could not fetch file metadata");
        let is_valid = path_metadata.is_file()
            && (name.ends_with(".md") || name.ends_with(".markdown") || name.ends_with(".mkd"));
        if is_valid {
            let result = Markdown {
                path: PathBuf::from(path),
                type_str: TYPE_STR,
            };
            Some(Box::new(result))
        } else {
            None
        }
    }

    fn initialize(&self, app_context: &mut crate::AppContext<'_>) -> Result<(), &'static str> {
        let header_hbs_path = app_context.root_source.join("partials/header.hbs");
        if !metadata(&header_hbs_path).is_ok() {
            return Err("Missing partials/header.hbs");
        }

        let footer_hbs_path = app_context.root_source.join("partials/footer.hbs");
        if !metadata(&footer_hbs_path).is_ok() {
            return Err("Missing partials/footer.hbs");
        }

        let note_hbs_path = app_context.root_source.join("layouts/note.hbs");
        if !metadata(&note_hbs_path).is_ok() {
            return Err("Missing /layouts/note.hbs");
        }

        let header_hbs_contents = read_file(&header_hbs_path)?;
        let footer_hbs_contents = read_file(&footer_hbs_path)?;
        let note_hbs_contents = read_file(&note_hbs_path)?;
        app_context
            .handlebars
            .register_template_string(
                TYPE_STR,
                format!(
                    "{}\n{}\n{}",
                    header_hbs_contents, note_hbs_contents, footer_hbs_contents
                ),
            )
            .ok()
            .expect("Error registering header|note|footer template");

        Ok(())
    }
}

pub struct Markdown {
    path: PathBuf,
    type_str: &'static str,
}

impl FileType for Markdown {
    fn get_url(&self, context: &crate::AppContext<'_>) -> String {
        let file_name = self.path.file_stem().unwrap().to_str().unwrap();
        let relative = self
            .path
            .my_relative_from(&context.root_notes)
            .expect("Problem parsing relative url");
        let parent_relative = if relative
            .parent()
            .map_or_else(|| true, |p| p == Path::new("/") || p == Path::new(""))
        {
            String::new()
        } else {
            format!("{}/", relative.parent().unwrap().to_str().unwrap())
        };
        format!(
            "{}{}{}",
            context.base_url,
            parent_relative,
            format!("{}.html", file_name)
        )
    }

    fn convert(&self, context: &crate::AppContext<'_>) {
        let relative = self
            .path
            .my_relative_from(&context.root_notes)
            .expect("Problem parsing relative url");
        let file_name = relative.file_stem().unwrap().to_str().unwrap();
        let dest_file = context
            .root_dest
            .clone()
            .join(relative.parent().unwrap())
            .join(format!("{}.html", file_name));
        let mut source_contents = String::new();
        File::open(&self.path)
            .ok()
            .unwrap()
            .read_to_string(&mut source_contents)
            .ok()
            .expect("Could not read markdown file");
        // Create Model
        let content = render_html(&source_contents);
        let parents = create_parent_links(&context.base_url, &relative, false);

        let model = MarkdownModel {
            name: String::from(file_name),
            parents: parents,
            content: format!("{}", content),
            base_url: context.base_url.clone(),
        };
        match context.handlebars.render(TYPE_STR, &model) {
            Ok(rendered) => {
                // Create File
                let mut file = File::create(&dest_file)
                    .ok()
                    .expect("Could not create markdown html file");
                //fs::chmod(&dest_file, USER_FILE).ok().expect("Couldn't chmod new file");
                file.write_all(rendered.as_bytes())
                    .ok()
                    .expect("Could not write html to file");
            }
            Err(why) => panic!("Error rendering markdown: {:?}", why),
        }
    }

    fn get_type_str(&self) -> &'static str {
        self.type_str
    }
}

#[derive(Serialize)]
struct MarkdownModel {
    name: String,
    parents: Vec<Link>,
    content: String,
    base_url: String,
}

// impl ToJson for MarkdownModel {
//     fn to_json(&self) -> Json {
//         Json::from_str(&json::encode(&self).unwrap()).unwrap()
//     }
// }

fn render_html(text: &str) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);
    let p = Parser::new(&text);
    html::push_html(&mut s, p);
    s
}
