use std::fs;
use std::path::Path;

macro_rules! load_config {
    ($($field:ident),*) => {
        pub struct ResourceLoader {
            $(pub $field: String,)*
        }

        impl ResourceLoader {
            pub fn load() -> Self {
                let mut value:Option<toml::Value> = None;
                let path = Path::new("resource/variable.toml");
                if path.is_file() {
                    if let Ok(content) = fs::read_to_string(path).map_err(|_| ()) {
                         value = toml::from_str(content.as_str()).ok();
                    }
                }
        
                Self {
                    $($field: get_value(
                        &(value.as_ref()),
                        stringify!($field),
                        crate::default_config::$field
                    ),)*
                }
            }
        }
    }
}

load_config!(
    REPO_TEMPLATE,
    LOCATOR_PATTERN,
    LOCATOR_TEMPLATE,
    MENDING_FILE_PATH,
    PACKAGE_FILE_STEM,
    EXE_FILE_NAME,
    CHECK_EXE_FILE_NAME
);

fn get_value(value: &Option<&toml::Value>, key: &str, default: &str) -> String {
    value
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}
