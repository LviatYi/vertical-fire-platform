use crate::constant;
use formatx::formatx;
use regex::Regex;
use std::fs;
use std::io::{Error, Write};
use std::path::{Path, PathBuf};

const PTN_PLACEHOLDER_FACTOR_ID: &str = "{ID}";
const PTN_PLACEHOLDER_AUTO_DETECT: &str = "{*}";
const REG_PLACEHOLDER_FACTOR_ID: &str = "REGEX_FACTOR_ID";
const REG_PLACEHOLDER_AUTO_DETECT: &str = "REGEX_AUTO_DETECT";
const REG_STR_FACTOR_ID: &str = r"(\d+)";
const REG_STR_AUTO_DETECT: &str = r".*";

pub fn remove_beginning_separator_in_relative_path(relative_path_str: &str) -> String {
    if relative_path_str.starts_with('\\') || relative_path_str.starts_with('/') {
        relative_path_str[1..].to_string()
    } else {
        relative_path_str.to_string()
    }
}

/// # Get sorted ci locators
///
/// get all main locators in the given path and sort them by ci number in descending order.
///
/// main locator is a directory name that starts with a number followed by a dash and a string.
///
/// like: `312-Hash.321312`
///
/// ## Arguments
///
/// * `path`: repo path to search
/// * `pattern`: pattern to query ci number. like: `{ID}-Hash.{*}`
///
/// returns: Vec<String, Global>
pub fn get_sorted_main_locators(path: PathBuf, pattern: &str) -> Vec<String> {
    let mut ci_package_names: Vec<String> = if let Ok(entries) = fs::read_dir(path) {
        entries
            .filter_map(|entry| match entry {
                Ok(e) => {
                    if e.path().is_dir() {
                        e.path()
                            .file_name()
                            .and_then(|v| v.to_str())
                            .map(|v| v.to_string())
                            .filter(|v| extract_ci_by_main_locator(pattern, v).is_some())
                    } else {
                        None
                    }
                }
                _err => None,
            })
            .collect()
    } else {
        vec![]
    };

    ci_package_names.sort_unstable_by(|a, b| {
        let a = extract_ci_by_main_locator(pattern, a);
        let b = extract_ci_by_main_locator(pattern, b);

        a.cmp(&b).reverse()
    });

    ci_package_names
}

/// # Extract ci by locator
///
/// get ci number from a main locator.
///
/// ## Arguments
///
/// * `pattern`: pattern to query ci number. like: `{ID}-Hash.{*}`
/// * `locator`: locator. like `312-Hash.321312`
///
/// returns: u32 like 312
pub fn extract_ci_by_main_locator(pattern: &str, locator: &str) -> Option<u32> {
    let mut regex = String::from(pattern);
    regex = regex
        .replace(PTN_PLACEHOLDER_FACTOR_ID, REG_PLACEHOLDER_FACTOR_ID)
        .replace(PTN_PLACEHOLDER_AUTO_DETECT, REG_PLACEHOLDER_AUTO_DETECT);
    regex = regex::escape(&regex);
    regex = regex
        .replace(REG_PLACEHOLDER_FACTOR_ID, REG_STR_FACTOR_ID)
        .replace(REG_PLACEHOLDER_AUTO_DETECT, REG_STR_AUTO_DETECT);
    regex = format!("^{}$", regex);

    Regex::new(regex.as_str()).ok().and_then(|re| {
        re.captures(locator)
            .and_then(|caps| caps.get(1).and_then(|v| v.as_str().parse::<u32>().ok()))
    })
}

pub fn clean_dir(dest: &Path) -> Result<Option<u128>, String> {
    if dest.is_dir() {
        let start_time = std::time::Instant::now();
        if fs::remove_dir_all(&dest).is_err() {
            Err(formatx!(
                constant::log::ERR_DIR_IN_USE,
                dest.to_str().unwrap_or(constant::log::ERR_INVALID_PATH)
            )
            .unwrap_or(constant::log::ERR_TEMPLATE_ENGINE_ERROR.to_string()))
        } else {
            let end_time = std::time::Instant::now();
            Ok(Some((end_time - start_time).as_millis()))
        }
    } else {
        Ok(None)
    }
}

