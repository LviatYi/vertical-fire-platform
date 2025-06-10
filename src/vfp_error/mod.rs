use crate::constant::log::*;
use crate::constant::util::get_hidden_sensitive_string;
use crate::pretty_log::{colored_println, ThemeColor};
use crate::LoginMethod;
use formatx::formatx;
use inquire::InquireError;
use jenkins_sdk::JenkinsError;
use std::fmt::Display;
use std::io::Stdout;
use std::ops::Add;

#[derive(Debug)]
pub enum VfpError {
    Custom(String),
    InquireError(InquireError),
    JenkinsLoginError {
        method: LoginMethod,
        url: String,
        username: String,
        key: String,
        e: JenkinsError,
    },
    JenkinsClientInvalid,
    MissingParam(String),
    EmptyRepo,
    RunTaskBuildFailed {
        build_number: u32,
        job_name: String,
        run_url: String,
        log: String,
    },
}

impl From<InquireError> for VfpError {
    fn from(value: InquireError) -> Self {
        VfpError::InquireError(value)
    }
}

impl Display for VfpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            VfpError::Custom(msg) => msg.clone(),
            VfpError::InquireError(err) => err.to_string(),
            VfpError::JenkinsLoginError {
                method,
                url,
                username,
                key,
                e,
            } => {
                let msg = match method {
                    LoginMethod::ApiToken => {
                        formatx!(
                            ERR_JENKINS_CLIENT_INVALID_MAY_BE_PWD_INVALID,
                            url,
                            username,
                            get_hidden_sensitive_string(
                                key,
                                crate::constant::util::SensitiveMode::Normal(4)
                            ),
                            e.to_string()
                        )
                    }
                    LoginMethod::Pwd => {
                        formatx!(
                            ERR_JENKINS_CLIENT_INVALID_MAY_BE_API_TOKEN_INVALID,
                            url,
                            username,
                            get_hidden_sensitive_string(
                                key,
                                crate::constant::util::SensitiveMode::Full
                            ),
                            e.to_string()
                        )
                    }
                }
                .unwrap_or_default();

                ERR_JENKINS_CLIENT_INVALID_SIMPLE
                    .to_string()
                    .add(msg.as_str())
            }
            VfpError::JenkinsClientInvalid => ERR_JENKINS_CLIENT_INVALID.to_string(),
            VfpError::MissingParam(param) => formatx!(ERR_NEED_PARAM, param).unwrap_or_default(),
            VfpError::EmptyRepo => ERR_EMPTY_REPO.to_string(),
            VfpError::RunTaskBuildFailed {
                build_number,
                job_name,
                ..
            } => formatx!(WATCHING_RUN_TASK_FAILURE, build_number, job_name).unwrap_or_default(),
        };
        write!(f, "{}", str)
    }
}

impl VfpError {
    pub fn colored_println(&self, stdout: &mut Stdout) {
        colored_println(stdout, ThemeColor::Error, self.to_string().as_str());
    }
}
