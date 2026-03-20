use crate::app_state::AppState;
use crate::constant::log::*;
use crate::db::db_data_proxy::DbDataProxy;
use crate::distribute::{distribute_pt, infer_blast_root_dir_name};
use crate::extract::extract_operation_info::{
    ExtractOperationInfo, OperationStatus, OperationStepType,
};
use crate::extract::extract_params::ExtractParams;
use crate::extract::extractor_util::{clean_dir, extract_zip_file, mending_user_ini};
use crate::interact::{
    input_ci_for_extract, input_directly_with_default, input_job_name, input_pwd,
    input_target_path, parse_without_input_with_default,
};
use crate::jenkins::query::{
    try_get_jenkins_async_client_by_api_token, try_get_jenkins_async_client_by_pwd,
    VfpJenkinsClient,
};
use crate::jenkins::watch::watch;
use crate::pretty_log::{colored_println, toast, ThemeColor};
use crate::run::set_server;
use crate::vfp_error::VfpFrontError;
use crate::{default_config, pretty_log, run_instance_with_log};
use crossterm::execute;
use crossterm::style::Color;
use formatx::formatx;
use inquire::InquireError;
use std::path::PathBuf;

/// # cli do extract
///
/// Extract zip file from repo to path.
///
/// Contains Inquire(input requests) and console output.
pub async fn cli_do_extract(
    app_state: &mut AppState,
    job_name_param: Option<String>,
    ci: Option<u32>,
    extract_params: ExtractParams,
    ignore_count_input: bool,
) -> Result<(), VfpFrontError> {
    let runtime_config = default_config::runtime();

    let job_name = {
        let db = app_state.get_mut_db();
        let result = input_job_name_with_err_handling(job_name_param, db)?;
        db.insert_job_name(result.as_str());
        result
    };

    let db = app_state.get_db();
    let used_extract_repo = parse_without_input_with_default(
        extract_params.build_target_repo_template,
        db.get_extract_repo().as_ref(),
        runtime_config.repo_template.as_str(),
    );
    let used_extract_locator_pattern = parse_without_input_with_default(
        extract_params.main_locator_pattern,
        db.get_extract_locator_pattern().as_ref(),
        runtime_config.locator_pattern.as_str(),
    );
    let used_extract_s_locator_template = parse_without_input_with_default(
        extract_params.secondary_locator_template,
        db.get_extract_s_locator_template().as_ref(),
        runtime_config.locator_template.as_str(),
    );
    let used_inner_version = input_ci_for_extract(app_state, job_name.as_str(), ci).await?;

    let db = app_state.get_db();
    let used_player_count = input_directly_with_default(
        extract_params.count,
        db.get_last_player_count(job_name.as_str()).as_ref(),
        ignore_count_input,
        default_config::COUNT,
        false,
        HINT_PLAYER_COUNT,
        Some(ERR_NEED_A_NUMBER),
    );
    let used_blast_path = input_target_path(
        extract_params.dest,
        db.get_blast_path(job_name.as_str()),
        job_name.as_str(),
        HINT_EXTRACT_TO,
        Some(ERR_INVALID_PATH),
    )
    .map_err(|_| VfpFrontError::MissingParam(PARAM_DEST.to_string()))?;

    {
        let db = app_state.get_mut_db();
        db.set_extract_repo(used_extract_repo.into());
        db.set_extract_locator_pattern(used_extract_locator_pattern.into());
        db.set_extract_s_locator_template(used_extract_s_locator_template.into());
        db.set_last_inner_version(job_name.as_str(), used_inner_version.into());
        db.set_last_player_count(job_name.as_str(), used_player_count.into());
        db.set_blast_path(job_name.as_str(), used_blast_path.clone().into());
    }

    app_state.commit(false);

    if let Some(path) = app_state
        .get_db()
        .get_repo_decoration()
        .get_full_path_by_ci(used_inner_version)
    {
        if let Some(file_name) = path.file_stem().and_then(|v| v.to_str()) {
            let pty_logger = pretty_log::VfpPrettyLogger::apply_for(
                &mut app_state.get_stdout(),
                used_player_count,
            );

            let mut working_status: Vec<ExtractOperationInfo> = (0..used_player_count)
                .map(|_| ExtractOperationInfo::default())
                .collect();

            let mut handles = vec![];
            let (tx, rx) = std::sync::mpsc::channel::<(u32, OperationStepType, OperationStatus)>();

            for i in 1..used_player_count + 1 {
                let tx = tx.clone();
                let dest_with_origin_name = used_blast_path
                    .as_path()
                    .join(format!("{}{}", file_name, i));
                let path_t = path.clone();
                let mend_file_path_t = runtime_config.mending_file_path.as_str();
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
                    let _ = pty_logger.pretty_log_operation_status(
                        &mut app_state.get_stdout(),
                        i,
                        used_player_count,
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
                        &mut app_state.get_stdout(),
                        index - 1,
                        used_player_count,
                        item,
                    );
                }
            }

            for handle in handles {
                handle.join().expect("Thread panicked");
            }

            toast("Extract", vec![EXTRACT_TASK_COMPLETED]);
        } else {
            let _ = execute!(
                &mut app_state.get_stdout(),
                crossterm::style::SetForegroundColor(Color::Red),
                crossterm::style::Print(format!(
                    "{}\n",
                    formatx!(ERR_INVALID_PATH).unwrap_or_default()
                ))
            );
        }
    } else {
        let _ = execute!(
            &mut app_state.get_stdout(),
            crossterm::style::SetForegroundColor(Color::Red),
            crossterm::style::Print(format!(
                "{}\n",
                formatx!(ERR_NO_SPECIFIED_PACKAGE).unwrap_or_default()
            ))
        );
    }
    let _ = execute!(&mut app_state.get_stdout(), crossterm::style::ResetColor);

    Ok(())
}

