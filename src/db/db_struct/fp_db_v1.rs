use crate::db::db_struct::fp_db_v2::FpDbV2;
use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use crate::define_versioned_data_type;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

pub const VERSION_FP_DB_V1: u32 = 1;

define_versioned_data_type!(FpDbV1, VERSION_FP_DB_V1, {
    pub b: Option<String>,
    pub ci: Option<u32>,
    pub c: Option<u32>,
    pub repo: Option<String>,
    pub locator_pattern: Option<String>,
    pub s_locator_template: Option<String>,
    pub d: Option<PathBuf>
    }
);

impl VersionedData for FpDbV1 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        let mut upg = FpDbV2::default();
        upg.branch = self.b;
        upg.last_inner_version = self.ci;
        upg.last_player_count = self.c;
        upg.extract_repo = self.repo;
        upg.extract_locator_pattern = self.locator_pattern;
        upg.extract_s_locator_template = self.s_locator_template;
        upg.blast_path = self.d;

        UpgradeValue::Upgraded(Box::new(upg))
    }
}
