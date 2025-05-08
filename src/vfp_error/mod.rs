use crate::constant::log::*;
use crate::constant::util::get_hidden_sensitive_string;
use crate::LoginMethod;
use formatx::formatx;
use inquire::InquireError;
use jenkins_sdk::JenkinsError;
use std::fmt::Display;
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
        };
        write!(f, "{}", str)
    }
}
