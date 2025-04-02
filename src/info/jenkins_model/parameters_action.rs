use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "_class")]
pub enum ParametersAction {
    #[serde(rename = "hudson.model.StringParameterValue")]
    StringParameterValue { name: String, value: String },
    #[serde(rename = "hudson.model.BooleanParameterValue")]
    BooleanParameterValue { name: String, value: bool },
}
