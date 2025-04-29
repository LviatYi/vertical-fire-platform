use crate::constant::log::*;
use crate::db::get_db;
use crate::interact::input_job_name;
use crate::jenkins::watch::{watch, VfpWatchError};
use crate::pretty_log::{colored_println, toast, ThemeColor};
use formatx::formatx;
use std::io::Stdout;

pub mod build;
pub mod jenkins_endpoint;
mod jenkins_model;
mod pwd_jenkins_async_client;
pub mod query;
pub mod watch;

pub async fn ci_do_watch(
    stdout: &mut Stdout,
    job_name: Option<String>,
    ci: Option<u32>,
) -> (Option<String>, Option<u32>) {
    let mut success_build_number = None;
    let mut used_job_name = None;
    let db = get_db(None);
    let client = db.try_get_jenkins_async_client(stdout, true).await;

    if let Ok(client) = client {
        let job_name = input_job_name(job_name, db.get_interest_job_name());

        if job_name.is_err() {
            println!("{}", ERR_EMPTY_REPO);
            return (used_job_name, success_build_number);
        }
        used_job_name = Some(job_name.unwrap());

        match db.get_jenkins_username() {
            Some(username) => {
                let result = watch(
                    stdout,
                    client,
                    username,
                    &used_job_name.clone().unwrap(),
                    ci,
                )
                .await;

                match result {
                    Ok(build_number) => {
                        success_build_number = Some(build_number);
                        colored_println(
                            stdout,
                            ThemeColor::Success,
                            &formatx!(
                                WATCHING_RUN_TASK_SUCCESS,
                                build_number,
                                used_job_name.as_ref().unwrap()
                            )
                            .unwrap_or_default(),
                        );

                        toast("Watch", vec![RUN_TASK_COMPLETED]);
                    }
                    Err(e) => match e {
                        VfpWatchError::JenkinsError(_) => {
                            colored_println(stdout, ThemeColor::Error, ERR_NO_IN_PROGRESS_RUN_TASK);
                        }
                        VfpWatchError::NoValidRunTask => {
                            colored_println(stdout, ThemeColor::Error, ERR_NO_VALID_RUN_TASK);
                        }
                        VfpWatchError::WatchTaskFailed(build_number, log) => {
                            colored_println(
                                stdout,
                                ThemeColor::Error,
                                &formatx!(
                                    WATCHING_RUN_TASK_FAILURE,
                                    build_number,
                                    used_job_name.as_ref().unwrap()
                                )
                                .unwrap_or_default(),
                            );
                            colored_println(stdout, ThemeColor::Main, &log);
                        }
                    },
                }
            }
            None => {
                colored_println(stdout, ThemeColor::Error, ERR_NEED_A_JENKINS_USERNAME);
            }
        }
    } else {
        colored_println(stdout, ThemeColor::Error, ERR_JENKINS_CLIENT_INVALID);
    }

    (used_job_name, success_build_number)
}

#[cfg(test)]
mod tests {
    use crate::jenkins::build::VfpJobBuildParam;
    use crate::jenkins::jenkins_model::job_config::FlowDefinition;
    use crate::jenkins::jenkins_model::workflow_run::WorkflowRun;
    use crate::jenkins::query::*;
    use crate::jenkins::watch::watch;
    use jenkins_sdk::{AsyncQuery, AsyncRawQuery, JenkinsAsyncClient, JobsInfo, TriggerBuild};
    use reqwest::header::COOKIE;
    use reqwest::Client;
    use serde_json::json;
    use std::time::Instant;
    use win_toast_notify::WinToastNotify;

    const URL: &str = "";
    const USERNAME: &str = "";
    const JOB_NAME: &str = "";
    const API_TOKEN: &str = "";
    const JENKINS_COOKIE: &str = "";
    const JENKINS_PWD: &str = "";

    #[tokio::test]
    async fn jenkins_sdk_lab() {
        let client = JenkinsAsyncClient::new(URL, USERNAME, API_TOKEN);

        let jobs: serde_json::Value = AsyncQuery::query(&JobsInfo, &client).await.unwrap();

        println!("Jobs: {:#?}", jobs);
    }

