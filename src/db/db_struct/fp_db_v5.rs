use crate::db::db_struct::fp_db_v6::FpDbV6;
use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use crate::define_versioned_data_type;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

pub const VERSION_FP_DB_V5: u32 = 5;

define_versioned_data_type!(FpDbV5, VERSION_FP_DB_V5, {
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
    }
);

impl VersionedData for FpDbV5 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        let mut upg = FpDbV6::default();
        upg.last_inner_version = self.last_inner_version;
        upg.last_player_count = self.last_player_count;
        upg.interest_job_name = self.interest_job_name;
        upg.extract_repo = self.extract_repo;
        upg.extract_locator_pattern = self.extract_locator_pattern;
        upg.extract_s_locator_template = self.extract_s_locator_template;
        upg.blast_path = self.blast_path;
        upg.jenkins_url = self.jenkins_url;
        upg.jenkins_username = self.jenkins_username;
        upg.jenkins_api_token = self.jenkins_api_token;
        upg.jenkins_pwd = None;

        UpgradeValue::Upgraded(Box::new(upg))
    }
}
