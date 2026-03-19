use crate::jenkins::build::{ToVfpJobBuildParam, VfpJobBuildParam};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct JobDefinitionJson {
    property: Vec<DefinitionProperty>,
}

impl JobDefinitionJson {
    pub fn get_parameters(&self) -> Vec<&ParameterDefinition> {
        self.property
            .iter()
            .map(|item| match item {
                DefinitionProperty::ParametersDefinitionProperty {
                    parameter_definitions,
                } => Some(parameter_definitions),
                DefinitionProperty::Other => None,
            })
            .filter(Option::is_some)
            .flat_map(Option::unwrap)
            .collect()
    }
}

impl ToVfpJobBuildParam for JobDefinitionJson {
    fn to_vfp_job_build_param(&self) -> VfpJobBuildParam {
        let params: HashMap<String, Value> = self
            .get_parameters()
            .into_iter()
            .filter(|item| !matches!(item, ParameterDefinition::Unknown))
            .map(|param| {
                let necessary = param.is_necessary();
                match param {
                    ParameterDefinition::String {
                        name,
                        default_value,
                        ..
                    } => (
                        name.clone(),
                        Value::String(
                            default_value
                                .as_ref()
                                .map(|item| item.value.clone())
                                .unwrap_or_default(),
                        ),
                    ),
                    ParameterDefinition::Bool {
                        name,
                        default_value,
                        ..
                    } => (
                        name.clone(),
                        Value::Bool(
                            default_value
                                .as_ref()
                                .map(|item| item.value)
                                .unwrap_or_default(),
                        ),
                    ),
                    ParameterDefinition::Choice {
                        name,
                        choices,
                        default_value,
                        ..
                    } => {
                        let default_choice = if necessary {
                            default_value
                                .as_ref()
                                .map(|item| &item.value)
                                .or_else(|| choices.iter().find(|item| !item.is_empty()))
                        } else {
                            default_value
                                .as_ref()
                                .map(|item| &item.value)
                                .or_else(|| choices.iter().next())
                        };
                        (
                            name.clone(),
                            Value::String(default_choice.cloned().unwrap_or_default()),
                        )
                    }
                    ParameterDefinition::Unknown => {
                        unreachable!()
                    }
                }
            })
            .collect();
        VfpJobBuildParam::new_with_override_recommend_param(params)
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "_class")]
enum DefinitionProperty {
    #[serde(rename = "hudson.model.ParametersDefinitionProperty")]
    ParametersDefinitionProperty {
        #[serde(default, rename = "parameterDefinitions")]
        parameter_definitions: Vec<ParameterDefinition>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "_class")]
pub enum ParameterDefinition {
    #[serde(rename = "hudson.model.StringParameterDefinition")]
    String {
        name: String,

        #[serde(default)]
        description: Option<String>,

        #[serde(default, rename = "defaultParameterValue")]
        default_value: Option<Val<String>>,
    },

    #[serde(rename = "hudson.model.BooleanParameterDefinition")]
    Bool {
        name: String,

        #[serde(default)]
        description: Option<String>,

        #[serde(default, rename = "defaultParameterValue")]
        default_value: Option<Val<bool>>,
    },

    #[serde(rename = "hudson.model.ChoiceParameterDefinition")]
    Choice {
        name: String,

        #[serde(default)]
        description: Option<String>,

        #[serde(default, rename = "defaultParameterValue")]
        default_value: Option<Val<String>>,

        #[serde(default)]
        choices: Vec<String>,
    },

    #[serde(other)]
    Unknown,
}

impl ParameterDefinition {
    pub fn is_necessary(&self) -> bool {
        let desc = match self {
            ParameterDefinition::String { description, .. } => description,
            ParameterDefinition::Bool { description, .. } => description,
            ParameterDefinition::Choice { description, .. } => description,
            ParameterDefinition::Unknown => return false,
        };

        desc.as_ref().is_some_and(|desc| desc.contains("color:red"))
    }
}

#[derive(Deserialize, Debug)]
pub struct Val<T>
where
    T: Debug,
{
    value: T,
}

impl<T> Deref for Val<T>
where
    T: Debug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_job_definition_json() {
        let content = r#"{
  "_class": "org.jenkinsci.plugins.workflow.job.WorkflowJob",
  "property": [
    {
      "_class": "hudson.model.ParametersDefinitionProperty",
      "parameterDefinitions": [
        {
          "_class": "hudson.model.StringParameterDefinition",
          "defaultParameterValue": {
            "_class": "hudson.model.StringParameterValue",
            "value": ""
          },
          "description": null,
          "name": "Changelist",
          "type": "StringParameterDefinition"
        },
        {
          "_class": "hudson.model.StringParameterDefinition",
          "defaultParameterValue": {
            "_class": "hudson.model.StringParameterValue",
            "value": ""
          },
          "description": "Give a Custom Server to connect",
          "name": "CustomServer",
          "type": "StringParameterDefinition"
        },
        {
          "_class": "hudson.model.ChoiceParameterDefinition",
          "defaultParameterValue": {
            "_class": "hudson.model.StringParameterValue",
            "value": ""
          },
          "description": "\u003Cspan style='color:red'\u003E Please select TestType before build.\u003C/span\u003E",
          "name": "TestType",
          "type": "ChoiceParameterDefinition",
          "choices": [
            "",
            "Some",
            "Other"
          ]
        },
        {
          "_class": "hudson.model.BooleanParameterDefinition",
          "defaultParameterValue": {
            "_class": "hudson.model.BooleanParameterValue",
            "value": true
          },
          "description": "",
          "name": "SetCustomServer",
          "type": "BooleanParameterDefinition"
        },
        {
          "_class": "hudson.model.BooleanParameterDefinition",
          "defaultParameterValue": {
            "_class": "hudson.model.BooleanParameterValue",
            "value": true
          },
          "description": "",
          "name": "Compile",
          "type": "BooleanParameterDefinition"
        }
      ]
    },
    {
      "_class": "com.sonyericsson.jenkins.plugins.bfa.model.ScannerJobProperty"
    },
    {
      "_class": "jenkins.model.BuildDiscarderProperty"
    }
  ]
}"#;
        let _: JobDefinitionJson = serde_json::from_str(content).unwrap();
    }
}
