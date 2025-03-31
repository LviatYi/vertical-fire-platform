use crate::extract::branch_types::BranchTypes;
use crate::extract::extractor_util::{
    extract_ci_by_main_locator, get_sorted_main_locators,
    remove_beginning_separator_in_relative_path,
};
use std::cell::{Ref, RefCell};
use std::path::PathBuf;

/// # RepoDecoration
///
/// This structure is intended to hold an absolute path to an unpacked source file.
///
#[derive(Debug)]
pub struct RepoDecoration {
    /// # Build target repo template
    ///
    /// root path to the build target repo.  
    /// In the build target directory, specify the part before the ci information.  
    ///
    /// The `{B}` in the path will be replaced by the branch type.
    ///
    /// Example:
    ///
    /// ```text
    /// # total
    /// \\home\package-Stage\312-Hash.321312\app.zip
    ///
    /// # build_target_repo_template
    /// \\home\package-{B}
    /// ```
    build_target_repo_template: String,

    ///# Main locator pattern
    ///
    /// pattern to query identify number.
    ///
    /// Example:
    ///
    /// ```text
    /// # total
    /// \\home\package-Stage\312-Hash.321312\app.zip
    ///
    /// # main locator pattern
    /// `{ID}-Hash.{*}`
    /// ```
    main_locator_pattern: String,

    /// # Secondary locator template
    ///
    /// secondary locator template.  
    /// In the build target directory, specify the part after the ci information.  
    ///
    /// The `{B}` in the path will be replaced by the branch type.
    ///
    /// Example:
    ///
    /// ```text
    /// # total
    /// \\home\package-Stage\312-Hash.321312\app.zip
    ///
    /// # secondary locator template
    /// \app.zip
    /// ```
    secondary_locator_template: String,

    branch: BranchTypes,

    sorted_ci_package_names_cached: RefCell<Option<Vec<String>>>,

    sorted_ci_list_cached: RefCell<Option<Vec<u32>>>,
}

impl RepoDecoration {
    pub fn new(
        build_target_repo_template: String,
        main_locator_pattern: String,
        secondary_locator_template: String,
        branch: BranchTypes,
    ) -> Self {
        let secondary_locator_template =
            remove_beginning_separator_in_relative_path(secondary_locator_template.as_str());

        Self {
            build_target_repo_template,
            main_locator_pattern,
            secondary_locator_template,
            branch,
            sorted_ci_package_names_cached: RefCell::new(None),
            sorted_ci_list_cached: RefCell::new(None),
        }
    }

    pub fn assemble_build_target_repo(&self) -> PathBuf {
        PathBuf::from(
            self.build_target_repo_template
                .replace("{B}", self.branch.to_string().as_str()),
        )
    }

    fn get_cached_locator_list(&self) -> Ref<Vec<String>> {
        if self.sorted_ci_package_names_cached.borrow().is_none() {
            self.sorted_ci_package_names_cached
                .replace(Some(get_sorted_main_locators(
                    self.assemble_build_target_repo(),
                    self.main_locator_pattern.as_str(),
                )));
        }

        Ref::map(self.sorted_ci_package_names_cached.borrow(), |v| {
            v.as_ref().unwrap()
        })
    }

    pub fn get_sorted_ci_list(&self) -> Ref<Vec<u32>> {
        if self.sorted_ci_list_cached.borrow().is_none() {
            self.sorted_ci_list_cached.replace(Some(
                self.get_cached_locator_list()
                    .iter()
                    .filter_map(|v| {
                        extract_ci_by_main_locator(self.main_locator_pattern.as_str(), v)
                    })
                    .collect(),
            ));
        }

        Ref::map(self.sorted_ci_list_cached.borrow(), |v| v.as_ref().unwrap())
    }

    pub fn get_full_path_by_ci(&self, ci: u32) -> Option<PathBuf> {
        self.get_cached_locator_list()
            .iter()
            .find(|&item| {
                extract_ci_by_main_locator(self.main_locator_pattern.as_str(), item) == Some(ci)
            })
            .map(|v| {
                self.assemble_build_target_repo()
                    .join(v)
                    .join(self.secondary_locator_template.as_str())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_config::{LOCATOR_PATTERN, LOCATOR_TEMPLATE, REPO_TEMPLATE};
    use rand::Rng;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::Write;
    use std::panic::AssertUnwindSafe;
    use std::{fs, panic};
    use tempfile::tempdir;

    #[test]
    fn test_assemble_build_target_repo() {
        let r = RepoDecoration::new(
            REPO_TEMPLATE.to_string(),
            LOCATOR_PATTERN.to_string(),
            LOCATOR_TEMPLATE.to_string(),
            BranchTypes::Stage,
        );

        assert_eq!(
            r.assemble_build_target_repo(),
            PathBuf::from(REPO_TEMPLATE.replace("{B}", "Stage"))
        );
    }

    #[test]
    fn test_get_latest() {
        let temp_root_dir = tempdir().unwrap();
        let mut temp_root_dir_path = temp_root_dir.path().to_path_buf();
        temp_root_dir_path.push("Packages-Stage");
        let mut max_ci = 0;
        let mut latest: u32 = 1;

        let r = RepoDecoration::new(
            temp_root_dir_path
                .to_str()
                .unwrap()
                .to_string()
                .replace("Stage", "{B}"),
            "{ID}-Hash.{*}".to_string(),
            "\\file.md".to_string(),
            BranchTypes::Stage,
        );

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut unique_numbers = HashSet::new();
            let max_create_count = 10;
            for _ in 0..max_create_count {
                let mut rand: u32;
                loop {
                    rand = rand::thread_rng().gen_range(1u32..=1000);
                    if unique_numbers.contains(&rand) {
                        continue;
                    } else {
                        unique_numbers.insert(rand);
                        break;
                    }
                }

                let ci_package_file_name = temp_root_dir_path
                    .join(format!("{}-Hash.{}", rand, rand))
                    .join("file.md");

                fs::create_dir_all(ci_package_file_name.parent().unwrap()).unwrap();
                let mut file = File::create(ci_package_file_name).unwrap();
                writeln!(file, "CONTENT of Hash#{}", rand).unwrap();

                max_ci = max_ci.max(rand);
            }

            latest = r.get_sorted_ci_list()[0];
        }));

        fs::remove_dir_all(temp_root_dir_path).unwrap();
        assert_eq!(latest, max_ci);
    }
}
