
use std::cmp::Ordering;
use std::path::{ Path, PathBuf };
use std::fs;
use std::fs::{ PathExt, File };
use std::io::{ Write };

use rustc_serialize::json;
use rustc_serialize::json::{ ToJson, Json };

use handlebars::Handlebars;

use ::file_type::{ create_parent_links, Link, read_file, FileType };

static TYPE_STR: &'static str = "dir";

pub struct DirFactory;

impl ::file_type::FileTypeFactory for DirFactory {
    fn try_create(&self, path: &Path) -> Option<Box<FileType>> {
        if path.is_dir() {
            Some(Box::new(Dir {
                path: PathBuf::from(path),
                type_str: TYPE_STR,
                file_type_manager: ::file_type::FileTypeManager::new()
            }))
        } else { None }
    }

    fn initialize(&self, source_root: &Path, handlebars: &mut Handlebars) -> Result<(), &'static str> {

        // Validate generic stuff
        let header_hbs_path = source_root.join("partials/header.hbs");
        if !header_hbs_path.exists() {
            return Err("Missing partials/header.hbs");
        }

        let footer_hbs_path = source_root.join("partials/footer.hbs");
        if !footer_hbs_path.exists() {
            return Err("Missing partials/footer.hbs");
        }

        // Validate Dir
        let dir_hbs_path = source_root.join("layouts/dir.hbs");
        if !dir_hbs_path.exists() {
            return Err("Missing /layouts/dir.hbs");
        }

        // Grab generic stuff
        let header_hbs_contents = try!(read_file(header_hbs_path));
        let footer_hbs_contents = try!(read_file(footer_hbs_path));

        // Create Dir
        let dir_template_name = TYPE_STR;
        let dir_hbs_contents = try!(read_file(&dir_hbs_path));

        handlebars.register_template_string(dir_template_name, format!("{}\n{}\n{}", header_hbs_contents, dir_hbs_contents, footer_hbs_contents))
            .ok().expect("Error registering header|dir|footer template");

        Ok(())
    }
}

pub struct Dir {
    path: PathBuf,
    type_str: &'static str
}

impl Dir {
    fn get_children(&self, context: &::AppContext) -> Vec<Child> {
        let mut result: Vec<Child> = Vec::new();

            match fs::read_dir(&self.path) {
                Ok(items) => {
                    for item in items {
                        let item = item.unwrap().path();
                        let child_opt = self.file_type_manager.create_file_type(&item)
                            .map(|ft| Child {
                                name: String::from_str(item.file_stem().unwrap().to_str().unwrap()),
                                url: ft.get_url(context),
                                file_type: String::from_str(ft.get_type_str())
                            });
                        if child_opt.is_some() { result.push(child_opt.unwrap()); }
                    }
                },
                Err(_) => ()
            }

        (&mut result).sort_by(|a, b| {
            if a.file_type == String::from_str(TYPE_STR) && b.file_type != String::from_str(TYPE_STR) {
                Ordering::Less
            } else {
                a.file_type.cmp(&b.file_type)
            }

        });

        result
    }
}

impl FileType for Dir {
    fn get_url(&self, context: &::AppContext) -> String {
        let relative = self.path.relative_from(&context.root_notes).expect("Problem parsing relative url");
        let relative = if relative.to_str().unwrap() == "." { String::new() } else {
            format!("{}/", relative.to_str().unwrap())
        };
        format!("{}{}", context.base_url, relative)
    }

    fn convert(&self, context: &::AppContext) {
        let relative = self.path.relative_from(&context.root_notes).expect("Problem parsing relative url");
        let new_dir = context.root_dest.join(&relative);
        let new_dir_index = new_dir.join("index.html");
        if !new_dir.exists() {
            fs::create_dir(&new_dir).ok().expect("Cannot create destination subdir");
        }
        let children = self.get_children(context);
        let name = match relative.file_name() {
            Some(_) => String::from_str(relative.file_name().unwrap().to_str().unwrap()),
            None => String::from_str("root")
        };
        let parents = create_parent_links(&context.base_url, &relative, true);
        let dir_model = DirModel {
            name: name,
            parents: parents,
            children: children,
            base_url: context.base_url.clone()
        };
        match context.handlebars.render(TYPE_STR, &dir_model) {
            Ok(rendered) => {
                // Create File
                let mut file = File::create(&new_dir_index).ok().expect("Could not create dir index.html file");
                //fs::chmod(&new_dir_index, USER_FILE).ok().expect("Couldn't chmod new file");
                file.write_all(rendered.as_bytes())
                    .ok().expect("Could not write html to file");
            },
            Err(why) => panic!("Error rendering markdown: {:?}", why)
        }
    }

    fn get_type_str(&self) -> &'static str {
        self.type_str
    }
}


#[derive(RustcEncodable, PartialEq)]
struct Child {
    name: String,
    url: String,
    file_type: String
}

#[derive(RustcEncodable)]
struct DirModel {
    name: String,
    parents: Vec<Link>,
    children: Vec<Child>,
    base_url: String
}

impl ToJson for DirModel {
    fn to_json(&self) -> Json {
        Json::from_str(&json::encode(&self).unwrap()).unwrap()
    }
}
