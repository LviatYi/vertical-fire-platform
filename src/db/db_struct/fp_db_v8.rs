use crate::db::db_struct::job_relative_data::JobRelativeData;
use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use crate::define_versioned_data_type;
use serde::{Deserialize, Serialize, Serializer};

pub const VERSION_FP_DB_V8: u32 = 8;

define_versioned_data_type!(FpDbV8, VERSION_FP_DB_V8, {
    pub extract_repo: Option<String>,
    pub extract_locator_pattern: Option<String>,
    pub extract_s_locator_template: Option<String>,

    pub jenkins_url: Option<String>,
    pub jenkins_username: Option<String>,
    pub jenkins_api_token: Option<String>,
    pub password_encrypt: Option<String>,

    #[serde(default)]
    pub job_relative_data_arr: Vec<JobRelativeData>,

    #[serde(default)]
    pub never_check_version: bool,
    #[serde(default)]
    pub auto_update_enabled: bool,
    pub latest_remote_version: Option<String>,
    }
);

impl VersionedData for FpDbV8 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        UpgradeValue::Latest(*self)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    impl PartialEq for FpDbV8 {
        fn eq(&self, other: &Self) -> bool {
            self.extract_repo == other.extract_repo
                && self.extract_locator_pattern == other.extract_locator_pattern
                && self.extract_s_locator_template == other.extract_s_locator_template
                && self.jenkins_url == other.jenkins_url
                && self.jenkins_username == other.jenkins_username
                && self.jenkins_api_token == other.jenkins_api_token
                && self.password_encrypt == other.password_encrypt
                && self.job_relative_data_arr == other.job_relative_data_arr
                && self.never_check_version == other.never_check_version
                && self.auto_update_enabled == other.auto_update_enabled
                && self.latest_remote_version == other.latest_remote_version
        }
    }
}
