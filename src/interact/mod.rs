use crate::constant::log::{
    ERR_INPUT_INVALID, ERR_INVALID_PATH, ERR_INVALID_PATH_NOT_EXIST, ERR_JENKINS_CLIENT_INVALID,
    ERR_NEED_A_NUMBER, ERR_NO_SPECIFIED_PACKAGE, HINT_CUSTOM, HINT_LAST_USED_CI_SUFFIX,
    HINT_LATEST_CI_SUFFIX, HINT_MY_LATEST_CI_SUFFIX, HINT_MY_LATEST_FAIL_CI_SUFFIX,
    HINT_MY_LATEST_IN_PROGRESS_CI_SUFFIX, HINT_NO_MY_LATEST_CI_SUFFIX, HINT_SELECT_CI,
    HINT_SET_CUSTOM_CI,
};
use crate::db::db_struct::LatestVersionData;
use crate::default_config;
use crate::extract::repo_decoration::{OrderedCiList, RepoDecoration};
use crate::jenkins::query::query_user_latest_info;
use crate::pretty_log::{clean_one_line, colored_println, ThemeColor};
use dirs::home_dir;
use formatx::formatx;
use inquire::error::InquireResult;
use inquire::validator::ErrorMessage::Custom;
use inquire::validator::Validation;
use inquire::{InquireError, Select, Text};
use std::io::Stdout;
use std::ops::Deref;
use std::path::PathBuf;

//region parse directly

/// # parse without input
///
/// parse an existed value from the command line argument or the memory.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined, return this value directly (priority in order of definition).
///
/// ### Returns
///
/// * `Ok` Some value.
/// * `Err` No value is available.
pub fn parse_without_input<T>(param_val: Option<T>, db_val: Option<&T>) -> Option<T>
where
    T: Clone,
{
    param_val.or_else(|| db_val.cloned())
}

/// # parse without input with default
///
/// parse an existed value from the command line argument or the memory. if not exist, return the default value.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined, return this value directly (priority in order of definition).
/// * `default`: The default value.
pub fn parse_without_input_with_default<T, D>(
    param_val: Option<T>,
    db_val: Option<&T>,
    default: D,
) -> T
where
    T: Clone,
    D: Into<T>,
{
    parse_without_input(param_val, db_val).unwrap_or_else(|| default.into())
}
//endregion

//region inquire::Text

/// # input directly with default
///
/// Input a value directly with default value as fallback.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined, return this value directly (priority in order of definition).
/// * `db_val_usable`: Whether the value from the memory can be used directly.
/// * `hint`: The hint for the selection.
/// * `default`: The default value to return if no selection is made.
/// * `err_hint`: The hint for error occurs.
pub fn input_directly_with_default<T, D>(
    param_val: Option<T>,
    db_val: Option<&T>,
    db_val_usable: bool,
    hint: &str,
    default: D,
    err_hint: Option<&str>,
) -> T
where
    T: Clone + ToString + std::str::FromStr,
    D: Clone + Into<T>,
{
    if let Some(val) = param_val {
        return val;
    }

    if db_val_usable {
        if let Some(val) = db_val {
            return val.clone();
        }
    }

    let mut input = Text::from(hint);

    let opt_default = db_val
        .cloned()
        .map(|db_val| db_val.to_string())
        .or(Some(default.clone().into().to_string()));
    if let Some(ref default) = opt_default {
        input = input.with_default(default.as_ref());
    }

    let err_msg = err_hint.unwrap_or(ERR_INPUT_INVALID).to_string();
    let input = input
        .with_validator(move |v: &str| {
            if v.parse::<T>().is_ok() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(Custom(err_msg.clone())))
            }
        })
        .prompt();

    input
        .ok()
        .and_then(|str| str.parse::<T>().ok())
        .unwrap_or_else(|| default.into())
}

/// # input directly
///
/// Input a value directly.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined, return this value directly (priority in order of definition).
/// * `db_val_usable`: Whether the value from the memory can be used directly.
/// * `hint`: The hint for the selection.
/// * `err_hint`: The hint for error occurs.
pub fn input_directly<T>(
    param_val: Option<T>,
    db_val: Option<&T>,
    db_val_usable: bool,
    hint: &str,
    err_hint: Option<&str>,
) -> InquireResult<T>
where
    T: Clone + ToString + std::str::FromStr,
{
    if let Some(val) = param_val {
        return Ok(val);
    }

    if db_val_usable {
        if let Some(val) = db_val {
            return Ok(val.clone());
        }
    }

    let mut input = Text::from(hint);

    let opt_default = db_val.cloned().map(|db_val| db_val.to_string());
    if let Some(ref default) = opt_default {
        input = input.with_default(default.as_ref());
    }

    let err_msg = err_hint.unwrap_or(ERR_INPUT_INVALID).to_string();
    let input = input
        .with_validator(move |v: &str| {
            if v.parse::<T>().is_ok() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(Custom(err_msg.clone())))
            }
        })
        .prompt();

    input.and_then(|str| {
        str.parse::<T>()
            .map_err(|_| InquireError::Custom(Box::from(ERR_INPUT_INVALID.to_string())))
    })
}

