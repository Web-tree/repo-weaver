use crate::ensure::{Ensure, EnsureContext, EnsurePlan};
use crate::plugin::ensure_wasm::LoadedEnsurePlugin;
use std::sync::Arc;

/// Wrapper that implements `Ensure` trait using a dynamically loaded WASM plugin.
pub struct EnsurePluginWrapper {
    pub plugin: Arc<LoadedEnsurePlugin>,
    pub config_json: String,
}

impl EnsurePluginWrapper {
    pub fn new(plugin: Arc<LoadedEnsurePlugin>, config_json: String) -> Self {
        Self {
            plugin,
            config_json,
        }
    }
}

impl Ensure for EnsurePluginWrapper {
    fn plan(&self, ctx: &EnsureContext) -> anyhow::Result<EnsurePlan> {
        let wasm_plan = self.plugin.plan(
            ctx.app_path.to_str().unwrap_or(""),
            ctx.dry_run,
            &self.config_json,
        )?;

        Ok(EnsurePlan {
            description: wasm_plan.description,
            actions: wasm_plan.actions,
        })
    }

    fn execute(&self, ctx: &EnsureContext) -> anyhow::Result<()> {
        let result = self.plugin.execute(
            ctx.app_path.to_str().unwrap_or(""),
            ctx.dry_run,
            &self.config_json,
        )?;

        println!("{}", result);
        Ok(())
    }
}
