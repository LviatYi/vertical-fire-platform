use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use crate::define_versioned_data_type;
use crate::jenkins::build::VfpJobBuildParam;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

pub const VERSION_FP_DB_V6: u32 = 6;

define_versioned_data_type!(FpDbV6, VERSION_FP_DB_V6, {
    pub last_inner_version: Option<u32>,
    pub last_player_count: Option<u32>,
    pub interest_job_name: Option<String>,
    pub extract_repo: Option<String>,
    pub extract_locator_pattern: Option<String>,
    pub extract_s_locator_template: Option<String>,
    pub blast_path: Option<PathBuf>,
    pub jenkins_url: Option<String>,
    pub jenkins_username: Option<String>,
    pub jenkins_api_token: Option<String>,
    pub jenkins_pwd: Option<String>,
    pub jenkins_build_params: Option<VfpJobBuildParam>,
    #[serde(default)]
    pub never_check_version: bool,
    #[serde(default)]
    pub auto_update_enabled: bool,
    pub latest_remote_version: Option<String>,
    }
);

impl VersionedData for FpDbV6 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        UpgradeValue::Latest(*self)
    }
}
