use crate::constant::log::{LOGIN_SUCCESS_BY_API_TOKEN, LOGIN_SUCCESS_BY_PWD};
use crate::constant::util::bring_element_to_first;
use crate::db::db_struct::db_status::DBStatus;
use crate::db::db_struct::fp_db_v7::JobRelativeData;
use crate::db::db_struct::version_only::VersionOnly;
use crate::db::db_struct::{parse_content_with_upgrade, LatestVersionData};
use crate::extract::repo_decoration::RepoDecoration;
use crate::jenkins::build::VfpJobBuildParam;
use crate::jenkins::query::{try_get_jenkins_async_client, VfpJenkinsClient};
use crate::pretty_log::{colored_println, ThemeColor};
use base64::Engine;
use jenkins_sdk::JenkinsError;
use std::fs::File;
use std::io::{Stdout, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Default)]
pub struct DbDataProxy {
    data: LatestVersionData,

    cached_repo_decoration: OnceLock<RepoDecoration>,
}

impl From<LatestVersionData> for DbDataProxy {
    fn from(data: LatestVersionData) -> Self {
        Self {
            data,
            cached_repo_decoration: OnceLock::new(),
        }
    }
}

impl DbDataProxy {
    pub async fn try_get_jenkins_async_client(
        &self,
        stdout: &mut Stdout,
        show_client_type: bool,
    ) -> Result<VfpJenkinsClient, JenkinsError> {
        let client = try_get_jenkins_async_client(
            self.get_jenkins_url(),
            self.get_jenkins_username(),
            &self.get_jenkins_pwd(),
            self.get_jenkins_api_token(),
        )
        .await;

        if show_client_type {
            if let Ok(ref client) = client {
                match client {
                    VfpJenkinsClient::ApiTokenClient(_) => {
                        colored_println(stdout, ThemeColor::Second, LOGIN_SUCCESS_BY_API_TOKEN)
                    }
                    VfpJenkinsClient::PwdClient(_) => {
                        colored_println(stdout, ThemeColor::Second, LOGIN_SUCCESS_BY_PWD)
                    }
                }
            }
        }

        client
    }

    pub fn get_interest_job_name(&self) -> Option<&str> {
        self.try_get_job_relative_data(None)
            .map(|data| data.job_name.as_ref())
    }

    pub fn insert_job_name(&mut self, val: &str) -> &mut Self {
        self.try_get_job_relative_data_mut(Some(val));
        self
    }

