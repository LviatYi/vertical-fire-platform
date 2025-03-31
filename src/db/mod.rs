use crate::db::db_struct::LatestVersionData;
use dirs::home_dir;
use std::path::{Path, PathBuf};

mod db_struct;

pub const DB_FILE_NAME: &str = ".vf-extract-db.toml";

pub fn get_default_db_file_path(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        Some(path.to_path_buf())
    } else if path.is_dir() {
        Some(path.join(DB_FILE_NAME))
    } else {
        None
    }
}

pub fn get_db(path: Option<&Path>) -> LatestVersionData {
    LatestVersionData::get_from_path(path).unwrap_or_default()
}

pub fn delete_db_file() {
    let db_path = home_dir().unwrap_or_default();
    let _ = db_path
        .is_dir()
        .then(|| std::fs::remove_file(db_path.join(DB_FILE_NAME)));
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PartialEq for LatestVersionData {
        fn eq(&self, other: &Self) -> bool {
            self.branch == other.branch
                && self.last_inner_version == other.last_inner_version
                && self.last_player_count == other.last_player_count
                && self.extract_repo == other.extract_repo
                && self.extract_locator_pattern == other.extract_locator_pattern
                && self.extract_s_locator_template == other.extract_s_locator_template
                && self.blast_path == other.blast_path
        }
    }

    #[test]
    fn test_get_db_not_exist() {
        let path = PathBuf::from("non_existent_path");
        let db = get_db(Some(&path));
        assert_eq!(db, LatestVersionData::default());
    }

    #[test]
    fn test_save() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();

        let mut db = LatestVersionData::default();

        db.branch = Some("test_branch".to_string());
        db.last_inner_version = Some(1);
        db.last_player_count = Some(2);
        db.extract_repo = Some("test_repo".to_string());
        db.extract_locator_pattern = Some("test_pattern".to_string());
        db.extract_s_locator_template = Some("test_template".to_string());
        db.blast_path = Some(PathBuf::from("test_blast_path"));

        db.save(temp_file.path()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();

        assert_eq!(
            content,
            r#"version = 2
branch = "test_branch"
last_inner_version = 1
last_player_count = 2
extract_repo = "test_repo"
extract_locator_pattern = "test_pattern"
extract_s_locator_template = "test_template"
blast_path = "test_blast_path"
"#
        );

        let loaded_db = LatestVersionData::get_from_path(Some(temp_file.path())).unwrap();

        assert_eq!(loaded_db, db);
    }
}
