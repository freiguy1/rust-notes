use std::path::{ Path, PathBuf };
use std::fs;

static TYPE_STR: &'static str = "unknown";

pub struct UnknownFactory;

impl ::file_type::FileTypeFactory for UnknownFactory {
    fn try_create(&self, path: &Path) -> Option<Box<::file_type::FileType>> {
        Some(Box::new(Unknown {
            path: PathBuf::from(path),
            type_str: TYPE_STR
        }))
    }

    fn initialize(&self, _: &mut ::AppContext) -> Result<(), &'static str> {
        Ok(())
    }
}

pub struct Unknown {
    path: PathBuf,
    type_str: &'static str
}

impl ::file_type::FileType for Unknown {
    fn get_url(&self, context: &::AppContext) -> String {
        let file_name = self.path.file_name().expect("Problem parsing relative url");
        let relative = self.path.relative_from(&context.root_notes).expect("Problem parsing relative url");
        let parent_relative = if relative.parent().unwrap() == Path::new("") {
            String::from_str("")
        } else {
            format!("{}/", relative.parent().unwrap().to_str().unwrap())
        };
        format!("{}{}{}", context.base_url, parent_relative, file_name.to_str().unwrap())
    }

    fn convert(&self, context: &::AppContext) {
        let relative = self.path.relative_from(&context.root_notes).expect("Problem parsing relative url");
        let destination = context.root_dest.join(&relative);
        fs::copy(&self.path, &destination).ok().expect("Problem copying unknown file");
    }

    fn get_type_str(&self) -> &'static str {
        self.type_str
    }
}