    pub fn get_from_path(path: &Path) -> Option<Self> {
        match VersionOnly::get_state_from_path(path) {
            DBStatus::Exist(version) => {
                let content = std::fs::read_to_string(path).ok()?;
                parse_content_with_upgrade(version, &content)
                    .map(|d| d.into())
                    .ok()
            }
            DBStatus::NotExist => None,
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let str = toml::to_string(&self.data).map_err(|e| e.to_string())?;
        File::create(path)
            .map_err(|e| e.to_string())?
            .write_all(str.as_bytes())
            .map_err(|e| e.to_string())
    }

    pub fn user_never_login(&self) -> bool {
        self.data.jenkins_username.is_none()
            || (self.data.jenkins_api_token.is_none() && self.data.jenkins_pwd.is_none())
    }

    pub fn get_repo_decoration(&self) -> &RepoDecoration {
        self.cached_repo_decoration.get_or_init(|| {
            let default = "";

            RepoDecoration::new(
                self.get_extract_repo()
                    .as_ref()
                    .map(|s| s.as_ref())
                    .unwrap_or(default),
                self.get_extract_locator_pattern()
                    .as_ref()
                    .map(|s| s.as_ref())
                    .unwrap_or(default),
                self.get_extract_s_locator_template()
                    .as_ref()
                    .map(|s| s.as_ref())
                    .unwrap_or(default),
                self.get_interest_job_name().unwrap_or(default),
            )
        })
    }

    pub fn has_latest_version(&self) -> bool {
        self.data.latest_remote_version.is_some()
    }

    pub fn consume_update_status(&mut self) {
        self.data.latest_remote_version = None;
    }

    //region getter & setter
    fn try_get_job_relative_data(&self, job_name: Option<&str>) -> Option<&JobRelativeData> {
        if let Some(name) = job_name {
            self.data
                .job_relative_data_arr
                .iter()
                .find(|data| data.job_name == name)
        } else {
            self.data.job_relative_data_arr.first()
        }
    }

    fn try_get_job_relative_data_mut(
        &mut self,
        job_name: Option<&str>,
    ) -> Option<&mut JobRelativeData> {
        if let Some(name) = job_name {
            let index = self
                .data
                .job_relative_data_arr
                .iter()
                .position(|data| data.job_name == name);

            if let Some(idx) = index {
                bring_element_to_first(&mut self.data.job_relative_data_arr, idx);
            } else {
                self.data.job_relative_data_arr.insert(
                    0,
                    JobRelativeData {
                        job_name: name.to_owned(),
                        last_inner_version: None,
                        last_player_count: None,
                        blast_path: None,
                        jenkins_build_params: None,
                    },
                );
            }
        }

        self.data.job_relative_data_arr.first_mut()
    }

    pub fn get_last_inner_version(&self, job_name: &str) -> Option<u32> {
        self.try_get_job_relative_data(Some(job_name))
            .and_then(|data| data.last_inner_version)
    }

    pub fn set_last_inner_version(&mut self, job_name: &str, val: Option<u32>) -> &mut Self {
        if let Some(data) = self.try_get_job_relative_data_mut(Some(job_name)) {
            data.last_inner_version = val
        }
        self
    }

    pub fn get_last_player_count(&self, job_name: &str) -> Option<u32> {
        self.try_get_job_relative_data(Some(job_name))
            .and_then(|data| data.last_player_count)
    }

    pub fn set_last_player_count(&mut self, job_name: &str, val: Option<u32>) -> &mut Self {
        if let Some(data) = self.try_get_job_relative_data_mut(Some(job_name)) {
            data.last_player_count = val
        }
        self
    }

    pub fn get_extract_repo(&self) -> &Option<String> {
        &self.data.extract_repo
    }

    pub fn set_extract_repo(&mut self, val: Option<String>) -> &mut Self {
        self.data.extract_repo = val;
        self
    }

    pub fn get_extract_locator_pattern(&self) -> &Option<String> {
        &self.data.extract_locator_pattern
    }

    pub fn set_extract_locator_pattern(&mut self, val: Option<String>) -> &mut Self {
        self.data.extract_locator_pattern = val;
        self
    }

    pub fn get_extract_s_locator_template(&self) -> &Option<String> {
        &self.data.extract_s_locator_template
    }

    pub fn set_extract_s_locator_template(&mut self, val: Option<String>) -> &mut Self {
        self.data.extract_s_locator_template = val;
        self
    }

    pub fn get_blast_path(&self, job_name: &str) -> Option<&PathBuf> {
        self.try_get_job_relative_data(Some(job_name))
            .and_then(|data| data.blast_path.as_ref())
    }

    pub fn set_blast_path(&mut self, job_name: &str, val: Option<PathBuf>) -> &mut Self {
        if let Some(data) = self.try_get_job_relative_data_mut(Some(job_name)) {
            data.blast_path = val
        }
        self
    }

    pub fn get_jenkins_url(&self) -> &Option<String> {
        &self.data.jenkins_url
    }

    pub fn set_jenkins_url(&mut self, val: Option<String>) -> &mut Self {
        self.data.jenkins_url = val;
        self
    }

    pub fn get_jenkins_username(&self) -> &Option<String> {
        &self.data.jenkins_username
    }

    pub fn set_jenkins_username(&mut self, val: Option<String>) -> &mut Self {
        self.data.jenkins_username = val;
        self
    }

    pub fn get_jenkins_api_token(&self) -> &Option<String> {
        &self.data.jenkins_api_token
    }

    pub fn set_jenkins_api_token(&mut self, val: Option<String>) -> &mut Self {
        self.data.jenkins_api_token = val;
        self
    }

    pub fn get_jenkins_pwd(&self) -> Option<String> {
        self.data.jenkins_pwd.clone().and_then(|v| {
            base64::prelude::BASE64_STANDARD
                .decode(v)
                .ok()
                .and_then(|v| String::from_utf8(v).ok())
        })
    }

    pub fn set_jenkins_pwd(&mut self, val: Option<String>) -> &mut Self {
        self.data.jenkins_pwd = val.map(|v| base64::prelude::BASE64_STANDARD.encode(v));
        self
    }

    pub fn get_jenkins_build_param(&self, job_name: &str) -> Option<&VfpJobBuildParam> {
        self.try_get_job_relative_data(Some(job_name))
            .and_then(|data| data.jenkins_build_params.as_ref())
    }

    pub fn get_mut_jenkins_build_param(&mut self, job_name: &str) -> Option<&mut VfpJobBuildParam> {
        self.try_get_job_relative_data_mut(Some(job_name))
            .and_then(|data| data.jenkins_build_params.as_mut())
    }

    pub fn set_jenkins_build_param(
        &mut self,
        job_name: &str,
        val: Option<VfpJobBuildParam>,
    ) -> &mut Self {
        if let Some(data) = self.try_get_job_relative_data_mut(Some(job_name)) {
            data.jenkins_build_params = val;
        }

        self
    }

    pub fn is_auto_update_enabled(&self) -> bool {
        self.data.auto_update_enabled
    }

    pub fn set_auto_update_enabled(&mut self, val: bool) -> &mut Self {
        self.data.auto_update_enabled = val;
        self
    }

    pub fn is_never_check_version(&self) -> bool {
        self.data.never_check_version
    }

    pub fn set_never_check_version(&mut self, val: bool) -> &mut Self {
        self.data.never_check_version = val;
        self
    }

    pub fn get_latest_remote_version(&self) -> Option<semver::Version> {
        self.data
            .latest_remote_version
            .to_owned()
            .and_then(|v| semver::Version::parse(&v).ok())
    }

    pub fn set_latest_remote_version(&mut self, val: Option<&str>) -> &mut Self {
        self.data.latest_remote_version = val.map(|v| v.to_owned());
        self
    }

    //endregion
}

#[cfg(test)]
mod tests {
    use crate::db::db_data_proxy::DbDataProxy;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    impl PartialEq for DbDataProxy {
        fn eq(&self, other: &Self) -> bool {
            self.data == other.data
        }
    }

    #[test]
    fn test_get_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        let content = r#"version = 7
never_check_version = false
auto_update_enabled = false

[[job_relative_data_arr]]
job_name = "test_job"
last_inner_version = 1024
last_player_count = 4
blast_path = 'C:\\Users\\LviatYi\\Desktop\\Temp'
"#;

        file.write_all(content.to_string().as_bytes()).unwrap();
        file.flush().unwrap();

        let config = DbDataProxy::get_from_path(file.path());

        assert!(config.is_some());

        let config = config.unwrap();

        let job_name = "test_job";

        assert_eq!(config.get_last_inner_version(job_name), Some(1024));
        assert_eq!(config.get_last_player_count(job_name), Some(4));
        assert_eq!(
            config.get_blast_path(job_name),
            Some(&PathBuf::from("C:\\Users\\LviatYi\\Desktop\\Temp"))
        );

        assert!(config.get_extract_locator_pattern().is_none());
    }

    #[test]
    fn test_get_file_not_exist() {
        let config = DbDataProxy::get_from_path(Path::new("Z:\\NOT_EXIST"));

        assert!(config.is_none());
    }
}
