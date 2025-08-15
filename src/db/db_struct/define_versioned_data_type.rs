#[macro_export]
macro_rules! define_versioned_data_type {
    ($name:ident, $version:expr, { $($field:tt)* }) => {
        #[derive(Deserialize, Serialize, Default, Debug)]
        pub(crate) struct $name{

            #[serde(
                skip_deserializing,
                serialize_with = "always_serialize_version",
                rename = "version",
                default
            )]
            _version: std::marker::PhantomData<()>,

            $($field)*
        }

        fn always_serialize_version<S>(
            _field: &std::marker::PhantomData<()>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            $crate::db::db_struct::version_only::serialize_version($version, _field, serializer)
        }
    };
}