/// # input path
///
/// Input a value representing the path.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined and `db_val_usable` is true, return this value directly (priority in order of definition).
/// * `db_val_usable`: Whether the value from the memory can be used directly.
/// * `hint`: The hint for the input.
/// * `existing_inquire`: Whether the input path should exist.
/// * `use_home_dir`: Whether to use the home directory as the default value.
/// * `err_hint`: The hint for error occurs.
pub fn input_path(
    param_val: Option<PathBuf>,
    db_val: Option<&PathBuf>,
    db_val_usable: bool,
    hint: &str,
    existing_inquire: bool,
    use_home_dir: bool,
    err_hint: Option<&str>,
) -> InquireResult<PathBuf> {
    if let Some(val) = param_val {
        return Ok(val);
    }

    if db_val_usable {
        if let Some(val) = db_val {
            return Ok(val.clone());
        }
    }

    let mut input = Text::from(hint);

    let opt_default = if use_home_dir {
        db_val.cloned().or(home_dir())
    } else {
        db_val.cloned()
    };

    let opt_default = opt_default.map(|db_val| db_val.to_string_lossy().into_owned());
    if let Some(ref default) = opt_default {
        input = input.with_default(default.as_ref());
    }

    let err_msg = err_hint.unwrap_or(ERR_INVALID_PATH).to_string();
    let input = input
        .with_validator(move |v: &str| match v.parse::<PathBuf>() {
            Ok(path) => {
                if existing_inquire {
                    if let Some(path) = path.canonicalize().ok() {
                        return if path.exists() {
                            Ok(Validation::Valid)
                        } else {
                            Ok(Validation::Invalid(Custom(
                                ERR_INVALID_PATH_NOT_EXIST.to_string(),
                            )))
                        };
                    }

                    Ok(Validation::Invalid(Custom(err_msg.clone())))
                } else {
                    Ok(Validation::Valid)
                }
            }
            Err(_) => Ok(Validation::Invalid(Custom(err_msg.clone()))),
        })
        .prompt();

    input.and_then(|str| {
        str.parse::<PathBuf>()
            .map_err(|_| InquireError::Custom(Box::from(ERR_INVALID_PATH.to_string())))
    })
}
//endregion

//region inquire::Selection

/// # input by selection
///
/// Select a value by selection.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined, return this value directly (priority in order of definition).
/// * `db_val_usable`: Whether the value from the memory can be used directly.
/// * `options`: The options to select from.
/// * `hint`: The hint for the selection.
/// * `default`: The default value to return if no selection is made.
///
/// ### Returns
///
/// * `Ok` The selected value.
/// * `Err` No value is available.
pub fn input_by_selection<T, D>(
    param_val: Option<T>,
    db_val: Option<&T>,
    db_val_usable: bool,
    options: Vec<String>,
    hint: &str,
    default: Option<D>,
) -> InquireResult<T>
where
    T: Clone + From<String>,
    D: Into<T>,
{
    if let Some(val) = param_val {
        return Ok(val);
    }

    if db_val_usable {
        if let Some(val) = db_val {
            return Ok(val.clone());
        }
    }

    let selection = Select::new(hint, options).prompt();
    match selection {
        Ok(choice) => Ok(choice.to_string().into()),
        Err(e) => default.map(|v| v.into()).ok_or(e),
    }
}
//endregion

//region Selection Options

pub fn get_job_name_options(last_used: &Option<String>) -> Vec<String> {
    let mut options = default_config::RECOMMEND_JOB_NAMES.to_vec();
    if let Some(last_used) = last_used.clone() {
        if let Some(index) = options.iter_mut().position(|&mut v| v == last_used) {
            let mut origin_options = options.clone();
            options = origin_options.split_off(index);
            let mut follow = options.split_off(1);

            options.append(&mut origin_options);
            options.append(&mut follow);
        }
    }

    options.iter().map(|v| v.to_string()).collect()
}
//endregion

