use crate::info::jenkins_model::workflow_build::WorkflowBuild;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WorkflowBuilds {
    pub builds: Vec<WorkflowBuild>,
}
