use crate::constant::log::{
    ERR_EMPTY_REPO, ERR_INPUT_INVALID, ERR_INVALID_PATH, ERR_NEED_A_NUMBER,
    ERR_NO_SPECIFIED_PACKAGE, HINT_EXTRACT_TO, HINT_JOB_NAME, HINT_PLAYER_COUNT,
};
use crate::db::{get_db, save_with_error_log};
use crate::extract::extract_operation_info::{
    ExtractOperationInfo, OperationStatus, OperationStepType,
};
use crate::extract::extractor_util::{clean_dir, extract_zip_file, mending_user_ini};
use crate::extract::repo_decoration::RepoDecoration;
use crate::interact::{
    input_by_selection, input_ci, input_directly_with_default, input_path,
    parse_without_input_with_default,
};
use crate::{default_config, pretty_log};
use crossterm::execute;
use crossterm::style::Color;
use formatx::formatx;
use std::io::Stdout;
use std::path::PathBuf;

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
    job_name: Option<String>,
    ci: Option<u32>,
    count: Option<u32>,
    build_target_repo_template: Option<String>,
    main_locator_pattern: Option<String>,
    secondary_locator_template: Option<String>,
    dest: Option<PathBuf>,
) {
    let mut db = get_db(None);

    if let Ok(val) = input_by_selection(
        job_name,
        None,
        false,
        crate::interact::get_job_name_options(&db.interest_job_name),
        HINT_JOB_NAME,
        default_config::RECOMMEND_JOB_NAMES
            .first()
            .map(|v| v.to_string())
            .as_ref(),
    ) {
        db.interest_job_name = Some(val);
    } else {
        println!("{}", ERR_EMPTY_REPO);
        return;
    }

    db.extract_repo = Some(parse_without_input_with_default(
        build_target_repo_template,
        db.extract_repo.as_ref(),
        default_config::REPO_TEMPLATE,
    ));

    db.extract_locator_pattern = Some(parse_without_input_with_default(
        main_locator_pattern,
        db.extract_locator_pattern.as_ref(),
        default_config::LOCATOR_PATTERN,
    ));

    db.extract_s_locator_template = Some(parse_without_input_with_default(
        secondary_locator_template,
        db.extract_s_locator_template.as_ref(),
        default_config::LOCATOR_TEMPLATE,
    ));

    let repo_decoration = RepoDecoration::new(
        &db.extract_repo.clone().unwrap(),
        &db.extract_locator_pattern.clone().unwrap(),
        &db.extract_s_locator_template.clone().unwrap(),
        &db.interest_job_name.clone().unwrap(),
    );

    let ci_temp = input_ci(stdout, &db, &repo_decoration, ci).await;

    if ci_temp.is_none() {
        println!("{}", ERR_EMPTY_REPO);
        return;
    }

    let ci_temp = ci_temp.unwrap();
    db.last_inner_version = ci_temp.into();

    db.last_player_count = Some(input_directly_with_default(
        count,
        db.last_player_count.as_ref(),
        false,
        HINT_PLAYER_COUNT,
        default_config::COUNT,
        Some(ERR_NEED_A_NUMBER),
    ));

    if let Ok(path) = input_path(
        dest,
        db.blast_path.as_ref(),
        true,
        HINT_EXTRACT_TO,
        false,
        true,
        Some(ERR_INVALID_PATH),
    ) {
        db.blast_path = Some(path);
    } else {
        println!("{}", ERR_INPUT_INVALID);
        return;
    }

    save_with_error_log(&db, None);

    if let Some(path) = repo_decoration.get_full_path_by_ci(ci_temp) {
        if let Some(file_name) = path.file_stem().and_then(|v| v.to_str()) {
            let count = db.last_player_count.unwrap();
            let pty_logger = pretty_log::VfpPrettyLogger::apply_for(stdout, count);

            let mut working_status: Vec<ExtractOperationInfo> = (0..count)
                .map(|_| ExtractOperationInfo::default())
                .collect();

            let mut handles = vec![];
            let (tx, rx) = std::sync::mpsc::channel::<(u32, OperationStepType, OperationStatus)>();

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

                            let extract_res = extract_zip_file(&path_t, &dest_with_origin_name);

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
                                        mend_file_path_t,
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
                            let _ =
                                tx.send((i, OperationStepType::Clean, OperationStatus::Err(msg)));
                        }
                    }
                });

                handles.push(handle);

                if let Some(item) = working_status.get((i - 1) as usize) {
                    let _ = pty_logger.pretty_log_operation_status(stdout, i, count, item);
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

                    let _ = pty_logger.pretty_log_operation_status(stdout, index - 1, count, item);
                }
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
