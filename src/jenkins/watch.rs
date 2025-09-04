use crate::app_state::AppState;
use crate::constant::log::*;
use crate::interact::input_ci_for_watch;
use crate::jenkins::jenkins_model::reasoned_run_status::ReasonedRunStatus;
use crate::jenkins::jenkins_model::run_status::RunStatus;
use crate::jenkins::query::{query_run_info, query_run_log, VfpJenkinsClient};
use crate::jenkins::util::get_jenkins_workflow_run_url;
use crate::pretty_log::{clean_one_line, colored_println, ThemeColor};
use crate::service::jenkins_rpc_service::JenkinsRpcService;
use crate::vfp_error::VfpFrontError;
use chrono::Local;
use formatx::formatx;
use jenkins_sdk::JenkinsError;
use std::sync::Arc;

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
    app_state: &mut AppState,
    client: VfpJenkinsClient,
    job_name: &str,
    ci: Option<u32>,
) -> Result<u32, VfpFrontError> {
    let build_number;
    let db = app_state.get_db();
    let arc_client = Arc::new(client);

    if let Some(ci) = ci {
        build_number = ci;
    } else {
        let username = db
            .get_jenkins_username()
            .as_ref()
            .ok_or(VfpFrontError::MissingParam(PARAM_USERNAME.to_string()))?;

        let latest_info = JenkinsRpcService::query_user_latest_info(
            arc_client.clone(),
            job_name,
            username.as_str(),
        )
        .await?;

        if let Some(in_progress) = latest_info.in_progress {
            build_number = in_progress.number;
        } else if let Some(failed) = latest_info.failed {
            let log = query_run_log(arc_client.as_ref(), job_name, failed.number).await?;

            return Err(VfpFrontError::RunTaskBuildFailed {
                build_number: failed.number,
                job_name: job_name.to_string(),
                run_url: get_jenkins_workflow_run_url(
                    db.get_jenkins_url().as_ref().unwrap(),
                    job_name,
                    failed.number,
                ),
                log,
            });
        } else if let Some(latest_success) = latest_info.latest_success {
            return Ok(latest_success.number);
        } else {
            let username = username.clone();
            colored_println(
                &mut app_state.get_stdout(),
                ThemeColor::Main,
                &format!("{} ({})", NO_IN_PROGRESS_RUN_TASK_OF_USER, username),
            );

            build_number = input_ci_for_watch(app_state, job_name, None).await?;
        }
    }

    colored_println(
        &mut app_state.get_stdout(),
        ThemeColor::Warn,
        &formatx!(
            WATCHING_RUN_TASK_IN_PROGRESS_PREPARE,
            build_number,
            job_name
        )
        .unwrap_or_default(),
    );

    colored_println(
        &mut app_state.get_stdout(),
        ThemeColor::Second,
        &format!(
            "{} {}",
            URL_OUTPUT,
            get_jenkins_workflow_run_url(
                app_state.get_db().get_jenkins_url().as_ref().unwrap(),
                job_name,
                build_number
            )
        ),
    );

    let mut clean_able = false;
    loop {
        let get_reasoned_run_status =
            get_reasoned_run_status(arc_client.as_ref(), job_name, build_number).await?;

        if clean_able {
            clean_one_line(&mut app_state.get_stdout());
        }
        clean_able = true;

        match get_reasoned_run_status {
            ReasonedRunStatus::Processing => {
                colored_println(
                    &mut app_state.get_stdout(),
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
                let db = app_state.get_db();
                return Err(VfpFrontError::RunTaskBuildFailed {
                    build_number,
                    job_name: job_name.to_string(),
                    run_url: get_jenkins_workflow_run_url(
                        db.get_jenkins_url().as_ref().unwrap(),
                        job_name,
                        build_number,
                    ),
                    log,
                });
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(
            crate::default_config::WATCH_INTERVAL,
        ))
        .await;
    }
}
