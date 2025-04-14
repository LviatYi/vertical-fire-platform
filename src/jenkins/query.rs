use crate::constant::log::*;
use crate::jenkins::cookied_jenkins_async_client::CookiedJenkinsAsyncClient;
use crate::jenkins::jenkins_endpoint::job_info::JobInfo;
use crate::jenkins::jenkins_endpoint::ping::{Ping, PingResult};
use crate::jenkins::jenkins_endpoint::run_info::RunInfo;
use crate::jenkins::jenkins_endpoint::run_log::RunLog;
use crate::jenkins::jenkins_model::run_status::RunStatus;
use crate::jenkins::jenkins_model::workflow_builds::WorkflowBuilds;
use crate::jenkins::jenkins_model::workflow_run::WorkflowRun;
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
