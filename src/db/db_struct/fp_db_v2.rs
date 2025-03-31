use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

pub const VERSION_FP_DB_V2: u32 = 2;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct FpDbV2 {
    #[serde(
        skip_deserializing,
        serialize_with = "always_serialize_version",
        default
    )]
    version: std::marker::PhantomData<()>,

    pub branch: Option<String>,

    pub last_inner_version: Option<u32>,

    pub last_player_count: Option<u32>,

    pub extract_repo: Option<String>,

    pub extract_locator_pattern: Option<String>,

    pub extract_s_locator_template: Option<String>,

    pub blast_path: Option<PathBuf>,
}

fn always_serialize_version<S>(
    _field: &std::marker::PhantomData<()>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    crate::db::db_struct::version_only::serialize_version(VERSION_FP_DB_V2, _field, serializer)
}

impl VersionedData for FpDbV2 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        UpgradeValue::Latest(*self)
    }
}
