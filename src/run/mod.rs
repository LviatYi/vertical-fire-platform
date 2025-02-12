use crate::constant::log::*;
use configparser::ini::Ini;
use formatx::formatx;
use std::path::Path;
use std::process::{Command, Stdio};

pub enum RunStatus {
    Running(Vec<u32>),
    Create,
    NotExist,
}

pub fn run_instance(
    home_path: &Path,
    package_name: &str,
    exe_file_name: &str,
    check_exe_file_name: &str,
    index: u32,
) -> RunStatus {
    let work_path = home_path.join(format!("{}{}", package_name, index));
    let exe_path = work_path.join(exe_file_name);
    let check_path = work_path.join(check_exe_file_name);
    println!(
        "{}",
        formatx!(OPERATION_RUN_CHECK, exe_path.display()).unwrap_or_default()
    );

    if !exe_path.is_file() {
        RunStatus::NotExist
    } else {
        let pids = check_running(&check_path);
        if !pids.is_empty() {
            return RunStatus::Running(pids);
        };

        match Command::new(exe_path).current_dir(work_path).spawn() {
            Ok(_) => RunStatus::Create,
            Err(_) => RunStatus::NotExist,
        }
    }
}

pub fn kill_by_pid(pid: u32) -> Result<(), String> {
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(formatx!(ERR_FAILED_TO_KILL_PROCESS_WITH_PID, pid).unwrap_or_default())
    }
}

/// # Check running
///
/// check specified executable file is running or not.
///
/// ## Arguments
///
/// - `exe_path`: executable file path.
///
/// returns: Option<u32> - process id if running, None if not running.
pub fn check_running(exe_path: &Path) -> Vec<u32> {
    if let Some(p) = exe_path
        .exists()
        .then_some(exe_path)
        .and_then(|v| v.to_str())
        .map(|v| v.replace("\\", "\\\\"))
    {
        match Command::new("wmic")
            .args([
                "process",
                "where",
                format!(r#"executablepath='{}'"#, p).as_str(),
                "get",
                "ProcessId",
            ])
            .stderr(Stdio::null())
            .output()
        {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|line| line.to_string())
                    .collect::<Vec<String>>();
                result
                    .iter()
                    .map(|line| line.trim().parse::<u32>())
                    .filter_map(|item| item.ok())
                    .collect()
            }
            Err(_) => {
                println!("{}", ERR_WMIC_FAILED);
                vec![]
            }
        }
    } else {
        vec![]
    }
}

pub fn set_server(
    home_path: &Path,
    package_name: &str,
    index: u32,
    mending_file_path: &str,
    server_str: &str,
) -> Result<(), String> {
    let work_path = home_path.join(format!("{}{}", package_name, index));
    let user_ini_path = work_path.join(mending_file_path);

    let mut config = Ini::new_cs();
    if config.load(&user_ini_path).is_ok() {
        config.set_default_section("NO_TREAT_default_AS_DEFAULT");
        if server_str.is_empty() || server_str.eq("localhost") || server_str.eq("local") {
            config.remove_key("default", "hostName");
        } else {
            config.set("default", "hostName", Some(server_str.to_string()));
        }

        config
            .write(&user_ini_path)
            .map_err(|_| ERR_WHEN_WRITE_USER_INI.to_string())
    } else {
        Err(ERR_USER_INI_NOT_FOUNT.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::run::check_running;
    use std::path::PathBuf;

    #[test]
    fn test_check_running() {
        assert!(!check_running(PathBuf::from("C:\\Windows\\explorer.exe").as_path()).is_empty());
    }
}
