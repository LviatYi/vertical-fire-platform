use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq,Default)]
pub enum RunStatus {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAILURE")]
    Failure,
    #[serde(other)]
    #[default]
    Processing,
}
