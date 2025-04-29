mod constant;
mod db;
mod default_config;
mod extract;
mod interact;
mod jenkins;
mod pretty_log;
mod run;

use crate::constant::log::*;
use crate::constant::util::{get_hidden_sensitive_string, SensitiveMode};
use crate::db::{delete_db_file, get_db, save_with_error_log};
use crate::extract::cli_do_extract;
use crate::interact::*;
use crate::jenkins::build::{query_job_config, request_build, VfpJobBuildParam};
use crate::jenkins::ci_do_watch;
use crate::jenkins::jenkins_model::shelves::Shelves;
use crate::jenkins::query::{
    try_get_jenkins_async_client_by_api_token, try_get_jenkins_async_client_by_pwd,
};
use crate::pretty_log::{colored_println, ThemeColor};
use crate::run::{kill_by_pid, run_instance, set_server, RunStatus};
use clap::builder::TypedValueParser;
use clap::{Parser, Subcommand};
use formatx::formatx;
use inquire::Select;
use jenkins_sdk::client::AsyncClient;
use jenkins_sdk::JenkinsError;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use strum_macros::Display;

#[derive(Parser)]
#[command(name="Vertical Fire Platform",
  author,
  version,
  about(env!("CARGO_PKG_DESCRIPTION")),
  long_about=None,
  arg_required_else_help=true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Display)]
enum Commands {
    /// Extract ci build package.
    Extract {
        /// job name.
        #[arg(short, long)]
        job_name: Option<String>,

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
        count: Option<u32>,

        /// expected instant index.
        #[arg(short, long)]
        index: Option<u32>,

        /// package name.
        #[arg(short = 'p', long = "package-name")]
        package_file_stem: Option<String>,

        /// executable file name.
        #[arg(short = 'e', long = "exe-name")]
        exe_file_name: Option<String>,

        /// name of executable file for check.
        #[arg(short = 'k', long = "check-name")]
        check_exe_file_name: Option<String>,

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

        /// Password of Jenkins.
        #[arg(short, long)]
        pwd: Option<String>,
    },
    /// Request start a Jenkins build task.
    Build {
        /// job name.
        #[arg(short, long)]
        job_name: Option<String>,

        /// change list number.
        #[arg(long)]
        cl: Option<u32>,

        /// shelved change list numbers.
        /// separated by ,
        #[arg(long)]
        sl: Option<String>,

        /// Custom build params.
        /// Repeated input --param can accept multiple sets of parameters
        /// like: --param "CustomServer" "http://127.0.0.1:8080"
        #[arg(long = "param",
            num_args = 2,
            value_names = ["PARAM_NAME", "PARAM_VALUE"],
            action = clap::ArgAction::Append
        )]
        params: Vec<String>,
    },
    /// Watch a Jenkins build task.
    Watch {
        /// job name.
        #[arg(short, long)]
        job_name: Option<String>,

        /// locator identity.
        #[arg(short = '#', long)]
        ci: Option<u32>,

        /// automatically extract the package after success.
        #[arg(short, long)]
        extract: bool,
    },
    /// Clean cache.
    Clean,
    /// Show debug info.
    Debug,
}

