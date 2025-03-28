use serde::Serialize;

#[derive(Serialize)]
pub struct VersionField {
    pub(crate) version: u32,
}
