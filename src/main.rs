mod app_state;
mod cli;
mod constant;
mod db;
mod default_config;
mod extract;
mod interact;
mod jenkins;
mod pretty_log;
mod run;
mod service;
mod update;
mod vfp_error;

use crate::app_state::AppState;
use crate::cli::{cli_do_login, cli_try_first_login, input_job_name_with_err_handling};
use crate::constant::log::*;
use crate::extract::extract_params::ExtractParams;
use crate::interact::*;
use crate::jenkins::build::{VfpJobBuildParam, query_job_config, request_build};
use crate::jenkins::jenkins_model::shelves::Shelves;
use crate::jenkins::jenkins_url_factor::JenkinsUrlFactor;
use crate::jenkins::query::{VfpJenkinsClient, query_builds_in_job, query_run_info};
use crate::jenkins::util::get_jenkins_workflow_run_url;
use crate::pretty_log::{ThemeColor, colored_println};
use crate::run::{RunStatus, kill_by_pid, run_instance, set_server};
use crate::update::{do_self_update_with_log, fetch_and_try_auto_update};
use crate::vfp_error::VfpFrontError;
use clap::{Parser, Subcommand};
use formatx::formatx;
use rand::Rng;
use semver::Version;
use std::fmt::Display;
use std::io::Write;
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

        /// the Jenkins run task URL.
        #[arg(short, long)]
        url: Option<String>,
    },
    /// Run game instance.
    Run {
        /// job name.
        #[arg(short, long)]
        job_name: Option<String>,

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

        /// the Jenkins run task URL.
        #[arg(short, long)]
        url: Option<String>,
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
    /// Open memory file directly.
    Db,
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
    let cli = Cli::parse();
    let mut app_state = AppState::new(None);

    if let Some(command) = cli.command {
        let command_name = command.to_string();
        show_welcome(Some(command_name.as_str()));

        {
            let db = app_state.get_mut_db();
            let mut upgrade_info_usable = false;
            if !db.is_never_check_version()
                && let Some(version) = db.get_latest_remote_version()
            {
                let curr_version = Version::parse(env!("CARGO_PKG_VERSION"));
                if let Ok(curr_version) = curr_version
                    && version.gt(&curr_version)
                {
                    upgrade_info_usable = true;
                    show_upgradable_hit(&mut app_state.get_stdout(), version.to_string().as_str());
                }

                if !upgrade_info_usable {
                    app_state.get_mut_db().consume_update_status();
                    app_state.commit(false);
                }
            }
        }

        if let Err(err) = main_cli(&mut app_state, command).await {
            err.colored_println(&mut app_state.get_stdout());
        }

        fetch_and_try_auto_update(&mut app_state);

        show_finished(Some(command_name.as_str()));
    }
}

