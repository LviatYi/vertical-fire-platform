mod constant;
mod db;
mod default_config;
mod extract;
mod info;
mod pretty_log;
mod run;

use crate::constant::log::*;
use crate::constant::util::get_hidden_sensitive_string;
use crate::db::{delete_db_file, get_db, save_with_error_log};
use crate::extract::extract_operation_info::{
    ExtractOperationInfo, OperationStatus, OperationStepType,
};
use crate::extract::extractor_util::{clean_dir, extract_zip_file, mending_user_ini};
use crate::extract::repo_decoration::RepoDecoration;
use crate::info::query::{
    query_user_latest_success_info, try_get_jenkins_async_client,
    try_get_jenkins_async_client_by_api_token, try_get_jenkins_async_client_by_cookie,
};
use crate::pretty_log::colored_println;
use crate::run::{kill_by_pid, run_instance, set_server, RunStatus};
use clap::{Parser, Subcommand};
use crossterm::execute;
use crossterm::style::Color;
use dirs::home_dir;
use formatx::formatx;
use inquire::validator::ErrorMessage::Custom;
use inquire::validator::Validation;
use inquire::{Select, Text};
use jenkins_sdk::client::AsyncClient;
use jenkins_sdk::JenkinsError;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::Duration;
use strum_macros::Display;

#[derive(Parser)]
#[command(name="Vertical Fire Platform", author, version, about(env!("CARGO_PKG_DESCRIPTION")), long_about=None,arg_required_else_help=true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Display)]
enum Commands {
    /// Extract ci build package.
    Extract {
        /// branch name.
        #[arg(short, long)]
        branch: Option<String>,

        /// locator identity.
        #[arg(short = '#', long)]
        ci: Option<u32>,

        /// expected quantity.
        #[arg(short, long)]
        count: Option<u32>,

        /// build target repo path.
        #[arg(long = "repo")]
        build_target_repo_template: Option<String>,

        /// main locator pattern.
        #[arg(long = "locator-pattern")]
        main_locator_pattern: Option<String>,

        #[arg(long = "s-locator-template")]
        /// secondary locator template.
        secondary_locator_template: Option<String>,

        #[arg(short, long)]
        /// target path to be extracted.
        dest: Option<PathBuf>,
    },
    /// Run game instance.
    Run {
        #[arg(short, long)]
        /// target path be extracted.
        dest: Option<PathBuf>,

        /// expected instant quantity.
        #[arg(short, long)]
        count_or_index: Option<u32>,

        /// package name.
        #[arg(short = 'p', long = "package-name")]
        package_file_stem: Option<String>,

        /// executable file name.
        #[arg(short = 'e', long = "exe-name")]
        exe_file_name: Option<String>,

        /// name of executable file for check.
        #[arg(short = 'k', long = "check-name")]
        check_exe_file_name: Option<String>,

        /// run an instance by index.
        #[arg(short, long)]
        single: bool,

        /// kill existing instance.
        #[arg(short, long)]
        force: bool,

        /// run with spec server.
        /// default: localhost
        #[arg(
            short = 'S',
            long,
            value_name = "URL",
            num_args = 0..=1,
            require_equals = false,
            default_missing_value = "localhost"
        )]
        server: Option<String>,
    },
    /// Login to Jenkins to get more information about build tasks.
    Login {
        /// Jenkins root URL.
        #[arg(long)]
        url: Option<String>,

        /// Username like "somebody@email.com"
        #[arg(short, long)]
        username: Option<String>,

        /// API token from Jenkins.
        /// You can get it from Jenkins web page.
        /// See also: https://www.jenkins.io/doc/book/using/remote-access-api/
        #[arg(short, long)]
        api_token: Option<String>,

        /// Cookie from Jenkins.
        /// [Unsafe] You can get it by F12 in any jenkins web page.
        #[arg(short, long)]
        cookie: Option<String>,

        /// Jenkins interested job name.
        #[arg(short, long)]
        job_name: Option<String>,
    },
    /// Clean cache.
    Clean,
}

#[derive(Debug, Display)]
enum LoginMethod {
    ApiToken,
    Cookie,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        let command_name = command.to_string();
        show_welcome(Some(command_name.as_str()));

