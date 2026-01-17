use anyhow::Result;
use std::path::PathBuf;

pub mod ai;
pub mod git;
pub mod plugin_wrapper;

// Core ensure types
pub struct EnsureContext {
    pub app_path: PathBuf,
    pub dry_run: bool,
    pub state: crate::state::State,
}

pub struct EnsurePlan {
    pub description: String,
    pub actions: Vec<String>,
}

pub trait Ensure {
    fn plan(&self, ctx: &EnsureContext) -> Result<EnsurePlan>;
    fn execute(&self, ctx: &EnsureContext) -> Result<()>;
}

// Builder function to create appropriate Ensure implementation from config
pub fn build_ensure(config: &crate::config::EnsureConfig) -> Result<Box<dyn Ensure>> {
    use crate::config::EnsureConfig;

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
        EnsureConfig::NpmScript { .. } | EnsureConfig::AiPatch { .. } => {
            // These require plugin/AI infrastructure that isn't set up yet
            // TODO: Implement when plugin system is ready (T018-T028 in tasks.md)
            Err(anyhow::anyhow!(
                "NpmScript and AiPatch ensures not yet implemented - requires plugin management system from Phase 3"
            ))
        }
    }
}
