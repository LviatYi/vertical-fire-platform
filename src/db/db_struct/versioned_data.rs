use crate::db::db_struct::LatestVersionData;
use toml::de::Error;

type AnyVersionedData = Box<dyn VersionedData>;

pub enum UpgradeValue {
    Latest(LatestVersionData),
    Upgraded(AnyVersionedData),
    Invalid,
}

pub trait VersionedData {
    /// # Parse self to the next version data
    ///
    /// if None, the version of the current data is the latest
    fn parse_next_version(self: Box<Self>) -> UpgradeValue {
        UpgradeValue::Invalid
    }

    fn parse_from_string(content: &str) -> Result<Self, Error>
    where
        Self: Sized,
        for<'de> Self: serde::Deserialize<'de>,
    {
        toml::from_str(content)
    }
}
