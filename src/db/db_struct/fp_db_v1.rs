use crate::db::db_struct::fp_db_v2::FpDbV2;
use crate::db::db_struct::versioned_data::{UpgradeValue, VersionedData};
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

pub const VERSION_FP_DB_V1: u32 = 1;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct FpDbV1 {
    #[serde(
        skip_deserializing,
        serialize_with = "always_serialize_version",
        default
    )]
    version: std::marker::PhantomData<()>,

    pub b: Option<String>,

    pub ci: Option<u32>,

    pub c: Option<u32>,

    pub repo: Option<String>,

    pub locator_pattern: Option<String>,

    pub s_locator_template: Option<String>,

    pub d: Option<PathBuf>,
}

fn always_serialize_version<S>(
    _field: &std::marker::PhantomData<()>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    crate::db::db_struct::version_only::serialize_version(VERSION_FP_DB_V1, _field, serializer)
}

impl VersionedData for FpDbV1 {
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        let mut upg = FpDbV2::default();
        upg.branch = self.b;
        upg.last_inner_version = self.ci;
        upg.last_player_count = self.c;
        upg.extract_repo = self.repo;
        upg.extract_locator_pattern = self.locator_pattern;
        upg.extract_s_locator_template = self.s_locator_template;
        upg.blast_path = self.d;

        UpgradeValue::Upgraded(Box::new(upg))
    }
}
