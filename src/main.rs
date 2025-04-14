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
use crate::jenkins::query::{
    try_get_jenkins_async_client_by_api_token, try_get_jenkins_async_client_by_cookie,
};
use crate::pretty_log::colored_println;
use crate::run::{kill_by_pid, run_instance, set_server, RunStatus};
use clap::{Parser, Subcommand};
use crossterm::style::Color;
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
                job_name,
                ci,
                count,
                build_target_repo_template,
                main_locator_pattern,
                secondary_locator_template,
                dest,
            } => {
                cli_do_extract(
                    job_name,
                    ci,
                    count,
                    build_target_repo_template,
                    main_locator_pattern,
                    secondary_locator_template,
                    dest,
                ).await;
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
                let dest =
                    input_blast_path(&get_db(None), dest, HINT_SET_PACKAGE_NEED_EXTRACT_HOME_PATH);

                let count_or_index = input_count_or_index(count_or_index, single);

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

                db.jenkins_url = input_url(&db, url);

                db.jenkins_username = input_user_name(&db, username);

                if db.jenkins_url.is_none() {
                    println!("{}", formatx!(ERR_NEED_A_JENKINS_URL).unwrap());
                    return;
                }

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
                        db.jenkins_api_token = Some(input_api_token(&db, api_token));

                        client = try_get_jenkins_async_client_by_api_token(
                            &db.jenkins_url,
                            &db.jenkins_username,
                            &db.jenkins_api_token,
                        )
                        .await
                        .map(|v| Box::new(v) as Box<dyn AsyncClient>);
                    }
                    LoginMethod::Cookie => {
                        db.jenkins_cookie = Some(input_cookie(&db, cookie));

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

                        db.interest_job_name = job_name.or_else(|| {
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

                            let existed = db.interest_job_name.clone().or(
                                if default_config::JENKINS_JOB_NAME.is_empty() {
                                    None
                                } else {
                                    Some(default_config::JENKINS_JOB_NAME.to_string())
                                },
                            );
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
