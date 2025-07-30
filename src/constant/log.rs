pub const ERR_INPUT_INVALID: &str = "Invalid input.";
pub const ERR_INPUT_INVALID_SHOULD_NOT_BE_EMPTY: &str = "Invalid input. Should not be empty.";
pub const ERR_EMPTY_REPO: &str = "There is no any package in the repo.";
pub const ERR_NO_SPECIFIED_PACKAGE: &str = "No package found.";
pub const ERR_NEED_A_NUMBER: &str = "Need a number.";
pub const ERR_NEED_PARAM: &str = "Need {}.";
pub const ERR_NEED_SHELVED: &str = "Need shelved change numbers.";
pub const ERR_INVALID_PATH: &str = "Invalid path.";
pub const ERR_INVALID_PATH_NOT_EXIST: &str = "Invalid path: not exist.";
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
pub const ERR_NEED_A_JENKINS_PWD: &str = "Need your jenkins password.";
pub const ERR_JENKINS_CLIENT_INVALID: &str =
    "Cannot connect to jenkins. Maybe you should check your password or api token.";
pub const ERR_JENKINS_CLIENT_INVALID_SIMPLE: &str = "Cannot connect to jenkins.";
pub const ERR_JENKINS_CLIENT_INVALID_MAY_BE_API_TOKEN_INVALID: &str =
    "Url: {}.\nUsername: {}\nApi-Token: {}\nMaybe you should check your api token.";
pub const ERR_JENKINS_CLIENT_INVALID_MAY_BE_PWD_INVALID: &str =
    "Url: {}.\nUsername: {}\nPassword: {}\nMaybe you should check your password.";
pub const ERR_JENKINS_CLIENT_GET_CRUMB_FAILED: &str = "Failed to get crumb. {}";
pub const ERR_NO_VALID_RUN_TASK: &str =
    "There is no valid run task (in progress, failure or success).";
pub const ERR_TOAST_SHOW_FAILED: &str = "Failed to show toast notification.";
pub const ERR_NEED_EVEN_PARAM: &str = "You must provide sufficient parameters.";
pub const ERR_QUERY_JOB_CONFIG: &str = "Failed to query job config. {}";
pub const ERR_REQUEST_BUILD_FAILED: &str = "Failed to request a build task. {}";
pub const ERR_VERSION_PARSE_FAILED: &str = "Failed to parse version. {}";
pub const ERR_VERSION_NOT_SUPPORT_UPDATE: &str =
    "This version does not support update. Please upgrade to a newer version great than {}.";
pub const ERR_UPDATE_FAILED: &str = "Update failed. {}";

pub const HINT_JOB_NAME: &str = "use job:";
pub const HINT_PLAYER_COUNT: &str = "use player count: ";
pub const HINT_GLOBAL_LATEST_IN_PROGRESS_CI_SUFFIX: &str = "GLOBAL latest in progress";
pub const HINT_GLOBAL_LATEST_SUCCESS_CI_SUFFIX: &str = "GLOBAL latest success";
pub const HINT_MY_LATEST_CI_SUFFIX: &str = "USER latest success {}";
pub const HINT_NO_MY_LATEST_CI_SUFFIX: &str = "there is no success run task in {}";
pub const HINT_MY_LATEST_IN_PROGRESS_CI_SUFFIX: &str = "({} in progress)";
pub const HINT_MY_LATEST_FAIL_CI_SUFFIX: &str = "({} failed)";
pub const HINT_LAST_USED_SUFFIX: &str = "last used";
pub const HINT_CUSTOM: &str = "Custom (tap enter to input)";
pub const HINT_NOT_SET: &str = "Do not set";
pub const HINT_SELECT_CI: &str = "use ci:";
pub const HINT_INPUT_CUSTOM: &str = "use custom: ";
pub const HINT_EXTRACT_TO: &str = "extract to: ";
pub const HINT_SET_PACKAGE_NEED_EXTRACT_HOME_PATH: &str = "game package home path: ";
pub const HINT_RUN_COUNT: &str = "run instance count: ";
pub const HINT_INPUT_JENKINS_URL: &str = "input the jenkins url: ";
pub const HINT_SELECT_LOGIN_METHOD: &str = "choose your login method: ";
pub const HINT_INPUT_JENKINS_USERNAME: &str = "input your jenkins username(somebody@email.com): ";
pub const HINT_INPUT_JENKINS_API_TOKEN: &str =
    "get your jenkins api token at {}/user/{}/configure\ncopy to here:";
