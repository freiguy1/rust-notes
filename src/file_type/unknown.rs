use std::fs;
use std::path::{Path, PathBuf};

use crate::util::RelativeFrom;

use crate::file_type::FileType;

static TYPE_STR: &'static str = "unknown";

pub struct UnknownFactory;

impl crate::file_type::FileTypeFactory for UnknownFactory {
    fn try_create(&self, path: &Path) -> Option<Box<dyn FileType>> {
        Some(Box::new(Unknown {
            path: PathBuf::from(path),
            type_str: TYPE_STR,
        }))
    }

    fn initialize(&self, _: &mut crate::AppContext) -> Result<(), &'static str> {
        Ok(())
    }
}

pub struct Unknown {
    path: PathBuf,
    type_str: &'static str,
}

impl crate::file_type::FileType for Unknown {
    fn get_url(&self, context: &crate::AppContext) -> String {
        let file_name = self.path.file_name().expect("Problem parsing relative url");
        let relative = self
            .path
            .my_relative_from(&context.root_notes)
            .expect("Problem parsing relative url");
        let parent_relative = if relative.parent().unwrap() == Path::new("") {
            String::from("")
        } else {
            format!("{}/", relative.parent().unwrap().to_str().unwrap())
        };
        format!(
            "{}{}{}",
            context.base_url,
            parent_relative,
            file_name.to_str().unwrap()
        )
    }

    fn convert(&self, context: &crate::AppContext) {
        let relative = self
            .path
            .my_relative_from(&context.root_notes)
            .expect("Problem parsing relative url");
        let destination = context.root_dest.join(&relative);
        fs::copy(&self.path, &destination)
            .ok()
            .expect("Problem copying unknown file");
    }

    fn get_type_str(&self) -> &'static str {
        self.type_str
    }
}