async fn main_cli(app_state: &mut AppState, command: Commands) -> Result<(), VfpFrontError> {
    match command {
        Commands::Extract {
            mut job_name,
            mut ci,
            extract_params,
            url,
        } => {
            // fp extract
            let url_factor = url.and_then(|str| JenkinsUrlFactor::from_url(str.as_str()).ok());
            job_name = job_name.or(url_factor
                .as_ref()
                .and_then(|factor| factor.get_job_name().map(|str| str.to_string())));
            ci = ci.or(url_factor
                .as_ref()
                .and_then(|factor| factor.get_build_number()));

            cli::cli_do_extract(app_state, job_name, ci, extract_params, false).await?;
        }
        Commands::Run {
            job_name,
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
                VfpFrontError::MissingParam(
                    formatx!(ERR_NEED_PARAM, PARAM_DEST).unwrap_or_default(),
                )
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
                if let Some(server) = server
                    && let Err(e) = set_server(
                        &dest,
                        &package_file_name,
                        count_or_index,
                        default_config::MENDING_FILE_PATH,
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
                            default_config::MENDING_FILE_PATH,
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
        }
        Commands::Login {
            url,
            username,
            api_token,
            pwd,
        } => {
            // fp login
            cli_do_login(app_state, false, url, username, api_token, pwd).await?;
            colored_println(
                &mut app_state.get_stdout(),
                ThemeColor::Success,
                JENKINS_LOGIN_RESULT,
            );
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
                return Err(VfpFrontError::Custom(ERR_NEED_EVEN_PARAM.to_string()));
            }

            cli_try_first_login(app_state, false).await?;

            let db = app_state.get_db();
            let mut client = db
                .try_get_jenkins_async_client(&mut app_state.get_stdout(), true)
                .await
                .map_err(|_| VfpFrontError::JenkinsClientInvalid)?;
            if let VfpJenkinsClient::PwdClient(ref mut client) = client {
                client.attach_crumb().await.map_err(|e| {
                    VfpFrontError::Custom(
                        formatx!(ERR_JENKINS_CLIENT_GET_CRUMB_FAILED, e.to_string())
                            .unwrap_or_default(),
                    )
                })?;
            }

            let db = app_state.get_mut_db();
            let job_name = input_job_name_with_err_handling(job_name, db)?;

            db.insert_job_name(job_name.as_str());

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

            let mut config_from_default: bool = false;
            let config_params_result =
                query_job_config(&client, &job_name).await.inspect_err(|_| {
                    config_from_default = true;
                });

            if let Err(ref e) = config_params_result {
                e.colored_println(&mut app_state.get_stdout());
            }

            let db = app_state.get_db();
            let build_params_template = config_params_result
                .map(VfpJobBuildParam::from)
                .unwrap_or_default();

            let mut build_params = build_params_template.clone();

            let mut used_cl: Option<u32> = None;
            let mut used_sl: Option<Shelves> = None;

            let db_latest_build_param = db.get_jenkins_build_param(job_name.as_ref());

            if let Some(db_params) = db_latest_build_param {
                used_cl = db_params.get_change_list();
                used_sl = db_params.get_shelve_changes();

                if build_params.from_default {
                    build_params.merge_from(db_params);

                    colored_println(
                        &mut app_state.get_stdout(),
                        ThemeColor::Warn,
                        DB_BUILD_PARAM_DIRECTLY_ADOPTED,
                    );
                    colored_println(
                        &mut app_state.get_stdout(),
                        ThemeColor::Second,
                        HINT_USE_PARAM_OPERATION,
                    );
                } else {
                    let excluded = build_params.exclusive_merge_from(db_params);
                    if !excluded.is_empty() {
                        colored_println(
                            &mut app_state.get_stdout(),
                            ThemeColor::Warn,
                            DB_BUILD_PARAM_NOT_IN_USED,
                        );

                        let mut stdout = app_state.get_stdout();
                        excluded.iter().for_each(|(k, v)| {
                            colored_println(
                                &mut stdout,
                                ThemeColor::Second,
                                &format!("{}: {}", k, v),
                            )
                        })
                    }
                }
            }

            build_params.set_change_list(input_cl(
                cl,
                &db_latest_build_param.and_then(|build_param| build_param.get_change_list()),
            )?);

            let sl = sl
                .filter(|str| !str.is_empty())
                .and_then(|v| Shelves::from_str(&v).ok());
            build_params.set_shelve_changes(input_sl(
                sl,
                &db_latest_build_param.and_then(|build_param| build_param.get_shelve_changes()),
            )?);

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

            let db = app_state.get_mut_db();
            db.set_jenkins_build_param(job_name.as_ref(), Some(build_params_to_save));
            app_state.commit(false);

            let need_query_used_cl = build_params.get_change_list().is_none();

            request_build(&client, &job_name, &build_params)
                .await
                .map_err(|e| {
                    VfpFrontError::Custom(
                        formatx!(ERR_REQUEST_BUILD_FAILED, e.to_string()).unwrap_or_default(),
                    )
                })?;

            colored_println(
                &mut app_state.get_stdout(),
                ThemeColor::Success,
                REQUEST_BUILD_SUCCESS,
            );
            colored_println(
                &mut app_state.get_stdout(),
                ThemeColor::Main,
                BUILD_USED_PARAMS,
            );

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
                colored_println(
                    &mut app_state.get_stdout(),
                    ThemeColor::Main,
                    &format!("{}: {}", k, v),
                );
            });

            let db = app_state.get_db();
            if let Ok(builds) = query_builds_in_job(&client, &job_name, Some(3))
                .await
                .map(|b| b.builds)
            {
                for build in builds {
                    if let Ok(run) = query_run_info(&client, &job_name, build.number).await
                        && run.is_mine(db.get_jenkins_username().as_ref().unwrap())
                    {
                        colored_println(
                            &mut app_state.get_stdout(),
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

            if no_watch_and_extract {
                return Ok(());
            }

            let (used_job_name, success_build_number) =
                cli::cli_do_watch(app_state, Some(job_name.clone()), None).await?;

            if let (true, Some(build_number)) = (need_query_used_cl, success_build_number) {
                let mut trial_count = 2;
                loop {
                    match query_run_info(&client, &job_name, build_number).await {
                        Ok(workflow_run) => {
                            if let Some(changelist) =
                                workflow_run.get_change_list_in_build_meta_data()
                            {
                                let db = app_state.get_mut_db();
                                let params =
                                    db.get_mut_jenkins_build_param(job_name.as_str()).unwrap();
                                params.set_change_list(Some(changelist));
                                colored_println(
                                    &mut app_state.get_stdout(),
                                    ThemeColor::Second,
                                    &formatx!(AUTO_FETCH_LATEST_USED_CL, changelist)
                                        .unwrap_or_default(),
                                );
                                app_state.commit(false);
                                break;
                            } else if trial_count == 0 {
                                colored_println(
                                    &mut app_state.get_stdout(),
                                    ThemeColor::Warn,
                                    AUTO_FETCH_LATEST_USED_CL_FAILED,
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            {
                                colored_println(
                                    &mut app_state.get_stdout(),
                                    ThemeColor::Error,
                                    &format!("Query build info failed because: {}", e),
                                );
                            }
                        }
                    }

                    colored_println(
                        &mut app_state.get_stdout(),
                        ThemeColor::Second,
                        AUTO_FETCH_LATEST_USED_CL_FAILED_AND_RETRY,
                    );

                    trial_count -= 1;
                    tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
                }
            }

            if no_extract {
                return Ok(());
            }

            if let Some(build_number) = success_build_number {
                let job_name = used_job_name;
                let ci = Some(build_number);

                cli::cli_do_extract(app_state, job_name, ci, extract_params, true).await?;
            }
        }
        Commands::Watch {
            mut job_name,
            mut ci,
            no_extract,
            extract_params,
            url,
        } => {
            // fp watch
            let url_factor = url.and_then(|str| JenkinsUrlFactor::from_url(str.as_str()).ok());
            job_name = job_name.or(url_factor
                .as_ref()
                .and_then(|factor| factor.get_job_name().map(|str| str.to_string())));
            ci = ci.or(url_factor
                .as_ref()
                .and_then(|factor| factor.get_build_number()));

            cli_try_first_login(app_state, false).await?;

            let (used_job_name, success_build_number) =
                cli::cli_do_watch(app_state, job_name, ci).await?;

            if !no_extract && let Some(build_number) = success_build_number {
                let job_name = used_job_name;
                let ci = Some(build_number);

                cli::cli_do_extract(app_state, job_name, ci, extract_params, true).await?;
            }
        }
        Commands::Update {
            auto_update,
            no_auto_update,
            never_check,
            version,
        } => {
            // fp update
            let db = app_state.get_mut_db();
            let mut want_update = true;

            if never_check {
                db.set_never_check_version(true);
                colored_println(
                    &mut app_state.get_stdout(),
                    ThemeColor::Warn,
                    NEVER_CHECK_VERSION,
                );
                want_update = false;
            }

            if no_auto_update {
                app_state.get_mut_db().set_auto_update_enabled(false);
                colored_println(
                    &mut app_state.get_stdout(),
                    ThemeColor::Main,
                    AUTO_UPDATE_DISABLED,
                );
            } else if auto_update {
                app_state.get_mut_db().set_auto_update_enabled(true);
                app_state.get_mut_db().set_never_check_version(false);
                colored_println(
                    &mut app_state.get_stdout(),
                    ThemeColor::Main,
                    AUTO_UPDATE_ENABLED,
                );
                want_update = false;
            }

            if want_update {
                app_state.get_mut_db().set_never_check_version(false);
                do_self_update_with_log(app_state, version.as_deref());
            }

            app_state.commit(false);
            return Ok(());
        }
        Commands::Clean => {
            // fp clean
            app_state.clean();
        }
        Commands::Db => app_state.open_db_file()?,
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

fn show_upgradable_hit<W: Write>(stdout: &mut W, latest_version: &str) {
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
            .ok_or_else(|| io::Error::other("no filename"))?;
        let out_path = into_dir.join(file_name);
        let mut out_file = fs::File::create(out_path)?;
        io::copy(&mut reader, &mut out_file)?;
        Ok(())
    }

    #[test]
    fn llab() {
        fn filter_even(x: &&i32) -> bool {
            *x % 2 == 0
        }

        let iters = [1, 2, 3, 4, 5].iter().filter(filter_even);

        let vec = iters.collect::<Vec<&i32>>();

        println!("{:?}", vec);
    }

    #[test]
    fn test_show_welcome() {
        show_welcome(Some("test"));
    }
}