pub async fn input_ci(
    stdout: &mut Stdout,
    db: &LatestVersionData,
    repo_decoration: &RepoDecoration,
    param_val: Option<u32>,
) -> Option<u32> {
    if param_val.is_some() {
        return param_val;
    }

    let latest = repo_decoration.get_sorted_ci_list().first().copied();
    let last_used = db.last_inner_version;

    let mut options: Vec<String> = Vec::new();

    let mut latest_mine_opt_index: usize = usize::MAX;
    let mut latest_opt_index: usize = usize::MAX;
    let mut last_used_index: usize = usize::MAX;

    //region latest mine ci
    let mut latest_mine_ci: Option<u32> = None;
    if let Some(job_name) = db.interest_job_name.clone() {
        let mut jenkins_client_invalid = false;
        let client = db.try_get_jenkins_async_client(stdout, true).await;

        colored_println(
            stdout,
            ThemeColor::Second,
            crate::constant::log::QUERYING_USER_LATEST_CI,
        );

        match client {
            Ok(client) => {
                let user_latest_info_result = query_user_latest_info(
                    &client,
                    &job_name,
                    &(db.jenkins_username.clone().unwrap()),
                    None,
                )
                .await;

                match user_latest_info_result {
                    Ok(user_latest_info) => match user_latest_info.latest_success {
                        Some(ref latest_success) => {
                            latest_mine_ci = Some(latest_success.number);
                            let mut opt_hint = latest_success.number.to_string()
                                + formatx!(
                                    HINT_MY_LATEST_CI_SUFFIX,
                                    db.jenkins_username.clone().unwrap_or_default()
                                )
                                .unwrap_or_default()
                                .as_str();

                            if let Some(ref in_progress) = user_latest_info.in_progress {
                                opt_hint += formatx!(
                                    HINT_MY_LATEST_IN_PROGRESS_CI_SUFFIX,
                                    in_progress.number
                                )
                                .unwrap_or_default()
                                .as_str();
                            }

                            if let Some(ref failed) = user_latest_info.failed {
                                opt_hint += formatx!(HINT_MY_LATEST_FAIL_CI_SUFFIX, failed.number)
                                    .unwrap_or_default()
                                    .as_str();
                            }

                            options.push(opt_hint);
                            latest_mine_opt_index = options.len() - 1;
                        }
                        None => {
                            let mut opt_hint = formatx!(
                                HINT_NO_MY_LATEST_CI_SUFFIX,
                                db.jenkins_username.clone().unwrap_or_default()
                            )
                            .unwrap_or_default();
                            if let Some(ref in_progress) = user_latest_info.in_progress {
                                opt_hint += formatx!(
                                    HINT_MY_LATEST_IN_PROGRESS_CI_SUFFIX,
                                    in_progress.number
                                )
                                .unwrap_or_default()
                                .as_str();
                            }

                            if let Some(ref failed) = user_latest_info.failed {
                                opt_hint += formatx!(HINT_MY_LATEST_FAIL_CI_SUFFIX, failed.number)
                                    .unwrap_or_default()
                                    .as_str();
                            }

                            colored_println(stdout, ThemeColor::Second, &opt_hint);
                        }
                    },

                    Err(_) => {
                        jenkins_client_invalid = true;
                    }
                }
            }
            Err(_) => {
                jenkins_client_invalid = true;
            }
        }

        clean_one_line(stdout);
        if jenkins_client_invalid {
            colored_println(stdout, ThemeColor::Error, ERR_JENKINS_CLIENT_INVALID);
        }
    }
    //endregion

    //region latest ci
    if let Some(latest) = latest {
        options.push(format!("{}{}", latest, HINT_LATEST_CI_SUFFIX));
        latest_opt_index = options.len() - 1;
    }
    //endregion

    let exist_ci_list = repo_decoration.get_sorted_ci_list();

    //region last used ci
    if let Some(ref last_used) = last_used {
        if exist_ci_list.deref().is_ci_exist(last_used) {
            options.push(format!("{}{}", last_used, HINT_LAST_USED_CI_SUFFIX));
            last_used_index = options.len() - 1;
        }
    }
    //endregion

    //region custom ci
    options.push(HINT_CUSTOM.to_string());
    //endregion

    let selection = Select::new(HINT_SELECT_CI, options)
        .without_filtering()
        .raw_prompt();

    match selection {
        Ok(choice) => {
            if choice.index == latest_mine_opt_index && latest_mine_ci.is_some() {
                latest_mine_ci
            } else if choice.index == latest_opt_index {
                latest
            } else if choice.index == last_used_index {
                last_used
            } else {
                let exist_ci_list_for_inquire =
                    repo_decoration.get_sorted_ci_list().deref().clone();

                let input = Text::from(HINT_SET_CUSTOM_CI)
                    .with_validator(move |v: &str| {
                        if let Ok(ref ci) = v.parse::<u32>() {
                            if exist_ci_list_for_inquire.is_ci_exist(ci) {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid(Custom(
                                    ERR_NO_SPECIFIED_PACKAGE.to_string(),
                                )))
                            }
                        } else {
                            Ok(Validation::Invalid(Custom(ERR_NEED_A_NUMBER.to_string())))
                        }
                    })
                    .prompt();

                input.ok().and_then(|str| str.parse::<u32>().ok())
            }
        }
        Err(_) => None,
    }
}
