mod constant;
mod db;
mod default_config;
mod extract;
mod interact;
mod jenkins;
mod pretty_log;
mod run;

use crate::constant::log::*;
use crate::constant::util::get_hidden_sensitive_string;
use crate::db::{delete_db_file, get_db, save_with_error_log};
use crate::extract::cli_do_extract;
use crate::interact::*;
use crate::jenkins::ci_do_watch;
use crate::jenkins::query::{
    try_get_jenkins_async_client_by_api_token, try_get_jenkins_async_client_by_cookie,
};
use crate::pretty_log::{colored_println, ThemeColor};
use crate::run::{kill_by_pid, run_instance, set_server, RunStatus};
use clap::{Parser, Subcommand};
use formatx::formatx;
use inquire::Select;
use jenkins_sdk::client::AsyncClient;
use jenkins_sdk::JenkinsError;
use std::ops::{Add, Deref};
use std::path::{Path, PathBuf};
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

        /// Cookie from Jenkins.
        /// [Unsafe] You can get it by F12 in any jenkins web page.
        #[arg(short, long)]
        cookie: Option<String>,

        /// Jenkins interested job name.
        #[arg(short, long)]
        job_name: Option<String>,
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
                    get_db(None).blast_path.as_ref(),
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
                cookie,
                job_name,
            } => {
                let mut db = get_db(None);

                db.jenkins_url = Some(input_directly_with_default(
                    url,
                    db.jenkins_url.as_ref(),
                    false,
                    HINT_INPUT_JENKINS_URL,
                    default_config::JENKINS_URL.to_string(),
                    Some(ERR_NEED_A_JENKINS_URL),
                ));

                db.jenkins_username = input_directly(
                    username,
                    db.jenkins_username.as_ref(),
                    false,
                    HINT_INPUT_JENKINS_USERNAME,
                    Some(ERR_NEED_A_JENKINS_USERNAME),
                )
                .ok();

                if db.jenkins_username.is_none() {
                    println!("{}", formatx!(ERR_NEED_A_JENKINS_USERNAME).unwrap());
                    return;
                }

                let login_method = Select::new(
                    HINT_SELECT_LOGIN_METHOD,
                    vec![LoginMethod::ApiToken, LoginMethod::Cookie],
                )
                .prompt()
                .unwrap_or(LoginMethod::ApiToken);

                let client: Result<Box<dyn AsyncClient>, JenkinsError>;

                match login_method {
                    LoginMethod::ApiToken => {
                        let hint = formatx!(
                            HINT_INPUT_JENKINS_API_TOKEN,
                            db.jenkins_url.clone().unwrap(),
                            db.jenkins_username.clone().unwrap()
                        )
                        .unwrap_or(HINT_JENKINS_API_TOKEN_DOC.to_string());

                        db.jenkins_api_token = input_directly(
                            api_token,
                            db.jenkins_api_token.as_ref(),
                            false,
                            &hint,
                            Some(ERR_NEED_A_JENKINS_API_TOKEN),
                        )
                        .ok();

                        client = try_get_jenkins_async_client_by_api_token(
                            &db.jenkins_url,
                            &db.jenkins_username,
                            &db.jenkins_api_token,
                        )
                        .await
                        .map(|v| Box::new(v) as Box<dyn AsyncClient>);
                    }
                    LoginMethod::Cookie => {
                        db.jenkins_cookie = input_directly(
                            cookie,
                            db.jenkins_cookie.as_ref(),
                            false,
                            HINT_INPUT_JENKINS_COOKIE,
                            Some(ERR_NEED_A_JENKINS_COOKIE),
                        )
                        .ok();

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
            Commands::Watch {
                job_name,
                ci,
                extract,
            } => {
                let (used_job_name, success_build_number) =
                    ci_do_watch(&mut stdout, job_name, ci).await;

                if extract {
                    let db = get_db(None);
                    if let Some(build_number) = success_build_number {
                        let job_name = used_job_name;
                        let ci = Some(build_number);
                        let count = db.last_player_count;
                        let build_target_repo_template = db.extract_repo.clone();
                        let main_locator_pattern = db.extract_locator_pattern.clone();
                        let secondary_locator_template = db.extract_s_locator_template.clone();
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