    #[tokio::test]
    async fn jenkins_build_lab() {
        let client = JenkinsAsyncClient::new(URL, USERNAME, API_TOKEN);

        let params = json!({
            "Changelist": "516882",
            "CustomServer": "",
            "HygeiaLogServer": "",
            "HygeiaServer": "",
            "ShelvedChange": "",
            "BackenDetailedProfile": false,
            "DisablePTUpdate": false,
            "EnableAudioAudition": false,
            "EnableBuildInPackageMetaScript": true,
            "EnableContentPreview": true,
            "EnableGPDebug": true,
            "EnableGameTalk": false,
            "Esports": false,
            "ForceSyncProjectBranch": false,
            "GPBECORE": false,
            "LoggingInFinal": false,
            "Pioneer": false,
            "SimulateAndroidGuestLogin": true,
            "UseICETool": false,
            "Clean": false,
            "SetCustomServer": true,
            "GenSLN": true,
            "Compile": true,
            "Publish_Blast": true
        });

        match (&TriggerBuild {
            job_name: JOB_NAME,
            params: &params,
        })
            .raw_query(&client)
            .await
        {
            Ok(result) => {
                println!("{:#?}", result);
            }
            Err(e) => {
                println!("Error: {:#?}", e);
            }
        };

        // match jenkins_sdk::AsyncQuery::<()>::query(
        //     &TriggerBuild {
        //         job_name: JOB_NAME,
        //         params: &params,
        //     },
        //     &client,
        // )
        // .await
        // {
        //     Ok(result) => {}
        //     Err(e) => {
        //         println!("Error: {:#?}", e);
        //     }
        // };
    }

    #[tokio::test]
    async fn test_direct_reqwest_jenkins() {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .no_proxy()
            .build()
            .unwrap();

        let req = client
            .request(
                "GET".parse().unwrap(),
                format!("{}/api/json?tree=ping", URL),
            )
            .header(COOKIE, COOKIE)
            .header("User-Agent", "jenkins-sdk-rust");

        let now = Instant::now();
        match req.send().await {
            Ok(resp) => {
                println!("Response: {:#?}", resp.text().await.unwrap());
            }
            Err(e) => {
                println!("Error: {:#?}", e);
            }
        }
        println!("Direct reqwest cost time: {:?}", now.elapsed());
    }

