use crate::constant::log::*;
use crate::constant::util::get_hidden_sensitive_string;
use crate::pretty_log::{colored_println, ThemeColor};
use crate::LoginMethod;
use formatx::formatx;
use inquire::InquireError;
use jenkins_sdk::JenkinsError;
use std::fmt::Display;
use std::io::Write;
use std::ops::Add;

#[derive(Debug)]
pub enum VfpFrontError {
    Quit,
    Custom(String),
    InquireError(InquireError),
    JenkinsLoginError {
        method: LoginMethod,
        url: String,
        username: String,
        key: String,
        #[allow(dead_code)]
        e: JenkinsError,
    },
    JenkinsClientInvalid,
    JenkinsTimeout,
    MissingParam(String),
    RunTaskBuildFailed {
        build_number: u32,
        job_name: String,
        run_url: String,
        log: String,
    },
    VersionParseFailed(String),
    SelfUpdateError(self_update::errors::Error),
    JobConfigParseError {
        e: String,
        content: String,
    },
    OpenDbFailed(String),
}

impl From<InquireError> for VfpFrontError {
    fn from(value: InquireError) -> Self {
        match value {
            InquireError::OperationCanceled => VfpFrontError::Quit,
            InquireError::OperationInterrupted => VfpFrontError::Quit,
            _ => VfpFrontError::InquireError(value),
        }
    }
}

impl From<JenkinsError> for VfpFrontError {
    fn from(_: JenkinsError) -> Self {
        VfpFrontError::JenkinsClientInvalid
    }
}

impl From<self_update::errors::Error> for VfpFrontError {
    fn from(value: self_update::errors::Error) -> Self {
        VfpFrontError::SelfUpdateError(value)
    }
}

impl Display for VfpFrontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            VfpFrontError::Quit => ERR_USER_FORCE_QUIT.to_string(),
            VfpFrontError::Custom(msg) => msg.clone(),
            VfpFrontError::InquireError(err) => err.to_string(),
            VfpFrontError::JenkinsLoginError {
                method,
                url,
                username,
                key,
                e: _,
            } => {
                let msg = match method {
                    LoginMethod::ApiToken => {
                        formatx!(
                            ERR_JENKINS_CLIENT_INVALID_MAY_BE_API_TOKEN_INVALID,
                            url,
                            username,
                            get_hidden_sensitive_string(
                                key,
                                crate::constant::util::SensitiveMode::Normal(4)
                            )
                        )
                    }
                    LoginMethod::Pwd => {
                        formatx!(
                            ERR_JENKINS_CLIENT_INVALID_MAY_BE_PWD_INVALID,
                            url,
                            username,
                            get_hidden_sensitive_string(
                                key,
                                crate::constant::util::SensitiveMode::Full
                            )
                        )
                    }
                }
                .unwrap_or_default();

                ERR_JENKINS_CLIENT_INVALID_SIMPLE
                    .to_string()
                    .add(msg.as_str())
            }
            VfpFrontError::JenkinsClientInvalid => ERR_JENKINS_CLIENT_INVALID.to_string(),
            VfpFrontError::JenkinsTimeout => ERR_JENKINS_TIMEOUT.to_string(),
            VfpFrontError::MissingParam(param) => {
                formatx!(ERR_NEED_PARAM, param).unwrap_or_default()
            }
            VfpFrontError::RunTaskBuildFailed {
                build_number,
                job_name,
                ..
            } => formatx!(WATCHING_RUN_TASK_FAILURE, build_number, job_name).unwrap_or_default(),
            VfpFrontError::VersionParseFailed(ver) => {
                formatx!(ERR_VERSION_PARSE_FAILED, ver).unwrap_or_default()
            }
            VfpFrontError::SelfUpdateError(e) => e.to_string(),
            VfpFrontError::JobConfigParseError { e, .. } => {
                formatx!(ERR_QUERY_JOB_CONFIG, e).unwrap_or_default()
            }
            VfpFrontError::OpenDbFailed(path) => {
                formatx!(ERR_OPEN_FILE_FAILED, path).unwrap_or_default()
            }
        };
        write!(f, "{}", str)
    }
}

impl VfpFrontError {
    pub fn colored_println<W: Write>(&self, stdout: &mut W) {
        match self {
            VfpFrontError::Quit => {
                colored_println(stdout, ThemeColor::Second, self.to_string().as_str());
            }
            VfpFrontError::JobConfigParseError { content, .. } => {
                colored_println(stdout, ThemeColor::Error, self.to_string().as_str());
                colored_println(
                    stdout,
                    ThemeColor::Second,
                    formatx!(HINT_JOB_CONFIG_CONTENT, content)
                        .unwrap_or_default()
                        .as_str(),
                );
            }
            VfpFrontError::RunTaskBuildFailed { log, run_url, .. } => {
                colored_println(stdout, ThemeColor::Error, self.to_string().as_str());
                colored_println(stdout, ThemeColor::Main, log.as_str());
                colored_println(
                    stdout,
                    ThemeColor::Warn,
                    formatx!(RUN_TASK_CONSOLE_OUTPUT_URL, run_url)
                        .unwrap_or_default()
                        .as_str(),
                );
            }
            _ => {
                colored_println(stdout, ThemeColor::Error, self.to_string().as_str());
            }
        }
    }
}
