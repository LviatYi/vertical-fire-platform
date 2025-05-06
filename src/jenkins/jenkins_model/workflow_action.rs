use crate::jenkins::jenkins_model::cause::Cause;
use crate::jenkins::jenkins_model::parameters_action::ParametersAction;
use crate::jenkins::jenkins_model::workflow_build_metadata::WorkflowBuildMetadata;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "_class")]
pub enum WorkflowAction {
    #[serde(rename = "hudson.model.ParametersAction")]
    Parameters { parameters: Vec<ParametersAction> },
    #[serde(rename = "hudson.model.CauseAction")]
    Causes { causes: Vec<Cause> },
    #[serde(rename = "jenkins.metrics.impl.TimeInQueueAction")]
    TimeInQueue,
    #[serde(rename = "org.jenkinsci.plugins.buildmetadata.plugin.action.BuildMetadataAction")]
    BuildMetadata {
        #[serde(rename = "buildMetadata")]
        build_metadata: Vec<WorkflowBuildMetadata> 
    },
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MaybeWorkflowAction {
    WorkflowAction(WorkflowAction),
    Unknown(#[allow(dead_code)] serde_json::Value),
}