#[derive(Debug, Display)]
enum LoginMethod {
    Pwd,
    ApiToken,
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
                job_name,
                ci,
                count,
                build_target_repo_template,
                main_locator_pattern,
                secondary_locator_template,
                dest,
            } => {
                cli_do_extract(
                    &mut stdout,
                    job_name,
                    ci,
                    count,
                    build_target_repo_template,
                    main_locator_pattern,
                    secondary_locator_template,
                    dest,
                )
                .await;
            }
            Commands::Run {
                dest,
                count,
                index,
                package_file_stem,
                exe_file_name,
                check_exe_file_name,
                force,
                server,
            } => {
                let dest = input_path(
                    dest,
                    get_db(None).get_blast_path().as_ref(),
                    true,
                    HINT_SET_PACKAGE_NEED_EXTRACT_HOME_PATH,
                    false,
                    true,
                    Some(ERR_INVALID_PATH),
                );

                if dest.is_err() {
                    println!("{}", ERR_INPUT_INVALID);
                    return;
                }
                let dest = dest.unwrap();

                let single = index.is_some();

                let count_or_index = index.or(count).unwrap_or_else(|| {
                    input_directly_with_default(
                        None,
                        None,
                        false,
                        HINT_RUN_COUNT,
                        default_config::RUN_COUNT,
                        Some(ERR_NEED_A_NUMBER),
                    )
                });

                let package_file_name = parse_without_input_with_default(
                    package_file_stem,
                    None,
                    default_config::PACKAGE_FILE_STEM,
                );
                let exe_file_name = parse_without_input_with_default(
                    exe_file_name,
                    None,
                    default_config::EXE_FILE_NAME,
                );
                let check_exe_file_name = parse_without_input_with_default(
                    check_exe_file_name,
                    None,
                    default_config::CHECK_EXE_FILE_NAME,
                );

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
                pwd,
            } => {
                let mut db = get_db(None);

                db.set_jenkins_url(Some(input_directly_with_default(
                    url,
                    db.get_jenkins_url().as_ref(),
                    false,
                    HINT_INPUT_JENKINS_URL,
                    default_config::JENKINS_URL.to_string(),
                    Some(ERR_NEED_A_JENKINS_URL),
                )));

                db.set_jenkins_username(
                    input_directly(
                        username,
                        db.get_jenkins_username().as_ref(),
                        false,
                        HINT_INPUT_JENKINS_USERNAME,
                        Some(ERR_NEED_A_JENKINS_USERNAME),
                    )
                    .ok(),
                );

                if db.get_jenkins_username().is_none() {
                    println!("{}", formatx!(ERR_NEED_A_JENKINS_USERNAME).unwrap());
                    return;
                }

                save_with_error_log(&db, None);

                let login_method = Select::new(
                    HINT_SELECT_LOGIN_METHOD,
                    vec![LoginMethod::Pwd, LoginMethod::ApiToken],
                )
                .prompt()
                .unwrap_or(LoginMethod::ApiToken);

                let client: Result<Box<dyn AsyncClient>, JenkinsError>;

                match login_method {
                    LoginMethod::ApiToken => {
                        let hint = formatx!(
                            HINT_INPUT_JENKINS_API_TOKEN,
                            db.get_jenkins_url().clone().unwrap(),
                            db.get_jenkins_username().clone().unwrap()
                        )
                        .unwrap_or(HINT_JENKINS_API_TOKEN_DOC.to_string());

                        db.set_jenkins_api_token(
                            input_directly(
                                api_token,
                                db.get_jenkins_api_token().as_ref(),
                                false,
                                &hint,
                                Some(ERR_NEED_A_JENKINS_API_TOKEN),
                            )
                            .ok(),
                        );

                        client = try_get_jenkins_async_client_by_api_token(
                            &db.get_jenkins_url(),
                            &db.get_jenkins_username(),
                            &db.get_jenkins_api_token(),
                        )
                        .await
                        .map(|v| Box::new(v) as Box<dyn AsyncClient>);
                    }
                    LoginMethod::Pwd => {
                        db.set_jenkins_pwd(
                            input_pwd(pwd, HINT_INPUT_JENKINS_PWD, Some(ERR_NEED_A_JENKINS_PWD))
                                .ok(),
                        );

                        client = try_get_jenkins_async_client_by_pwd(
                            &db.get_jenkins_url(),
                            &db.get_jenkins_username(),
                            &db.get_jenkins_pwd(),
                        )
                        .await
                        .map(|v| Box::new(v) as Box<dyn AsyncClient>);
                    }
                }

                match client {
                    Ok(_) => {
                        colored_println(
                            &mut stdout,
                            ThemeColor::Success,
                            format!("{}", JENKINS_LOGIN_RESULT).as_str(),
                        );

                        save_with_error_log(&db, None);
                    }
                    Err(e) => {
                        let err_msg = match login_method {
                            LoginMethod::ApiToken => {
                                formatx!(
                                    ERR_JENKINS_CLIENT_INVALID_MAY_BE_API_TOKEN_INVALID,
                                    db.get_jenkins_url().clone().unwrap(),
                                    db.get_jenkins_username().clone().unwrap(),
                                    get_hidden_sensitive_string(
                                        &db.get_jenkins_api_token().clone().unwrap(),
                                        SensitiveMode::Normal(4)
                                    ),
                                    e.to_string()
                                )
                            }
                            LoginMethod::Pwd => {
                                formatx!(
                                    ERR_JENKINS_CLIENT_INVALID_MAY_BE_PWD_INVALID,
                                    db.get_jenkins_url().clone().unwrap(),
                                    get_hidden_sensitive_string(
                                        &db.get_jenkins_pwd().clone().unwrap(),
                                        SensitiveMode::Full
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
            Commands::Build {
                job_name,
                cl,
                sl,
                params,
            } => {
                if params.len() % 2 != 0 {
                    colored_println(&mut stdout, ThemeColor::Error, ERR_NEED_EVEN_PARAM);
                    return;
                }

                let mut db = get_db(None);
                if let Ok(val) = input_job_name(job_name, db.get_interest_job_name()) {
                    db.set_interest_job_name(Some(val));
                } else {
                    println!("{}", ERR_NEED_A_JOB_NAME);
                    return;
                }

                let param_pairs: Vec<(String, serde_json::Value)> = params
                    .chunks(2)
                    .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
                    .map(|(k, v)| {
                        if v.eq("true") {
                            (k, serde_json::Value::Bool(true))
                        } else if v.eq("false") {
                            (k, serde_json::Value::Bool(false))
                        } else {
                            (k, serde_json::Value::String(v))
                        }
                    })
                    .collect();

                let client = db.try_get_jenkins_async_client(&mut stdout, true).await;
                if let Ok(client) = client {
                    let job_name = db.get_interest_job_name().clone().unwrap();
                    match query_job_config(&client, &job_name).await {
                        Ok(recommend_params) => {
                            let mut params = VfpJobBuildParam::from(recommend_params);

                            if let Some(val) = input_cl(
                                cl,
                                &(db.get_jenkins_build_param()
                                    .as_ref()
                                    .and_then(|db| db.get_change_list())),
                            ) {
                                params.set_change_list(val);
                            }

                            let sl = sl.and_then(|v| Shelves::from_str(&v).ok());
                            if let Some(val) = input_sl(
                                sl,
                                &(db.get_jenkins_build_param()
                                    .as_ref()
                                    .and_then(|db| db.get_shelve_changes())),
                            ) {
                                params.set_shelve_changes(val);
                            }
                            
                            param_pairs.into_iter().for_each(|(k, v)| {
                                params.params.insert(k, v);
                            });

                            match request_build(&client, &job_name, &params).await {
                                Ok(_) => {
                                    colored_println(
                                        &mut stdout,
                                        ThemeColor::Success,
                                        REQUEST_BUILD_SUCCESS,
                                    );

                                    colored_println(
                                        &mut stdout,
                                        ThemeColor::Main,
                                        BUILD_USED_PARAMS,
                                    );

                                    params.params.iter().for_each(|(k, v)| {
                                        colored_println(
                                            &mut stdout,
                                            ThemeColor::Main,
                                            &format!("{}: {}", k, v),
                                        );
                                    })
                                }
                                Err(e) => {
                                    colored_println(
                                        &mut stdout,
                                        ThemeColor::Error,
                                        &formatx!(ERR_REQUEST_BUILD_FAILED, e.to_string())
                                            .unwrap_or_default(),
                                    );
                                    return;
                                }
                            };
                        }
                        Err(e) => {
                            colored_println(
                                &mut stdout,
                                ThemeColor::Error,
                                &formatx!(ERR_QUERY_JOB_CONFIG, e.to_string()).unwrap_or_default(),
                            );
                            return;
                        }
                    };

                    //TODO_LviatYi 后续操作：-w -e
                } else {
                    colored_println(&mut stdout, ThemeColor::Error, ERR_JENKINS_CLIENT_INVALID);
                    return;
                }
            }
            Commands::Watch {
                job_name,
                ci,
                extract,
            } => {
                let (used_job_name, success_build_number) =
                    ci_do_watch(&mut stdout, job_name, ci).await;

                if extract {
                    if let Some(build_number) = success_build_number {
                        let db = get_db(None);
                        let job_name = used_job_name;
                        let ci = Some(build_number);
                        let count = *db.get_last_player_count();
                        let build_target_repo_template = db.get_extract_repo().clone();
                        let main_locator_pattern = db.get_extract_locator_pattern().clone();
                        let secondary_locator_template =
                            db.get_extract_s_locator_template().clone();
                        let dest = None;

                        cli_do_extract(
                            &mut stdout,
                            job_name,
                            ci,
                            count,
                            build_target_repo_template,
                            main_locator_pattern,
                            secondary_locator_template,
                            dest,
                        )
                        .await;
                    }
                }
            }
            Commands::Clean => {
                delete_db_file(None);
            }
            Commands::Debug => {
                println!("Debug info:");
                println!("COUNT: {:#?}", default_config::COUNT);
                println!("RUN_COUNT: {:#?}", default_config::RUN_COUNT);
                println!(
                    "RECOMMEND_JOB_NAMES: {:#?}",
                    default_config::RECOMMEND_JOB_NAMES
                );
                println!("REPO_TEMPLATE: {:#?}", default_config::REPO_TEMPLATE);
                println!("LOCATOR_PATTERN: {:#?}", default_config::LOCATOR_PATTERN);
                println!("LOCATOR_TEMPLATE: {:#?}", default_config::LOCATOR_TEMPLATE);
                println!(
                    "MENDING_FILE_PATH: {:#?}",
                    default_config::MENDING_FILE_PATH
                );
                println!(
                    "PACKAGE_FILE_STEM: {:#?}",
                    default_config::PACKAGE_FILE_STEM
                );
                println!("EXE_FILE_NAME: {:#?}", default_config::EXE_FILE_NAME);
                println!(
                    "CHECK_EXE_FILE_NAME: {:#?}",
                    default_config::CHECK_EXE_FILE_NAME
                );
                println!("JENKINS_URL: {:#?}", default_config::JENKINS_URL);
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
