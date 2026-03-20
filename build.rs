const BUILD_CONFIG_ENV_KEYS: [&str; 11] = [
    "RECOMMEND_JOB_NAMES",
    "REPO_TEMPLATE",
    "LOCATOR_PATTERN",
    "LOCATOR_TEMPLATE",
    "MENDING_FILE_PATH",
    "PT_RELATIVE_PATH",
    "PACKAGE_FILE_STEM",
    "EXE_FILE_NAME",
    "CHECK_EXE_FILE_NAME",
    "JENKINS_URL",
    "QUERY_TOKEN_GITHUB",
];

fn main() {
    for key in BUILD_CONFIG_ENV_KEYS {
        println!("cargo:rerun-if-env-changed={key}");
    }
}
