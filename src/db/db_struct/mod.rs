use crate::db::db_struct::fp_db_v1::FpDbV1;
use crate::db::db_struct::fp_db_v2::{FpDbV2, VERSION_FP_DB_V2};
use crate::db::db_struct::version_only::VersionOnly;
use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use db_status::DBStatus::{Exist, NotExist};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub mod fp_db_v1;
pub mod versioned_data;

mod db_status;
mod define_versioned_data_type;
mod fp_db_v2;
mod version_field;
mod version_only;

pub type LatestVersionData = FpDbV2;

/// # parse content with upgrade
///
/// parse & upgrade the versioned data to latest.
pub fn parse_content_with_upgrade(
    curr_version: u32,
    content: &str,
) -> Result<LatestVersionData, toml::de::Error> {
    let mut db: Box<dyn VersionedData> = parse_content_by_version(curr_version, content)?;
    loop {
        let upgrade_value = db.parse_next_version();
        match upgrade_value {
            UpgradeValue::Latest(latest) => return Ok(latest),
            UpgradeValue::Upgraded(d) => db = d,
            UpgradeValue::Invalid => {
                return Err(serde::de::Error::custom(
                    crate::constant::log::ERR_UPGRADE_NOT_DEFINED,
                ));
            }
        }
    }
}

/// # parse content by version
///
/// get VersionedData from str by version.
fn parse_content_by_version(
    version: u32,
    content: &str,
) -> Result<Box<dyn VersionedData>, toml::de::Error> {
    match version {
        VERSION_FP_DB_V2 => {
            FpDbV2::parse_from_string(content).map(|v| Box::new(v) as Box<dyn VersionedData>)
        }
        _ => FpDbV1::parse_from_string(content).map(|v| Box::new(v) as Box<dyn VersionedData>),
    }
}

impl LatestVersionData {
    pub fn get_from_path(path: &Path) -> Option<Self> {
        match VersionOnly::get_state_from_path(path) {
            Exist(version) => {
                let content = std::fs::read_to_string(path).ok()?;
                parse_content_with_upgrade(version, &content).ok()
            }
            NotExist => None,
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let str = toml::to_string(self).map_err(|e| e.to_string())?;
        File::create(path)
            .map_err(|e| e.to_string())?
            .write_all(str.as_bytes())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_old_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        let content = r#"b = "Dev"
ci = 2025
c = 9
d = 'C:\Users\LviatYi\Desktop\Temp'
"#;

        file.write_all(content.to_string().as_bytes()).unwrap();
        file.flush().unwrap();

        let config = LatestVersionData::get_from_path(file.path());

        assert!(config.is_some());

        let config = config.unwrap();

        assert_eq!(config.branch, Some("Dev".to_string()));
        assert_eq!(config.last_inner_version, Some(2025));
        assert_eq!(config.last_player_count, Some(9));
        assert_eq!(
            config.blast_path,
            Some(PathBuf::from("C:\\Users\\LviatYi\\Desktop\\Temp"))
        );

        assert!(config.extract_locator_pattern.is_none());
    }

    #[test]
    fn test_get_file_not_exist() {
        let config = LatestVersionData::get_from_path(Path::new("Z:\\NOT_EXIST"));

        assert!(config.is_none());
    }

    #[test]
    fn test_get_file_version_2() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        let content = r#"version = 2
branch = "Dev"
last_inner_version = 2025
last_player_count = 9
blast_path = "C:\\Users\\LviatYi\\Desktop\\Temp"
"#;

        file.write_all(content.to_string().as_bytes()).unwrap();
        file.flush().unwrap();

        let config = LatestVersionData::get_from_path(file.path());

        assert!(config.is_some());

        let config = config.unwrap();
        assert_eq!(config.branch, Some("Dev".to_string()));
        assert_eq!(config.last_inner_version, Some(2025));
        assert_eq!(config.last_player_count, Some(9));
        assert_eq!(
            config.blast_path,
            Some(PathBuf::from("C:\\Users\\LviatYi\\Desktop\\Temp"))
        );
        assert_eq!(config.extract_locator_pattern, None);
    }
}
