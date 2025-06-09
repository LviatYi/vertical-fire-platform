use crate::constant::log::*;
use crate::db::db_data_proxy::DbDataProxy;
use crate::db::{get_db, save_with_error_log};
use crate::extract::extract_operation_info::{
    ExtractOperationInfo, OperationStatus, OperationStepType,
};
use crate::extract::extractor_util::{clean_dir, extract_zip_file, mending_user_ini};
use crate::extract::repo_decoration::RepoDecoration;
use crate::interact::{
    input_ci, input_directly_with_default, input_job_name, input_path, input_pwd,
    parse_without_input_with_default,
};
use crate::jenkins::query::{
    try_get_jenkins_async_client_by_api_token, try_get_jenkins_async_client_by_pwd,
    VfpJenkinsClient,
};
use crate::jenkins::util::get_jenkins_workflow_run_url;
use crate::jenkins::watch::{watch, VfpWatchError};
use crate::pretty_log::{colored_println, toast, ThemeColor};
use crate::vfp_error::VfpError;
use crate::{default_config, pretty_log};
use crossterm::execute;
use crossterm::style::Color;
use formatx::formatx;
use std::io::Stdout;
use std::path::PathBuf;

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
    dest: Option<PathBuf>,
    build_target_repo_template: Option<String>,
    main_locator_pattern: Option<String>,
    secondary_locator_template: Option<String>,
) {
    let mut db = get_db(None);

    if let Ok(val) = input_job_name(job_name, db.get_interest_job_name()) {
        db.set_interest_job_name(Some(val));
    } else {
        println!("{}", ERR_EMPTY_REPO);
        return;
    }

    db.set_extract_repo(Some(parse_without_input_with_default(
        build_target_repo_template,
        db.get_extract_repo().as_ref(),
        default_config::REPO_TEMPLATE,
    )));

    db.set_extract_locator_pattern(Some(parse_without_input_with_default(
        main_locator_pattern,
        db.get_extract_locator_pattern().as_ref(),
        default_config::LOCATOR_PATTERN,
    )));

    db.set_extract_s_locator_template(Some(parse_without_input_with_default(
        secondary_locator_template,
        db.get_extract_s_locator_template().as_ref(),
        default_config::LOCATOR_TEMPLATE,
    )));

    let repo_decoration = RepoDecoration::new(
        &db.get_extract_repo().clone().unwrap(),
        &db.get_extract_locator_pattern().clone().unwrap(),
        &db.get_extract_s_locator_template().clone().unwrap(),
        &db.get_interest_job_name().clone().unwrap(),
    );

    let ci_temp = input_ci(stdout, ci, &db, &repo_decoration).await;

    if ci_temp.is_none() {
        println!("{}", ERR_EMPTY_REPO);
        return;
    }

    let ci_temp = ci_temp.unwrap();
    db.set_last_inner_version(ci_temp.into());

    db.set_last_player_count(Some(input_directly_with_default(
        count,
        db.get_last_player_count().as_ref(),
        false,
        default_config::COUNT,
        false,
        HINT_PLAYER_COUNT,
        Some(ERR_NEED_A_NUMBER),
    )));

    if let Ok(path) = input_path(
        dest,
        db.get_blast_path().as_ref(),
        true,
        HINT_EXTRACT_TO,
        false,
        true,
        Some(ERR_INVALID_PATH),
    ) {
        db.set_blast_path(Some(path));
    } else {
        println!("{}", ERR_INPUT_INVALID);
        return;
    }

    save_with_error_log(&db, None);

    if let Some(path) = repo_decoration.get_full_path_by_ci(ci_temp) {
        if let Some(file_name) = path.file_stem().and_then(|v| v.to_str()) {
            let count = db.get_last_player_count().unwrap();
            let pty_logger = pretty_log::VfpPrettyLogger::apply_for(stdout, count);

            let mut working_status: Vec<ExtractOperationInfo> = (0..count)
                .map(|_| ExtractOperationInfo::default())
                .collect();

            let mut handles = vec![];
            let (tx, rx) = std::sync::mpsc::channel::<(u32, OperationStepType, OperationStatus)>();

            for i in 1..count + 1 {
                let tx = tx.clone();
                let dest_with_origin_name = db
                    .get_blast_path()
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

            toast("Extract", vec![EXTRACT_TASK_COMPLETED]);
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

/// # cli do login
///
/// Login to Jenkins server.
///
/// Contains Inquire(input requests) and console output.
///
/// ### Arguments
///
/// * `db`: db file.
/// * `simplified`: When simplifying, only re-enter the login key (password api-token etc.).
/// * `url`: jenkins url root from cli param.
/// * `username`: jenkins username from cli param.
/// * `api_token`: jenkins api token from cli param.
/// * `pwd`: jenkins password from cli param.
pub async fn cli_do_login(
    db: &mut DbDataProxy,
    simplified: bool,
    url: Option<impl AsRef<str>>,
    username: Option<impl AsRef<str>>,
    api_token: Option<impl AsRef<str>>,
    pwd: Option<impl AsRef<str>>,
) -> Result<VfpJenkinsClient, VfpError> {
    db.set_jenkins_url(Some(input_directly_with_default(
        url.map(|v| v.as_ref().to_string()),
        db.get_jenkins_url().as_ref(),
        simplified,
        default_config::JENKINS_URL.to_string(),
        true,
        HINT_INPUT_JENKINS_URL,
        Some(ERR_NEED_A_JENKINS_URL),
    )));

    let username = crate::interact::input_directly(
        username.map(|v| v.as_ref().to_string()),
        db.get_jenkins_username().as_ref(),
        simplified,
        true,
        HINT_INPUT_JENKINS_USERNAME,
        Some(ERR_NEED_A_JENKINS_USERNAME),
    )?;

    db.set_jenkins_username(Some(username));

    save_with_error_log(&db, None);

    let login_method = inquire::Select::new(
        HINT_SELECT_LOGIN_METHOD,
        vec![crate::LoginMethod::Pwd, crate::LoginMethod::ApiToken],
    )
    .prompt()
    .unwrap_or(crate::LoginMethod::ApiToken);

    let client = match login_method {
        crate::LoginMethod::ApiToken => {
            let hint = formatx!(
                HINT_INPUT_JENKINS_API_TOKEN,
                db.get_jenkins_url().clone().unwrap(),
                db.get_jenkins_username().clone().unwrap()
            )
            .unwrap_or(HINT_JENKINS_API_TOKEN_DOC.to_string());

            db.set_jenkins_api_token(Some(crate::interact::input_directly(
                api_token.map(|v| v.as_ref().to_string()),
                db.get_jenkins_api_token().as_ref(),
                false,
                true,
                &hint,
                Some(ERR_NEED_A_JENKINS_API_TOKEN),
            )?));

            try_get_jenkins_async_client_by_api_token(
                db.get_jenkins_url(),
                db.get_jenkins_username(),
                db.get_jenkins_api_token(),
            )
            .await
        }
        crate::LoginMethod::Pwd => {
            db.set_jenkins_pwd(
                input_pwd(
                    pwd.map(|v| v.as_ref().to_string()),
                    HINT_INPUT_JENKINS_PWD,
                    Some(ERR_NEED_A_JENKINS_PWD),
                )
                .ok(),
            );

            try_get_jenkins_async_client_by_pwd(
                db.get_jenkins_url(),
                db.get_jenkins_username(),
                &db.get_jenkins_pwd(),
            )
            .await
        }
    };

    match client {
        Ok(client) => {
            save_with_error_log(&db, None);
            Ok(client)
        }
        Err(e) => {
            let key = match login_method {
                crate::LoginMethod::ApiToken => db.get_jenkins_api_token().clone().unwrap(),
                crate::LoginMethod::Pwd => db.get_jenkins_pwd().clone().unwrap(),
            };

            Err(VfpError::JenkinsLoginError {
                method: login_method,
                url: db.get_jenkins_url().clone().unwrap(),
                username: db.get_jenkins_username().clone().unwrap(),
                key,
                e,
            })
        }
    }
}

/// # ci do watch
///
/// Watch jenkins run task status.
///
/// Contains Inquire(input requests) and console output.
pub async fn cli_do_watch(
    stdout: &mut Stdout,
    job_name: Option<String>,
    ci: Option<u32>,
) -> (Option<String>, Option<u32>) {
    let mut success_build_number = None;
    let mut used_job_name = None;
    let db = get_db(None);
    let client = db.try_get_jenkins_async_client(stdout, true).await;

    if let Ok(client) = client {
        let job_name = input_job_name(job_name, db.get_interest_job_name());

        if job_name.is_err() {
            println!("{}", ERR_EMPTY_REPO);
            return (used_job_name, success_build_number);
        }
        used_job_name = Some(job_name.unwrap());

        match db.get_jenkins_username() {
            Some(username) => {
                let result = watch(
                    stdout,
                    client,
                    username,
                    &used_job_name.clone().unwrap(),
                    ci,
                )
                .await;

                match result {
                    Ok(build_number) => {
                        success_build_number = Some(build_number);
                        colored_println(
                            stdout,
                            ThemeColor::Success,
                            &formatx!(
                                WATCHING_RUN_TASK_SUCCESS,
                                build_number,
                                used_job_name.as_ref().unwrap()
                            )
                            .unwrap_or_default(),
                        );

                        toast("Watch", vec![RUN_TASK_COMPLETED]);
                    }
                    Err(e) => match e {
                        VfpWatchError::JenkinsError(_) => {
                            colored_println(stdout, ThemeColor::Error, ERR_NO_IN_PROGRESS_RUN_TASK);
                        }
                        VfpWatchError::NoValidRunTask => {
                            colored_println(stdout, ThemeColor::Error, ERR_NO_VALID_RUN_TASK);
                        }
                        VfpWatchError::WatchTaskFailed(build_number, log) => {
                            let default_url = "".to_string();
                            colored_println(
                                stdout,
                                ThemeColor::Error,
                                &formatx!(
                                    WATCHING_RUN_TASK_FAILURE,
                                    build_number,
                                    used_job_name.as_ref().unwrap(),
                                    get_jenkins_workflow_run_url(
                                        db.get_jenkins_url().as_ref().unwrap_or(&default_url),
                                        used_job_name.as_ref().unwrap(),
                                        build_number
                                    )
                                )
                                .unwrap_or_default(),
                            );
                            colored_println(stdout, ThemeColor::Main, &log);
                        }
                    },
                }
            }
            None => {
                colored_println(stdout, ThemeColor::Error, ERR_NEED_A_JENKINS_USERNAME);
            }
        }
    } else {
        colored_println(stdout, ThemeColor::Error, ERR_JENKINS_CLIENT_INVALID);
    }

    (used_job_name, success_build_number)
}

pub async fn cli_try_first_login(db: &mut DbDataProxy, stdout: Option<&mut Stdout>) -> bool {
    if db.user_never_login() {
        match cli_do_login(
            db,
            false,
            None::<String>,
            None::<String>,
            None::<String>,
            None::<String>,
        )
        .await
        {
            Ok(_) => {
                if let Some(stdout) = stdout {
                    colored_println(stdout, ThemeColor::Success, JENKINS_LOGIN_RESULT);
                }
                true
            }
            Err(e) => {
                if let Some(stdout) = stdout {
                    colored_println(stdout, ThemeColor::Error, e.to_string().as_str());
                }
                false
            }
        }
    } else {
        true
    }
}
