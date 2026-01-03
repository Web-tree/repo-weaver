use crate::config::EnsureConfig;
use anyhow::Result;
use std::path::PathBuf;

pub mod git;

use crate::state::State;

#[derive(Debug)]
pub struct EnsureContext {
    pub app_path: PathBuf,
    pub dry_run: bool,
    pub state: State,
}

#[derive(Debug)]
pub struct EnsurePlan {
    pub description: String,
}

pub trait Ensure: Send + Sync {
    fn plan(&self, ctx: &EnsureContext) -> Result<EnsurePlan>;
    fn execute(&self, ctx: &EnsureContext) -> Result<()>;
}

pub fn build_ensure(config: &EnsureConfig) -> Result<Box<dyn Ensure>> {
    match config {
        EnsureConfig::GitSubmodule { url, path, r#ref } => Ok(Box::new(git::EnsureGitSubmodule {
            url: url.clone(),
            path: PathBuf::from(path),
            ref_: r#ref.clone(),
        })),
        EnsureConfig::GitClonePinned { url, path, r#ref } => {
            Ok(Box::new(git::EnsureGitClonePinned {
                url: url.clone(),
                path: PathBuf::from(path),
                ref_: r#ref.clone(),
            }))
        }
    }
}
