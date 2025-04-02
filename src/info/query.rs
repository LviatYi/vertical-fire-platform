use crate::info::jenkins_endpoint::job_info::JobInfo;
use crate::info::jenkins_endpoint::run_info::RunInfo;
use crate::info::jenkins_model::workflow_builds::WorkflowBuilds;
use crate::info::jenkins_model::workflow_run::WorkflowRun;
use jenkins_sdk::{AsyncQuery, JenkinsAsyncClient, JenkinsError};

pub async fn query_builds_in_job(
    client: &JenkinsAsyncClient,
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
    client: &JenkinsAsyncClient,
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

pub async fn query_user_latest_info(
    client: &JenkinsAsyncClient,
    job_name: &str,
    user_id: &str,
    count: Option<u32>,
) -> Result<Option<WorkflowRun>, JenkinsError> {
    let builds = query_builds_in_job(client, job_name, count).await?;
    let mut user_latest_build_number: Option<WorkflowRun> = None;

    for b in builds.builds {
        let run_info = query_run_info(client, job_name, b.number).await?;
        if run_info.is_mine(user_id) {
            user_latest_build_number = Some(run_info);
            break;
        }
    }

    Ok(user_latest_build_number)
}
