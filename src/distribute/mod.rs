use crate::app_state::AppState;
use crate::constant::log::{
    DISTRIBUTE_SUCCESS, ERR_COPY_FOR_DISTRIBUTE_FAILED, ERR_DEST_PATH_NOT_EXIST,
    ERR_SRC_PT_NOT_EXIST,
};
use crate::pretty_log::colored_println;
use crate::pretty_log::ThemeColor::{Success, Warn};
use crate::vfp_error::VfpFrontError;
use formatx::formatx;
use std::collections::HashMap;
use std::path::Path;

pub fn infer_blast_root_dir_name(path: impl AsRef<Path>) -> Option<(String, Vec<String>)> {
    let path = path.as_ref();
    if !path.is_dir() {
        return None;
    }

    let mut dirs: HashMap<String, Vec<String>> = HashMap::new();

    for entry in std::fs::read_dir(path).ok()?.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_dir() {
            continue;
        }

        let raw = entry.file_name();
        let name = raw.to_string_lossy();

        let prefix = name.trim_end_matches(|c: char| c.is_ascii_digit());

        if !prefix.is_empty() && prefix.len() < name.len() {
            dirs.entry(prefix.to_string())
                .or_default()
                .push(name.to_string());
        }
    }

    dirs.into_iter().max_by_key(|(_, dirs)| dirs.len())
}

pub fn distribute_pt(
    app_state: &mut AppState,
    src_pt_path: impl AsRef<Path>,
    dest_pt_paths: Vec<impl AsRef<Path>>,
) -> Result<(), VfpFrontError> {
    let src_pt_path = src_pt_path.as_ref();
    if !src_pt_path.is_file() {
        return Err(VfpFrontError::DistributeError(
            formatx!(ERR_SRC_PT_NOT_EXIST, src_pt_path.to_string_lossy()).unwrap_or_default(),
        ));
    }

    for dest_pt_path in dest_pt_paths {
        let dest_pt_path = dest_pt_path.as_ref();

        if let Some(parent) = dest_pt_path.parent()
            && parent.is_dir()
        {
            if let Err(e) = std::fs::copy(src_pt_path, dest_pt_path) {
                colored_println(
                    &mut app_state.get_stdout(),
                    Warn,
                    &formatx!(
                        ERR_COPY_FOR_DISTRIBUTE_FAILED,
                        dest_pt_path.to_string_lossy(),
                        e
                    )
                        .unwrap_or_default(),
                );
            } else {
                colored_println(
                    &mut app_state.get_stdout(),
                    Success,
                    &formatx!(
                        DISTRIBUTE_SUCCESS,
                        src_pt_path.to_string_lossy(),
                        dest_pt_path.to_string_lossy()
                    )
                        .unwrap_or_default(),
                )
            }
        } else {
            colored_println(
                &mut app_state.get_stdout(),
                Warn,
                &formatx!(ERR_DEST_PATH_NOT_EXIST, dest_pt_path.to_string_lossy())
                    .unwrap_or_default(),
            );
        }
    }

    Ok(())
}
