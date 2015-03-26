
use std::cmp::Ordering;
use std::path::Path;
use std::fs;
use std::fs::{ PathExt, File };
use std::io::{ Write };

use rustc_serialize::json;
use rustc_serialize::json::{ ToJson, Json };

use handlebars::Handlebars;

use ::file_type::{ create_parent_links, Link, read_file };

pub fn register_handlebars<P: AsRef<Path>>(source_root: P, handlebars: &mut Handlebars) -> Result<(), &'static str> {
    let source_root: &Path = source_root.as_ref();

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
    let dir_template_name = type_str();
    let dir_hbs_contents = try!(read_file(&dir_hbs_path));

    handlebars.register_template_string(dir_template_name, format!("{}\n{}\n{}", header_hbs_contents, dir_hbs_contents, footer_hbs_contents))
        .ok().expect("Error registering header|dir|footer template");

    Ok(())
}

pub fn get_url<P: AsRef<Path>>(context: &::AppContext, path: P) -> String {
    let path: &Path = path.as_ref();
    let relative = path.relative_from(&context.root_notes).expect("Problem parsing relative url");
    let relative = if relative.to_str().unwrap() == "." { String::new() } else { 
        format!("{}/", relative.to_str().unwrap())
    };
    format!("{}{}", context.base_url, relative)
}


pub fn type_str() -> &'static str {
    "dir"
}

pub fn is_valid_path<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
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
        Json::from_str(&json::encode(&self).unwrap()).unwrap()
    }
}

pub fn convert<P: AsRef<Path>>(context: &::AppContext, path: P) {
    let path = path.as_ref();
    let relative = path.relative_from(&context.root_notes).expect("Problem parsing relative url");
    let new_dir = context.root_dest.join(&relative);
    let new_dir_index = new_dir.join("index.html");
    if !new_dir.exists() {
        fs::create_dir(&new_dir).ok().expect("Cannot create destination subdir");
    }
    let children = get_children(context, path);
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
    match context.handlebars.render(type_str(), &dir_model) {
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

fn get_children<P: AsRef<Path>>(context: &::AppContext, path: P) -> Vec<Child> {
    let path = path.as_ref();
    let mut result: Vec<Child> = Vec::new();

        match fs::read_dir(&path) {
            Ok(items) => {
                for item in items {
                    let item = item.unwrap().path();
                    let child_opt = ::file_type::FileType::new(&item)
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
