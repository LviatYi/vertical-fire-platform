/// Endpoint for get config.xml about Jenkins job.
pub struct JobConfigXML {
    /// Name of the Jenkins job.
    pub job_name: String,
}

impl jenkins_sdk::Endpoint for JobConfigXML {
    /// HTTP method used (GET).
    fn method(&self) -> &str {
        "GET"
    }

    /// API path for get job config.
    fn endpoint(&self) -> String {
        format!("job/{}/config.xml", self.job_name)
    }
}
