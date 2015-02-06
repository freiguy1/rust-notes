
use std::old_io::fs;
use std::old_io::fs::PathExtensions;
use std::old_io::{ FileType, File, USER_DIR, USER_FILE };

use rustc_serialize::json;
use rustc_serialize::json::{ ToJson, Json };

use rustdoc::html::markdown::Markdown;

use handlebars::Handlebars;

use std::rc::Rc;

use ::file_converter::{ create_parent_links, Link };


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

pub fn register_handlebars(source_root: &Path, handlebars: &mut Handlebars) -> Result<(), &'static str> {
    let header_hbs_path = source_root.clone().join("partials/header.hbs");
    if !header_hbs_path.exists() {
        return Err("Missing partials/header.hbs");
    }

    let footer_hbs_path = source_root.clone().join("partials/footer.hbs");
    if !footer_hbs_path.exists() {
        return Err("Missing partials/footer.hbs");
    }

    let note_hbs_path = source_root.clone().join("layouts/note.hbs");
    if !note_hbs_path.exists() {
        return Err("Missing /layouts/note.hbs");
    }

    let header_hbs_contents = File::open(&header_hbs_path).read_to_string().unwrap();
    let footer_hbs_contents = File::open(&footer_hbs_path).read_to_string().unwrap();
    let note_hbs_contents = File::open(&note_hbs_path).read_to_string().unwrap();
    handlebars.register_template_string(type_str(), format!("{}\n{}\n{}", header_hbs_contents, note_hbs_contents, footer_hbs_contents))
        .ok().expect("Error registering header|note|footer template");

    Ok(())

}

pub fn is_valid_path(path: &Path) -> bool {
    let name = path.filename_str().unwrap();
    path.is_file() && (
        name.ends_with(".md") || 
        name.ends_with(".markdown") || 
        name.ends_with(".mkd"))
}

pub fn convert(context: &::AppContext, path: &Path) {
    let relative = path.path_relative_from(&context.root_notes).expect("Problem parsing relative url");
    let file_name = relative.filestem_str().unwrap();
    let dest_file = context.root_dest.clone().join(relative.dirname_str().unwrap()).join(format!("{}.html", file_name));
    let source_contents = File::open(path).read_to_string().unwrap();
    // Create Model
    let content = Markdown(source_contents.as_slice());
    let parents = create_parent_links(context.base_url.as_slice(), &relative, false);

    let model = MarkdownModel {
        name: String::from_str(file_name),
        parents : parents,
        content : format!("{}", content),
        base_url: context.base_url.clone()
    };
    match context.handlebars.render(type_str(), &model) {
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

pub fn get_url(context: &::AppContext, path: &Path) -> String {
    let file_name = path.filestem_str().unwrap();
    let relative = path.path_relative_from(&context.root_notes).expect("Problem parsing relative url");
    let parent_relative = if relative.dirname_str().unwrap() == "." { 
        String::from_str("") 
    } else {
        format!("{}/", relative.dirname_str().unwrap())
    };
    format!("{}{}{}", context.base_url, parent_relative, format!("{}.html", file_name))
}

pub fn type_str() -> &'static str {
    "markdown"
}
