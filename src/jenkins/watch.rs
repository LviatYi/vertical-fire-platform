use crate::db::db_struct::LatestVersionData;
use crate::jenkins::jenkins_model::reasoned_run_status::ReasonedRunStatus;
use crate::jenkins::jenkins_model::run_status::RunStatus;
use crate::jenkins::query::{
    query_run_info, query_run_log, try_get_jenkins_async_client, VfpJenkinsClient,
};
use jenkins_sdk::JenkinsError;

pub async fn get_reasoned_run_status(
    client: &VfpJenkinsClient,
    job_name: &str,
    build_number: u32,
) -> Result<ReasonedRunStatus, JenkinsError> {
    let run_info = query_run_info(client, job_name, build_number).await?;

    match run_info.result {
        RunStatus::Success => Ok(ReasonedRunStatus::Success),
        RunStatus::Processing => Ok(ReasonedRunStatus::Processing),
        RunStatus::Failure => {
            let result = query_run_log(client, job_name, build_number).await?;
            Ok(ReasonedRunStatus::Failure(result))
        }
    }
}

pub async fn watch(db: &LatestVersionData, ci: Option<u32>) {
    let client = try_get_jenkins_async_client(
        &db.jenkins_url,
        &db.jenkins_cookie,
        &db.jenkins_username,
        &db.jenkins_api_token,
    )
    .await
    .unwrap();
    let job_name = "vfp-jenkins-test";
    let build_number = ci.unwrap_or(1);

    loop {
        match get_reasoned_run_status(&client, job_name, build_number).await {
            Ok(status) => {
                println!("Current status: {:?}", status);
                if status.is_final() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