/// # cli do log in
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
    app_state: &mut AppState,
    simplified: bool,
    url: Option<impl AsRef<str>>,
    username: Option<impl AsRef<str>>,
    api_token: Option<impl AsRef<str>>,
    pwd: Option<impl AsRef<str>>,
) -> Result<VfpJenkinsClient, VfpFrontError> {
    let runtime_config = default_config::runtime();
    let db = app_state.get_mut_db();
    db.set_jenkins_url(Some(input_directly_with_default(
        url.map(|v| v.as_ref().to_string()),
        db.get_jenkins_url().as_ref().filter(|v| !v.is_empty()),
        simplified,
        runtime_config.jenkins_url.clone(),
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

    app_state.commit(false);
    let db = app_state.get_mut_db();

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
            app_state.commit(false);
            Ok(client)
        }
        Err(e) => {
            let key = match login_method {
                crate::LoginMethod::ApiToken => db.get_jenkins_api_token().clone().unwrap(),
                crate::LoginMethod::Pwd => db.get_jenkins_pwd().clone().unwrap(),
            };

            Err(VfpFrontError::JenkinsLoginError {
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
    app_state: &mut AppState,
    job_name: Option<String>,
    ci: Option<u32>,
) -> Result<(Option<String>, Option<u32>), VfpFrontError> {
    let db = app_state.get_db();
    let client = db
        .try_get_jenkins_async_client(&mut app_state.get_stdout(), true)
        .await
        .map_err(|_| VfpFrontError::JenkinsClientInvalid)?;

    let db = app_state.get_mut_db();
    let used_job_name = Some(input_job_name_with_err_handling(job_name, db)?);

    let result = watch(app_state, client, &used_job_name.clone().unwrap(), ci).await;

    let success_build_number = match result {
        Ok(build_number) => {
            colored_println(
                &mut app_state.get_stdout(),
                ThemeColor::Success,
                &formatx!(
                    WATCHING_RUN_TASK_SUCCESS,
                    build_number,
                    used_job_name.as_ref().unwrap()
                )
                .unwrap_or_default(),
            );

            toast("Watch", vec![RUN_TASK_COMPLETED]);
            Some(build_number)
        }
        Err(e) => return Err(e),
    };

    Ok((used_job_name, success_build_number))
}

pub async fn cli_try_first_login(
    app_state: &mut AppState,
    silence: bool,
) -> Result<(), VfpFrontError> {
    let db = app_state.get_db();
    if db.user_never_login() {
        match cli_do_login(
            app_state,
            false,
            None::<String>,
            None::<String>,
            None::<String>,
            None::<String>,
        )
        .await
        {
            Ok(_) => {
                if !silence {
                    colored_println(
                        &mut app_state.get_stdout(),
                        ThemeColor::Success,
                        JENKINS_LOGIN_RESULT,
                    );
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else {
        Ok(())
    }
}

pub async fn cli_do_distribute(
    app_state: &mut AppState,
    job_name: &str,
    src_distr_index: u32,
) -> Result<(), VfpFrontError> {
    let runtime_config = default_config::runtime();
    let db = app_state.get_db();

    let blast_path = db
        .get_blast_path(job_name)
        .ok_or(VfpFrontError::DistributeError(
            ERR_BLAST_PATH_NOT_FOUND.to_string(),
        ))?;

    let (prefix, dirs) = infer_blast_root_dir_name(blast_path.as_path()).ok_or(
        VfpFrontError::DistributeError(ERR_BLAST_PATH_NOT_FOUND_ANY_BLAST_PACKAGE.to_string()),
    )?;

    let src_pt_path = blast_path
        .join(format!("{}{}", prefix, src_distr_index))
        .join(
            runtime_config
                .pt_relative_path
                .trim_start_matches(['/', '\\']),
        );

    let limit = db.get_last_player_count(job_name);
    let src_dir_name = format!("{}{}", prefix, src_distr_index);
    let mut filtered_dirs = dirs
        .into_iter()
        .filter(|dir| dir != &src_dir_name)
        .collect::<Vec<_>>();

    let selected_dirs = if let Some(limit) = limit {
        let take_count = limit.max(1).saturating_sub(1) as usize;

        filtered_dirs.sort_by(|a, b| {
            let parse_suffix = |dir: &str| {
                dir.strip_prefix(prefix.as_str())
                   .and_then(|suffix| suffix.parse::<u32>().ok())
            };

            match (parse_suffix(a), parse_suffix(b)) {
                (Some(av), Some(bv)) => av.cmp(&bv).then_with(|| a.cmp(b)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.cmp(b),
            }
        });

        filtered_dirs.into_iter().take(take_count).collect::<Vec<_>>()
    } else {
        filtered_dirs
    };

    let dest_pt_paths = selected_dirs
        .into_iter()
        .map(|path| {
            blast_path
                .join(path)
                .join(
                    runtime_config
                        .pt_relative_path
                        .trim_start_matches(['/', '\\']),
                )
        })
        .collect::<Vec<_>>();

    distribute_pt(app_state, src_pt_path, dest_pt_paths)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn cli_do_run(
    app_state: &mut AppState,
    job_name: Option<String>,
    dest: Option<PathBuf>,
    count: Option<u32>,
    index: Option<u32>,
    package_file_stem: Option<String>,
    exe_file_name: Option<String>,
    check_exe_file_name: Option<String>,
    force: bool,
    server: Option<String>,
) -> Result<(), VfpFrontError> {
    let runtime_config = default_config::runtime();
    let db = app_state.get_db();

    let job_name = input_job_name_with_err_handling(job_name, db)?;

    let dest = input_target_path(
        dest,
        db.get_blast_path(job_name.as_str()),
        job_name.as_str(),
        HINT_SET_PACKAGE_NEED_EXTRACT_HOME_PATH,
        Some(ERR_INVALID_PATH),
    )
        .map_err(|_| {
            VfpFrontError::MissingParam(formatx!(ERR_NEED_PARAM, PARAM_DEST).unwrap_or_default())
        })?;

    let single = index.is_some();

    let count_or_index = index.or(count).unwrap_or_else(|| {
        input_directly_with_default(
            None,
            None,
            false,
            default_config::RUN_COUNT,
            false,
            HINT_RUN_COUNT,
            Some(ERR_NEED_A_NUMBER),
        )
    });

    let package_file_name = parse_without_input_with_default(
        package_file_stem,
        None,
        runtime_config.package_file_stem.as_str(),
    );
    let exe_file_name =
        parse_without_input_with_default(exe_file_name, None, runtime_config.exe_file_name.as_str());
    let check_exe_file_name = parse_without_input_with_default(
        check_exe_file_name,
        None,
        runtime_config.check_exe_file_name.as_str(),
    );

    if single {
        if let Some(server) = server
            && let Err(e) = set_server(
            &dest,
            &package_file_name,
            count_or_index,
            runtime_config.mending_file_path.as_str(),
            &server,
        )
        {
            colored_println(&mut app_state.get_stdout(), ThemeColor::Error, e.as_str());
        }

        run_instance_with_log(
            &dest,
            &package_file_name,
            &exe_file_name,
            &check_exe_file_name,
            count_or_index,
            force,
        );
    } else {
        for i in 1..count_or_index + 1 {
            if let Some(server) = server.clone()
                && let Err(e) = set_server(
                &dest,
                &package_file_name,
                i,
                runtime_config.mending_file_path.as_str(),
                &server,
            )
            {
                colored_println(&mut app_state.get_stdout(), ThemeColor::Error, e.as_str());
            }

            run_instance_with_log(
                &dest,
                &package_file_name,
                &exe_file_name,
                &check_exe_file_name,
                i,
                force,
            );
        }
    }

    Ok(())
}

pub fn input_job_name_with_err_handling(
    param_val: Option<String>,
    db: &DbDataProxy,
) -> Result<String, VfpFrontError> {
    input_job_name(param_val, db).map_err(|e| match e {
        InquireError::OperationCanceled | InquireError::OperationInterrupted => e.into(),
        _ => VfpFrontError::MissingParam(PARAM_JOB_NAME.to_string()),
    })
}
