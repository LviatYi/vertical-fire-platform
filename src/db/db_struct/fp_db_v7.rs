use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use crate::define_versioned_data_type;
use crate::jenkins::build::VfpJobBuildParam;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

pub const VERSION_FP_DB_V7: u32 = 7;

define_versioned_data_type!(FpDbV7, VERSION_FP_DB_V7, {
    pub extract_repo: Option<String>,
    pub extract_locator_pattern: Option<String>,
    pub extract_s_locator_template: Option<String>,

    pub jenkins_url: Option<String>,
    pub jenkins_username: Option<String>,
    pub jenkins_api_token: Option<String>,
    pub jenkins_pwd: Option<String>,

    pub job_relative_data_arr: Vec<JobRelativeData>,

    #[serde(default)]
    pub never_check_version: bool,
    #[serde(default)]
    pub auto_update_enabled: bool,
    pub latest_remote_version: Option<String>,
    }
);

#[derive(Deserialize, Serialize, Default, Debug)]
pub(crate) struct JobRelativeData {
    pub job_name: String,

    pub last_inner_version: Option<u32>,
    pub last_player_count: Option<u32>,
    pub blast_path: Option<PathBuf>,

    pub jenkins_build_params: Option<VfpJobBuildParam>,
}

impl VersionedData for FpDbV7 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        UpgradeValue::Latest(*self)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    impl PartialEq for JobRelativeData {
        fn eq(&self, other: &Self) -> bool {
            self.job_name == other.job_name
                && self.last_inner_version == other.last_inner_version
                && self.last_player_count == other.last_player_count
                && self.blast_path == other.blast_path
                && self.jenkins_build_params == other.jenkins_build_params
        }
    }

    impl PartialEq for FpDbV7 {
        fn eq(&self, other: &Self) -> bool {
            self.extract_repo == other.extract_repo
                && self.extract_locator_pattern == other.extract_locator_pattern
                && self.extract_s_locator_template == other.extract_s_locator_template
                && self.jenkins_url == other.jenkins_url
                && self.jenkins_username == other.jenkins_username
                && self.jenkins_api_token == other.jenkins_api_token
                && self.jenkins_pwd == other.jenkins_pwd
                && self.job_relative_data_arr == other.job_relative_data_arr
                && self.never_check_version == other.never_check_version
                && self.auto_update_enabled == other.auto_update_enabled
                && self.latest_remote_version == other.latest_remote_version
        }
    }
}
