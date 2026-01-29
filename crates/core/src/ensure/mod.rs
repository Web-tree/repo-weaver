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
pub async fn build_ensure(
    config: &crate::config::EnsureConfig,
    plugin_resolver: Option<&crate::plugin::resolver::PluginResolver>,
) -> Result<Box<dyn Ensure>> {
    use crate::config::EnsureConfig;
    use crate::plugin::ensure_wasm::EnsurePluginEngine;
    use std::sync::Arc;

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
        EnsureConfig::NpmScript { name, command } => {
            let resolver = plugin_resolver.ok_or_else(|| {
                anyhow::anyhow!("Plugin resolver required for npm.script ensures")
            })?;

            // Resolve plugin for npm.script type
            let resolved = resolver.resolve_ensure_type("npm.script").await?;

            // Load the plugin using EnsurePluginEngine
            let engine = EnsurePluginEngine::new()?;
            let plugin = Arc::new(engine.load_plugin(&resolved.wasm_path)?);

            // Serialize config to JSON for the plugin
            let config_json = serde_json::json!({
                "type": "npm.script",
                "name": name,
                "command": command,
            })
            .to_string();

            Ok(Box::new(plugin_wrapper::EnsurePluginWrapper::new(
                plugin,
                config_json,
            )))
        }
        EnsureConfig::AiPatch {
            prompt,
            verify_command,
        } => {
            let resolver = plugin_resolver
                .ok_or_else(|| anyhow::anyhow!("Plugin resolver required for ai.patch ensures"))?;

            // Resolve plugin for ai.patch type
            let resolved = resolver.resolve_ensure_type("ai.patch").await?;

            // Load the plugin using EnsurePluginEngine
            let engine = EnsurePluginEngine::new()?;
            let plugin = Arc::new(engine.load_plugin(&resolved.wasm_path)?);

            // Serialize config to JSON for the plugin
            let config_json = serde_json::json!({
                "type": "ai.patch",
                "prompt": prompt,
                "verify_command": verify_command,
            })
            .to_string();

            Ok(Box::new(plugin_wrapper::EnsurePluginWrapper::new(
                plugin,
                config_json,
            )))
        }
    }
}
