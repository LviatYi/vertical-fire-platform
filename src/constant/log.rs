pub const ERR_EMPTY_REPO: &str = "There is no any package in the repo.";
pub const ERR_NO_SPECIFIED_PACKAGE: &str = "No package found.";
pub const ERR_NEED_A_NUMBER: &str = "Need a number.";
pub const ERR_INVALID_PATH: &str = "Invalid path.";
pub const ERR_DIR_IN_USE: &str = "Maybe {} is in use. skip.";
pub const ERR_TEMPLATE_ENGINE_ERROR: &str = "Template engine error.";
pub const ERR_ZIP_CANNOT_OPEN: &str = "Cannot open the zip file.";
pub const ERR_USER_INI_NOT_FOUNT: &str = "user.ini not found.";
pub const ERR_ALREADY_RUNNING: &str = "Instance {} is running. Skip.";
pub const ERR_RUN_PACKAGE_NOT_FOUND: &str =
    "Instance {} is not exist. Please extract to here first.";
pub const ERR_WMIC_FAILED: &str = "Failed to execute wmic.";
pub const ERR_WHEN_WRITE_USER_INI: &str = "When write user.ini, error occurred.";
pub const ERR_FAILED_TO_KILL_PROCESS_WITH_PID: &str = "Failed to kill process with PID {}.";
pub const ERR_FAILED_TO_KILL_PROCESS: &str = "Failed to kill process.";
pub const ERR_UPGRADE_NOT_DEFINED: &str = "Upgrade not defined.";
pub const ERR_DB_SAVE_FAILURE: &str = "Archive storage failure: {}";
pub const ERR_NEED_A_JENKINS_URL: &str = "Need a jenkins url.";
pub const ERR_NEED_A_JENKINS_USERNAME: &str = "Need your jenkins username.";
pub const ERR_NEED_A_JENKINS_API_TOKEN: &str = "Need your jenkins api token.";
pub const ERR_JENKINS_CLIENT_INVALID: &str =
    "Cannot connect to jenkins.\nUrl: {}.\nUsername: {}\nMay be you should check your api token.\nErr: {}";

pub const HINT_BRANCH: &str = "use branch:";
pub const HINT_PLAYER_COUNT: &str = "use player count: ";
pub const HINT_LATEST_CI_SUFFIX: &str = "(latest)";
pub const HINT_LAST_USED_CI_SUFFIX: &str = "(last used)";
pub const HINT_CUSTOM: &str = "Custom";
pub const HINT_SELECT_CI: &str = "use ci:";
pub const HINT_SET_CUSTOM_CI: &str = "use custom ci #";
pub const HINT_EXTRACT_TO: &str = "extract to: ";
pub const HINT_SET_PACKAGE_NEED_EXTRACT_HOME_PATH: &str = "game package home path: ";
pub const HINT_RUN_INDEX: &str = "run instance index: ";
pub const HINT_RUN_COUNT: &str = "run instance count: ";
pub const HINT_INPUT_JENKINS_URL: &str = "input the jenkins url: ";
pub const HINT_INPUT_JENKINS_USERNAME: &str = "input your jenkins username(somebody@email.com): ";
pub const HINT_INPUT_JENKINS_API_TOKEN: &str =
    "get your jenkins api token at https://{}/user/{}/configure\ncopy to here:";
pub const HINT_JENKINS_API_TOKEN_DOC: &str =
    "https://www.jenkins.io/doc/book/using/remote-access-api/";

pub const OPERATION_TITLE: &str = "Work at index {}.";
pub const OPERATION_FINISHED: &str = "Finished at index {}.";
pub const OPERATION_FAILED: &str = "Failed at index {}.";
pub const OPERATION_ALL_COST: &str = "All cost {}ms.";
pub const OPERATION_CLEAN: &str = "Cleaning.";
pub const RESULT_CLEAN: &str = "Clean {}ms.";
pub const OPERATION_EXTRACT: &str = "Extracting.";
pub const RESULT_EXTRACT: &str = "Extract {}ms.";
pub const OPERATION_MEND: &str = "Mending.";
pub const RESULT_MEND: &str = "Mend {}ms.";
pub const OPERATION_RUN_CHECK: &str = "Checking {}";
pub const RESULT_RUN: &str = "Instance {} is created.";
pub const OPERATION_KILL_AND_RETRY: &str = "Killing and retrying...";
pub const JENKINS_LOGIN_RESULT: &str = "Jenkins login success!";

pub const CONFIG_APPEND_LINE: &str = "\neadpClientIndex={}\n";
