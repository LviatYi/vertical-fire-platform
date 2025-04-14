use serde::Deserialize;

/// ping Jenkins.
pub struct Ping;

impl jenkins_sdk::Endpoint for Ping {
    /// HTTP method used (GET).
    fn method(&self) -> &str {
        "GET"
    }

    /// API path for ping.
    fn endpoint(&self) -> String {
        "api/json?tree=ping".to_string()
    }
}

#[derive( Deserialize, Debug)]
pub struct PingResult {}