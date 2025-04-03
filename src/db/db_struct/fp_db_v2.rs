use crate::db::db_struct::fp_db_v3::FpDbV3;
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
        let mut upg = FpDbV3::default();
        upg.branch = self.branch;
        upg.last_inner_version = self.last_inner_version;
        upg.last_player_count = self.last_player_count;
        upg.extract_repo = self.extract_repo;
        upg.extract_locator_pattern = self.extract_locator_pattern;
        upg.extract_s_locator_template = self.extract_s_locator_template;
        upg.blast_path = self.blast_path;

        UpgradeValue::Upgraded(Box::new(upg))
    }
}
