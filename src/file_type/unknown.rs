use std::path::{ Path, PathBuf };

static TYPE_STR: &'static str = "unknown";

struct UnknownFactory;

impl ::file_type::FileTypeFactory for UnknownFactory {
    fn try_create(&self, path: &Path) -> Option<Box<FileType>> {
        Some(Box::new(Unknown {
            path: PathBuf::from(path),
            type_str: TYPE_STR
        }))
    }

    fn initialize(&self, app_context: &mut ::AppContext) -> Result<(), &'static str> {
        Ok(())
    }
}

struct Unknown {
    path: PathBuf,
    type_str: &'static str
}

impl ::file_type::FileType for Unknown {
    fn get_url(&self, context: &::AppContext) -> String {
        /*
        let file_name = self.path.file_stem().unwrap().to_str().unwrap();
        let relative = self.path.relative_from(&context.root_notes).expect("Problem parsing relative url");
        let parent_relative = if relative.parent().map_or_else(|| true,
            |p| p == Path::new("/") || p == Path::new("")) {
            String::from_str("")
        } else {
            format!("{}/", relative.parent().unwrap().to_str().unwrap())
        };
        format!("{}{}{}", context.base_url, parent_relative, format!("{}.html", file_name))
        */
        let file_name = self.path.file_name().unwrap();
    }

    fn convert(&self, context: &::AppContext) {
    }

    fn get_type_str(&self) -> &'static str {
    }
}
