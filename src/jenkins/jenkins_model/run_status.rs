use serde::Deserialize;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub enum RunStatus {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAILURE")]
    Failure,
    #[serde(other)]
    #[default]
    Processing,
}
