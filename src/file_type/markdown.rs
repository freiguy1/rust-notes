
use std::path::{ AsPath, Path };
use std::fs::{ File, PathExt };
use std::io::{ Read, Write };

use rustc_serialize::json;
use rustc_serialize::json::{ ToJson, Json };

use rustdoc::html::markdown::Markdown;

use handlebars::Handlebars;

use ::file_type::{ create_parent_links, Link, read_file };


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

pub fn register_handlebars<P: AsPath>(source_root: P, handlebars: &mut Handlebars) -> Result<(), &'static str> {
    let source_root = source_root.as_path();
    let header_hbs_path = source_root.clone().join("partials/header.hbs");
    if !header_hbs_path.exists() {
        return Err("Missing partials/header.hbs");
    }

    let footer_hbs_path = source_root.clone().join("partials/footer.hbs");
    if !footer_hbs_path.exists() {
        return Err("Missing partials/footer.hbs");
    }

    let note_hbs_path = source_root.join("layouts/note.hbs");
    if !note_hbs_path.exists() {
        return Err("Missing /layouts/note.hbs");
    }

    let header_hbs_contents = try!(read_file(&header_hbs_path));//File::open(&header_hbs_path).read_to_string().unwrap();
    let footer_hbs_contents = try!(read_file(&note_hbs_path));//File::open(&footer_hbs_path).read_to_string().unwrap();
    let note_hbs_contents = try!(read_file(&note_hbs_path));//File::open(&note_hbs_path).read_to_string().unwrap();
    handlebars.register_template_string(type_str(), format!("{}\n{}\n{}", header_hbs_contents, note_hbs_contents, footer_hbs_contents))
        .ok().expect("Error registering header|note|footer template");

    Ok(())

}

pub fn is_valid_path<P: AsPath>(path: P) -> bool {
    let path = path.as_path();
    let name = path.file_name().unwrap().to_str().unwrap();
    path.is_file() && (
        name.ends_with(".md") || 
        name.ends_with(".markdown") || 
        name.ends_with(".mkd"))
}

pub fn convert<P: AsPath>(context: &::AppContext, path: P) {
    let path = path.as_path();
    let relative = path.relative_from(&context.root_notes).expect("Problem parsing relative url");
    let file_name = relative.file_stem().unwrap().to_str().unwrap();
    let dest_file = context.root_dest.clone().join(relative.parent().unwrap()).join(format!("{}.html", file_name));
    let mut source_contents = String::new();
    File::open(path).ok().unwrap().read_to_string(&mut source_contents);
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
            //fs::chmod(&dest_file, USER_FILE).ok().expect("Couldn't chmod new file");
            file.write_all(rendered.as_slice().as_bytes())
                .ok().expect("Could not write html to file");
        },
        Err(why) => panic!("Error rendering markdown: {:?}", why)
    }
}

pub fn get_url<P: AsPath>(context: &::AppContext, path: P) -> String {
    let path = path.as_path();
    let file_name = path.file_stem().unwrap().to_str().unwrap();
    let relative = path.relative_from(&context.root_notes).expect("Problem parsing relative url");
    let parent_relative = if relative.parent().is_none() {
        String::from_str("") 
    } else {
        format!("{}/", relative.parent().unwrap().to_str().unwrap())
    };
    format!("{}{}{}", context.base_url, parent_relative, format!("{}.html", file_name))
}

pub fn type_str() -> &'static str {
    "markdown"
}
