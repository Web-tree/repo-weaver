use anyhow::Result;
use std::path::PathBuf;

pub mod git;
pub mod npm;
pub mod plugin_wrapper;

// Core ensure types
pub struct EnsureContext {
    pub app_path: PathBuf,
    pub dry_run: bool,
}

pub struct EnsurePlan {
    pub description: String,
    pub actions: Vec<String>,
}

pub trait Ensure {
    fn plan(&self, ctx: &EnsureContext) -> Result<EnsurePlan>;
    fn execute(&self, ctx: &EnsureContext) -> Result<()>;
}

/// Build an `Ensure` from a manifest entry.
///
/// Built-in [`EnsureConfig`] types are handled in-process; any other `type:`
/// is routed to a WASM plugin named after the type (`go.dep` -> `go-dep`),
/// with the entry's config forwarded verbatim as JSON.
pub async fn build_ensure(
    entry: &crate::config::EnsureEntry,
    plugin_resolver: Option<&crate::plugin::resolver::PluginResolver>,
) -> Result<Box<dyn Ensure>> {
    use crate::config::EnsureEntry;

    match entry {
        EnsureEntry::Known(config) => build_known_ensure(config, plugin_resolver).await,
        EnsureEntry::Plugin(p) => build_plugin_ensure(p, plugin_resolver).await,
    }
}

/// Dispatch a generic plugin-backed ensure: resolve the plugin by type name and
/// forward the entry's config as JSON.
async fn build_plugin_ensure(
    p: &crate::config::PluginEnsure,
    plugin_resolver: Option<&crate::plugin::resolver::PluginResolver>,
) -> Result<Box<dyn Ensure>> {
    use crate::plugin::ensure_wasm::EnsurePluginEngine;
    use std::sync::Arc;

    let resolver = plugin_resolver.ok_or_else(|| {
        anyhow::anyhow!(
            "plugin resolver required for ensure type '{}'",
            p.type_name
        )
    })?;
    let resolved = resolver.resolve_ensure_type(&p.type_name).await?;

    let engine = EnsurePluginEngine::new()?;
    let plugin = Arc::new(engine.load_plugin(&resolved.wasm_path)?);

    // Forward `{ "type": <type>, ...config }` to the plugin.
    let mut obj = p.config.clone();
    obj.insert(
        "type".to_string(),
        serde_json::Value::String(p.type_name.clone()),
    );
    let config_json = serde_json::to_string(&obj)?;

    Ok(Box::new(plugin_wrapper::EnsurePluginWrapper::new(
        plugin, config_json,
    )))
}

// Builder for the built-in (typed) ensure configs.
async fn build_known_ensure(
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
            // Handled natively (deterministic JSON edit, no npm required) —
            // shares the implementation with the app-level `ensure.npm.script`.
            // The npm-script WASM plugin is the reference SDK example, not the
            // built-in handler.
            Ok(Box::new(npm::EnsureNpmScript {
                name: name.clone(),
                command: command.clone(),
            }))
        }
        EnsureConfig::AiPatch {
            prompt,
            verify_command,
            tool,
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
                "tool": tool,
            })
            .to_string();

            Ok(Box::new(plugin_wrapper::EnsurePluginWrapper::new(
                plugin,
                config_json,
            )))
        }
    }
}
