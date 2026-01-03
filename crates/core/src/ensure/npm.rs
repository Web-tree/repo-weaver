use crate::ensure::{Ensure, EnsureContext, EnsurePlan};
use repo_weaver_ops::npm;

pub struct EnsureNpmScript {
    pub name: String,
    pub command: String,
}

impl Ensure for EnsureNpmScript {
    fn plan(&self, ctx: &EnsureContext) -> anyhow::Result<EnsurePlan> {
        // Check if script exists and matches
        let current = npm::npm_pkg_get_script(&self.name, &ctx.app_path)?;

        if let Some(existing_cmd) = current {
            if existing_cmd == self.command {
                return Ok(EnsurePlan {
                    description: format!("Ensure npm script '{}' (already set)", self.name),
                    actions: vec![],
                });
            }
        }

        // Need to set
        // Need to set
        // (variables used directly in format! macro, so local bindings not strictly needed if we use self.name)

        // EnsurePlan actions are usually strings or structured descriptions in this system?
        // Wait, EnsurePlan struct definition:
        /*
        pub struct EnsurePlan {
            pub description: String,
            // Actions? If we are just reporting plan, we don't return closures usually.
            // The execute phase runs the ensure.
            // But if `EnsurePlan` is just info, then `execute` does the work.
            // Let's check `crates/core/src/ensure.rs`.
        */
        // Assuming EnsurePlan just has description for now based on previous knowledge or I will check it.
        // But wait, the previous conversation mentioned "EnsurePlan struct for plan output".

        // If I need to return an action, maybe I just return the plan description?
        // The `execute` method is separate. I'll assume `plan` is for display/dry-run.

        Ok(EnsurePlan {
            description: format!("Set npm script '{}' to '{}'", self.name, self.command),
            actions: vec![format!(
                "npm pkg set scripts.{}='{}'",
                self.name, self.command
            )],
        })
    }

    fn execute(&self, ctx: &EnsureContext) -> anyhow::Result<()> {
        let pkg_json = ctx.app_path.join("package.json");
        if !pkg_json.exists() {
            anyhow::bail!("package.json not found in {}", ctx.app_path.display());
        }

        npm::npm_pkg_set_script(&self.name, &self.command, &ctx.app_path)?;
        println!("Ensured npm script '{}'", self.name);
        Ok(())
    }
}
