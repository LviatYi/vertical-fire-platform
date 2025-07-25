mod cli;
mod constant;
mod db;
mod default_config;
mod extract;
mod interact;
mod jenkins;
mod pretty_log;
mod run;
mod update;
mod vfp_error;

use crate::cli::{cli_do_login, cli_try_first_login};
use crate::constant::log::*;
use crate::db::db_data_proxy::DbDataProxy;
use crate::db::{delete_db_file, get_db, save_with_error_log};
use crate::extract::extract_params::ExtractParams;
use crate::interact::*;
use crate::jenkins::build::{query_job_config, request_build, VfpJobBuildParam};
use crate::jenkins::jenkins_model::shelves::Shelves;
use crate::jenkins::query::{query_builds_in_job, query_run_info, VfpJenkinsClient};
use crate::jenkins::util::get_jenkins_workflow_run_url;
use crate::pretty_log::{colored_println, ThemeColor};
use crate::run::{kill_by_pid, run_instance, set_server, RunStatus};
use crate::update::{do_self_update_with_log, fetch_and_try_auto_update};
use crate::vfp_error::VfpError;
use clap::{Parser, Subcommand};
use formatx::formatx;
use rand::Rng;
use semver::Version;
use std::error::Error;
use std::fmt::Display;
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

        #[command(flatten)]
        extract_params: ExtractParams,
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
    /// Watch a Jenkins build task.
    Watch {
        /// job name.
        #[arg(short, long)]
        job_name: Option<String>,

        /// locator identity.
        #[arg(short = '#', long)]
        ci: Option<u32>,

        /// do not automatically extract the package after success.
        #[arg(long)]
        no_extract: bool,

        #[command(flatten)]
        extract_params: ExtractParams,
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

        /// custom build params.
        /// repeated input --param can accept multiple sets of parameters
        /// like: --param "CustomServer" "http://127.0.0.1:8080"
        #[arg(long = "param",
            num_args = 2,
            value_names = ["PARAM_NAME", "PARAM_VALUE"],
            action = clap::ArgAction::Append
        )]
        params: Vec<String>,

        /// do not automatically watch and extract the package after success.
        #[arg(long)]
        no_watch_and_extract: bool,

        /// do not automatically extract the package after success.
        #[arg(long)]
        no_extract: bool,

        #[command(flatten)]
        extract_params: ExtractParams,
    },
    /// Upgrade to the latest version.
    Update {
        #[arg(long, conflicts_with("no_auto_update"), conflicts_with("never_check"))]
        auto_update: bool,

        #[arg(long, conflicts_with("auto_update"))]
        no_auto_update: bool,

        #[arg(long, conflicts_with("auto_update"))]
        never_check: bool,

        #[arg(short, long)]
        version: Option<String>,
    },
    /// Clean cache.
    Clean,
    #[cfg(debug_assertions)]
    /// Show debug info.
    Debug,
}

#[derive(Debug)]
enum LoginMethod {
    Pwd,
    ApiToken,
}

impl Display for LoginMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginMethod::Pwd => write!(f, "Password"),
            LoginMethod::ApiToken => write!(f, "API Token"),
        }
    }
}

#[tokio::main]
async fn main() {
    let mut stdout = std::io::stdout();
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        let command_name = command.to_string();
        show_welcome(Some(command_name.as_str()));

        {
            let mut db = get_db(None);
            let mut upgrade_info_usable = false;
            if !db.is_never_check_version() {
                if let Some(version) = db.get_latest_remote_version() {
                    let curr_version = Version::parse(env!("CARGO_PKG_VERSION"));
                    if let Ok(curr_version) = curr_version {
                        if version.gt(&curr_version) {
                            upgrade_info_usable = true;
                            show_upgradable_hit(&mut stdout, version.to_string().as_str());
                        }
                    }

                    if !upgrade_info_usable {
                        db.consume_update_status();
                        save_with_error_log(&db, None);
                    }
                }
            }
        }

        match main_cli(command, &mut stdout).await {
            Ok(_) => {}
            Err(err) => match &err {
                e @ VfpError::RunTaskBuildFailed { log, run_url, .. } => {
                    e.colored_println(&mut stdout);

                    colored_println(&mut stdout, ThemeColor::Main, log.as_str());
                    colored_println(
                        &mut stdout,
                        ThemeColor::Warn,
                        formatx!(RUN_TASK_CONSOLE_OUTPUT_URL, run_url)
                            .unwrap_or_default()
                            .as_str(),
                    );
                }
                _ => {
                    err.colored_println(&mut stdout);
                }
            },
        }

        fetch_and_try_auto_update(&mut stdout);

        show_finished(Some(command_name.as_str()));
    }
}

