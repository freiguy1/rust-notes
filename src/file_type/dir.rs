
use std::cmp::Ordering;
use std::old_io::fs;
use std::old_io::fs::PathExtensions;
use std::old_io::{ File, USER_DIR, USER_FILE };

use rustc_serialize::json;
use rustc_serialize::json::{ ToJson, Json };

use handlebars::Handlebars;

use ::file_type::{ create_parent_links, Link };

pub fn register_handlebars(source_root: &Path, handlebars: &mut Handlebars) -> Result<(), &'static str> {
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

    // Create Dir
    let dir_template_name = type_str();
    let dir_hbs_contents = File::open(&dir_hbs_path).read_to_string().unwrap();
    handlebars.register_template_string(dir_template_name, format!("{}\n{}\n{}", header_hbs_contents, dir_hbs_contents, footer_hbs_contents))
        .ok().expect("Error registering header|dir|footer template");

    Ok(())
}

pub fn get_url(context: &::AppContext, path: &Path) -> String {
    let relative = path.path_relative_from(&context.root_notes).expect("Problem parsing relative url");
    let relative = if relative.as_str().unwrap() == "." { String::new() } else { 
        format!("{}/", relative.as_str().unwrap())
    };
    format!("{}{}", context.base_url, relative)
}


pub fn type_str() -> &'static str {
    "dir"
}

pub fn is_valid_path(path: &Path) -> bool {
    path.is_dir()
}

#[derive(RustcEncodable, Debug, PartialEq)]
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
        Json::from_str(json::encode(&self).unwrap().as_slice()).unwrap()
    }
}

pub fn convert(context: &::AppContext, path: &Path) {
    let relative = path.path_relative_from(&context.root_notes).expect("Problem parsing relative url");
    let new_dir = context.root_dest.join(&relative);
    let new_dir_index = new_dir.join("index.html");
    if !new_dir.exists() {
        fs::mkdir(&new_dir, USER_DIR).ok().expect("Cannot create destination subdir");
    }
    let children = get_children(context, path);
    let name = match relative.filename() {
        Some(_) => String::from_str(relative.filename_str().unwrap()),
        None => String::from_str("root")
    };
    let parents = create_parent_links(context.base_url.as_slice(), &relative, true);
    let dir_model = DirModel {
        name: name,
        parents: parents,
        children: children,
        base_url: context.base_url.clone()
    };
    match context.handlebars.render(type_str(), &dir_model) {
        Ok(rendered) => {
            // Create File
            let mut file = File::create(&new_dir_index).ok().expect("Could not create dir index.html file");
            fs::chmod(&new_dir_index, USER_FILE).ok().expect("Couldn't chmod new file");
            file.write_str(rendered.as_slice())
                .ok().expect("Could not write html to file");
        },
        Err(why) => panic!("Error rendering markdown: {:?}", why)
    }
}

fn get_children(context: &::AppContext, path: &Path) -> Vec<Child> {
    let mut result: Vec<Child> = Vec::new();

        match fs::readdir(&path) {
            Ok(items) => {
                for item in items.iter() {
                    let child_opt = ::file_type::FileType::new(item)
                        .map(|ft| Child {
                            name: String::from_str(item.filestem_str().unwrap()),
                            url: ft.get_url(context),
                            file_type: String::from_str(ft.get_type_str())
                        });
                    if child_opt.is_some() { result.push(child_opt.unwrap()); }
                }
            },
            Err(_) => ()
        }

    result.as_mut_slice().sort_by(|a, b| {
        if a.file_type == String::from_str(type_str()) && b.file_type != String::from_str(type_str()) {
            Ordering::Less
        } else {
            a.file_type.cmp(&b.file_type)
        }

    });

    result
}

#[test]
fn test() {
    let mut handlebars = Handlebars::new();
    register_handlebars(&Path::new("/home/freied/dev/git/notes-site"), &mut handlebars).ok().unwrap();

    let context = ::AppContext {
        root_source: Path::new("/home/freied/dev/git/notes-site"),
        root_dest: Path::new("/home/freied/temp/dest"),
        root_notes: Path::new("/home/freied/dev/git/notes-site/notes"),
        handlebars: handlebars,
        base_url: String::from_str("/abcd/")
    };

    let path = Path::new("/home/freied/dev/git/notes-site/notes/recipes");
    convert(&context, &path);

    assert!(true);

}
