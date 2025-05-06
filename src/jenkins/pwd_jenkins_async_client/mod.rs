use crate::jenkins::jenkins_endpoint::get_crumb::GetCrumb;
use crate::jenkins::jenkins_model::crumb::Crumb;
use base64::Engine;
use jenkins_sdk::client::AsyncClient;
use jenkins_sdk::{AsyncQuery, JenkinsError};
use reqwest::header::{HeaderValue, AUTHORIZATION};
use reqwest::Client;

/// Asynchronous Jenkins API client.
pub struct PwdJenkinsAsyncClient {
    url: String,
    username: String,
    pwd: String,
    client: Client,
    crumb: Option<Crumb>,
}

impl PwdJenkinsAsyncClient {
    /// Creates a new asynchronous Jenkins API client authentic by Password.
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL of the Jenkins server.
    /// * `username` - Username for authentication.
    /// * `pwd` - Password for authentication.
    pub fn new(url: &str, username: &str, pwd: &str) -> Self {
        Self {
            url: url.into(),
            username: username.into(),
            pwd: pwd.into(),
            client: Client::builder()
                .cookie_store(true)
                .danger_accept_invalid_certs(true)
                .no_proxy()
                .build()
                .unwrap(),
            crumb: None,
        }
    }

    pub async fn attach_crumb(&mut self) -> Result<(), JenkinsError> {
        self.crumb = Some(AsyncQuery::<Crumb>::query(&GetCrumb, self).await?);
        Ok(())
    }
}

#[async_trait::async_trait]
impl AsyncClient for PwdJenkinsAsyncClient {
    /// Sends an asynchronous HTTP request to the Jenkins server.
    async fn request(
        &self,
        method: &str,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<String, JenkinsError> {
        let url = format!("{}/{}", self.url, endpoint);
        let auth = format!(
            "BASIC {}",
            base64::prelude::BASE64_STANDARD
                .encode(format!("{}:{}", self.username, self.pwd).as_bytes())
        );

        let mut req = self
            .client
            .request(method.parse()?, url)
            .header(AUTHORIZATION, auth)
            .header("User-Agent", "jenkins-sdk-rust");

        if let Some(ref crumb) = self.crumb {
            req = req.header(
                crumb.crumb_request_field.clone(),
                HeaderValue::from_str(&crumb.crumb).map_err(|e| JenkinsError::RequestError(e.to_string()))?,
            );
        }

        let resp = if let Some(p) = params {
            req.form(&p).send().await?
        } else {
            req.send().await?
        };

        Ok(resp.text().await?)
    }
}
