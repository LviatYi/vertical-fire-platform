use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Crumb {
    #[serde(rename = "crumbRequestField")]
    pub crumb_request_field: String,

    pub crumb: String,
}
