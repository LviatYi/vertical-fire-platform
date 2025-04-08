use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use crate::define_versioned_data_type;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

pub const VERSION_FP_DB_V3: u32 = 3;

define_versioned_data_type!(FpDbV3, VERSION_FP_DB_V3, {
    pub branch: Option<String>,
    pub last_inner_version: Option<u32>,
    pub last_player_count: Option<u32>,
    pub extract_repo: Option<String>,
    pub extract_locator_pattern: Option<String>,
    pub extract_s_locator_template: Option<String>,
    pub blast_path: Option<PathBuf>,
    pub jenkins_url:Option<String>,
    pub jenkins_username:Option<String>,
    pub jenkins_api_token:Option<String>,
    pub jenkins_cookie:Option<String>,
    pub jenkins_interested_job_name:Option<String>,
    }
);

impl VersionedData for FpDbV3 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        UpgradeValue::Latest(*self)
    }
}