    #[tokio::test]
    async fn test_direct_reqwest_jenkins_run_info() {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .no_proxy()
            .build()
            .unwrap();

        let job_name = JOB_NAME;
        let run_id = 2182;

        let req = (&client)
            .request(
                "GET".parse().unwrap(),
                format!("{}/job/{}/{}/api/json?tree=number,actions[causes[userId],parameters[name,value]],result", URL, job_name, run_id),
            )
            .header(COOKIE, JENKINS_COOKIE)
            .header("User-Agent", "jenkins-sdk-rust");

        match req.send().await {
            Ok(resp) => {
                println!("Response: {:#?}", resp.text().await.unwrap());
            }
            Err(e) => {
                println!("Error: {:#?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_client() {
        let client = try_get_jenkins_async_client(
            &Some(format!("{}/api/json?tree=ping", URL).to_string()),
            &None,
            &Some(JENKINS_PWD.to_string()),
            &None,
        )
        .await;

        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_ping_jenkins() {
        let client_valid =
            VfpJenkinsClient::ApiTokenClient(JenkinsAsyncClient::new(URL, USERNAME, API_TOKEN));

        assert!(ping_jenkins(&client_valid).await.is_ok());

        let client_invalid = VfpJenkinsClient::ApiTokenClient(JenkinsAsyncClient::new(
            "https://what",
            "who?",
            "none",
        ));
        assert!(ping_jenkins(&client_invalid).await.is_err());
    }

    #[tokio::test]
    async fn test_query_builds_in_job() {
        let client =
            VfpJenkinsClient::ApiTokenClient(JenkinsAsyncClient::new(URL, USERNAME, API_TOKEN));

        let builds = query_builds_in_job(&client, JOB_NAME, Some(200))
            .await
            .unwrap();

        println!("Builds: {:#?}", builds);
    }

    #[tokio::test]
    async fn test_query_runs_in_job() {
        let my_user_id = USERNAME;
        let client =
            VfpJenkinsClient::ApiTokenClient(JenkinsAsyncClient::new(URL, my_user_id, API_TOKEN));
        let job_name = JOB_NAME.to_string();

        match query_builds_in_job(&client, &job_name, Some(20)).await {
            Ok(builds) => {
                for b in builds.builds {
                    let run_info = query_run_info(&client, &job_name, b.number).await;
                    match run_info {
                        Ok(run_info) => {
                            println!("Run Info of {}: {:#?}", b.number, run_info);

                            if run_info.is_mine(my_user_id) {
                                println!("Found my run. build number: {}", b.number);
                            }
                        }
                        Err(e) => {
                            println!("Error: {:#?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {:#?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_query_run_info() {
        let my_user_id = USERNAME;
        let client =
            VfpJenkinsClient::ApiTokenClient(JenkinsAsyncClient::new(URL, my_user_id, API_TOKEN));
        let job_name = JOB_NAME.to_string();
        let run_number = 2182;
        let run_info = query_run_info(&client, &job_name, run_number).await;

        println!("RunInfo: {:#?}", run_info);
    }

    #[tokio::test]
    async fn test_query_user_latest_info() {
        let my_user_id = USERNAME;
        let client =
            VfpJenkinsClient::ApiTokenClient(JenkinsAsyncClient::new(URL, my_user_id, API_TOKEN));
        let job_name = JOB_NAME.to_string();

        match query_user_latest_info(&client, &job_name, my_user_id, Some(200)).await {
            Ok(user_latest_workflow_info) => {
                if let Some(run) = user_latest_workflow_info.latest_success {
                    println!("Found my latest build: {:#?}", run);
                } else {
                    println!("No builds found for user: {}", my_user_id);
                }
            }
            Err(e) => {
                println!("Error: {:#?}", e);
            }
        };
    }

    #[tokio::test]
    async fn test_query_run_log() {
        let my_user_id = USERNAME;
        let client =
            VfpJenkinsClient::ApiTokenClient(JenkinsAsyncClient::new(URL, my_user_id, API_TOKEN));
        let job_name = JOB_NAME.to_string();

        match query_run_log(&client, &job_name, 2090).await {
            Ok(content) => {
                println!("log: \n{:#?}", content);
            }
            Err(e) => {
                println!("Error: {:#?}", e);
            }
        };
    }

    #[test]
    fn parse_json_to_run_info() {
        let content = r#"{"_class":"org.jenkinsci.plugins.workflow.job.WorkflowRun","actions":[{"_class":"hudson.model.CauseAction","causes":[{"_class":"hudson.model.Cause$UserIdCause","userId":"jiajunyi@contractor.ea.com"},{"_class":"com.sonyericsson.rebuild.RebuildCause"}]},{"_class":"hudson.model.ParametersAction","parameters":[{"_class":"hudson.model.StringParameterValue","name":"Changelist","value":"515786"},{"_class":"hudson.model.StringParameterValue","name":"CustomServer","value":""},{"_class":"hudson.model.StringParameterValue","name":"HygeiaLogServer","value":""},{"_class":"hudson.model.StringParameterValue","name":"HygeiaServer","value":""},{"_class":"hudson.model.StringParameterValue","name":"ShelvedChange","value":"510597,515928"},{"_class":"hudson.model.BooleanParameterValue","name":"BackenDetailedProfile","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"DisablePTUpdate","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"EnableAudioAudition","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"EnableBuildInPackageMetaScript","value":true},{"_class":"hudson.model.BooleanParameterValue","name":"EnableContentPreview","value":true},{"_class":"hudson.model.BooleanParameterValue","name":"EnableGPDebug","value":true},{"_class":"hudson.model.BooleanParameterValue","name":"EnableGameTalk","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"Esports","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"ForceSyncProjectBranch","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"GPBECORE","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"LoggingInFinal","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"Pioneer","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"SimulateAndroidGuestLogin","value":true},{"_class":"hudson.model.BooleanParameterValue","name":"UseICETool","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"Clean","value":false},{"_class":"hudson.model.BooleanParameterValue","name":"SetCustomServer","value":true},{"_class":"hudson.model.BooleanParameterValue","name":"GenSLN","value":true},{"_class":"hudson.model.BooleanParameterValue","name":"Compile","value":true},{"_class":"hudson.model.BooleanParameterValue","name":"Publish_Blast","value":true}]},{"_class":"jenkins.metrics.impl.TimeInQueueAction"},{"_class":"org.jenkinsci.plugins.workflow.libs.LibrariesAction"},{},{"_class":"hudson.plugins.git.util.BuildData"},{"_class":"hudson.plugins.git.util.BuildData"},{"_class":"org.jenkinsci.plugins.workflow.cps.EnvActionImpl"},{},{},{},{},{},{},{},{},{"_class":"org.jenkinsci.plugins.displayurlapi.actions.RunDisplayAction"},{"_class":"org.jenkinsci.plugins.pipeline.modeldefinition.actions.RestartDeclarativePipelineAction"},{},{},{},{"_class":"org.jenkinsci.plugins.workflow.job.views.FlowGraphAction"},{},{},{},{}],"number":2183,"result":null}"#;

        match serde_json::from_str::<WorkflowRun>(content) {
            Ok(run_info) => {
                println!("Parsed RunInfo: {:#?}", run_info);
            }
            Err(e) => {
                println!("Failed to parse JSON: {:#?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_watch() {
        let mut stdout = std::io::stdout();
        let client_valid =
            VfpJenkinsClient::ApiTokenClient(JenkinsAsyncClient::new(URL, USERNAME, API_TOKEN));
        let result = watch(&mut stdout, client_valid, USERNAME, JOB_NAME, None).await;

        match result {
            Ok(_) => {
                println!("Watch completed successfully. Send toast.");
                WinToastNotify::new()
                    .set_title("Vertical FP")
                    .set_messages(vec!["Run Task Completed with Success."])
                    .show()
                    .expect("Failed to show toast notification")
            }
            Err(e) => {
                println!("Error: {:#?}", e);
            }
        }
    }

    #[test]
    fn test_vfp_job_build_param_from_xml_to_json() {
        let xml_content = r#"<?xml version='1.1' encoding='UTF-8'?>
<flow-definition plugin="workflow-job@1385.vb_58b_86ea_fff1">
  <description></description>
  <keepDependencies>false</keepDependencies>
  <properties>
    <hudson.model.ParametersDefinitionProperty>
      <parameterDefinitions>
        <hudson.model.StringParameterDefinition>
          <name>Changelist</name>
          <trim>true</trim>
        </hudson.model.StringParameterDefinition>
        <hudson.model.BooleanParameterDefinition>
          <name>EnableContentPreview</name>
          <description>Enable Content Preview</description>
          <defaultValue>false</defaultValue>
        </hudson.model.BooleanParameterDefinition>
      </parameterDefinitions>
    </hudson.model.ParametersDefinitionProperty>
    <jenkins.model.BuildDiscarderProperty>
      <strategy class="hudson.tasks.LogRotator">
        <daysToKeep>-1</daysToKeep>
        <numToKeep>100</numToKeep>
        <artifactDaysToKeep>-1</artifactDaysToKeep>
        <artifactNumToKeep>-1</artifactNumToKeep>
      </strategy>
    </jenkins.model.BuildDiscarderProperty>
  </properties>
  <triggers/>
  <disabled>false</disabled>
</flow-definition>
"#;
        let def = quick_xml::de::from_str::<FlowDefinition>(xml_content);

        assert!(def.is_ok());
        let def = def.unwrap();

        let mut param = VfpJobBuildParam::from(def);
        assert_eq!(
            param.to_json_value(),
            json!({
                "Changelist": "",
                "EnableContentPreview": true,
                "SimulateAndroidGuestLogin": true,
            })
        );

        param.set_change_list(1234);
        assert_eq!(
            param.to_json_value(),
            json!({
                "Changelist": "1234",
                "EnableContentPreview": true,
                "SimulateAndroidGuestLogin": true,
            })
        );

        param.set_shelve_changes(vec![1230, 1231]);
        assert_eq!(
            param.to_json_value(),
            json!({
                "Changelist": "1234",
                "EnableContentPreview": true,
                "SimulateAndroidGuestLogin": true,
                "ShelvedChange": "1230,1231",
            })
        );
    }
}
