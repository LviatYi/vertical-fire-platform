use crate::constant::log::*;
use crate::db::db_data_proxy::DbDataProxy;
use crate::default_config;
use crate::extract::repo_decoration::{OrderedCiList, RepoDecoration};
use crate::jenkins::jenkins_model::shelves::Shelves;
use crate::jenkins::query::query_user_latest_info;
use crate::pretty_log::{clean_one_line, colored_println, ThemeColor};
use dirs::home_dir;
use formatx::formatx;
use inquire::error::InquireResult;
use inquire::validator::{ErrorMessage, Validation};
use inquire::{InquireError, Password, PasswordDisplayMode, Select, Text};
use std::fmt::{Display, Formatter};
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

fn prompt_input_with_parse_validator<T>(
    input: Text,
    not_empty: bool,
    err_msg: &str,
) -> InquireResult<String>
where
    T: std::str::FromStr,
{
    let err_msg = err_msg.to_string();
    input
        .with_validator(move |v: &str| {
            if not_empty && v.is_empty() {
                return Ok(Validation::Invalid(ErrorMessage::Custom(
                    ERR_INPUT_INVALID_SHOULD_NOT_BE_EMPTY.to_string(),
                )));
            }

            if v.parse::<T>().is_ok() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(ErrorMessage::Custom(err_msg.clone())))
            }
        })
        .prompt()
}

/// # input directly with default
///
/// Input a value directly with default value as fallback.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined, return this value directly (priority in order of definition).
/// * `db_val_directly_usable`: Whether the value from the memory can be used directly.
/// * `default`: The default value to return if no selection is made.
/// * `not_empty`: The input should not be empty.
/// * `hint`: The hint for the selection.
/// * `err_hint`: The hint for error occurs.
pub fn input_directly_with_default<T, D>(
    param_val: Option<T>,
    db_val: Option<&T>,
    db_val_directly_usable: bool,
    default: D,
    not_empty: bool,
    hint: &str,
    err_hint: Option<&str>,
) -> T
where
    T: Clone + ToString + std::str::FromStr,
    D: Clone + Into<T>,
{
    if let Some(val) = param_val {
        return val;
    }

    if let (true, Some(val)) = (db_val_directly_usable, db_val) {
        return val.clone();
    }

    let mut input = Text::from(hint);

    let opt_default = db_val
        .cloned()
        .map(|db_val| db_val.to_string())
        .or(Some(default.clone().into().to_string()));
    if let Some(ref default) = opt_default {
        input = input.with_default(default.as_ref());
    }

    prompt_input_with_parse_validator::<String>(
        input,
        not_empty,
        err_hint.unwrap_or(ERR_INPUT_INVALID),
    )
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
/// * `db_val_directly_usable`: Whether the value from the memory can be used directly.
/// * `not_empty`: The input should not be empty.
/// * `hint`: The hint for the selection.
/// * `err_hint`: The hint for error occurs.
pub fn input_directly<T>(
    param_val: Option<T>,
    db_val: Option<&T>,
    db_val_directly_usable: bool,
    not_empty: bool,
    hint: &str,
    err_hint: Option<&str>,
) -> InquireResult<T>
where
    T: Clone + ToString + std::str::FromStr,
{
    if let Some(val) = param_val {
        return Ok(val);
    }

    if db_val_directly_usable {
        if let Some(val) = db_val {
            return Ok(val.clone());
        }
    }

    let mut input = Text::from(hint);

    let opt_default = db_val.cloned().map(|db_val| db_val.to_string());
    if let Some(ref default) = opt_default {
        input = input.with_default(default.as_ref());
    }

    prompt_input_with_parse_validator::<String>(
        input,
        not_empty,
        err_hint.unwrap_or(ERR_INPUT_INVALID),
    )
    .and_then(|str| {
        str.parse::<T>()
            .map_err(|_| InquireError::Custom(Box::from(ERR_INPUT_INVALID.to_string())))
    })
}

/// # input pwd
///
/// Input a password directly.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined, return this value directly (priority in order of definition).
/// * `db_val_directly_usable`: Whether the value from the memory can be used directly.
/// * `hint`: The hint for the selection.
/// * `err_hint`: The hint for error occurs.
pub fn input_pwd(
    param_val: Option<String>,
    hint: &str,
    err_hint: Option<&str>,
) -> InquireResult<String> {
    if let Some(val) = param_val {
        return Ok(val);
    }

    let input = Password::from(hint);

    let err_msg = err_hint.unwrap_or(ERR_INPUT_INVALID).to_string();
    input
        .without_confirmation()
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_validator(move |v: &str| {
            if !v.is_empty() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(ErrorMessage::Custom(err_msg.clone())))
            }
        })
        .prompt()
}

