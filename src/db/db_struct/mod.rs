use crate::db::db_struct::fp_db_v1::FpDbV1;
use crate::db::db_struct::fp_db_v2::{FpDbV2, VERSION_FP_DB_V2};
use crate::db::db_struct::fp_db_v3::{FpDbV3, VERSION_FP_DB_V3};
use crate::db::db_struct::fp_db_v4::{FpDbV4, VERSION_FP_DB_V4};
use crate::db::db_struct::fp_db_v5::{FpDbV5, VERSION_FP_DB_V5};
use crate::db::db_struct::fp_db_v6::{FpDbV6, VERSION_FP_DB_V6};
use crate::db::db_struct::fp_db_v7::{FpDbV7, VERSION_FP_DB_V7};
use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};

pub mod db_status;
mod define_versioned_data_type;
pub mod fp_db_v1;
pub mod fp_db_v2;
pub mod fp_db_v3;
pub mod fp_db_v4;
pub mod fp_db_v5;
pub mod fp_db_v6;
pub mod fp_db_v7;
mod version_field;
pub mod version_only;
pub mod versioned_data;

pub type LatestVersionData = FpDbV7;

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
        VERSION_FP_DB_V7 => {
            FpDbV7::parse_from_string(content).map(|v| Box::new(v) as Box<dyn VersionedData>)
        }
        VERSION_FP_DB_V6 => {
            FpDbV6::parse_from_string(content).map(|v| Box::new(v) as Box<dyn VersionedData>)
        }
        VERSION_FP_DB_V5 => {
            FpDbV5::parse_from_string(content).map(|v| Box::new(v) as Box<dyn VersionedData>)
        }
        VERSION_FP_DB_V4 => {
            FpDbV4::parse_from_string(content).map(|v| Box::new(v) as Box<dyn VersionedData>)
        }
        VERSION_FP_DB_V3 => {
            FpDbV3::parse_from_string(content).map(|v| Box::new(v) as Box<dyn VersionedData>)
        }
        VERSION_FP_DB_V2 => {
            FpDbV2::parse_from_string(content).map(|v| Box::new(v) as Box<dyn VersionedData>)
        }
        _ => FpDbV1::parse_from_string(content).map(|v| Box::new(v) as Box<dyn VersionedData>),
    }
}
