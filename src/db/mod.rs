use crate::db::db_struct::LatestVersionData;
use dirs::home_dir;
use formatx::formatx;
use std::fs::create_dir_all;
use std::ops::Not;
use std::path::{Path, PathBuf};

pub mod db_struct;

pub const DB_FILE_NAME: &str = ".vf-extract-db.toml";

fn get_default_db_file_path(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        Some(path.to_path_buf())
    } else if path.is_dir() {
        Some(path.join(DB_FILE_NAME))
    } else {
        None
    }
}

fn touch_default_db_file_path(path: &Path) -> PathBuf {
    if path.is_file() {
        return path.to_path_buf();
    } else if path.is_dir() {
        return path.join(DB_FILE_NAME);
    }

    let mut path = path.to_path_buf();
    if path.is_relative() {
        path = home_dir().unwrap().join(path)
    }

    if !path
        .file_name()
        .and_then(|item| item.to_str())
        .is_some_and(|n| n.ends_with(".toml"))
    {
        path = path.join(DB_FILE_NAME);
    }

    if let Some(p) = path.parent() {
        p.exists().not().then(|| create_dir_all(p).ok());
    }

    path
}

pub fn get_db(path: Option<&Path>) -> LatestVersionData {
    get_default_db_file_path(&get_path_or_home_path(path))
        .and_then(|item| LatestVersionData::get_from_path(&item))
        .unwrap_or_default()
}

pub fn delete_db_file(path: Option<&Path>) {
    get_default_db_file_path(&get_path_or_home_path(path)).inspect(|p| {
        p.is_file().then(|| std::fs::remove_file(p));
    });
}

pub fn save_with_error_log(db: &LatestVersionData, path: Option<&Path>) {
    let path = touch_default_db_file_path(&get_path_or_home_path(path));

    if let Err(e) = db.save(&path) {
        eprintln!(
            "{}",
            formatx!(crate::constant::log::ERR_DB_SAVE_FAILURE, e).unwrap_or_default()
        );
    }
}

fn get_path_or_home_path(path: Option<&Path>) -> PathBuf {    
    path.unwrap_or(&home_dir().unwrap_or_default())
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PartialEq for LatestVersionData {
        fn eq(&self, other: &Self) -> bool {
            self.last_inner_version == other.last_inner_version
                && self.last_player_count == other.last_player_count
                && self.interest_job_name == other.interest_job_name
                && self.extract_repo == other.extract_repo
                && self.extract_locator_pattern == other.extract_locator_pattern
                && self.extract_s_locator_template == other.extract_s_locator_template
                && self.blast_path == other.blast_path
                && self.jenkins_url == other.jenkins_url
                && self.jenkins_username == other.jenkins_username
                && self.jenkins_api_token == other.jenkins_api_token
                && self.jenkins_pwd == other.jenkins_pwd
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

        db.last_inner_version = Some(1);
        db.last_player_count = Some(2);

        db.save(temp_file.path()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();

        assert_eq!(
            content,
            r#"version = 4
last_inner_version = 1
last_player_count = 2
"#
        );

        let loaded_db = get_db(Some(temp_file.path()));

        assert_eq!(loaded_db, db);
    }

    #[test]
    fn test_delete() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let db = LatestVersionData::default();
        db.save(&path).unwrap();

        assert!(path.is_file());

        delete_db_file(Some(&path));

        assert!(!path.exists());
    }
}
