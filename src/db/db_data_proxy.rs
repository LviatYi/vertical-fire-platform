use crate::constant::log::{LOGIN_SUCCESS_BY_API_TOKEN, LOGIN_SUCCESS_BY_PWD};
use crate::db::db_struct::db_status::DBStatus;
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
                self.get_interest_job_name()
                    .as_ref()
                    .map(|s| s.as_ref())
                    .unwrap_or(default),
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
    pub fn get_last_inner_version(&self) -> &Option<u32> {
        &self.data.last_inner_version
    }

    pub fn set_last_inner_version(&mut self, val: Option<u32>) -> &mut Self {
        self.data.last_inner_version = val;
        self
    }

    pub fn get_last_player_count(&self) -> &Option<u32> {
        &self.data.last_player_count
    }

    pub fn set_last_player_count(&mut self, val: Option<u32>) -> &mut Self {
        self.data.last_player_count = val;
        self
    }

    pub fn get_interest_job_name(&self) -> &Option<String> {
        &self.data.interest_job_name
    }

    pub fn set_interest_job_name(&mut self, val: Option<String>) -> &mut Self {
        self.data.interest_job_name = val;
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

    pub fn get_blast_path(&self) -> &Option<PathBuf> {
        &self.data.blast_path
    }

    pub fn set_blast_path(&mut self, val: Option<PathBuf>) -> &mut Self {
        self.data.blast_path = val;
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

    pub fn get_jenkins_build_param(&self) -> &Option<VfpJobBuildParam> {
        &self.data.jenkins_build_params
    }

    pub fn get_mut_jenkins_build_param(&mut self) -> Option<&mut VfpJobBuildParam> {
        self.data.jenkins_build_params.as_mut()
    }

    pub fn set_jenkins_build_param(&mut self, val: Option<VfpJobBuildParam>) -> &mut Self {
        self.data.jenkins_build_params = val;
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
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_old_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        let content = r#"ci = 2025
c = 9
d = 'C:\Users\LviatYi\Desktop\Temp'
"#;

        file.write_all(content.to_string().as_bytes()).unwrap();
        file.flush().unwrap();

        let config = DbDataProxy::get_from_path(file.path());

        assert!(config.is_some());

        let config = config.unwrap();

        assert_eq!(*config.get_last_inner_version(), Some(2025));
        assert_eq!(*config.get_last_player_count(), Some(9));
        assert_eq!(
            *config.get_blast_path(),
            Some(PathBuf::from("C:\\Users\\LviatYi\\Desktop\\Temp"))
        );

        assert!(config.get_extract_locator_pattern().is_none());
    }

    #[test]
    fn test_get_file_not_exist() {
        let config = DbDataProxy::get_from_path(Path::new("Z:\\NOT_EXIST"));

        assert!(config.is_none());
    }

    #[test]
    fn test_get_file_version_2() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        let content = r#"version = 2
last_inner_version = 2025
last_player_count = 9
blast_path = "C:\\Users\\LviatYi\\Desktop\\Temp"
"#;

        file.write_all(content.to_string().as_bytes()).unwrap();
        file.flush().unwrap();

        let config = DbDataProxy::get_from_path(file.path());

        assert!(config.is_some());

        let config = config.unwrap();
        assert_eq!(*config.get_last_inner_version(), Some(2025));
        assert_eq!(*config.get_last_player_count(), Some(9));
        assert_eq!(
            *config.get_blast_path(),
            Some(PathBuf::from("C:\\Users\\LviatYi\\Desktop\\Temp"))
        );
        assert_eq!(*config.get_extract_locator_pattern(), None);
    }
}
