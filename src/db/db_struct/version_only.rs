use crate::db::db_struct::db_status::DBStatus;
use crate::db::get_default_db_file_path;
use serde::{Deserialize, Serialize, Serializer};
use std::path::Path;
use toml::from_str;

pub const EARLY_STAGE_VERSION: u32 = 0;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VersionOnly {
    pub version: Option<u32>,
}

impl VersionOnly {
    fn from_path(path: &Path) -> Option<Self>
    where
        Self: Sized,
        for<'de> Self: serde::Deserialize<'de>,
    {
        get_default_db_file_path(path)
            .and_then(|file_path| std::fs::read_to_string(file_path).ok())
            .and_then(|content| from_str(&content).ok())
    }

    pub fn get_state_from_path(path: &Path) -> DBStatus {
        Self::from_path(path)
            .map(|p| p.version.unwrap_or(EARLY_STAGE_VERSION))
            .map(DBStatus::Exist)
            .unwrap_or(DBStatus::NotExist)
    }
}

pub fn serialize_version<S>(
    version: u32,
    _field: &std::marker::PhantomData<()>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32(version)
}

#[cfg(test)]
mod tests {
    use crate::db::db_struct::version_only::VersionOnly;
    use std::io::{Read, Seek, SeekFrom, Write};

    impl PartialEq for VersionOnly {
        fn eq(&self, other: &Self) -> bool {
            self.version == other.version
        }
    }

    #[test]
    fn test_read_version_only() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        let content = r#"version = 1
b = "Dev"
ci = 1905
"#;

        file.write_all(content.to_string().as_bytes()).unwrap();
        file.flush().unwrap();

        file.seek(SeekFrom::Start(0)).unwrap();

        let mut str: String = String::new();
        let _ = file.read_to_string(&mut str);

        assert_eq!(str, content);

        let version_info = VersionOnly::from_path(file.path());

        assert_eq!(version_info, Some(VersionOnly { version: Some(1) }));
    }
}