pub fn extract_zip_file(from: &Path, dest: &Path) -> Result<u128, String> {
    let start_time = std::time::Instant::now();
    let _ = fs::create_dir_all(dest);

    if let Ok(zip_file) = fs::File::open(from) {
        let mut archive = zip::ZipArchive::new(zip_file).unwrap();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let out_path = match file.enclosed_name() {
                Some(path) => path,
                None => continue,
            };

            if file.is_dir() {
                fs::create_dir_all(dest.join(out_path)).unwrap();
            } else {
                if let Some(p) = out_path.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap();
                    }
                }

                let _ = fs::File::create(dest.join(out_path)).map(|mut outfile| {
                    let _ = std::io::copy(&mut file, &mut outfile);
                });
            }
        }

        let end_time = std::time::Instant::now();
        Ok((end_time - start_time).as_millis())
    } else {
        Err(constant::log::ERR_ZIP_CANNOT_OPEN.to_string())
    }
}

pub fn mending_user_ini(dest: &Path, index: u32, mend_file_path: &str) -> Result<u128, Error> {
    let start_time = std::time::Instant::now();
    let user_ini_path = dest.join(mend_file_path);
    if user_ini_path.is_file() {
        fs::OpenOptions::new()
            .append(true)
            .open(&user_ini_path)
            .and_then(|mut file| {
                file.write_all(
                    formatx!(constant::log::CONFIG_APPEND_LINE, index)
                        .unwrap_or_default()
                        .as_bytes(),
                )
            })
            .map(|_| {
                let end_time = std::time::Instant::now();
                (end_time - start_time).as_millis()
            })
    } else {
        Err(Error::new(
            std::io::ErrorKind::NotFound,
            constant::log::ERR_USER_INI_NOT_FOUNT,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::collections::HashSet;
    use std::io::Write;
    use std::panic;
    use std::panic::AssertUnwindSafe;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    #[test]
    fn test_extract_ci_by_main_locator() {
        assert_eq!(
            extract_ci_by_main_locator("{ID}-Hash.{*}", "312-Hash.321312"),
            Some(360)
        );
        assert_eq!(
            extract_ci_by_main_locator("{ID}-Hash.{*}", "-312-Hash.321312"),
            None
        );
        assert_eq!(
            extract_ci_by_main_locator("{ID}-Hash.{*}", "-CL.451065"),
            None
        );
    }

    #[test]
    fn test_get_sorted_ci_locators() {
        let temp_root_dir = tempdir().unwrap();
        let temp_root_dir_path = temp_root_dir.path().to_path_buf();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut max_ci = 0;
            let mut unique_numbers = HashSet::new();
            let max_create_count = 10;
            for _ in 0..max_create_count {
                let mut rand: u32;
                loop {
                    rand = rand::thread_rng().gen_range(1u32..=1000);
                    if unique_numbers.contains(&rand) {
                        continue;
                    } else {
                        unique_numbers.insert(rand);
                        break;
                    }
                }

                let ci_package_dir_name = temp_root_dir.path().join(format!("{}-CL.451065", rand));
                max_ci = max_ci.max(rand);
                fs::create_dir(&ci_package_dir_name).expect("create dir failed.");
            }

            let sorted_ci_package_names =
                get_sorted_main_locators(temp_root_dir.into_path(), "{ID}-Hash.{*}");
            let ci = extract_ci_by_main_locator("{ID}-Hash.{*}", &sorted_ci_package_names[0]);

            assert_eq!(ci.unwrap(), max_ci);
        }));

        fs::remove_dir_all(temp_root_dir_path).unwrap();
    }

    #[test]
    fn test_extract_file_in_zip() {
        let temp_root_dir = tempdir().unwrap();
        let temp_root_dir_path = temp_root_dir.path().to_path_buf();

        let zip_file_path = temp_root_dir_path.join("test.zip");
        let file_path = temp_root_dir_path.join("test").join("file.txt");
        let path_str = temp_root_dir_path.to_str().unwrap();
        println!("path_str: {}", path_str);
        let _ = std::io::stdout().flush();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut zip = zip::ZipWriter::new(fs::File::create(&zip_file_path).unwrap());
            zip.start_file("file.txt", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"hello world").unwrap();
            zip.finish().unwrap();

            let _ = extract_zip_file(&zip_file_path, temp_root_dir_path.as_path());

            assert_eq!(fs::read_to_string(file_path).unwrap(), "hello world");
        }));
    }
}
