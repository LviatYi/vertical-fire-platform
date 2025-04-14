use crate::constant::log::{ERR_EMPTY_REPO, ERR_INVALID_PATH, ERR_JENKINS_CLIENT_INVALID, ERR_NO_SPECIFIED_PACKAGE, HINT_EXTRACT_TO};
use crate::db::{get_db, save_with_error_log};
use crate::extract::extract_operation_info::{ExtractOperationInfo, OperationStatus, OperationStepType};
use crate::extract::extractor_util::{clean_dir, extract_zip_file, mending_user_ini};
use crate::extract::repo_decoration::RepoDecoration;
use crate::interact::{input_blast_path, input_branch, input_ci, input_player_count, parse_extract_locator_pattern, parse_extract_repo, parse_extract_s_locator_template};
use crate::jenkins::query::{query_user_latest_success_info, try_get_jenkins_async_client};
use crate::pretty_log::colored_println;
use crate::{default_config, pretty_log};
use crossterm::execute;
use crossterm::style::Color;
use formatx::formatx;
use std::io::Stdout;
use std::path::PathBuf;
use std::time::Duration;

pub mod branch_types;
pub mod extract_operation_info;
pub mod extractor_util;
pub mod repo_decoration;

/// # cli do extract
///
/// Extract zip file from repo to path.
///
/// Contains Inquire(input requests) and console output.
pub async fn cli_do_extract(
    stdout: &mut Stdout,
    branch: Option<String>,
    mut ci: Option<u32>,
    count: Option<u32>,
    build_target_repo_template: Option<String>,
    main_locator_pattern: Option<String>,
    secondary_locator_template: Option<String>,
    dest: Option<PathBuf>,
) {
    let mut db = get_db(None);

    db.branch = Some(input_branch(&db, branch));

    db.extract_repo = Some(parse_extract_repo(&db, build_target_repo_template));

    db.extract_locator_pattern =
        Some(parse_extract_locator_pattern(&db, main_locator_pattern));

    db.extract_s_locator_template = Some(parse_extract_s_locator_template(
        &db,
        secondary_locator_template,
    ));

    let repo_decoration = RepoDecoration::new(
        db.extract_repo.clone().unwrap(),
        db.extract_locator_pattern.clone().unwrap(),
        db.extract_s_locator_template.clone().unwrap(),
        db.branch.clone().unwrap().parse().unwrap_or_default(),
    );

    let ci_list = repo_decoration.get_sorted_ci_list();
    let ci_list_clone_for_inquire = ci_list.clone();

    ci = ci
        .and_then(|v| {
            if ci_list
                .binary_search_by(|probe| probe.cmp(&v).reverse())
                .is_ok()
            {
                Some(v)
            } else {
                None
            }
        })
        .filter(|v| *v != 0);

    let mut latest_mine_ci: Option<u32> = None;

    if let Some(job_name) = db.jenkins_interested_job_name.clone() {
        let client = try_get_jenkins_async_client(
            &db.jenkins_url,
            &db.jenkins_cookie,
            &db.jenkins_username,
            &db.jenkins_api_token,
        )
            .await;

        let mut jenkins_client_invalid = false;
        match client {
            Ok(client) => {
                let user_latest_info = query_user_latest_success_info(
                    &client,
                    &job_name,
                    &(db.jenkins_username.clone().unwrap()),
                    None,
                )
                    .await;

                match user_latest_info {
                    Ok(Some(info)) => {
                        latest_mine_ci = Some(info.number);
                    }
                    Ok(None) => {
                        latest_mine_ci = None;
                    }
                    Err(_) => {
                        jenkins_client_invalid = true;
                    }
                }
            }
            Err(_) => {
                jenkins_client_invalid = true;
            }
        }

        if jenkins_client_invalid {
            let _ =
                colored_println(stdout, Color::Red, ERR_JENKINS_CLIENT_INVALID);
        }
    }

    let ci_temp = input_ci(
        &db,
        ci_list.first().copied(),
        latest_mine_ci,
        db.last_inner_version.and_then(|v| {
            if ci_list
                .binary_search_by(|probe| probe.cmp(&v).reverse())
                .is_ok()
            {
                Some(v)
            } else {
                None
            }
        }),
        Some(&ci_list_clone_for_inquire),
    );

    if ci_temp.is_none() {
        println!("{}", ERR_EMPTY_REPO);
        return;
    }

    db.last_inner_version = ci_temp;
    let ci_temp = ci_temp.unwrap();

    db.last_player_count = Some(input_player_count(&db, count));

    db.blast_path = Some(input_blast_path(&db, dest, HINT_EXTRACT_TO));

    save_with_error_log(&db, None);

    if let Some(path) = repo_decoration.get_full_path_by_ci(ci_temp) {
        if let Some(file_name) = path.file_stem().and_then(|v| v.to_str()) {
            let count = db.last_player_count.unwrap();
            let pty_logger = pretty_log::VfpPrettyLogger::apply_for(stdout, count);

            let mut working_status: Vec<ExtractOperationInfo> = (0..count)
                .map(|_| ExtractOperationInfo::default())
                .collect();

            let mut handles = vec![];
            let (tx, rx) =
                std::sync::mpsc::channel::<(u32, OperationStepType, OperationStatus)>();

            for i in 1..count + 1 {
                let tx = tx.clone();
                let dest_with_origin_name = db
                    .blast_path
                    .clone()
                    .unwrap()
                    .as_path()
                    .join(format!("{}{}", file_name, i));
                let path_t = path.clone();
                let mend_file_path_t = default_config::MENDING_FILE_PATH;
                let handle = std::thread::spawn(move || {
                    let clean_res = clean_dir(&dest_with_origin_name);
                    match clean_res {
                        Ok(cost_opt) => {
                            let _ = tx.send((
                                i,
                                OperationStepType::Clean,
                                OperationStatus::Done(cost_opt),
                            ));

                            let extract_res =
                                extract_zip_file(&path_t, &dest_with_origin_name);

                            match extract_res {
                                Ok(cost) => {
                                    let _ = tx.send((
                                        i,
                                        OperationStepType::Extract,
                                        OperationStatus::Done(Some(cost)),
                                    ));

                                    let mend_res = mending_user_ini(
                                        &dest_with_origin_name,
                                        i,
                                        &mend_file_path_t,
                                    );

                                    match mend_res {
                                        Ok(cost) => {
                                            let _ = tx.send((
                                                i,
                                                OperationStepType::Mend,
                                                OperationStatus::Done(Some(cost)),
                                            ));
                                        }
                                        Err(e) => {
                                            let _ = tx.send((
                                                i,
                                                OperationStepType::Mend,
                                                OperationStatus::Err(e.to_string()),
                                            ));
                                        }
                                    }
                                }
                                Err(msg) => {
                                    let _ = tx.send((
                                        i,
                                        OperationStepType::Extract,
                                        OperationStatus::Err(msg),
                                    ));
                                }
                            }
                        }
                        Err(msg) => {
                            let _ = tx.send((
                                i,
                                OperationStepType::Clean,
                                OperationStatus::Err(msg),
                            ));
                        }
                    }
                });

                handles.push(handle);

                if let Some(item) = working_status.get((i - 1) as usize) {
                    let _ = pty_logger.pretty_log_operation_status(
                        stdout,
                        i,
                        count,
                        item,
                    );
                };
            }

            drop(tx);

            while let Ok((index, op_type, op_stat)) = rx.recv() {
                if let Some(item) = working_status.get_mut((index - 1) as usize) {
                    match op_type {
                        OperationStepType::Clean => {
                            item.clean_state = op_stat;
                        }
                        OperationStepType::Extract => {
                            item.extract_state = op_stat;
                        }
                        OperationStepType::Mend => {
                            item.mend_state = op_stat;
                        }
                    }

                    let _ = pty_logger.pretty_log_operation_status(
                        stdout,
                        index - 1,
                        count,
                        item,
                    );
                }
                std::thread::sleep(Duration::from_millis(50));
            }

            for handle in handles {
                handle.join().expect("Thread panicked");
            }
        } else {
            let _ = execute!(
                            stdout,
                            crossterm::style::SetForegroundColor(Color::Red),
                            crossterm::style::Print(format!(
                                "{}\n",
                                formatx!(ERR_INVALID_PATH).unwrap_or_default()
                            ))
                        );
        }
    } else {
        let _ = execute!(
                        stdout,
                        crossterm::style::SetForegroundColor(Color::Red),
                        crossterm::style::Print(format!(
                            "{}\n",
                            formatx!(ERR_NO_SPECIFIED_PACKAGE).unwrap_or_default()
                        ))
                    );
    }
    let _ = execute!(stdout, crossterm::style::ResetColor);
}