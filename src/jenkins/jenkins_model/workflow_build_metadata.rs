use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WorkflowBuildMetadata {
    #[serde(rename = "StringBuildMetadata")]
    StringBuildMetadata(StringBuildMetadata),
}

#[derive(Deserialize, Debug)]
pub struct StringBuildMetadata {
    pub name: String,

    #[serde(rename = "stringValue")]
    pub value: String,
}
