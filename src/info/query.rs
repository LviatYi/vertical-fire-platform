use crate::constant::log::*;
use crate::info::cookied_jenkins_async_client::CookiedJenkinsAsyncClient;
use crate::info::jenkins_endpoint::job_info::JobInfo;
use crate::info::jenkins_endpoint::ping::{Ping, PingResult};
use crate::info::jenkins_endpoint::run_info::RunInfo;
use crate::info::jenkins_endpoint::run_log::RunLog;
use crate::info::jenkins_model::run_status::RunStatus;
use crate::info::jenkins_model::workflow_builds::WorkflowBuilds;
use crate::info::jenkins_model::workflow_run::WorkflowRun;
use jenkins_sdk::client::AsyncClient;
use jenkins_sdk::{AsyncQuery, JenkinsAsyncClient, JenkinsError};

pub enum VfpJenkinsClient {
    ApiTokenClient(JenkinsAsyncClient),
    CookiedClient(CookiedJenkinsAsyncClient),
}

#[async_trait::async_trait]
impl AsyncClient for VfpJenkinsClient {
    async fn request(
        &self,
        method: &str,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<String, JenkinsError> {
        match self {
            VfpJenkinsClient::ApiTokenClient(c) => c.request(method, endpoint, params).await,
            VfpJenkinsClient::CookiedClient(c) => c.request(method, endpoint, params).await,
        }
    }
}

pub async fn ping_jenkins(client: &VfpJenkinsClient) -> Result<(), JenkinsError> {
    AsyncQuery::<PingResult>::query(&Ping, client)
        .await
        .map(|_| ())
}

pub async fn try_get_jenkins_async_client(
    url: &Option<String>,
    cookie: &Option<String>,
    username: &Option<String>,
    api_token: &Option<String>,
) -> Result<VfpJenkinsClient, JenkinsError> {
    if cookie.is_some() {
        try_get_jenkins_async_client_by_cookie(url, cookie).await
    } else {
        try_get_jenkins_async_client_by_api_token(url, username, api_token).await
    }
}

pub async fn try_get_jenkins_async_client_by_api_token(
    url: &Option<String>,
    username: &Option<String>,
    api_token: &Option<String>,
) -> Result<VfpJenkinsClient, JenkinsError> {
    if url.is_none() || username.is_none() || api_token.is_none() {
        return Err(JenkinsError::RequestError(
            ERR_JENKINS_CLIENT_INVALID_SIMPLE.to_string(),
        ));
    }
    let client = VfpJenkinsClient::ApiTokenClient(JenkinsAsyncClient::new(
        url.as_deref().unwrap(),
        username.as_deref().unwrap(),
        api_token.as_deref().unwrap(),
    ));
    let result = ping_jenkins(&client).await;

    match result {
        Ok(_) => Ok(client),
        Err(e) => Err(e),
    }
}

pub async fn try_get_jenkins_async_client_by_cookie(
    url: &Option<String>,
    cookie: &Option<String>,
) -> Result<VfpJenkinsClient, JenkinsError> {
    if url.is_none() || cookie.is_none() {
        return Err(JenkinsError::RequestError(
            ERR_JENKINS_CLIENT_INVALID_SIMPLE.to_string(),
        ));
    }
    let client = VfpJenkinsClient::CookiedClient(CookiedJenkinsAsyncClient::new(
        url.as_deref().unwrap(),
        cookie.as_deref().unwrap(),
    ));
    let result = ping_jenkins(&client).await;

    match result {
        Ok(_) => Ok(client),
        Err(e) => Err(e),
    }
}

pub async fn query_builds_in_job(
    client: &VfpJenkinsClient,
    job_name: &str,
    count: Option<u32>,
) -> Result<WorkflowBuilds, JenkinsError> {
    AsyncQuery::query(
        &JobInfo {
            job_name: job_name.into(),
            count,
        },
        client,
    )
    .await
}

pub async fn query_run_info(
    client: &VfpJenkinsClient,
    job_name: &str,
    build_number: u32,
) -> Result<WorkflowRun, JenkinsError> {
    AsyncQuery::query(
        &RunInfo {
            job_name: job_name.into(),
            build_number,
        },
        client,
    )
    .await
}

pub async fn query_user_latest_success_info(
    client: &VfpJenkinsClient,
    job_name: &str,
    user_id: &str,
    count: Option<u32>,
) -> Result<Option<WorkflowRun>, JenkinsError> {
    let builds = query_builds_in_job(client, job_name, count).await?;
    let mut user_latest_build_number: Option<WorkflowRun> = None;

    for b in builds.builds {
        if let Ok(run_info) = query_run_info(client, job_name, b.number).await {
            if run_info.is_mine(user_id) && run_info.result == RunStatus::Success {
                user_latest_build_number = Some(run_info);
                break;
            }
        }
    }

    Ok(user_latest_build_number)
}

pub async fn query_run_log(
    client: &VfpJenkinsClient,
    job_name: &str,
    build_number: u32,
) -> Result<String, JenkinsError> {
    jenkins_sdk::AsyncRawQuery::raw_query(
        &RunLog {
            job_name: job_name.into(),
            build_number,
        },
        client,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use jenkins_sdk::{AsyncQuery, JenkinsAsyncClient, JobsInfo};
    use reqwest::header::COOKIE;
    use reqwest::Client;
    use serde_json::json;
    use std::time::Instant;

    const URL: &str = "";
    const USERNAME: &str = "";
    const JOB_NAME: &str = "";
    const API_TOKEN: &str = "";
    const JENKINS_COOKIE: &str = "";

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
            "Changelist": "505990",
            "CustomServer": "",
            "HygeiaLogServer": "",
            "HygeiaServer": "",
            "ShelvedChange": "484152,497925",
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

        // if let Err(e) = client.trigger_build_with_params(job_name, &params).await {
        //     eprintln!("Failed to trigger build: {}", e);
        // }
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
                format!("{}/job/{}/{}/api/json?tree=number,actions[causes[userId],parameters[name,value]],result",URL, job_name, run_id),
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
            &Some(JENKINS_COOKIE.to_string()),
            &None,
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

        let builds = super::query_builds_in_job(&client, JOB_NAME, Some(200))
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

        match super::query_builds_in_job(&client, &job_name, Some(20)).await {
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

        match query_user_latest_success_info(&client, &job_name, my_user_id, Some(200)).await {
            Ok(run) => {
                if let Some(run) = run {
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
}
