use crate::constant::log::*;
use crate::jenkins::jenkins_endpoint::job_info::JobInfo;
use crate::jenkins::jenkins_endpoint::ping::{Ping, PingResult};
use crate::jenkins::jenkins_endpoint::run_info::RunInfo;
use crate::jenkins::jenkins_endpoint::run_log::RunLog;
use crate::jenkins::jenkins_model::run_status::RunStatus;
use crate::jenkins::jenkins_model::workflow_builds::WorkflowBuilds;
use crate::jenkins::jenkins_model::workflow_run::WorkflowRun;
use crate::jenkins::pwd_jenkins_async_client::PwdJenkinsAsyncClient;
use jenkins_sdk::client::AsyncClient;
use jenkins_sdk::{AsyncQuery, JenkinsAsyncClient, JenkinsError};

pub enum VfpJenkinsClient {
    PwdClient(PwdJenkinsAsyncClient),
    ApiTokenClient(JenkinsAsyncClient),
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
            VfpJenkinsClient::PwdClient(c) => c.request(method, endpoint, params).await,
            VfpJenkinsClient::ApiTokenClient(c) => c.request(method, endpoint, params).await,
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
    username: &Option<String>,
    pwd: &Option<String>,
    api_token: &Option<String>,
) -> Result<VfpJenkinsClient, JenkinsError> {
    if pwd.is_some() {
        try_get_jenkins_async_client_by_pwd(url, username, pwd).await
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

pub async fn try_get_jenkins_async_client_by_pwd(
    url: &Option<String>,
    username: &Option<String>,
    pwd: &Option<String>,
) -> Result<VfpJenkinsClient, JenkinsError> {
    if url.is_none() || username.is_none() || pwd.is_none() {
        return Err(JenkinsError::RequestError(
            ERR_JENKINS_CLIENT_INVALID_SIMPLE.to_string(),
        ));
    }
    let client = VfpJenkinsClient::PwdClient(PwdJenkinsAsyncClient::new(
        url.as_deref().unwrap(),
        username.as_deref().unwrap(),
        pwd.as_deref().unwrap(),
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

/// # UserLatestWorkflowInfo
///
/// This struct holds the latest workflow information for a user.
/// It contains the latest successful workflow run, the latest in-progress workflow run, and the latest failed workflow run.
#[derive(Debug)]
pub struct UserLatestWorkflowInfo {
    /// user latest successful workflow run
    pub latest_success: Option<WorkflowRun>,

    /// user latest in-progress workflow run
    pub in_progress: Option<WorkflowRun>,

    /// user latest failed workflow run
    pub failed: Option<WorkflowRun>,
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
