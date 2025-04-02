use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub enum RunStatus {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAILURE")]
    Failure,
    #[serde(other)]
    Processing,
}
