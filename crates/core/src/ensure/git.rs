use crate::ensure::{Ensure, EnsureContext, EnsurePlan};
use anyhow::Result;
use repo_weaver_ops::git;
use std::path::PathBuf;

pub struct EnsureGitSubmodule {
    pub url: String,
    pub path: PathBuf,
    pub ref_: String,
}

impl Ensure for EnsureGitSubmodule {
    fn plan(&self, _ctx: &EnsureContext) -> Result<EnsurePlan> {
        Ok(EnsurePlan {
            description: format!(
                "Ensure git submodule {} at {:?} is {}",
                self.url, self.path, self.ref_
            ),
        })
    }

    fn execute(&self, ctx: &EnsureContext) -> Result<()> {
        let target_path = ctx.app_path.join(&self.path);

        // Safe Checkout check
        if target_path.exists() {
            // If it's a repo, check if dirty
            if target_path.join(".git").exists() {
                if !git::is_worktree_clean(&target_path)? {
                    return Err(anyhow::anyhow!(
                        "Target path {:?} has dirty working tree. Safe Checkout aborted.",
                        target_path
                    ));
                }
            }
        }

        if ctx.dry_run {
            return Ok(());
        }

        if git::is_submodule_registered(&target_path)? {
            git::submodule_sync_init_update(&target_path, &self.ref_)
        } else {
            git::submodule_add(&self.url, &target_path, &self.ref_)
        }
    }
}

pub struct EnsureGitClonePinned {
    pub url: String,
    pub path: PathBuf,
    pub ref_: String,
}

impl Ensure for EnsureGitClonePinned {
    fn plan(&self, _ctx: &EnsureContext) -> Result<EnsurePlan> {
        Ok(EnsurePlan {
            description: format!(
                "Ensure git clone {} at {:?} pinned to {}",
                self.url, self.path, self.ref_
            ),
        })
    }

    fn execute(&self, ctx: &EnsureContext) -> Result<()> {
        let target_path = ctx.app_path.join(&self.path);

        if target_path.exists() {
            // Check clean
            if target_path.join(".git").exists() {
                if !git::is_worktree_clean(&target_path)? {
                    return Err(anyhow::anyhow!(
                        "Target path {:?} has dirty working tree.",
                        target_path
                    ));
                }
            }
        }

        if ctx.dry_run {
            return Ok(());
        }

        git::clone_pinned(&self.url, &target_path, &self.ref_)
    }
}
