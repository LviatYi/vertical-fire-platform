use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() {
    println!("cargo build with build.rs is running.");

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:outdir={}", out_dir);
    let root = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    println!("cargo:root={:?}", root);

    let readme_path = root.join("README.md");
    let version = env::var("CARGO_PKG_VERSION").unwrap();

    if let Ok(file) = File::open(&readme_path) {
        println!("Updating README.md with version {}", version);
        let version_regex = Regex::new(r"^v\d+\.\d+\.\d+[ab]?\s*$").unwrap();
        let buf_reader = BufReader::new(file);
        let lines: Result<Vec<_>, _> = buf_reader.lines().collect();
        let mut new_content = String::new();

        if let Ok(lines) = lines {
            for line in lines {
                if version_regex.is_match(&line) {
                    println!("Found version line: {}", line);
                    new_content.push_str(&format!("v{}  \n", version));
                } else {
                    new_content.push_str(&line);
                    new_content.push('\n');
                }
            }
        }

        if let Ok(mut file) = File::create(&readme_path) {
            file.write_all(new_content.as_bytes()).unwrap();
        }
    }

    let _ = inject_sensitive_data();
}

fn inject_sensitive_data() -> Result<(), ()> {
    let path = Path::new("src/default_config/mod.rs");
    if !path.is_file() {
        println!("File not found: {:?}", path);
        return Err(());
    }

    let recommend_job_names = env::var("RECOMMEND_JOB_NAMES").unwrap_or_default();
    let repo_template = env::var("REPO_TEMPLATE").unwrap_or_default();
    let locator_pattern = env::var("LOCATOR_PATTERN").unwrap_or_default();
    let locator_template = env::var("LOCATOR_TEMPLATE").unwrap_or_default();
    let mending_file_path = env::var("MENDING_FILE_PATH").unwrap_or_default();
    let package_file_stem = env::var("PACKAGE_FILE_STEM").unwrap_or_default();
    let exe_file_name = env::var("EXE_FILE_NAME").unwrap_or_default();
    let check_exe_file_name = env::var("CHECK_EXE_FILE_NAME").unwrap_or_default();
    let jenkins_url = env::var("JENKINS_URL").unwrap_or_default();

    let recommend_job_names = recommend_job_names
        .split([',', ';'])
        .filter(|s| !s.is_empty())
        .map(|s| format!("\t\t\"{}\"", s.trim()))
        .collect::<Vec<_>>();
    
    if repo_template.is_empty() {
        println!("ENV VARIABLE NOT SET");
    }

    let content = format!(
        "// Auto Generated by build.rs. Do not edit it manually.
// Used for sensitive data injection.

pub const COUNT: u32 = 4;
pub const RUN_COUNT: u32 = 1;

pub const RECOMMEND_JOB_NAMES: [&str; {}] = [
{}];

pub const REPO_TEMPLATE: &str = \"{}\";
pub const LOCATOR_PATTERN: &str = \"{}\";
pub const LOCATOR_TEMPLATE: &str = \"{}\";
pub const MENDING_FILE_PATH: &str = \"{}\";

pub const PACKAGE_FILE_STEM: &str = \"{}\";
pub const EXE_FILE_NAME: &str = \"{}\";
pub const CHECK_EXE_FILE_NAME: &str = \"{}\";

pub const JENKINS_URL: &str = \"{}\";
",
        recommend_job_names.len(),
        recommend_job_names.join(",\n"),
        repo_template,
        locator_pattern,
        locator_template,
        mending_file_path,
        package_file_stem,
        exe_file_name,
        check_exe_file_name,
        jenkins_url,
    );

    if let Ok(mut file) = File::create(path) {
        if file.write_all(content.as_bytes()).is_ok() {
            println!("Injected sensitive data into src/default_config/mod.rs");
            Ok(())
        } else {
            println!("Failed to inject sensitive data into src/default_config/mod.rs");
            Err(())
        }
    } else {
        println!("Failed to create src/default_config/mod.rs");
        Err(())
    }
}
