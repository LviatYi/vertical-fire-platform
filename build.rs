use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() {
    // 从环境变量获取输出目录
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
}