/// # input path
///
/// Input a value representing the path.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined and `db_val_directly_usable` is true, return this value directly (priority in order of definition).
/// * `db_val_directly_usable`: Whether the value from the memory can be used directly.
/// * `hint`: The hint for the input.
/// * `existing_inquire`: Whether the input path should exist.
/// * `use_home_dir`: Whether to use the home directory as the default value.
/// * `err_hint`: The hint for error occurs.
pub fn input_path(
    param_val: Option<PathBuf>,
    db_val: Option<&PathBuf>,
    db_val_directly_usable: bool,
    hint: &str,
    existing_inquire: bool,
    use_home_dir: bool,
    err_hint: Option<&str>,
) -> InquireResult<PathBuf> {
    if let Some(val) = param_val {
        return Ok(val);
    }

    if db_val_directly_usable {
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
                    if let Ok(path) = path.canonicalize() {
                        return if path.exists() {
                            Ok(Validation::Valid)
                        } else {
                            Ok(Validation::Invalid(ErrorMessage::Custom(
                                ERR_INVALID_PATH_NOT_EXIST.to_string(),
                            )))
                        };
                    }

                    Ok(Validation::Invalid(ErrorMessage::Custom(err_msg.clone())))
                } else {
                    Ok(Validation::Valid)
                }
            }
            Err(_) => Ok(Validation::Invalid(ErrorMessage::Custom(err_msg.clone()))),
        })
        .prompt();

    input.and_then(|str| {
        str.parse::<PathBuf>()
            .map_err(|_| InquireError::Custom(Box::from(ERR_INVALID_PATH.to_string())))
    })
}
//endregion

//region inquire::Selection

pub enum SelectionOptionVal<T> {
    Data(T),
    DataWithHintSuffix(T, String),
}

impl<T> SelectionOptionVal<T> {
    fn get_data(self) -> T {
        match self {
            SelectionOptionVal::Data(d) => d,
            SelectionOptionVal::DataWithHintSuffix(d, _) => d,
        }
    }
}

impl<T> Display for SelectionOptionVal<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionOptionVal::Data(d) => {
                write!(f, "{}", d)
            }
            SelectionOptionVal::DataWithHintSuffix(d, hint) => {
                write!(f, "{} {}", d, &hint)
            }
        }
    }
}

impl<T> From<T> for SelectionOptionVal<T> {
    fn from(value: T) -> Self {
        SelectionOptionVal::Data(value)
    }
}

pub enum SelectionCustomizableOptionVal<T> {
    DataContain(SelectionOptionVal<T>),
    Custom,
    None,
}

impl<T> SelectionCustomizableOptionVal<T> {
    fn from_with_hint(d: T, hint: &str) -> Self {
        SelectionCustomizableOptionVal::DataContain(SelectionOptionVal::DataWithHintSuffix(
            d,
            hint.to_string(),
        ))
    }

    fn from_data(d: T) -> Self {
        SelectionCustomizableOptionVal::DataContain(SelectionOptionVal::Data(d))
    }
}

impl<T> Display for SelectionCustomizableOptionVal<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionCustomizableOptionVal::DataContain(d) => {
                write!(f, "{}", d)
            }
            SelectionCustomizableOptionVal::Custom => {
                write!(f, "{}", HINT_CUSTOM)
            }
            SelectionCustomizableOptionVal::None => {
                write!(f, "{}", HINT_NOT_SET)
            }
        }
    }
}

impl<T> From<T> for SelectionCustomizableOptionVal<T> {
    fn from(value: T) -> Self {
        SelectionCustomizableOptionVal::from_data(value)
    }
}

