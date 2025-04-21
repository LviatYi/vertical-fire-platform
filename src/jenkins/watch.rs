use crate::constant::log::*;
use crate::db::db_struct::LatestVersionData;
use crate::jenkins::jenkins_model::reasoned_run_status::ReasonedRunStatus;
use crate::jenkins::jenkins_model::run_status::RunStatus;
use crate::jenkins::query::{
    query_run_info, query_run_log, query_user_latest_info, try_get_jenkins_async_client,
    VfpJenkinsClient,
};
use crate::pretty_log::{clean_one_line, colored_println, ThemeColor};
use formatx::formatx;
use jenkins_sdk::JenkinsError;
use std::io::Stdout;
use thiserror::Error;

async fn get_reasoned_run_status(
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

pub async fn watch(
    stdout: &mut Stdout,
    client: VfpJenkinsClient,
    username: &str,
    job_name: &str,
    ci: Option<u32>,
) -> Result<(), VfpWatchError> {
    let build_number;
    if ci.is_none() {
        let latest_info = query_user_latest_info(&client, job_name, username, None).await?;

        if let Some(in_progress) = latest_info.in_progress {
            build_number = in_progress.number;
        } else {
            return Err(VfpWatchError::NoInProgressRunTask);
        }
    } else {
        build_number = ci.unwrap();
    }

    colored_println(
        stdout,
        ThemeColor::Second,
        &formatx!(WATCHING_RUN_TASK_IN_PROGRESS, build_number, job_name).unwrap_or_default(),
    );

    loop {
        let get_reasoned_run_status =
            get_reasoned_run_status(&client, job_name, build_number).await?;

        match get_reasoned_run_status {
            ReasonedRunStatus::Processing => {
                clean_one_line(stdout);
                colored_println(
                    stdout,
                    ThemeColor::Warn,
                    &formatx!(WATCHING_RUN_TASK_IN_PROGRESS, build_number, job_name)
                        .unwrap_or_default(),
                );
            }
            ReasonedRunStatus::Success => {
                clean_one_line(stdout);
                colored_println(
                    stdout,
                    ThemeColor::Success,
                    &formatx!(WATCHING_RUN_TASK_SUCCESS, build_number, job_name)
                        .unwrap_or_default(),
                );
                return Ok(());
            }
            ReasonedRunStatus::Failure(log) => {
                clean_one_line(stdout);
                colored_println(
                    stdout,
                    ThemeColor::Error,
                    &formatx!(WATCHING_RUN_TASK_FAILURE, build_number, job_name)
                        .unwrap_or_default(),
                );
                colored_println(stdout, ThemeColor::Main, &log);
                return Err(VfpWatchError::WatchTaskFailed);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(
            crate::default_config::WATCH_INTERVAL,
        ))
            .await;
    }
}

#[derive(Error, Debug)]
pub enum VfpWatchError {
    #[error(transparent)]
    JenkinsError(#[from] JenkinsError),

    #[error("{msg}", msg = ERR_NO_IN_PROGRESS_RUN_TASK)]
    NoInProgressRunTask,

    #[error("{msg}", msg = ERR_WATCH_RUN_TASK_FAILED)]
    WatchTaskFailed,
}
