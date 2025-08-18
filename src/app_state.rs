use crate::constant::log::HINT_NO_VALID_PATH;
use crate::db::DB_FILE_NAME;
use crate::db::db_data_proxy::DbDataProxy;
use crate::pretty_log::{ThemeColor, colored_println};
use crate::vfp_error::VfpError;
use dirs::home_dir;
use formatx::formatx;
use std::cell::OnceCell;
use std::fs::create_dir_all;
use std::io::{Stdout, StdoutLock};
use std::ops::Not;
use std::path::{Path, PathBuf};

fn get_path_or_home_path(path: Option<&Path>) -> PathBuf {
    path.unwrap_or(&home_dir().unwrap_or_default())
        .to_path_buf()
}

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

pub struct AppState {
    db: OnceCell<DbDataProxy>,

    db_path: Option<PathBuf>,

    stdout: Stdout,
}

impl AppState {
    pub fn new(path: Option<&Path>) -> AppState {
        AppState {
            db: OnceCell::new(),
            db_path: path.map(|path| path.into()),
            stdout: std::io::stdout(),
        }
    }

    pub fn get_stdout(&self) -> StdoutLock<'_> {
        self.stdout.lock()
    }

    pub fn get_mut_db(&mut self) -> &mut DbDataProxy {
        let _ = self.ensure_init();

        self.db.get_mut().unwrap()
    }

    pub fn get_db(&self) -> &DbDataProxy {
        self.ensure_init()
    }

    pub fn commit(&self, silence: bool) {
        let path = touch_default_db_file_path(&get_path_or_home_path(self.db_path.as_deref()));

        if let Err(e) = self.get_db().save(&path)
            && !silence
        {
            colored_println(
                &mut self.stdout.lock(),
                ThemeColor::Error,
                &formatx!(crate::constant::log::ERR_DB_SAVE_FAILURE, e).unwrap_or_default(),
            )
        }
    }

    pub fn clean(&mut self) {
        get_default_db_file_path(&get_path_or_home_path(self.db_path.as_deref())).inspect(|p| {
            p.is_file().then(|| std::fs::remove_file(p));
        });

        let _ = self.db.take();
    }

    pub fn open_db_file(&self) -> Result<(), VfpError> {
        if let Some(path) =
            get_default_db_file_path(&get_path_or_home_path(self.db_path.as_deref()))
        {
            open::that(&path).map_err(|_| {
                VfpError::OpenDbFailed(path.to_str().map(|str| str.to_string()).unwrap_or_default())
            })?;
            colored_println(
                &mut self.stdout.lock(),
                ThemeColor::Success,
                crate::constant::log::OPEN_DB_SUCCESS,
            );
        }

        Err(VfpError::OpenDbFailed(HINT_NO_VALID_PATH.to_string()))
    }

    fn ensure_init(&self) -> &DbDataProxy {
        self.db.get_or_init(|| {
            get_default_db_file_path(&get_path_or_home_path(self.db_path.as_deref()))
                .and_then(|item| DbDataProxy::get_from_path(&item))
                .unwrap_or_default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_not_exist() {
        let path = PathBuf::from("non_existent_path");
        let app_state = AppState::new(Some(&path));

        assert_eq!(*app_state.ensure_init(), DbDataProxy::default());
    }

    #[test]
    fn test_save() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();

        let mut db = DbDataProxy::default();
        let job_name = "test_job";

        db.set_last_inner_version(job_name, Some(1024));
        db.set_last_player_count(job_name, Some(4));

        db.save(temp_file.path()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();

        assert_eq!(
            content,
            r#"version = 7
never_check_version = false
auto_update_enabled = false

[[job_relative_data_arr]]
job_name = "test_job"
last_inner_version = 1024
last_player_count = 4
"#
        );

        let app_state = AppState::new(Some(temp_file.path()));

        assert_eq!(*app_state.ensure_init(), db);
    }

    #[test]
    fn test_delete() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let db = DbDataProxy::default();
        db.save(&path).unwrap();

        assert!(path.is_file());

        let mut app_state = AppState::new(Some(&path));
        app_state.ensure_init();

        app_state.clean();

        assert!(!path.exists());
        assert!(app_state.db.take().is_none());
    }
}
