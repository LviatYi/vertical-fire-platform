use crate::db::db_struct::fp_db_v8::FpDbV8;
use crate::db::db_struct::job_relative_data::JobRelativeData;
use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use crate::define_versioned_data_type;
use crate::security::{decrypt_jenkins_pwd, encrypt_jenkins_pwd};
use serde::{Deserialize, Serialize, Serializer};

pub const VERSION_FP_DB_V7: u32 = 7;

define_versioned_data_type!(FpDbV7, VERSION_FP_DB_V7, {
    pub extract_repo: Option<String>,
    pub extract_locator_pattern: Option<String>,
    pub extract_s_locator_template: Option<String>,

    pub jenkins_url: Option<String>,
    pub jenkins_username: Option<String>,
    pub jenkins_api_token: Option<String>,
    pub jenkins_pwd: Option<String>,

    #[serde(default)]
    pub job_relative_data_arr: Vec<JobRelativeData>,

    #[serde(default)]
    pub never_check_version: bool,
    #[serde(default)]
    pub auto_update_enabled: bool,
    pub latest_remote_version: Option<String>,
    }
);

impl VersionedData for FpDbV7 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        let mut upg = FpDbV8::default();

        upg.extract_repo = self.extract_repo;
        upg.extract_locator_pattern = self.extract_locator_pattern;
        upg.extract_s_locator_template = self.extract_s_locator_template;
        upg.jenkins_url = self.jenkins_url;
        upg.jenkins_username = self.jenkins_username;
        upg.jenkins_api_token = self.jenkins_api_token;
        upg.password_encrypt = self
            .jenkins_pwd
            .as_deref()
            .and_then(decrypt_jenkins_pwd)
            .and_then(|plain| encrypt_jenkins_pwd(plain.as_str()).ok());
        upg.job_relative_data_arr = self.job_relative_data_arr;
        upg.never_check_version = self.never_check_version;
        upg.auto_update_enabled = self.auto_update_enabled;
        upg.latest_remote_version = self.latest_remote_version;

        UpgradeValue::Upgraded(Box::new(upg))
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

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
