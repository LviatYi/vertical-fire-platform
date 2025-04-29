use crate::jenkins::jenkins_endpoint::job_config::JobConfig;
use crate::jenkins::jenkins_model::job_config::{FlowDefinition, ParameterDefinition};
use crate::jenkins::jenkins_model::shelves::Shelves;
use crate::jenkins::query::VfpJenkinsClient;
use jenkins_sdk::{JenkinsError, TriggerBuild};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct VfpJobBuildParam {
    pub params: HashMap<String, Value>,
}

impl VfpJobBuildParam {
    pub const PARAM_NAME_CHANGE_LIST: &'static str = "Changelist";

    pub const PARAM_NAME_SHELVED_CHANGE: &'static str = "ShelvedChange";

    pub const PARAM_NAME_ENABLE_CONTENT_PREVIEW: &'static str = "EnableContentPreview";

    pub const PARAM_NAME_SIMULATE_ANDROID_GUEST_LOGIN: &'static str = "SimulateAndroidGuestLogin";

    pub fn override_recommend_param(&mut self) -> &mut Self {
        self.set_enable_content_preview(true);
        self.set_simulate_android_guest_login(true);
        self
    }

    pub fn to_json_value(&self) -> Value {
        serde_json::to_value(&self.params).unwrap_or_default()
    }

    pub fn set_change_list(&mut self, val: u32) -> &mut Self {
        self.params.insert(
            Self::PARAM_NAME_CHANGE_LIST.to_string(),
            Value::String(val.to_string()),
        );
        self
    }

    pub fn get_change_list(&self) -> Option<u32> {
        self.params
            .get(Self::PARAM_NAME_CHANGE_LIST)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok())
    }

    pub fn set_shelve_changes(&mut self, val:Shelves) -> &mut Self {
        self.params.insert(
            Self::PARAM_NAME_SHELVED_CHANGE.to_string(),
            Value::String(
                val.0.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        );
        self
    }

    pub fn get_shelve_changes(&self) -> Option<Shelves> {
        self.params
            .get(Self::PARAM_NAME_SHELVED_CHANGE)
            .and_then(|v| v.as_str())
            .and_then(|s| Shelves::from_str(s).ok())
    }

    pub fn set_enable_content_preview(&mut self, val: bool) -> &mut Self {
        self.params.insert(
            Self::PARAM_NAME_ENABLE_CONTENT_PREVIEW.to_string(),
            Value::Bool(val),
        );
        self
    }

    pub fn set_simulate_android_guest_login(&mut self, val: bool) -> &mut Self {
        self.params.insert(
            Self::PARAM_NAME_SIMULATE_ANDROID_GUEST_LOGIN.to_string(),
            Value::Bool(val),
        );
        self
    }
}

impl From<FlowDefinition> for VfpJobBuildParam {
    fn from(value: FlowDefinition) -> Self {
        let params = value
            .get_parameters()
            .iter()
            .map(|param| match param {
                ParameterDefinition::StringParam {
                    name,
                    default_value,
                    ..
                } => (
                    name.clone(),
                    Value::String(default_value.clone().unwrap_or_default()),
                ),
                ParameterDefinition::BoolParam {
                    name,
                    default_value,
                    ..
                } => (name.clone(), Value::Bool(default_value.unwrap_or_default())),
            })
            .collect();

        let mut result = Self { params };
        result.override_recommend_param();
        result
    }
}

pub async fn query_job_config(
    client: &VfpJenkinsClient,
    job_name: &str,
) -> Result<FlowDefinition, JenkinsError> {
    let content = jenkins_sdk::AsyncRawQuery::raw_query(
        &JobConfig {
            job_name: job_name.to_string(),
        },
        client,
    )
    .await?;

    match quick_xml::de::from_str::<FlowDefinition>(&content) {
        Ok(result) => Ok(result),
        Err(e) => Err(JenkinsError::RequestError(e.to_string())),
    }
}

pub async fn request_build(
    client: &VfpJenkinsClient,
    job_name: &str,
    param: &VfpJobBuildParam,
) -> Result<(), JenkinsError> {
    match jenkins_sdk::AsyncQuery::<()>::query(
        &TriggerBuild {
            job_name,
            params: &param.to_json_value(),
        },
        client,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
