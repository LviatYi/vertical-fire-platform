use crate::info::jenkins_model::user_id_cause::UserIdCause;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "_class")]
pub enum Cause {
    #[serde(rename = "hudson.model.Cause$UserIdCause")]
    UserIdCause(UserIdCause),
    #[serde(rename = "com.sonyericsson.rebuild.RebuildCause")]
    RebuildCause,
    #[serde(other)]
    Unknown,
}