        let mut stdout = std::io::stdout();
        match command {
            Commands::Extract {
                branch,
                mut ci,
                count,
                build_target_repo_template,
                main_locator_pattern,
                secondary_locator_template,
                dest,
            } => {
                let mut db = get_db(None);

                db.branch = branch.or_else(|| {
                    let mut options = vec!["Dev", "Stage", "Next"];
                    if let Some(last_used) = db.branch {
                        if let Some(v) = options.iter_mut().position(|&mut v| v == last_used) {
                            options.swap(0, v);
                        }
                    }

                    let selection = Select::new(HINT_BRANCH, options).prompt();

                    match selection {
                        Ok(choice) => Some(choice.to_string()),
                        Err(_) => Some("Dev".to_string()),
                    }
                });

                db.extract_repo = Some(build_target_repo_template.unwrap_or_else(|| {
                    db.extract_repo
                        .clone()
                        .unwrap_or(default_config::REPO_TEMPLATE.to_string())
                }));

                db.extract_locator_pattern = Some(main_locator_pattern.unwrap_or_else(|| {
                    db.extract_locator_pattern
                        .clone()
                        .unwrap_or(default_config::LOCATOR_PATTERN.to_string())
                }));

                db.extract_s_locator_template =
                    Some(secondary_locator_template.unwrap_or_else(|| {
                        db.extract_s_locator_template
                            .clone()
                            .unwrap_or(default_config::LOCATOR_TEMPLATE.to_string())
                    }));

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

                    match client {
                        Ok(client) => {
                            let user_latest_info = query_user_latest_success_info(
                                &client,
                                &job_name,
                                &(db.jenkins_username.clone().unwrap()),
                                None,
                            )
                            .await;

                            if let Ok(Some(info)) = user_latest_info {
                                latest_mine_ci = Some(info.number);
                            }
                        }
                        Err(_) => {
                            println!("{}", ERR_JENKINS_CLIENT_INVALID);
                        }
                    }
                }

                let ci_temp = ci.unwrap_or_else(|| {
                    if let Some(latest) = ci_list.first().copied() {
                        let last_used: Option<u32> = db.last_inner_version.and_then(|v| {
                            if ci_list
                                .binary_search_by(|probe| probe.cmp(&v).reverse())
                                .is_ok()
                            {
                                Some(v)
                            } else {
                                None
                            }
                        });
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
                                    let input = Text::from(HINT_SET_CUSTOM_CI)
                                        .with_validator(move |v: &str| {
                                            if let Ok(ci) = v.parse::<u32>() {
                                                if ci_list_clone_for_inquire
                                                    .binary_search_by(|probe| {
                                                        probe.cmp(&ci).reverse()
                                                    })
                                                    .is_ok()
                                                {
                                                    Ok(Validation::Valid)
                                                } else {
                                                    Ok(Validation::Invalid(Custom(
                                                        ERR_NO_SPECIFIED_PACKAGE.to_string(),
                                                    )))
                                                }
                                            } else {
                                                Ok(Validation::Invalid(Custom(
                                                    ERR_NEED_A_NUMBER.to_string(),
                                                )))
                                            }
                                        })
                                        .prompt();

                                    input.unwrap().to_string().parse::<u32>().unwrap()
                                } else if choice.index == latest_mine_opt_index
                                    && latest_mine_ci.is_some()
                                {
                                    latest_mine_ci.unwrap()
                                } else if choice.index == last_used_index && last_used.is_some() {
                                    last_used.unwrap()
                                } else {
                                    latest
                                }
                            }
                            Err(_) => 0,
                        }
                    } else {
                        0
                    }
                });
                if ci_temp == 0 {
                    println!("{}", ERR_EMPTY_REPO);
                    return;
                }

                db.last_inner_version = Some(ci_temp);

                db.last_player_count = Some(count.unwrap_or_else(|| {
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

                    match input {
                        Ok(choice) => choice.parse::<u32>().unwrap(),
                        Err(_) => 4,
                    }
                }));

                db.blast_path = Some(dest.or(db.blast_path.clone()).unwrap_or_else(|| {
                    if let Some(home_path) = home_dir() {
                        home_path
                    } else {
                        let input = Text::from(HINT_EXTRACT_TO)
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
                }));

                save_with_error_log(&db, None);

                if let Some(path) = repo_decoration.get_full_path_by_ci(ci_temp) {
                    if let Some(file_name) = path.file_stem().and_then(|v| v.to_str()) {
                        let count = db.last_player_count.unwrap();
                        let pty_logger = pretty_log::VfpPrettyLogger::apply_for(&mut stdout, count);

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
                                    &mut stdout,
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
                                    &mut stdout,
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
            Commands::Run {
                dest,
                count_or_index,
                package_file_stem,
                exe_file_name,
                check_exe_file_name,
                single,
                force,
                server,
            } => {
                let dest = dest.or(get_db(None).blast_path).unwrap_or_else(|| {
                    if let Some(home_path) = home_dir() {
                        home_path
                    } else {
                        let input = Text::from(HINT_SET_PACKAGE_NEED_EXTRACT_HOME_PATH)
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
                });

                let count_or_index = count_or_index.unwrap_or_else(|| {
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
                });

                let package_file_name =
                    package_file_stem.unwrap_or(default_config::PACKAGE_FILE_STEM.to_string());
                let exe_file_name =
                    exe_file_name.unwrap_or(default_config::EXE_FILE_NAME.to_string());
                let check_exe_file_name =
                    check_exe_file_name.unwrap_or(default_config::CHECK_EXE_FILE_NAME.to_string());

                if single {
                    if let Some(server) = server {
                        if let Err(e) = set_server(
                            &dest,
                            &package_file_name,
                            count_or_index,
                            default_config::MENDING_FILE_PATH,
                            &server,
                        ) {
                            println!("{}", e);
                        }
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
                        if let Some(server) = server.clone() {
                            if let Err(e) = set_server(
                                &dest,
                                &package_file_name,
                                i,
                                &default_config::MENDING_FILE_PATH,
                                &server,
                            ) {
                                println!("{}", e);
                            }
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
            }
            Commands::Login {
                url,
                username,
                api_token,
                cookie,
                job_name,
            } => {
                let mut db = get_db(None);

                db.jenkins_url = url.or_else(|| {
                    let mut input = Text::from(HINT_INPUT_JENKINS_URL).with_validator(|v: &str| {
                        if !v.is_empty() {
                            Ok(Validation::Valid)
                        } else {
                            Ok(Validation::Invalid(Custom(
                                ERR_NEED_A_JENKINS_URL.to_string(),
                            )))
                        }
                    });

                    let existed =
                        db.jenkins_url
                            .clone()
                            .or(if default_config::JENKINS_URL.is_empty() {
                                None
                            } else {
                                Some(default_config::JENKINS_URL.to_string())
                            });
                    if existed.is_some() {
                        input = input.with_default(existed.as_deref().unwrap());
                    }

                    let input = input.prompt();

                    input.ok()
                });

                let login_method = Select::new(
                    HINT_SELECT_LOGIN_METHOD,
                    vec![LoginMethod::ApiToken, LoginMethod::Cookie],
                )
                .prompt()
                .unwrap_or(LoginMethod::ApiToken);

                let client: Result<Box<dyn AsyncClient>, JenkinsError>;

                match login_method {
                    LoginMethod::ApiToken => {
                        db.jenkins_username = username.or_else(|| {
                            let mut input = Text::from(HINT_INPUT_JENKINS_USERNAME).with_validator(
                                |v: &str| {
                                    if !v.is_empty() {
                                        Ok(Validation::Valid)
                                    } else {
                                        Ok(Validation::Invalid(Custom(
                                            ERR_NEED_A_JENKINS_USERNAME.to_string(),
                                        )))
                                    }
                                },
                            );

                            let existed = db.jenkins_username.clone();
                            if existed.is_some() {
                                input = input.with_default(existed.as_deref().unwrap());
                            }

                            let input = input.prompt();

                            input.ok()
                        });

                        db.jenkins_api_token = api_token.or_else(|| {
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

                            let input = input.prompt().map(|v| {
                                if v.ends_with("/") || v.ends_with("\\") {
                                    v[0..v.len() - 1].to_string()
                                } else {
                                    v
                                }
                            });

                            input.ok()
                        });

                        if db.jenkins_url.is_none() {
                            println!("{}", formatx!(ERR_NEED_A_JENKINS_URL).unwrap());
                            return;
                        }

                        if db.jenkins_username.is_none() {
                            println!("{}", formatx!(ERR_NEED_A_JENKINS_USERNAME).unwrap());
                            return;
                        }

                        if db.jenkins_api_token.is_none() {
                            println!("{}", formatx!(ERR_NEED_A_JENKINS_API_TOKEN).unwrap());
                            return;
                        }

                        client = try_get_jenkins_async_client_by_api_token(
                            &db.jenkins_url,
                            &db.jenkins_username,
                            &db.jenkins_api_token,
                        )
                        .await
                        .map(|v| Box::new(v) as Box<dyn AsyncClient>);
                    }
                    LoginMethod::Cookie => {
                        db.jenkins_cookie = cookie.or_else(|| {
                            let mut input =
                                Text::from(HINT_INPUT_JENKINS_COOKIE).with_validator(|v: &str| {
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

                            input.ok()
                        });

                        client = try_get_jenkins_async_client_by_cookie(
                            &db.jenkins_url,
                            &db.jenkins_cookie,
                        )
                        .await
                        .map(|v| Box::new(v) as Box<dyn AsyncClient>);
                    }
                }

                match client {
                    Ok(_) => {
                        let _ = colored_println(
                            &mut stdout,
                            Color::Green,
                            format!("{}", JENKINS_LOGIN_RESULT).as_str(),
                        );

                        db.jenkins_interested_job_name = job_name.or_else(|| {
                            let mut input = Text::from(HINT_INPUT_JENKINS_JOB_NAME).with_validator(
                                |v: &str| {
                                    if !v.is_empty() {
                                        Ok(Validation::Valid)
                                    } else {
                                        Ok(Validation::Invalid(Custom(
                                            ERR_NEED_A_JENKINS_JOB_NAME.to_string(),
                                        )))
                                    }
                                },
                            );

                            let existed = db.jenkins_interested_job_name.clone();
                            if existed.is_some() {
                                input = input.with_default(existed.as_deref().unwrap());
                            }

                            let input = input.prompt();

                            input.ok()
                        });

                        save_with_error_log(&db, None);
                    }
                    Err(e) => {
                        let err_msg = match login_method {
                            LoginMethod::ApiToken => {
                                formatx!(
                                    ERR_JENKINS_CLIENT_INVALID_MAY_BE_API_TOKEN_INVALID,
                                    db.jenkins_url.clone().unwrap(),
                                    db.jenkins_username.clone().unwrap(),
                                    get_hidden_sensitive_string(
                                        &db.jenkins_api_token.clone().unwrap()
                                    ),
                                    e.to_string()
                                )
                            }
                            LoginMethod::Cookie => {
                                formatx!(
                                    ERR_JENKINS_CLIENT_INVALID_MAY_BE_COOKIE_INVALID,
                                    db.jenkins_url.clone().unwrap(),
                                    get_hidden_sensitive_string(
                                        &db.jenkins_cookie.clone().unwrap()
                                    ),
                                    e.to_string()
                                )
                            }
                        }
                        .unwrap_or_default();

                        let err_msg = ERR_JENKINS_CLIENT_INVALID_SIMPLE.to_string().add(&err_msg);
                        println!("{}", err_msg)
                    }
                }
            }
            Commands::Clean => {
                delete_db_file(None);
            }
        }

        show_finished(Some(command_name.as_str()));
    }
}

fn show_welcome(title: Option<&str>) {
    let title = if let Some(t) = title {
        format!("| {}", t.to_uppercase())
    } else {
        String::new()
    };

    println!(
        "⠄⠄⠄V-F Platform {} ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼",
        title
    );
}

fn show_finished(title: Option<&str>) {
    let title = if let Some(t) = title {
        format!("| {}", t.to_uppercase())
    } else {
        String::new()
    };

    println!(
        "⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄All Finished {} ⠄⠄⠄",
        title
    );
}

fn run_instance_with_log(
    home_path: &Path,
    package_name: &str,
    exe_file_name: &str,
    check_exe_file_name: &str,
    index: u32,
    force: bool,
) {
    let mut max_retry = 3;
    while max_retry > 0 {
        match run_instance(
            home_path,
            package_name,
            exe_file_name,
            check_exe_file_name,
            index,
        ) {
            RunStatus::Running(pids) => {
                if !force {
                    println!(
                        "{}",
                        formatx!(ERR_ALREADY_RUNNING, index).unwrap_or_default()
                    );
                    return;
                } else {
                    println!("{}", OPERATION_KILL_AND_RETRY);
                    for pid in pids {
                        let _ = kill_by_pid(pid);
                    }

                    std::thread::sleep(Duration::from_millis(300));
                }
            }
            RunStatus::Create => {
                println!("{}", formatx!(RESULT_RUN, index).unwrap_or_default());
                return;
            }
            RunStatus::NotExist => {
                println!(
                    "{}",
                    formatx!(ERR_RUN_PACKAGE_NOT_FOUND, index).unwrap_or_default()
                );
                return;
            }
        };
        max_retry -= 1;
    }

    println!(
        "{}",
        formatx!(ERR_ALREADY_RUNNING, index).unwrap_or_default()
    );
    println!(
        "{}",
        formatx!(ERR_FAILED_TO_KILL_PROCESS, index).unwrap_or_default()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab() -> std::io::Result<()> {
        Ok(())
    }

    #[test]
    fn test_show_welcome() {
        show_welcome(Some("test"));
    }
}
