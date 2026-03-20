pub const COUNT: u32 = 4;
pub const RUN_COUNT: u32 = 1;
pub const WATCH_INTERVAL: u64 = 10;
pub const WATCH_QUERY_BUILDS_COUNT: u32 = 10;
pub const OLDEST_SUPPORT_UPDATE_VERSION: &str = "1.5.0";
pub const MAX_JOB_RELATIVE_DATA_COUNT: usize = 8;
pub const USER_QUERY_JENKINS_BUILD_COUNT: usize = 50;
pub const JENKINS_QUERY_CONCURRENCY_COUNT: usize = 20;
pub const RELEASE_URL: &str = "https://github.com/LviatYi/vertical-fire-platform/releases/tag/v";

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const DEFAULT_RUNTIME_CONFIG_PATH: &str = "fp-config.toml";
const BUILD_DEFAULT_RECOMMEND_JOB_NAMES: &str =
    if let Some(v) = option_env!("RECOMMEND_JOB_NAMES") { v } else { "" };
const BUILD_DEFAULT_REPO_TEMPLATE: &str =
    if let Some(v) = option_env!("REPO_TEMPLATE") { v } else { "" };
const BUILD_DEFAULT_LOCATOR_PATTERN: &str =
    if let Some(v) = option_env!("LOCATOR_PATTERN") { v } else { "" };
const BUILD_DEFAULT_LOCATOR_TEMPLATE: &str =
    if let Some(v) = option_env!("LOCATOR_TEMPLATE") { v } else { "" };
const BUILD_DEFAULT_MENDING_FILE_PATH: &str =
    if let Some(v) = option_env!("MENDING_FILE_PATH") { v } else { "" };
const BUILD_DEFAULT_PT_RELATIVE_PATH: &str =
    if let Some(v) = option_env!("PT_RELATIVE_PATH") { v } else { "" };
const BUILD_DEFAULT_PACKAGE_FILE_STEM: &str =
    if let Some(v) = option_env!("PACKAGE_FILE_STEM") { v } else { "" };
const BUILD_DEFAULT_EXE_FILE_NAME: &str =
    if let Some(v) = option_env!("EXE_FILE_NAME") { v } else { "" };
const BUILD_DEFAULT_CHECK_EXE_FILE_NAME: &str =
    if let Some(v) = option_env!("CHECK_EXE_FILE_NAME") { v } else { "" };
const BUILD_DEFAULT_JENKINS_URL: &str =
    if let Some(v) = option_env!("JENKINS_URL") { v } else { "" };
const BUILD_DEFAULT_QUERY_TOKEN_GITHUB: &str =
    if let Some(v) = option_env!("QUERY_TOKEN_GITHUB") { v } else { "" };

#[derive(Debug, Default, Deserialize)]
struct RuntimeConfigFile {
    recommend_job_names: Option<Vec<String>>,
    repo_template: Option<String>,
    locator_pattern: Option<String>,
    locator_template: Option<String>,
    mending_file_path: Option<String>,
    pt_relative_path: Option<String>,
    package_file_stem: Option<String>,
    exe_file_name: Option<String>,
    check_exe_file_name: Option<String>,
    jenkins_url: Option<String>,
    query_token_github: Option<String>,
}

#[derive(Debug, Default)]
pub struct RuntimeConfig {
    pub recommend_job_names: Vec<String>,
    pub repo_template: String,
    pub locator_pattern: String,
    pub locator_template: String,
    pub mending_file_path: String,
    pub pt_relative_path: String,
    pub package_file_stem: String,
    pub exe_file_name: String,
    pub check_exe_file_name: String,
    pub jenkins_url: String,
    pub query_token_github: String,
}

static RUNTIME_CONFIG: LazyLock<RuntimeConfig> = LazyLock::new(load_runtime_config);

pub fn runtime() -> &'static RuntimeConfig {
    &RUNTIME_CONFIG
}

fn load_runtime_config() -> RuntimeConfig {
    let file_config = load_runtime_config_file();

    RuntimeConfig {
        recommend_job_names: resolve_recommend_job_names(
            file_config.recommend_job_names,
            BUILD_DEFAULT_RECOMMEND_JOB_NAMES,
        ),
        repo_template: resolve_string(file_config.repo_template, BUILD_DEFAULT_REPO_TEMPLATE),
        locator_pattern: resolve_string(file_config.locator_pattern, BUILD_DEFAULT_LOCATOR_PATTERN),
        locator_template: resolve_string(file_config.locator_template, BUILD_DEFAULT_LOCATOR_TEMPLATE),
        mending_file_path: resolve_string(
            file_config.mending_file_path,
            BUILD_DEFAULT_MENDING_FILE_PATH,
        ),
        pt_relative_path: resolve_string(file_config.pt_relative_path, BUILD_DEFAULT_PT_RELATIVE_PATH),
        package_file_stem: resolve_string(
            file_config.package_file_stem,
            BUILD_DEFAULT_PACKAGE_FILE_STEM,
        ),
        exe_file_name: resolve_string(file_config.exe_file_name, BUILD_DEFAULT_EXE_FILE_NAME),
        check_exe_file_name: resolve_string(
            file_config.check_exe_file_name,
            BUILD_DEFAULT_CHECK_EXE_FILE_NAME,
        ),
        jenkins_url: resolve_string(file_config.jenkins_url, BUILD_DEFAULT_JENKINS_URL),
        query_token_github: resolve_string(
            file_config.query_token_github,
            BUILD_DEFAULT_QUERY_TOKEN_GITHUB,
        ),
    }
}

fn load_runtime_config_file() -> RuntimeConfigFile {
    let raw = match fs::read_to_string(resolve_runtime_config_path()) {
        Ok(raw) => raw,
        Err(_) => return RuntimeConfigFile::default(),
    };

    toml::from_str::<RuntimeConfigFile>(&raw).unwrap_or_default()
}

fn resolve_runtime_config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .as_deref()
        .and_then(Path::parent)
        .map(runtime_config_path_from_root)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_RUNTIME_CONFIG_PATH))
}

fn runtime_config_path_from_root(root: &Path) -> PathBuf {
    root.join(DEFAULT_RUNTIME_CONFIG_PATH)
}

fn resolve_recommend_job_names(from_file: Option<Vec<String>>, from_build: &str) -> Vec<String> {
    let from_file = from_file
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();

    if !from_file.is_empty() {
        return from_file;
    }

    split_trimmed(from_build)
}

fn resolve_string(from_file: Option<String>, from_build: &str) -> String {
    let from_file = from_file.unwrap_or_default().trim().to_string();
    if !from_file.is_empty() {
        return from_file;
    }

    from_build.trim().to_string()
}

fn split_trimmed(value: &str) -> Vec<String> {
    value
        .split([',', ';'])
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config_path_from_root() {
        let root = Path::new(r"C:\tools\fp");

        assert_eq!(
            runtime_config_path_from_root(root),
            PathBuf::from(r"C:\tools\fp\fp-config.toml")
        );
    }
}
