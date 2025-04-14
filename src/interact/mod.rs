use crate::constant::log::{
    ERR_INVALID_PATH, ERR_NEED_A_JENKINS_API_TOKEN, ERR_NEED_A_JENKINS_COOKIE,
    ERR_NEED_A_JENKINS_URL, ERR_NEED_A_JENKINS_USERNAME, ERR_NEED_A_NUMBER,
    ERR_NO_SPECIFIED_PACKAGE, HINT_CUSTOM, HINT_INPUT_JENKINS_API_TOKEN, HINT_INPUT_JENKINS_COOKIE,
    HINT_INPUT_JENKINS_URL, HINT_INPUT_JENKINS_USERNAME, HINT_JENKINS_API_TOKEN_DOC,
    HINT_JOB_NAME, HINT_LAST_USED_CI_SUFFIX, HINT_LATEST_CI_SUFFIX,
    HINT_MY_LATEST_CI_SUFFIX, HINT_PLAYER_COUNT, HINT_RUN_COUNT, HINT_RUN_INDEX, HINT_SELECT_CI,
    HINT_SET_CUSTOM_CI,
};
use crate::db::db_struct::LatestVersionData;
use crate::default_config;
use dirs::home_dir;
use formatx::formatx;
use inquire::validator::ErrorMessage::Custom;
use inquire::validator::Validation;
use inquire::{Select, Text};
use std::path::PathBuf;

pub fn input_job_name(db: &LatestVersionData, val: Option<String>) -> String {
    val.unwrap_or_else(|| {
        let mut options = default_config::RECOMMEND_JOB_NAMES.to_vec();
        if let Some(last_used) = db.interest_job_name.clone() {
            if let Some(index) = options.iter_mut().position(|&mut v| v == last_used) {
                let mut origin_options = options.clone();
                let mut options = origin_options.split_off(index);
                let mut follow = options.split_off(1);

                options.append(&mut origin_options);
                options.append(&mut follow);
            }
        }

        let selection = Select::new(HINT_JOB_NAME, options).prompt();

        match selection {
            Ok(choice) => choice.to_string(),
            Err(_) => "Dev".to_string(),
        }
    })
}

pub fn parse_extract_repo(db: &LatestVersionData, val: Option<String>) -> String {
    val.unwrap_or_else(|| {
        db.extract_repo
            .clone()
            .unwrap_or(default_config::REPO_TEMPLATE.to_string())
    })
}

pub fn parse_extract_locator_pattern(db: &LatestVersionData, val: Option<String>) -> String {
    val.unwrap_or_else(|| {
        db.extract_locator_pattern
            .clone()
            .unwrap_or(default_config::LOCATOR_PATTERN.to_string())
    })
}

pub fn parse_extract_s_locator_template(db: &LatestVersionData, val: Option<String>) -> String {
    val.unwrap_or_else(|| {
        db.extract_s_locator_template
            .clone()
            .unwrap_or(default_config::LOCATOR_TEMPLATE.to_string())
    })
}

pub fn input_ci<'list>(
    db: &LatestVersionData,
    latest: Option<u32>,
    latest_mine_ci: Option<u32>,
    last_used: Option<u32>,
    exist_ci_list: Option<&Vec<u32>>,
) -> Option<u32> {
    if latest.is_none() {
        return None;
    };
    let latest = latest.unwrap();

    let mut options: Vec<String> = Vec::new();

    let mut latest_mine_opt_index: usize = usize::MAX;
    let mut _latest_opt_index: usize = usize::MAX;
    let mut last_used_index: usize = usize::MAX;
    let custom_index: usize;

    if let Some(latest_mine_ci) = latest_mine_ci {
        options.push(format!(
            "{}{}",
            latest_mine_ci,
            formatx!(
                HINT_MY_LATEST_CI_SUFFIX,
                db.jenkins_username.clone().unwrap_or_default()
            )
            .unwrap_or_default()
        ));
        latest_mine_opt_index = options.len() - 1;
    }

    options.push(format!("{}{}", latest, HINT_LATEST_CI_SUFFIX));
    _latest_opt_index = options.len() - 1;

    if let Some(last_used) = last_used {
        options.push(format!("{}{}", last_used, HINT_LAST_USED_CI_SUFFIX));
        last_used_index = options.len() - 1;
    }
    options.push(HINT_CUSTOM.to_string());
    custom_index = options.len() - 1;

    let selection = Select::new(HINT_SELECT_CI, options)
        .without_filtering()
        .raw_prompt();

    match selection {
        Ok(choice) => {
            if choice.index == custom_index {
                let exist_ci_list_for_inquire = exist_ci_list.cloned();
                let input = Text::from(HINT_SET_CUSTOM_CI)
                    .with_validator(move |v: &str| {
                        if let Ok(ci) = v.parse::<u32>() {
                            if let Some(ref exist_ci_list_for_inquire) = exist_ci_list_for_inquire {
                                if exist_ci_list_for_inquire
                                    .binary_search_by(|probe| probe.cmp(&ci).reverse())
                                    .is_ok()
                                {
                                    Ok(Validation::Valid)
                                } else {
                                    Ok(Validation::Invalid(Custom(
                                        ERR_NO_SPECIFIED_PACKAGE.to_string(),
                                    )))
                                }
                            } else {
                                Ok(Validation::Valid)
                            }
                        } else {
                            Ok(Validation::Invalid(Custom(ERR_NEED_A_NUMBER.to_string())))
                        }
                    })
                    .prompt();

                input.ok().and_then(|str| str.parse::<u32>().ok())
            } else if choice.index == latest_mine_opt_index && latest_mine_ci.is_some() {
                latest_mine_ci
            } else if choice.index == last_used_index && last_used.is_some() {
                last_used
            } else {
                Some(latest)
            }
        }
        Err(_) => None,
    }
}

