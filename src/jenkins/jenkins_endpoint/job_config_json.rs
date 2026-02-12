/// Endpoint for get job definitions and bypass config.xml.
pub struct JobConfigJson {
    /// Name of the Jenkins job.
    pub job_name: String,
}

impl jenkins_sdk::Endpoint for JobConfigJson {
    /// HTTP method used (GET).
    fn method(&self) -> &str {
        "GET"
    }

    /// API path for get job definitions.
    fn endpoint(&self) -> String {
        format!(
            "job/{}/api/json?tree=property[parameterDefinitions[name,type,description,defaultParameterValue[value],choices]]",
            self.job_name
        )
    }
}
