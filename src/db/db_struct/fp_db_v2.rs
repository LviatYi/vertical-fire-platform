use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use crate::define_versioned_data_type;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

pub const VERSION_FP_DB_V2: u32 = 2;

define_versioned_data_type!(FpDbV2, VERSION_FP_DB_V2, {
    pub branch: Option<String>,
    pub last_inner_version: Option<u32>,
    pub last_player_count: Option<u32>,
    pub extract_repo: Option<String>,
    pub extract_locator_pattern: Option<String>,
    pub extract_s_locator_template: Option<String>,
    pub blast_path: Option<PathBuf>
    }
);

impl VersionedData for FpDbV2 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        UpgradeValue::Latest(*self)
    }
}