/// # input by selection various
///
/// Select a value by selection. Allow various options.
///
/// ### Arguments
///
/// * `param_val`: The value from the command line argument. If defined, return this value directly (priority in order of definition).
/// * `db_val`: The value from the memory. If defined, return this value directly (priority in order of definition).
/// * `db_val_directly_usable`: Whether the value from the memory can be used directly.
/// * `options`: The options to select from SelectionOptionVal.
/// * `hint`: The hint for the selection.
/// * `default`: The default value to return if no selection is made.
///
/// ### Returns
///
/// * `Ok` The selected value.
/// * `Err` No value is available.
pub fn input_by_selection_various<T>(
    param_val: Option<T>,
    db_val: Option<&T>,
    db_val_directly_usable: bool,
    options: Vec<SelectionCustomizableOptionVal<T>>,
    hint: &str,
    default: Option<impl Into<T>>,
) -> InquireResult<SelectionCustomizableOptionVal<T>>
where
    T: Display + Clone + std::str::FromStr,
{
    if let Some(val) = param_val {
        return Ok(val.into());
    }

    if db_val_directly_usable {
        if let Some(val) = db_val {
            return Ok(val.clone().into());
        }
    }

    match Select::new(hint, options).prompt() {
        Err(e) => default.map(|v| v.into().into()).ok_or(e),
        res => res,
    }
}
//endregion

pub async fn input_ci(
    stdout: &mut Stdout,
    param_val: Option<u32>,
    db: &DbDataProxy,
    repo_decoration: &RepoDecoration,
) -> Option<u32> {
    if param_val.is_some() {
        return param_val;
    }

    let latest = repo_decoration.get_sorted_ci_list().first().copied();
    let last_used = *db.get_last_inner_version();

    let mut options: Vec<String> = Vec::new();

    let mut latest_mine_opt_index: usize = usize::MAX;
    let mut latest_opt_index: usize = usize::MAX;
    let mut last_used_index: usize = usize::MAX;

    //region latest mine ci
    let mut latest_mine_ci: Option<u32> = None;
    if let Some(job_name) = db.get_interest_job_name().clone() {
        let mut jenkins_client_invalid = false;
        let client = db.try_get_jenkins_async_client(stdout, true).await;

        colored_println(stdout, ThemeColor::Second, QUERYING_USER_LATEST_CI);

        match client {
            Ok(client) => {
                let user_latest_info_result = query_user_latest_info(
                    &client,
                    &job_name,
                    &(db.get_jenkins_username().clone().unwrap()),
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
                                    db.get_jenkins_username().clone().unwrap_or_default()
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
                                db.get_jenkins_username().clone().unwrap_or_default()
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
            options.push(format!("{}{}", last_used, HINT_LAST_USED_SUFFIX));
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

                let input = Text::from(HINT_INPUT_CUSTOM)
                    .with_validator(move |v: &str| {
                        if let Ok(ref ci) = v.parse::<u32>() {
                            if exist_ci_list_for_inquire.is_ci_exist(ci) {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid(ErrorMessage::Custom(
                                    ERR_NO_SPECIFIED_PACKAGE.to_string(),
                                )))
                            }
                        } else {
                            Ok(Validation::Invalid(ErrorMessage::Custom(
                                ERR_NEED_A_NUMBER.to_string(),
                            )))
                        }
                    })
                    .prompt();

                input.ok().and_then(|str| str.parse::<u32>().ok())
            }
        }
        Err(_) => None,
    }
}

