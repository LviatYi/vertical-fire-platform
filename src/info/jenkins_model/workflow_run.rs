use crate::info::jenkins_model::cause::Cause;
use crate::info::jenkins_model::run_status::RunStatus;
use crate::info::jenkins_model::workflow_action::{MaybeWorkflowAction, WorkflowAction};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WorkflowRun {
    pub number: u32,

    pub actions: Vec<MaybeWorkflowAction>,

    pub result: RunStatus,
}

impl WorkflowRun {
    pub fn is_mine(&self, my_user_id: &str) -> bool {
        for action in &self.actions {
            if let MaybeWorkflowAction::WorkflowAction(WorkflowAction::Causes { causes }) = action {
                if causes.iter().any(|cause| {
                    matches!(cause,
                        Cause::UserIdCause(user_id_cause)
                        if user_id_cause.is_mine(my_user_id)
                    )
                }) {
                    return true;
                }
            }
        }

        false
    }
}
