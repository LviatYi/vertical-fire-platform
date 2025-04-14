/// Endpoint for retrieving information about Jenkins job.
pub struct JobInfo {
    /// Name of the Jenkins job.
    pub job_name: String,

    pub count: Option<u32>,
}

impl jenkins_sdk::Endpoint for JobInfo {
    /// HTTP method used (GET).
    fn method(&self) -> &str {
        "GET"
    }

    /// API path for retrieving job information.
    fn endpoint(&self) -> String {
        let count_str = match self.count {
            Some(count) => format!("{{0,{}}}", count),
            None => "".to_string(),
        };
        format!(
            "job/{}/api/json/?tree=builds[number]{}",
            self.job_name, count_str
        )
    }
}
