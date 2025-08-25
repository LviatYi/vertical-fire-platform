use crate::default_config;
use crate::jenkins::jenkins_model::run_status::RunStatus;
use crate::jenkins::jenkins_model::workflow_build::WorkflowBuild;
use crate::jenkins::jenkins_model::workflow_run::WorkflowRun;
use crate::jenkins::query::{
    query_builds_in_job, query_run_info, UserLatestWorkflowInfo, VfpJenkinsClient,
};
use crate::vfp_error::VfpError;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;

pub struct JenkinsRpcService;

impl JenkinsRpcService {
    pub async fn query_user_latest_info(
        client: Arc<VfpJenkinsClient>,
        job_name: &str,
        user_id: &str,
    ) -> Result<UserLatestWorkflowInfo, VfpError> {
        let builds = query_builds_in_job(
            client.as_ref(),
            job_name,
            Some(default_config::USER_QUERY_JENKINS_BUILD_COUNT as u32),
        )
        .await?;

        let mut tasks_set = JoinSet::new();
        let mut results: Vec<Option<WorkflowRun>> = std::iter::repeat_with(|| None)
            .take(default_config::USER_QUERY_JENKINS_BUILD_COUNT)
            .collect();

        let mut next_query_idx = 0;
        let mut oldest_query_in_queue_success_idx = None;

        fn fill_task_set_window(
            client: &Arc<VfpJenkinsClient>,
            tasks_set: &mut JoinSet<(usize, Result<WorkflowRun, VfpError>)>,
            next_query_idx: &mut usize,
            builds: &[WorkflowBuild],
            job_name: &str,
        ) {
            while tasks_set.len() < default_config::JENKINS_QUERY_CONCURRENCY_COUNT
                && *next_query_idx < builds.len()
            {
                let joined_idx = *next_query_idx;
                *next_query_idx += 1;
                let build_number = builds.get(joined_idx).unwrap().number;
                let job_name = job_name.to_string();
                let arc_client = client.to_owned();
                tasks_set.spawn(async move {
                    return match tokio::time::timeout(
                        Duration::from_secs(10),
                        query_run_info(arc_client.as_ref(), job_name.as_ref(), build_number),
                    )
                    .await
                    {
                        Ok(resp) => (joined_idx, resp.map_err(VfpError::from)),
                        Err(_) => (joined_idx, Err(VfpError::JenkinsTimeout)),
                    };
                });
            }
        }

        // start the initial concurrent window
        fill_task_set_window(
            &client,
            &mut tasks_set,
            &mut next_query_idx,
            &builds.builds,
            job_name,
        );

        // handle results as they complete. If necessary results have been obtained, suspend all ongoing tasks in advance
        let mut latest_success_idx = None;
        let mut latest_failed_idx = None;
        let mut latest_in_progress_idx = None;
        while let Some(joined) = tasks_set.join_next().await {
            let (query_idx, run_info) = match joined {
                Ok((idx, Ok(run_info))) => (idx, run_info),
                Ok((idx, Err(e))) => {
                    #[cfg(debug_assertions)]
                    {
                        println!("Error querying run info for index {}: {:?}", idx, e);
                    }

                    continue;
                }
                Err(_) => continue,
            };

            results[query_idx] = Some(run_info);

            if query_idx
                == oldest_query_in_queue_success_idx
                    .unwrap_or(usize::MAX)
                    .wrapping_add(1)
            {
                for (curr_handle_idx, run_info) in results.iter().enumerate().skip(
                    oldest_query_in_queue_success_idx
                        .map(|offset| offset + 1)
                        .unwrap_or_default(),
                ) {
                    if let Some(run_info) = run_info {
                        oldest_query_in_queue_success_idx = Some(curr_handle_idx);

                        if run_info.is_mine(user_id) {
                            match run_info.result {
                                RunStatus::Success => {
                                    if latest_success_idx.is_none() {
                                        latest_success_idx = Some(curr_handle_idx);
                                    }
                                    tasks_set.abort_all();
                                    break;
                                }
                                RunStatus::Failure => {
                                    if latest_failed_idx.is_none() {
                                        latest_failed_idx = Some(curr_handle_idx);
                                    }
                                }
                                RunStatus::Processing => {
                                    if latest_in_progress_idx.is_none() {
                                        latest_in_progress_idx = Some(curr_handle_idx);
                                    }
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
            }

            // hold the next query in the queue
            fill_task_set_window(
                &client,
                &mut tasks_set,
                &mut next_query_idx,
                &builds.builds,
                job_name,
            );
        }

        Ok(UserLatestWorkflowInfo {
            latest_success: latest_success_idx.and_then(|idx| results[idx].take()),
            in_progress: latest_in_progress_idx.and_then(|idx| results[idx].take()),
            failed: latest_failed_idx.and_then(|idx| results[idx].take()),
        })
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::app_state::AppState;
    use crate::service::jenkins_rpc_service::JenkinsRpcService;
    use std::sync::Arc;

    #[tokio::test]
    async fn lab() {
        let app_state = AppState::new(None);
        let mut stdout = app_state.get_stdout();
        if let Ok(client) = app_state
            .get_db()
            .try_get_jenkins_async_client(&mut stdout, true)
            .await
        {
            let now = std::time::Instant::now();
            let arc_client = Arc::new(client);
            let result = JenkinsRpcService::query_user_latest_info(
                arc_client.clone(),
                app_state.get_db().get_interest_job_name().unwrap(),
                app_state
                    .get_db()
                    .get_jenkins_username()
                    .clone()
                    .unwrap()
                    .as_str(),
            )
            .await;

            println!("Result: {:#?}", result);
            println!("Elapsed time: {:?}", now.elapsed());
        }
    }
}
