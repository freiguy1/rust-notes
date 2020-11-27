use std::cmp::Ordering;
use std::fs;
use std::fs::{metadata, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::file_type::{create_parent_links, read_file, FileType, Link};
use crate::util::RelativeFrom;

static TYPE_STR: &'static str = "dir";

pub struct DirFactory;

impl crate::file_type::FileTypeFactory for DirFactory {
    fn try_create(&self, path: &Path) -> Option<Box<dyn FileType>> {
        if metadata(&path)
            .ok()
            .expect("Error fetching metadata for file")
            .is_dir()
        {
            Some(Box::new(Dir {
                path: PathBuf::from(path),
                type_str: TYPE_STR,
                file_type_manager: crate::file_type::FileTypeManager::new(),
            }))
        } else {
            None
        }
    }

    fn initialize(&self, app_context: &mut crate::AppContext) -> Result<(), &'static str> {
        // Validate generic stuff
        let header_hbs_path = app_context.root_source.join("partials/header.hbs");
        if !metadata(&header_hbs_path).is_ok() {
            return Err("Missing partials/header.hbs");
        }

        let footer_hbs_path = app_context.root_source.join("partials/footer.hbs");
        if !metadata(&footer_hbs_path).is_ok() {
            return Err("Missing partials/footer.hbs");
        }

        // Validate Dir
        let dir_hbs_path = app_context.root_source.join("layouts/dir.hbs");
        if !metadata(&dir_hbs_path).is_ok() {
            return Err("Missing /layouts/dir.hbs");
        }

        // Grab generic stuff
        let header_hbs_contents = read_file(header_hbs_path)?;
        let footer_hbs_contents = read_file(footer_hbs_path)?;

        // Create Dir
        let dir_template_name = TYPE_STR;
        let dir_hbs_contents = read_file(&dir_hbs_path)?;

        app_context
            .handlebars
            .register_template_string(
                dir_template_name,
                format!(
                    "{}\n{}\n{}",
                    header_hbs_contents, dir_hbs_contents, footer_hbs_contents
                ),
            )
            .ok()
            .expect("Error registering header|dir|footer template");

        Ok(())
    }
}

pub struct Dir {
    path: PathBuf,
    type_str: &'static str,
    file_type_manager: crate::file_type::FileTypeManager,
}

impl Dir {
    fn get_children(&self, context: &crate::AppContext) -> Vec<Child> {
        let mut result: Vec<Child> = Vec::new();

        match fs::read_dir(&self.path) {
            Ok(items) => {
                for item in items {
                    let item = item.unwrap().path();
                    let child = self.file_type_manager.create_file_type(&item);
                    result.push(Child {
                        name: String::from(item.file_stem().unwrap().to_str().unwrap()),
                        url: child.get_url(context),
                        file_type: String::from(child.get_type_str()),
                    });
                }
            }
            Err(_) => (),
        }

        (&mut result).sort_by(|a, b| {
            if a.file_type == String::from(TYPE_STR) && b.file_type != String::from(TYPE_STR) {
                Ordering::Less
            } else if a.file_type != String::from(TYPE_STR) && b.file_type == String::from(TYPE_STR)
            {
                Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        result
    }
}

impl FileType for Dir {
    fn get_url(&self, context: &crate::AppContext) -> String {
        let relative = self
            .path
            .my_relative_from(&context.root_notes)
            .expect("Problem parsing relative url");
        let relative = if relative.to_str().unwrap() == "." {
            String::new()
        } else {
            format!("{}/", relative.to_str().unwrap())
        };
        format!("{}{}", context.base_url, relative)
    }

    fn convert(&self, context: &crate::AppContext) {
        let relative = self
            .path
            .my_relative_from(&context.root_notes)
            .expect("Problem parsing relative url");
        let new_dir = context.root_dest.join(&relative);
        let new_dir_index = new_dir.join("index.html");
        if !metadata(&new_dir).is_ok() {
            fs::create_dir(&new_dir)
                .ok()
                .expect("Cannot create destination subdir");
        }
        let children = self.get_children(context);
        let name = match relative.file_name() {
            Some(_) => String::from(relative.file_name().unwrap().to_str().unwrap()),
            None => String::from("root"),
        };
        let parents = create_parent_links(&context.base_url, &relative, true);
        let dir_model = DirModel {
            name: name,
            parents: parents,
            children: children,
            base_url: context.base_url.clone(),
        };
        match context.handlebars.render(TYPE_STR, &dir_model) {
            Ok(rendered) => {
                // Create File
                let mut file = File::create(&new_dir_index)
                    .ok()
                    .expect("Could not create dir index.html file");
                //fs::chmod(&new_dir_index, USER_FILE).ok().expect("Couldn't chmod new file");
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

#[derive(Serialize, PartialEq)]
struct Child {
    name: String,
    url: String,
    file_type: String,
}

#[derive(Serialize)]
struct DirModel {
    name: String,
    parents: Vec<Link>,
    children: Vec<Child>,
    base_url: String,
}

// impl ToJson for DirModel {
//         Json::from_str(&json::encode(&self).unwrap()).unwrap()
//     }
// }