pub fn input_player_count(db: &LatestVersionData, val: Option<u32>) -> u32 {
    val.unwrap_or_else(|| {
        let input = Text::from(HINT_PLAYER_COUNT)
            .with_default(db.last_player_count.unwrap_or(4).to_string().as_str())
            .with_validator(|v: &str| {
                if v.parse::<u32>().is_ok() {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(Custom(ERR_NEED_A_NUMBER.to_string())))
                }
            })
            .prompt();

        input
            .ok()
            .and_then(|str| str.parse::<u32>().ok())
            .unwrap_or_else(|| default_config::COUNT)
    })
}

pub fn input_blast_path(db: &LatestVersionData, val: Option<PathBuf>, hint: &str) -> PathBuf {
    val.or(db.blast_path.clone()).unwrap_or_else(|| {
        if let Some(home_path) = home_dir() {
            home_path
        } else {
            let input = Text::from(hint)
                .with_validator(|v: &str| {
                    if v.parse::<PathBuf>().is_ok() {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid(Custom(ERR_INVALID_PATH.to_string())))
                    }
                })
                .prompt();

            match input {
                Ok(p) => p.parse::<PathBuf>().unwrap(),
                Err(_) => PathBuf::new(),
            }
        }
    })
}

pub fn input_count_or_index(val: Option<u32>, single: bool) -> u32 {
    val.unwrap_or_else(|| {
        let input = Text::from(if single {
            HINT_RUN_INDEX
        } else {
            HINT_RUN_COUNT
        })
        .with_default(1.to_string().as_str())
        .with_validator(|v: &str| {
            if v.parse::<u32>().is_ok() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(Custom(ERR_NEED_A_NUMBER.to_string())))
            }
        })
        .prompt();

        match input {
            Ok(choice) => choice.parse::<u32>().unwrap(),
            Err(_) => 1,
        }
    })
}

pub fn input_url(db: &LatestVersionData, val: Option<String>) -> Option<String> {
    val.or_else(|| {
        let mut input = Text::from(HINT_INPUT_JENKINS_URL).with_validator(|v: &str| {
            if !v.is_empty() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(Custom(
                    ERR_NEED_A_JENKINS_URL.to_string(),
                )))
            }
        });

        let existed = db
            .jenkins_url
            .clone()
            .or(if default_config::JENKINS_URL.is_empty() {
                None
            } else {
                Some(default_config::JENKINS_URL.to_string())
            });
        if existed.is_some() {
            input = input.with_default(existed.as_deref().unwrap());
        }

        let input = input.prompt().map(|v| {
            if v.ends_with("/") || v.ends_with("\\") {
                v[0..v.len() - 1].to_string()
            } else {
                v
            }
        });

        input.ok()
    })
}

pub fn input_user_name(db: &LatestVersionData, val: Option<String>) -> Option<String> {
    val.or_else(|| {
        let mut input = Text::from(HINT_INPUT_JENKINS_USERNAME).with_validator(|v: &str| {
            if !v.is_empty() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(Custom(
                    ERR_NEED_A_JENKINS_USERNAME.to_string(),
                )))
            }
        });

        let existed = db.jenkins_username.clone();
        if existed.is_some() {
            input = input.with_default(existed.as_deref().unwrap());
        }

        let input = input.prompt();

        input.ok()
    })
}

pub fn input_api_token(db: &LatestVersionData, val: Option<String>) -> String {
    val.unwrap_or_else(|| {
        let hint = formatx!(
            HINT_INPUT_JENKINS_API_TOKEN,
            db.jenkins_url.clone().unwrap(),
            db.jenkins_username.clone().unwrap()
        )
        .unwrap_or(HINT_JENKINS_API_TOKEN_DOC.to_string());

        let mut input = Text::from(hint.as_str()).with_validator(|v: &str| {
            if !v.is_empty() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(Custom(
                    ERR_NEED_A_JENKINS_API_TOKEN.to_string(),
                )))
            }
        });

        let existed = db.jenkins_api_token.clone();
        if existed.is_some() {
            input = input.with_default(existed.as_deref().unwrap());
        }

        let input = input.prompt();

        input.ok().unwrap_or_default()
    })
}

pub fn input_cookie(db: &LatestVersionData, val: Option<String>) -> String {
    val.unwrap_or_else(|| {
        let mut input = Text::from(HINT_INPUT_JENKINS_COOKIE).with_validator(|v: &str| {
            if !v.is_empty() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(Custom(
                    ERR_NEED_A_JENKINS_COOKIE.to_string(),
                )))
            }
        });

        let existed = db.jenkins_cookie.clone();
        if existed.is_some() {
            input = input.with_default(existed.as_deref().unwrap());
        }

        let input = input.prompt();

        input.ok().unwrap_or_default()
    })
}