pub fn input_job_name(param_val: Option<String>, db_val: &Option<String>) -> InquireResult<String> {
    let mut origin_options: Vec<String> = default_config::RECOMMEND_JOB_NAMES
        .to_vec()
        .iter()
        .map(|v| v.to_string())
        .collect();
    let mut options: Vec<SelectionCustomizableOptionVal<String>>;
    if let Some(last_used) = db_val.clone() {
        if let Some(index) = origin_options.iter_mut().position(|v| (*v).eq(&last_used)) {
            let mut cut_off_at_index: Vec<String> = origin_options.split_off(index);
            let mut cut_off_back: Vec<SelectionCustomizableOptionVal<String>> = cut_off_at_index
                .split_off(1)
                .iter()
                .map(|v| SelectionCustomizableOptionVal::from_data(v.to_string()))
                .collect();

            options = cut_off_at_index
                .into_iter()
                .map(|v| SelectionCustomizableOptionVal::from_with_hint(v, HINT_LAST_USED_SUFFIX))
                .collect();

            options.append(
                &mut origin_options
                    .into_iter()
                    .map(|v| SelectionCustomizableOptionVal::from_data(v.to_string()))
                    .collect(),
            );

            options.append(&mut cut_off_back);
        } else {
            options = vec![SelectionCustomizableOptionVal::DataContain(
                SelectionOptionVal::DataWithHintSuffix(
                    last_used,
                    HINT_LAST_USED_SUFFIX.to_string(),
                ),
            )];
            options.append(
                &mut origin_options
                    .into_iter()
                    .map(SelectionCustomizableOptionVal::from_data)
                    .collect(),
            );
        }
    } else {
        options = origin_options
            .into_iter()
            .map(SelectionCustomizableOptionVal::from_data)
            .collect();
    }

    options.push(SelectionCustomizableOptionVal::Custom);

    match input_by_selection_various(
        param_val,
        None,
        false,
        options,
        HINT_JOB_NAME,
        default_config::RECOMMEND_JOB_NAMES
            .first()
            .map(|v| v.to_string())
            .as_ref(),
    ) {
        Ok(SelectionCustomizableOptionVal::DataContain(d)) => Ok(d.get_data()),
        Ok(SelectionCustomizableOptionVal::Custom) => Text::from(HINT_INPUT_CUSTOM).prompt(),
        Ok(SelectionCustomizableOptionVal::None) => panic!("should not be possible"),
        Err(e) => Err(e),
    }
}

pub fn input_cl(param_val: Option<u32>, db_val: &Option<u32>) -> Option<u32> {
    let options: Vec<SelectionCustomizableOptionVal<u32>> = if let Some(last_used) = *db_val {
        vec![
            SelectionCustomizableOptionVal::None,
            SelectionCustomizableOptionVal::from_with_hint(last_used, HINT_LAST_USED_SUFFIX),
            SelectionCustomizableOptionVal::Custom,
        ]
    } else {
        vec![
            SelectionCustomizableOptionVal::None,
            SelectionCustomizableOptionVal::Custom,
        ]
    };

    match input_by_selection_various::<u32>(
        param_val,
        None,
        false,
        options,
        HINT_SELECT_CL,
        None::<u32>,
    )
    .and_then(|v| match v {
        SelectionCustomizableOptionVal::Custom => {
            let input = Text::from(HINT_INPUT_CUSTOM)
                .with_validator(|input: &str| {
                    if input.parse::<u32>().is_ok() {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid(ErrorMessage::Custom(
                            ERR_NEED_A_NUMBER.to_string(),
                        )))
                    }
                })
                .prompt();

            input.map(|v| v.parse::<u32>().unwrap().into())
        }
        other => Ok(other),
    }) {
        Ok(SelectionCustomizableOptionVal::DataContain(d)) => Some(d.get_data()),
        Ok(SelectionCustomizableOptionVal::Custom) => None,
        Ok(SelectionCustomizableOptionVal::None) => None,
        Err(_) => None,
    }
}

pub fn input_sl(param_val: Option<Shelves>, db_val: &Option<Shelves>) -> Option<Shelves> {
    let options: Vec<SelectionCustomizableOptionVal<Shelves>> =
        if let Some(last_used) = db_val.clone() {
            vec![
                SelectionCustomizableOptionVal::None,
                SelectionCustomizableOptionVal::from_with_hint(last_used, HINT_LAST_USED_SUFFIX),
                SelectionCustomizableOptionVal::Custom,
            ]
        } else {
            vec![
                SelectionCustomizableOptionVal::None,
                SelectionCustomizableOptionVal::Custom,
            ]
        };

    match input_by_selection_various::<Shelves>(
        param_val,
        None,
        false,
        options,
        HINT_SELECT_SL,
        None::<Shelves>,
    )
    .and_then(|v| match v {
        SelectionCustomizableOptionVal::Custom => {
            let input = Text::from(HINT_INPUT_CUSTOM)
                .with_validator(|input: &str| {
                    if input.parse::<Shelves>().is_ok() {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid(ErrorMessage::Custom(
                            ERR_NEED_SHELVED.to_string(),
                        )))
                    }
                })
                .prompt();

            input.map(|v| v.parse::<Shelves>().unwrap().into())
        }
        other => Ok(other),
    }) {
        Ok(SelectionCustomizableOptionVal::DataContain(d)) => Some(d.get_data()),
        Ok(SelectionCustomizableOptionVal::Custom) => None,
        Ok(SelectionCustomizableOptionVal::None) => None,
        Err(_) => None,
    }
}
