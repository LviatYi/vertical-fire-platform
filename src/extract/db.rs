use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::from_str;

pub const DB_FILE_NAME: &str = ".vf-extract-db.toml";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExtractDb {
    pub b: Option<String>,

    pub ci: Option<u32>,

    pub c: Option<u32>,

    pub repo: Option<String>,

    pub locator_pattern: Option<String>,

    pub s_locator_template: Option<String>,

    pub d: Option<PathBuf>,
}

impl ExtractDb {
    pub fn from_path(path: &Path) -> Option<Self> {
        get_valid_path(path)
            .and_then(|file_path| std::fs::read_to_string(file_path).ok())
            .and_then(|content| from_str(&content).ok())
    }

    fn save(&self, path: &Path) -> Result<(), String> {
        let save_path = get_valid_path(path).ok_or_else(|| "Invalid path".to_string())?;
        let str = toml::to_string(self).map_err(|e| e.to_string())?;
        File::create(save_path)
            .map_err(|e| e.to_string())?
            .write_all(str.as_bytes())
            .map_err(|e| e.to_string())
    }

    pub fn save_with_error_log(&self, path: &Path) {
        if let Err(e) = self.save(path) {
            eprintln!("Archive storage failure: {}", e);
        }
    }
}

fn get_valid_path(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        Some(path.to_path_buf())
    } else if path.is_dir() {
        Some(path.join(DB_FILE_NAME))
    } else {
        None
    }
}
