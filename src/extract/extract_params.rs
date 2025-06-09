use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ExtractParams {
    /// expected quantity.
    #[arg(short, long)]
    pub count: Option<u32>,

    #[arg(short, long)]
    /// target path to be extracted.
    pub dest: Option<PathBuf>,

    /// build target repo path.
    #[arg(long = "repo")]
    pub build_target_repo_template: Option<String>,

    /// main locator pattern.
    #[arg(long = "locator-pattern")]
    pub main_locator_pattern: Option<String>,

    #[arg(long = "s-locator-template")]
    /// secondary locator template.
    pub secondary_locator_template: Option<String>,
}