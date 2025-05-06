pub fn get_jenkins_workflow_run_url(
    jenkins_url: &str,
    job_name: &str,
    build_number: u32,
) -> String {
    format!("{}/job/{}/{}", jenkins_url, job_name, build_number)
}
