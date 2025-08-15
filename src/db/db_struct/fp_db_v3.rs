use crate::db::db_struct::fp_db_v4::FpDbV4;
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
        let mut upg = FpDbV4::default();
        upg.interest_job_name = self.jenkins_interested_job_name;

        if let Some(old_extract_repo) = self.extract_repo {
            if old_extract_repo.contains("{B}") {
                let index = find_job_name_template_head_in_old_extract_repo(&old_extract_repo);
                upg.extract_repo = Some(old_extract_repo[..index].to_string());

                if upg.interest_job_name.is_none() && self.branch.is_some() {
                    upg.interest_job_name = Some(
                        old_extract_repo[index..]
                            .to_string()
                            .replace("{B}", &self.branch.clone().unwrap()),
                    )
                }
            } else {
                upg.extract_repo = Some(old_extract_repo);
            }
        }

        upg.last_inner_version = self.last_inner_version;
        upg.last_player_count = self.last_player_count;
        upg.extract_locator_pattern = self.extract_locator_pattern;
        upg.extract_s_locator_template = self.extract_s_locator_template;
        upg.blast_path = self.blast_path;
        upg.jenkins_url = self.jenkins_url;
        upg.jenkins_username = self.jenkins_username;
        upg.jenkins_api_token = self.jenkins_api_token;
        upg.jenkins_cookie = self.jenkins_cookie;

        UpgradeValue::Upgraded(Box::new(upg))
    }
}

fn find_job_name_template_head_in_old_extract_repo(old_extract_repo: &str) -> usize {
    old_extract_repo
        .rfind(['/', '\\'])
        .map(|v| v + 1)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_extract_repo_tail_as_job_name_template() {
        let old_extract_repo = r#"\\root\sub\TEMP_{B}_LATE"#;
        assert_eq!(
            old_extract_repo[find_job_name_template_head_in_old_extract_repo(old_extract_repo)..]
                .to_string(),
            "TEMP_{B}_LATE"
        );

        let old_extract_repo = r#"\TEMP_{B}_LATE"#;
        assert_eq!(
            old_extract_repo[find_job_name_template_head_in_old_extract_repo(old_extract_repo)..]
                .to_string(),
            "TEMP_{B}_LATE"
        );

        let old_extract_repo = r#"TEMP_{B}_LATE"#;
        assert_eq!(
            old_extract_repo[find_job_name_template_head_in_old_extract_repo(old_extract_repo)..]
                .to_string(),
            "TEMP_{B}_LATE"
        );
    }
}