async fn main_cli(command: Commands, stdout: &mut std::io::Stdout) -> Result<(), VfpError> {
    match command {
        Commands::Extract {
            job_name,
            ci,
            extract_params,
        } => {
            // fp extract
            cli::cli_do_extract(stdout, job_name, ci, extract_params, false).await?;
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
            // fp run
            let dest = input_path(
                dest,
                get_db(None).get_blast_path().as_ref(),
                true,
                HINT_SET_PACKAGE_NEED_EXTRACT_HOME_PATH,
                false,
                true,
                Some(ERR_INVALID_PATH),
            )
            .map_err(|_| {
                VfpError::MissingParam(formatx!(ERR_NEED_PARAM, PARAM_DEST).unwrap_or_default())
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
            // fp login
            let mut db: DbDataProxy = get_db(None);
            cli_do_login(&mut db, false, url, username, api_token, pwd).await?;
            colored_println(stdout, ThemeColor::Success, JENKINS_LOGIN_RESULT);
        }
        Commands::Build {
            job_name,
            cl,
            sl,
            params,
            no_extract,
            no_watch_and_extract,
            extract_params,
        } => {
            // fp build
            if params.len() % 2 != 0 {
                return Err(VfpError::Custom(ERR_NEED_EVEN_PARAM.to_string()));
            }

            let mut db = get_db(None);

            cli_try_first_login(&mut db, Some(stdout)).await?;

            let mut client = db
                .try_get_jenkins_async_client(stdout, true)
                .await
                .map_err(|_| VfpError::JenkinsClientInvalid)?;
            if let VfpJenkinsClient::PwdClient(ref mut client) = client {
                client.attach_crumb().await.map_err(|e| {
                    VfpError::Custom(
                        formatx!(ERR_JENKINS_CLIENT_GET_CRUMB_FAILED, e.to_string())
                            .unwrap_or_default(),
                    )
                })?;
            }

            db.set_interest_job_name(Some(
                input_job_name(job_name, db.get_interest_job_name())
                    .map_err(|_| VfpError::MissingParam(PARAM_JOB_NAME.to_string()))?,
            ));

            let param_pairs: Vec<(String, serde_json::Value)> = params
                .chunks(2)
                .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
                .map(|(k, v)| {
                    if let Ok(val) = v.parse::<bool>() {
                        (k, serde_json::Value::Bool(val))
                    } else {
                        (k, serde_json::Value::String(v))
                    }
                })
                .collect();

            let job_name = db.get_interest_job_name().clone().unwrap();

            let config_params = query_job_config(&client, &job_name).await.map_err(|e| {
                VfpError::Custom(formatx!(ERR_QUERY_JOB_CONFIG, e.to_string()).unwrap_or_default())
            })?;

            let build_params_template = VfpJobBuildParam::from(config_params);
            let mut build_params = build_params_template.clone();

            let mut used_cl: Option<u32> = None;
            let mut used_sl: Option<Shelves> = None;

            if let Some(ref db_params) = db.get_jenkins_build_param() {
                used_cl = db_params.get_change_list();
                used_sl = db_params.get_shelve_changes();

                let excluded = build_params.exclusive_merge_from(db_params);
                if !excluded.is_empty() {
                    colored_println(stdout, ThemeColor::Warn, DB_BUILD_PARAM_NOT_IN_USED);

                    excluded.iter().for_each(|(k, v)| {
                        colored_println(stdout, ThemeColor::Second, &format!("{}: {}", k, v))
                    })
                }
            }

            build_params.set_change_list(input_cl(
                cl,
                &(db.get_jenkins_build_param()
                    .as_ref()
                    .and_then(|db| db.get_change_list())),
            ));

            let sl = sl
                .filter(|str| !str.is_empty())
                .and_then(|v| Shelves::from_str(&v).ok());
            build_params.set_shelve_changes(input_sl(
                sl,
                &(db.get_jenkins_build_param()
                    .as_ref()
                    .and_then(|db| db.get_shelve_changes())),
            ));

            param_pairs.into_iter().for_each(|(k, v)| {
                build_params.params.insert(k, v);
            });

            let mut build_params_to_save = build_params.clone();
            build_params_to_save.retain_differing_params(&build_params_template);

            if build_params_to_save.get_change_list().is_none() {
                build_params_to_save.set_change_list(used_cl);
            }
            if build_params_to_save.get_shelve_changes().is_none() {
                build_params_to_save.set_shelve_changes(used_sl);
            }

            db.set_jenkins_build_param(Some(build_params_to_save));
            save_with_error_log(&db, None);

            let need_query_used_cl = build_params.get_shelve_changes().is_none();

            request_build(&client, &job_name, &build_params)
                .await
                .map_err(|e| {
                    VfpError::Custom(
                        formatx!(ERR_REQUEST_BUILD_FAILED, e.to_string()).unwrap_or_default(),
                    )
                })?;
            colored_println(stdout, ThemeColor::Success, REQUEST_BUILD_SUCCESS);

            colored_println(stdout, ThemeColor::Main, BUILD_USED_PARAMS);

            let mut sorted_params_for_show: Vec<(&String, &serde_json::Value)> =
                build_params.params.iter().collect();

            sorted_params_for_show.sort_by(|&(lk, lv), &(rk, rv)| {
                if (lv.is_string() && rv.is_string()) || (lv.is_boolean() && rv.is_boolean()) {
                    lk.cmp(rk)
                } else if lv.is_string() {
                    std::cmp::Ordering::Less
                } else if rv.is_string() {
                    std::cmp::Ordering::Greater
                } else if lv.is_boolean() {
                    std::cmp::Ordering::Less
                } else if rv.is_boolean() {
                    std::cmp::Ordering::Greater
                } else {
                    lk.cmp(rk)
                }
            });
            sorted_params_for_show.iter().for_each(|(k, v)| {
                colored_println(stdout, ThemeColor::Main, &format!("{}: {}", k, v));
            });

            if let Ok(builds) = query_builds_in_job(&client, &job_name, Some(3))
                .await
                .map(|b| b.builds)
            {
                for build in builds {
                    if let Ok(run) = query_run_info(&client, &job_name, build.number).await {
                        if run.is_mine(db.get_jenkins_username().as_ref().unwrap()) {
                            colored_println(
                                stdout,
                                ThemeColor::Second,
                                &format!(
                                    "{} {}",
                                    URL_OUTPUT,
                                    get_jenkins_workflow_run_url(
                                        db.get_jenkins_url().as_ref().unwrap(),
                                        &job_name,
                                        build.number,
                                    )
                                ),
                            );
                            break;
                        }
                    }
                }
            }

            if no_watch_and_extract {
                return Ok(());
            }

            let (used_job_name, success_build_number) =
                cli::cli_do_watch(stdout, Some(job_name.clone()), None).await?;

            if let (true, Some(build_number)) = (need_query_used_cl, success_build_number) {
                if let Ok(workflow_run) = query_run_info(&client, &job_name, build_number).await {
                    if let Some(changelist) = workflow_run.get_change_list_in_build_meta_data() {
                        let params = db.get_mut_jenkins_build_param().unwrap();
                        colored_println(
                            stdout,
                            ThemeColor::Second,
                            &formatx!(AUTO_FETCH_LATEST_USED_CL, changelist).unwrap_or_default(),
                        );
                        params.set_change_list(Some(changelist));
                        save_with_error_log(&db, None);
                    } else {
                        colored_println(stdout, ThemeColor::Warn, AUTO_FETCH_LATEST_USED_CL_FAILED);
                    }
                }
            }

            if no_extract {
                return Ok(());
            }

            if let Some(build_number) = success_build_number {
                let job_name = used_job_name;
                let ci = Some(build_number);

                cli::cli_do_extract(stdout, job_name, ci, extract_params, true).await?;
            }
        }
        Commands::Watch {
            job_name,
            ci,
            no_extract,
            extract_params,
        } => {
            // fp watch
            cli_try_first_login(&mut get_db(None), Some(stdout)).await?;

            let (used_job_name, success_build_number) =
                cli::cli_do_watch(stdout, job_name, ci).await?;

            if !no_extract {
                if let Some(build_number) = success_build_number {
                    let job_name = used_job_name;
                    let ci = Some(build_number);

                    cli::cli_do_extract(stdout, job_name, ci, extract_params, true).await?;
                }
            }
        }
        Commands::Update {
            auto_update,
            no_auto_update,
            never_check,
            version,
        } => {
            // fp update
            let mut db = get_db(None);
            let mut want_update = true;

            if no_auto_update {
                db.set_auto_update_enabled(false);
                colored_println(stdout, ThemeColor::Main, AUTO_UPDATE_DISABLED);
                want_update = false;
            }

            if never_check {
                db.set_never_check_version(true);
                colored_println(stdout, ThemeColor::Warn, NEVER_CHECK_VERSION);
                want_update = false;
            }

            if !want_update {
                save_with_error_log(&db, None);
                return Ok(());
            }

            db.set_never_check_version(false);
            if auto_update {
                db.set_auto_update_enabled(true);
                colored_println(stdout, ThemeColor::Main, AUTO_UPDATE_ENABLED);
            }

            do_self_update_with_log(stdout, &mut db, version.as_deref());
        }
        Commands::Clean => {
            // fp clean
            delete_db_file(None);
        }
        #[cfg(debug_assertions)]
        Commands::Debug => {
            // fp debug
            println!("Debug info:");

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
            println!(
                "QUERY_TOKEN_GITHUB: {:#?}",
                default_config::QUERY_TOKEN_GITHUB
            );
        }
    }

    Ok(())
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

fn show_upgradable_hit(stdout: &mut std::io::Stdout, latest_version: &str) {
    colored_println(
        stdout,
        ThemeColor::Main,
        formatx!(HINT_UPGRADABLE, latest_version, env!("CARGO_PKG_VERSION"))
            .unwrap_or_default()
            .as_str(),
    );

    colored_println(stdout, ThemeColor::Second, HINT_UPGRADE_OPERATION);
    let mut rng = rand::rng();
    if rng.random_range(0..100) < 10 {
        // 10%
        colored_println(stdout, ThemeColor::Second, HINT_AUTO_UPGRADE_OPERATION);
    } else if rng.random_range(0..100) < 56 {
        // ~50%
        colored_println(stdout, ThemeColor::Second, HINT_UPGRADE_SILENT_OPERATION);
    }
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
    use std::{fs, io};

    #[test]
    fn lab() -> io::Result<()> {
        let mut reader = fs::File::open("C:\\Workspace\\self-tools\\bin\\temp\\fp.exe")?;
        let into_dir = PathBuf::from("C:\\Workspace\\self-tools\\bin\\temp");
        let file_to_extract = PathBuf::from("fp.exe");
        match fs::create_dir_all(&into_dir) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                    return Err(e);
                }
            }
        }
        let file_name = file_to_extract
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no filename"))?;
        let out_path = into_dir.join(file_name);
        let mut out_file = fs::File::create(out_path)?;
        io::copy(&mut reader, &mut out_file)?;
        Ok(())
    }

    #[test]
    fn test_show_welcome() {
        show_welcome(Some("test"));
    }
}
