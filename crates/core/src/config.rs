use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod loader;
pub use loader::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default, rename = "ref")]
    pub git_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaverConfig {
    pub version: String,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub modules: Vec<ModuleConfig>,
    #[serde(default)]
    pub apps: Vec<AppConfig>,
    #[serde(default)]
    pub checks: Vec<CheckDef>,
    #[serde(default)]
    pub secrets: HashMap<String, SecretConfig>,
    #[serde(default)]
    pub plugins: HashMap<String, PluginConfig>,
}

impl WeaverConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for (name, plugin) in &self.plugins {
            plugin.validate(name)?;
        }
        Ok(())
    }
}

impl PluginConfig {
    pub fn validate(&self, name: &str) -> anyhow::Result<()> {
        if self.git.is_some() && self.path.is_some() {
            anyhow::bail!(
                "Plugin '{}' cannot have both 'git' and 'path' properties. Please choose one.",
                name
            );
        }
        if self.git.is_none() && self.path.is_none() {
            anyhow::bail!(
                "Plugin '{}' must have either 'git' or 'path' property.",
                name
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub source: String,
    pub r#ref: String, // "ref" is a keyword in some languages, but okay in Rust struct field? No, "ref" is keyword. Use raw identifier.
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub module: String,
    pub path: String,
    #[serde(default)]
    pub inputs: HashMap<String, serde_yml::Value>,
    #[serde(default)]
    pub checks: Vec<CheckDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckDef {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretConfig {
    pub provider: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    #[serde(default)]
    pub inputs: HashMap<String, InputDef>,
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    #[serde(default)]
    pub tasks: HashMap<String, TaskDef>,
    #[serde(default)]
    pub ensures: Vec<EnsureConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EnsureConfig {
    #[serde(rename = "git.submodule")]
    GitSubmodule {
        url: String,
        path: String,
        r#ref: String,
    },
    #[serde(rename = "git.clone_pinned")]
    GitClonePinned {
        url: String,
        path: String,
        r#ref: String,
    },
    #[serde(rename = "npm.script")]
    NpmScript { name: String, command: String },
    #[serde(rename = "ai.patch")]
    AiPatch {
        prompt: String,
        #[serde(default)]
        verify_command: String,
    },
}

impl ModuleManifest {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Self = serde_yml::from_str(&content)?;
        Ok(manifest)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDef {
    pub r#type: String,
    pub default: Option<serde_yml::Value>,
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDef {
    pub command: String,
    pub description: Option<String>,
}
