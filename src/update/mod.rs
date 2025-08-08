use crate::constant::log::*;
use crate::db::db_data_proxy::DbDataProxy;
use crate::db::{get_db, save_with_error_log};
use crate::pretty_log::{colored_println, ThemeColor};
use crate::vfp_error::VfpError;
use crate::{default_config, update};
use formatx::formatx;
use self_update::update::UpdateStatus;
use self_update::Status;
use semver::Version;

/// # self update
///
/// Do self update.
///
/// ### Returns
///
/// Returns `Ok(None)` if the current version is up to date.
/// Returns `Ok(Some(version))` if the update was successful, where `version` is the new version.
pub fn self_update(
    specified_version: Option<&str>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner("LviatYi")
        .repo_name(env!("CARGO_PKG_NAME"))
        .bin_name("fp.exe")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .show_output(false)
        .no_confirm(true)
        .auth_token(default_config::QUERY_TOKEN_GITHUB);

    if let Some(specified_version) = specified_version {
        let specified_version_tag = if !specified_version.starts_with("v") {
            format!("v{}", specified_version)
        } else {
            specified_version.to_string()
        };

        builder.target_version_tag(&specified_version_tag);
    }

    let status = builder.build()?.update()?;

    match status {
        Status::UpToDate(_) => Ok(None),
        Status::Updated(_) => Ok(Some(status.version().to_owned())),
    }
}

/// # fetch latest version
///
/// Fetch the latest version from GitHub and compare it with the current version.
pub fn fetch_latest_version() -> Result<UpdateStatus, VfpError> {
    let curr_version_str = env!("CARGO_PKG_VERSION");
    let curr_version = Version::parse(curr_version_str)
        .map_err(|_| VfpError::VersionParseFailed(curr_version_str.to_string()))?;

    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("LviatYi")
        .repo_name(env!("CARGO_PKG_NAME"))
        .auth_token(default_config::QUERY_TOKEN_GITHUB)
        .build()?
        .fetch()?;

    if let Some(remote_latest) = releases.first() {
        let remote_version = Version::parse(&remote_latest.version)
            .map_err(|_| VfpError::VersionParseFailed(remote_latest.version.clone()))?;

        if remote_version.ge(&curr_version) {
            return Ok(UpdateStatus::Updated(releases.first().unwrap().to_owned()));
        }
    }

    Ok(UpdateStatus::UpToDate)
}

pub fn fetch_and_try_auto_update(stdout: &mut std::io::Stdout) {
    let mut db = get_db(None);

    if db.is_never_check_version() {
        return;
    }

    if let Ok(update_status) = fetch_latest_version() {
        match update_status {
            UpdateStatus::UpToDate => {
                db.set_latest_remote_version(None);
            }
            UpdateStatus::Updated(v) => {
                db.set_latest_remote_version(Some(v.version.as_str()));
            }
        }
    }

    save_with_error_log(&db, None);

    if db.has_latest_version() && db.is_auto_update_enabled() {
        colored_println(stdout, ThemeColor::Second, AUTO_UPDATE_ENABLED);

        do_self_update_with_log(stdout, &mut db, None);
    }
}

pub fn do_self_update_with_log(
    stdout: &mut std::io::Stdout,
    db: &mut DbDataProxy,
    specified_version: Option<&str>,
) {
    if let Some(specified_version) = specified_version
        && let Ok(version) = Version::parse(specified_version)
        && version.lt(&Version::parse(default_config::OLDEST_SUPPORT_UPDATE_VERSION).unwrap())
    {
        colored_println(
            stdout,
            ThemeColor::Error,
            &formatx!(
                ERR_VERSION_NOT_SUPPORT_UPDATE,
                default_config::OLDEST_SUPPORT_UPDATE_VERSION
            )
            .unwrap_or_default(),
        );
        return;
    }

    match update::self_update(specified_version) {
        Ok(update_result) => match update_result {
            None => {
                colored_println(stdout, ThemeColor::Success, CURRENT_VERSION_UP_TO_DATE);
            }
            Some(v) => {
                colored_println(
                    stdout,
                    ThemeColor::Success,
                    formatx!(UPGRADE_TO_VERSION_SUCCESS, v)
                        .unwrap_or_default()
                        .as_str(),
                );
            }
        },
        Err(e) => {
            colored_println(
                stdout,
                ThemeColor::Error,
                formatx!(ERR_UPDATE_FAILED, e.to_string())
                    .unwrap_or_default()
                    .as_str(),
            );

            colored_println(
                stdout,
                ThemeColor::Second,
                DISABLE_AUTO_UPDATE_BECAUSE_OF_UPDATE_FAILED,
            );

            db.set_auto_update_enabled(false);
        }
    }

    db.consume_update_status();
    save_with_error_log(db, None);
}
