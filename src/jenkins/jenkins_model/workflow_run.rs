use crate::jenkins::jenkins_model::cause::Cause;
use crate::jenkins::jenkins_model::run_status::RunStatus;
use crate::jenkins::jenkins_model::workflow_action::{MaybeWorkflowAction, WorkflowAction};
use crate::jenkins::jenkins_model::workflow_build_metadata::WorkflowBuildMetadata::StringBuildMetadata;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WorkflowRun {
    pub number: u32,

    pub actions: Vec<MaybeWorkflowAction>,

    #[serde(deserialize_with = "deserialize_run_status")]
    pub result: RunStatus,
}

fn deserialize_run_status<'de, D>(deserializer: D) -> Result<RunStatus, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    let opt = Option::<RunStatus>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
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

    pub fn get_change_list_in_build_meta_data(&self) -> Option<u32> {
        for action in &self.actions {
            if let MaybeWorkflowAction::WorkflowAction(WorkflowAction::BuildMetadata {
                build_metadata: metadata_list,
            }) = action
            {
                for data in metadata_list {
                    match data {
                        StringBuildMetadata(data) => {
                            if data.name == "P4CL" {
                                return data.value.parse::<u32>().ok();
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_workflow_run() {
        //region content
        let content = r###"{
  "_class": "org.jenkinsci.plugins.workflow.job.WorkflowRun",
  "actions": [
    {
      "_class": "hudson.model.ParametersAction",
      "parameters": [
        {
          "_class": "hudson.model.StringParameterValue",
          "name": "Changelist",
          "value": ""
        },
        {
          "_class": "hudson.model.StringParameterValue",
          "name": "ShelvedChange",
          "value": ""
        },
        {
          "_class": "hudson.model.BooleanParameterValue",
          "name": "Publish_Blast",
          "value": true
        }
      ]
    },
    {
      "_class": "hudson.model.CauseAction",
      "causes": [
        {
          "_class": "hudson.model.Cause$UserIdCause",
          "shortDescription": "Started by user LviatYi",
          "userId": "LviatYi@foxmail.com",
          "userName": "LviatYi"
        }
      ]
    },
    {
      "_class": "jenkins.metrics.impl.TimeInQueueAction",
      "blockedDurationMillis": 0,
      "blockedTimeMillis": 0,
      "buildableDurationMillis": 0,
      "buildableTimeMillis": 4,
      "buildingDurationMillis": 519510,
      "executingTimeMillis": 517166,
      "executorUtilization": 1,
      "subTaskCount": 2,
      "waitingDurationMillis": 0,
      "waitingTimeMillis": 1
    },
    {
      "_class": "org.jenkinsci.plugins.buildmetadata.plugin.action.BuildMetadataAction",
      "buildMetadata": [
        {
          "_class": "org.jenkinsci.plugins.buildmetadata.plugin.StringBuildMetadata",
          "description": null,
          "name": "P4CL",
          "stringValue": "532097",
          "type": "StringBuildMetadata"
        },
        {
          "_class": "org.jenkinsci.plugins.buildmetadata.plugin.StringBuildMetadata",
          "description": null,
          "name": "P4ShelvedCL",
          "stringValue": "",
          "type": "StringBuildMetadata"
        },
        {
          "_class": "org.jenkinsci.plugins.buildmetadata.plugin.StringBuildMetadata",
          "description": null,
          "name": "NodeName",
          "stringValue": "eamc-dre-wb036",
          "type": "StringBuildMetadata"
        },
        {
          "_class": "org.jenkinsci.plugins.buildmetadata.plugin.StringBuildMetadata",
          "description": null,
          "name": "CustomServer",
          "stringValue": "",
          "type": "StringBuildMetadata"
        },
        {
          "_class": "org.jenkinsci.plugins.buildmetadata.plugin.StringBuildMetadata",
          "description": null,
          "name": "HygeiaServer",
          "stringValue": "",
          "type": "StringBuildMetadata"
        },
        {
          "_class": "org.jenkinsci.plugins.buildmetadata.plugin.StringBuildMetadata",
          "description": null,
          "name": "FileShare",
          "stringValue": "\\\\eamc-sha-filer1.eamobile.ad.ea.com\\DREBuilds\\fifamobile\\builds\\eamc-fcmobile\\FCM.EAMC.FCM-Stage.Client.Blast.Opt\\851-CL.532097\\pc64-vc-opt",
          "type": "StringBuildMetadata"
        },
        {
          "_class": "org.jenkinsci.plugins.buildmetadata.plugin.StringBuildMetadata",
          "description": null,
          "name": "OTA",
          "stringValue": "https://eamc-ota.ad.ea.com/#/Home/FIFAMobile/Builds/eamc-fcmobile/FCM.EAMC.FCM-Stage.Client.Blast.Opt/851-CL.532097/pc64-vc-opt/",
          "type": "StringBuildMetadata"
        }
      ]
    },
    {
      "_class": "org.jenkinsci.plugins.workflow.job.views.FlowGraphAction"
    }
  ],
  "building": false,
  "fullDisplayName": "FCM.EAMC.FCM-Stage.Client.Blast.Opt #851-CL532097-CustomServer",
  "id": "851",
  "number": 851,
  "result": "SUCCESS",
  "timestamp": 1746759736434,
  "url": "https://eamc-fcmobile.eamobile.ad.ea.com/job/FCM.EAMC.FCM-Stage.Client.Blast.Opt/851/",
  "inProgress": false
}
"###;
        //endregion

        match serde_json::from_str::<WorkflowRun>(content) {
            Ok(workflow_run) => {
                println!("{:#?}", workflow_run);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
