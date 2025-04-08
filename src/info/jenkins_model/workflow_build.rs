use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WorkflowBuild {
    pub number: u32,
}