pub const HINT_INPUT_JENKINS_PWD: &str = "input your jenkins password: ";
pub const HINT_JENKINS_API_TOKEN_DOC: &str =
    "https://www.jenkins.io/doc/book/using/remote-access-api/";
pub const HINT_SELECT_CL: &str = "use change list: ";
pub const HINT_SELECT_SL: &str = "use shelved changes: ";
pub const HINT_UPGRADABLE: &str = "There is a new version available. Latest: {}. Current: {}";
pub const HINT_UPGRADE_OPERATION: &str = "Use `fp update` to upgrade to the latest version.";
pub const HINT_AUTO_UPGRADE_OPERATION: &str =
    "Use `fp update --auto-update` to enable auto update.";
pub const HINT_UPGRADE_SILENT_OPERATION: &str =
    "Use `fp update --never-check` to disable update check and hints.";
pub const HINT_USE_PARAM_OPERATION: &str =
    "Use `fp build --param <PARAM_NAME> <PARAM_VALUE>` to set the parameter value.";

pub const LOGIN_SUCCESS_BY_PWD: &str = "Login success by password!";
pub const LOGIN_SUCCESS_BY_API_TOKEN: &str = "Login success by api token!";
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
pub const QUERYING_USER_LATEST_CI: &str = "Querying user latest ci (It might take some time)...";
pub const WATCHING_RUN_TASK_IN_PROGRESS_PREPARE: &str =
    "Prepare to watching run task {} of {} in progress...";
pub const WATCHING_RUN_TASK_IN_PROGRESS: &str =
    "Watching run task {} of {} in progress...(last check at {})";
pub const NO_IN_PROGRESS_RUN_TASK_OF_USER: &str = "There is no in progress run task of yours.";
pub const WATCHING_RUN_TASK_SUCCESS: &str = "Run task {} of {} finished with SUCCESS.";
pub const WATCHING_RUN_TASK_FAILURE: &str = "Run task {} of {} finished with FAILURE.";
pub const RUN_TASK_CONSOLE_OUTPUT_URL: &str = "Console output url: {}";
pub const RUN_TASK_COMPLETED: &str = "Run Task Completed with Success.";
pub const EXTRACT_TASK_COMPLETED: &str = "All Extract Completed.";
pub const DB_BUILD_PARAM_NOT_IN_USED: &str = "Some build param from memery not in used.";
pub const DB_BUILD_PARAM_DIRECTLY_ADOPTED: &str =
    "Build param directly adopted because of template is invalid.";
pub const REQUEST_BUILD_SUCCESS: &str = "Request build success.";
pub const BUILD_USED_PARAMS: &str = "Build used params:";
pub const AUTO_FETCH_LATEST_USED_CL: &str = "Auto fetch your latest used CL: {}";
pub const AUTO_FETCH_LATEST_USED_CL_FAILED: &str =
    "Auto fetch used CL in latest build failed. The data may be invalid";
pub const AUTO_FETCH_LATEST_USED_CL_FAILED_AND_RETRY: &str =
    "Auto fetch used CL in latest build failed. Retrying...";
pub const URL_OUTPUT: &str = "Url: ";
pub const AUTO_UPDATE_ENABLED: &str = "Auto update enabled.";
pub const AUTO_UPDATE_DISABLED: &str = "Auto update disabled.";
pub const UPGRADE_TO_VERSION_SUCCESS: &str = "Upgrade to version {} success!";
pub const CURRENT_VERSION_UP_TO_DATE: &str = "Current version is up to date.";
pub const DISABLE_AUTO_UPDATE_BECAUSE_OF_UPDATE_FAILED: &str = "Auto-Update disabled.";
pub const NEVER_CHECK_VERSION: &str = "Never check version.";

pub const PARAM_JOB_NAME: &str = "job name";
pub const PARAM_DEST: &str = "path as destination";
pub const PARAM_USERNAME: &str = "your jenkins username";

pub const CONFIG_APPEND_LINE: &str = "\neadpClientIndex={}\n";
