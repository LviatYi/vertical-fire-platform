/// Endpoint for get crumb for authentic.
pub struct GetCrumb;

impl jenkins_sdk::Endpoint for GetCrumb {
    /// HTTP method used (GET).
    fn method(&self) -> &str {
        "GET"
    }

    /// API path for retrieving job information.
    fn endpoint(&self) -> String {
        "/crumbIssuer/api/json".to_string()
    }
}
