use jenkins_sdk::client::AsyncClient;
use jenkins_sdk::JenkinsError;
use reqwest::header::COOKIE;
use reqwest::Client;

/// Asynchronous Jenkins API client.
pub struct CookiedJenkinsAsyncClient {
    url: String,
    cookie: String,
    client: Client,
}

impl CookiedJenkinsAsyncClient {
    /// Creates a new asynchronous Jenkins API client authentic by Cookie.
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL of the Jenkins server.
    /// * `cookie` - Cookie for authentication.
    pub fn new(url: &str, cookie: &str) -> Self {
        Self {
            url: url.into(),
            cookie: cookie.into(),
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .no_proxy()
                .build()
                .unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl AsyncClient for CookiedJenkinsAsyncClient {
    /// Sends an asynchronous HTTP request to the Jenkins server.
    async fn request(
        &self,
        method: &str,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<String, JenkinsError> {
        let url = format!("{}/{}", self.url, endpoint);
        let req = self
            .client
            .request(method.parse()?, url)
            .header(COOKIE, &self.cookie)
            .header("User-Agent", "jenkins-sdk-rust");

        let resp = if let Some(p) = params {
            req.form(&p).send().await?
        } else {
            req.send().await?
        };

        Ok(resp.text().await?)
    }
}
