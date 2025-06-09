use crate::jenkins::jenkins_model::workflow_build::WorkflowBuild;
use serde::Deserialize;
use std::slice::{Iter, IterMut};

#[derive(Deserialize, Debug)]
pub struct WorkflowBuilds {
    pub builds: Vec<WorkflowBuild>,
}

impl<'a> IntoIterator for &'a WorkflowBuilds {
    type Item = &'a WorkflowBuild;
    type IntoIter = Iter<'a, WorkflowBuild>;

    fn into_iter(self) -> Self::IntoIter {
        self.builds.iter()
    }
}

impl<'a> IntoIterator for &'a mut WorkflowBuilds {
    type Item = &'a mut WorkflowBuild;
    type IntoIter = IterMut<'a, WorkflowBuild>;

    fn into_iter(self) -> Self::IntoIter {
        self.builds.iter_mut()
    }
}
