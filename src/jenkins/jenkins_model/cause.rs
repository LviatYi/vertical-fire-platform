use crate::jenkins::jenkins_model::user_id_cause::UserIdCause;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "_class")]
pub enum Cause {
    #[serde(rename = "hudson.model.Cause$UserIdCause")]
    UserId(UserIdCause),
    #[serde(rename = "com.sonyericsson.rebuild.RebuildCause")]
    Rebuild,
    #[serde(other)]
    Unknown,
}