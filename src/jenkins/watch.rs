use crate::constant::log::*;
use crate::jenkins::jenkins_model::reasoned_run_status::ReasonedRunStatus;
use crate::jenkins::jenkins_model::run_status::RunStatus;
use crate::jenkins::query::{
    query_run_info, query_run_log, query_user_latest_info, VfpJenkinsClient,
};
use crate::pretty_log::{clean_one_line, colored_println, ThemeColor};
use chrono::Local;
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

/// # watch
///
/// Watch the run task status by interval.
///
/// ### Arguments
///
/// * `stdout`:
/// * `client`:
/// * `username`:
/// * `job_name`:
/// * `ci`: focus build number of run task. if None, will query the latest run task.
///
/// ### Returns
///
/// if Ok(build_number), the run task is success. You can take the build_number to do something.
pub async fn watch(
    stdout: &mut Stdout,
    client: VfpJenkinsClient,
    username: &str,
    job_name: &str,
    ci: Option<u32>,
) -> Result<u32, VfpWatchError> {
    let build_number;
    if ci.is_none() {
        let latest_info = query_user_latest_info(&client, job_name, username, None).await?;

        if let Some(in_progress) = latest_info.in_progress {
            build_number = in_progress.number;
        } else if let Some(failed) = latest_info.failed {
            let log = query_run_log(&client, job_name, failed.number).await?;

            return Err(VfpWatchError::WatchTaskFailed(failed.number, log));
        } else if let Some(latest_success) = latest_info.latest_success {
            return Ok(latest_success.number);
        } else {
            return Err(VfpWatchError::NoValidRunTask);
        }
    } else {
        build_number = ci.unwrap();
    }

    colored_println(
        stdout,
        ThemeColor::Warn,
        &formatx!(
            WATCHING_RUN_TASK_IN_PROGRESS_PREPARE,
            build_number,
            job_name
        )
        .unwrap_or_default(),
    );

    loop {
        let get_reasoned_run_status =
            get_reasoned_run_status(&client, job_name, build_number).await?;
        clean_one_line(stdout);

        match get_reasoned_run_status {
            ReasonedRunStatus::Processing => {
                colored_println(
                    stdout,
                    ThemeColor::Warn,
                    &formatx!(
                        WATCHING_RUN_TASK_IN_PROGRESS,
                        build_number,
                        job_name,
                        Local::now().format("%Y-%m-%d %H:%M:%S")
                    )
                    .unwrap_or_default(),
                );
            }
            ReasonedRunStatus::Success => {
                return Ok(build_number);
            }
            ReasonedRunStatus::Failure(log) => {
                return Err(VfpWatchError::WatchTaskFailed(build_number, log));
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
    NoValidRunTask,

    #[error("{msg}", msg = ERR_WATCH_RUN_TASK_FAILED)]
    WatchTaskFailed(u32, String),
}
