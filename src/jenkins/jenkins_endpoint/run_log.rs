/// Endpoint for retrieving information about Jenkins run.
pub struct RunLog {
    /// Name of the Jenkins job.
    pub job_name: String,

    /// Build number of the Jenkins run.
    pub build_number: u32,
}

impl jenkins_sdk::Endpoint for RunLog {
    /// HTTP method used (GET).
    fn method(&self) -> &str {
        "GET"
    }

    /// API path for retrieving job information.
    fn endpoint(&self) -> String {
        format!("job/{}/{}/logText/progressiveText?start=0", self.job_name, self.build_number)
    }
}
