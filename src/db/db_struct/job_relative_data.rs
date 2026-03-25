use crate::jenkins::build::VfpJobBuildParam;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Default, Debug)]
pub(crate) struct JobRelativeData {
    pub job_name: String,

    pub last_inner_version: Option<u32>,
    pub last_player_count: Option<u32>,
    pub blast_path: Option<PathBuf>,

    pub jenkins_build_params: Option<VfpJobBuildParam>,

    pub distr_src_index: Option<u32>,
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    impl PartialEq for JobRelativeData {
        fn eq(&self, other: &Self) -> bool {
            self.job_name == other.job_name
                && self.last_inner_version == other.last_inner_version
                && self.last_player_count == other.last_player_count
                && self.blast_path == other.blast_path
                && self.jenkins_build_params == other.jenkins_build_params
                && self.distr_src_index == other.distr_src_index
        }
    }
}
