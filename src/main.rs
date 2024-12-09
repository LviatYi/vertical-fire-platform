mod constant;
mod default_config;
mod extract;
mod pretty_log;
mod run;

use crate::constant::log::*;
use crate::default_config::*;
use crate::extract::db;
use crate::extract::db::ExtractDb;
use crate::extract::extract_operation_info::{
    ExtractOperationInfo, OperationStatus, OperationStepType,
};
use crate::extract::extractor_util::{clean_dir, extract_zip_file, mending_user_ini};
use crate::extract::repo_decoration::RepoDecoration;
use crate::pretty_log::{pretty_log_operation_start, pretty_log_operation_status};
use crate::run::{kill_by_pid, run_instance, set_server, RunStatus};
use clap::{Parser, Subcommand};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use dirs::home_dir;
use formatx::formatx;
use inquire::validator::ErrorMessage::Custom;
use inquire::validator::Validation;
use inquire::{Select, Text};
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use std::{fs, thread};
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

        /// source root path.
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

        #[arg(short, long)]
        /// reset storage.
        reset: bool,
    },
    /// Run game instance.
    Run {
        #[arg(short, long)]
        /// target path be extracted.
        dest: Option<PathBuf>,

        /// expected instant quantity.
        #[arg(short, long)]
        count_or_index: Option<u32>,

        /// origin zip file name.
        #[arg(short = 'p', long = "package-name")]
        package_file_stem: Option<String>,

        /// executable file name.
        #[arg(short = 'e', long = "exe-name")]
        exe_file_name: Option<String>,

        /// name of executable file for check.
        #[arg(short = 'k', long = "check-name")]
        check_exe_file_name: Option<String>,

        /// run an instance by index.
        /// default: 1
        #[arg(short, long)]
        single: bool,

        /// kill existing instance.
        #[arg(short, long)]
        force: bool,

        /// run with spec server.
        /// default: http://localhost:8080
        #[arg(short = 'S', long)]
        server: Option<String>,
    },
    /// Clean cache.
    Clean,
}

fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        let command_name = command.to_string();
        show_welcome(Some(command_name.as_str()));
        let mut stdout = stdout();
        match command {
            Commands::Extract {
                branch,
                mut ci,
                count,
                build_target_repo_template,
                main_locator_pattern,
                secondary_locator_template,
                dest,
                reset,
            } => {
                let db_path = home_dir().unwrap_or_default();
                let mut db = reset
                    .then_some(ExtractDb::default())
                    .or(ExtractDb::from_path(&db_path))
                    .unwrap_or_default();

                db.b = branch.or_else(|| {
                    let mut options = vec!["Dev", "Stage", "Next"];
                    if let Some(last_used) = db.b {
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

                db.repo = Some(build_target_repo_template.unwrap_or_else(|| {
                    db.repo.clone().unwrap_or(DEFAULT_REPO_TEMPLATE.to_string())
                }));

                db.locator_pattern = Some(main_locator_pattern.unwrap_or_else(|| {
                    db.locator_pattern
                        .clone()
                        .unwrap_or(DEFAULT_LOCATOR_PATTERN.to_string())
                }));

                db.s_locator_template = Some(secondary_locator_template.unwrap_or_else(|| {
                    db.s_locator_template
                        .clone()
                        .unwrap_or(DEFAULT_LOCATOR_TEMPLATE.to_string())
                }));

                let repo_decoration = RepoDecoration::new(
                    db.repo.clone().unwrap(),
                    db.locator_pattern.clone().unwrap(),
                    db.s_locator_template.clone().unwrap(),
                    db.b.clone().unwrap().parse().unwrap_or_default(),
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

                let ci_temp = ci.unwrap_or_else(|| {
                    if let Some(latest) = ci_list.first().copied() {
                        let last_used: Option<u32> = db.ci.and_then(|v| {
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

                        options.push(format!("{}{}", latest, HINT_LATEST_CI_SUFFIX));
                        if let Some(last_used) = last_used {
                            options.push(format!("{}{}", last_used, HINT_LAST_USED_CI_SUFFIX));
                        }
                        options.push(HINT_CUSTOM.to_string());
                        let options_len = options.len();

                        let selection = Select::new(HINT_SELECT_CI, options)
                            .without_filtering()
                            .raw_prompt();

                        match selection {
                            Ok(choice) => {
                                if choice.index == options_len - 1 {
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
                                } else if choice.index == options_len - 2 && last_used.is_some() {
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
                db.ci = Some(ci_temp);

                db.c = Some(count.unwrap_or_else(|| {
                    let input = Text::from(HINT_PLAYER_COUNT)
                        .with_default(db.c.unwrap_or(4).to_string().as_str())
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

                db.d = Some(dest.or(db.d.clone()).unwrap_or_else(|| {
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

                db.save_with_error_log(&db_path);

                if let Some(path) = repo_decoration.get_full_path_by_ci(ci_temp) {
                    if let Some(file_name) = path.file_stem().and_then(|v| v.to_str()) {
                        let count = db.c.unwrap();
                        pretty_log_operation_start(&mut stdout, count);

                        let mut working_status: Vec<ExtractOperationInfo> = (0..count)
                            .map(|_| ExtractOperationInfo::default())
                            .collect();

                        let mut handles = vec![];
                        let (tx, rx) = mpsc::channel::<(u32, OperationStepType, OperationStatus)>();

                        for i in 1..count + 1 {
                            let tx = tx.clone();
                            let dest_with_origin_name =
                                db.d.clone()
                                    .unwrap()
                                    .as_path()
                                    .join(format!("{}{}", file_name, i));
                            let path_t = path.clone();
                            let handle = thread::spawn(move || {
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

                                                let mend_res =
                                                    mending_user_ini(&dest_with_origin_name, i);

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
                                let _ = pretty_log_operation_status(&mut stdout, i, count, item);
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

                                let _ = pretty_log_operation_status(
                                    &mut stdout,
                                    index - 1,
                                    count,
                                    item,
                                );
                            }
                            thread::sleep(Duration::from_millis(50));
                        }

                        for handle in handles {
                            handle.join().expect("Thread panicked");
                        }
                    } else {
                        let _ = execute!(
                            stdout,
                            SetForegroundColor(Color::Red),
                            Print(format!(
                                "{}\n",
                                formatx!(ERR_INVALID_PATH).unwrap_or_default()
                            ))
                        );
                    }
                } else {
                    let _ = execute!(
                        stdout,
                        SetForegroundColor(Color::Red),
                        Print(format!(
                            "{}\n",
                            formatx!(ERR_NO_SPECIFIED_PACKAGE).unwrap_or_default()
                        ))
                    );
                }
                let _ = execute!(stdout, ResetColor);
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
                let db_path = home_dir().unwrap_or_default();
                let dest = dest
                    .or(ExtractDb::from_path(&db_path).and_then(|c| c.d))
                    .unwrap_or_else(|| {
                        if let Some(home_path) = home_dir() {
                            home_path
                        } else {
                            let input = Text::from(HINT_SET_PACKAGE_NEED_EXTRACT_HOME_PATH)
                                .with_validator(|v: &str| {
                                    if v.parse::<PathBuf>().is_ok() {
                                        Ok(Validation::Valid)
                                    } else {
                                        Ok(Validation::Invalid(Custom(
                                            ERR_INVALID_PATH.to_string(),
                                        )))
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
                    package_file_stem.unwrap_or(DEFAULT_PACKAGE_FILE_STEM.to_string());
                let exe_file_name =
                    exe_file_name.unwrap_or(DEFAULT_CHECK_EXE_FILE_NAME.to_string());
                let check_exe_file_name =
                    check_exe_file_name.unwrap_or(DEFAULT_CHECK_EXE_FILE_NAME.to_string());

                if single {
                    if let Some(server) = server {
                        if let Err(e) =
                            set_server(&dest, &package_file_name, count_or_index, &server)
                        {
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
                            if let Err(e) = set_server(&dest, &package_file_name, i, &server) {
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
            Commands::Clean => {
                let db_path = home_dir().unwrap_or_default();
                let _ = db_path
                    .is_dir()
                    .then(|| fs::remove_file(db_path.join(db::DB_FILE_NAME)));
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
    use std::fs;
    use std::fs::create_dir_all;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    #[test]
    fn lab() -> std::io::Result<()> {
        // let mut stdout = stdout();
        //
        // println!("start");
        // stdout.flush();
        //
        // let mut info1 = ExtractOperationInfo {
        //     clean_state: OperationStatus::Pending,
        //     extract_state: OperationStatus::Pending,
        //     mend_state: OperationStatus::Pending,
        // };
        // let mut info2 = ExtractOperationInfo {
        //     clean_state: OperationStatus::Pending,
        //     extract_state: OperationStatus::Pending,
        //     mend_state: OperationStatus::Pending,
        // };
        // let mut info3 = ExtractOperationInfo {
        //     clean_state: OperationStatus::Pending,
        //     extract_state: OperationStatus::Pending,
        //     mend_state: OperationStatus::Pending,
        // };
        //
        // pretty_log_operation_start(&mut stdout, 3);
        //
        // pretty_log_operation_status(&mut stdout, 0, 3, &info1)?;
        // pretty_log_operation_status(&mut stdout, 1, 3, &info2)?;
        // pretty_log_operation_status(&mut stdout, 2, 3, &info3)?;
        //
        // info1.clean_state = OperationStatus::Done(100);
        // info2.clean_state = OperationStatus::Done(100);
        // info2.extract_state = OperationStatus::Done(3000);
        // info3.clean_state = OperationStatus::Done(100);
        // info3.extract_state = OperationStatus::Done(3000);
        // info3.mend_state = OperationStatus::Done(10);
        //
        // pretty_log_operation_status(&mut stdout, 0, 3, &info1)?;
        // pretty_log_operation_status(&mut stdout, 1, 3, &info2)?;
        // pretty_log_operation_status(&mut stdout, 2, 3, &info3)?;
        //
        // info1.extract_state = OperationStatus::Done(3000);
        // info1.mend_state = OperationStatus::Done(10);
        // info2.mend_state = OperationStatus::Done(10);
        //
        // pretty_log_operation_status(&mut stdout, 0, 3, &info1)?;
        // pretty_log_operation_status(&mut stdout, 1, 3, &info2)?;
        // pretty_log_operation_status(&mut stdout, 2, 3, &info3)?;
        Ok(())
    }

    #[test]
    fn test_show_welcome() {
        show_welcome(Some("test"));
    }
}