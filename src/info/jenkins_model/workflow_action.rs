use crate::info::jenkins_model::cause::Cause;
use crate::info::jenkins_model::parameters_action::ParametersAction;
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
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MaybeWorkflowAction {
    WorkflowAction(WorkflowAction),
    Unknown(serde_json::Value),
}
