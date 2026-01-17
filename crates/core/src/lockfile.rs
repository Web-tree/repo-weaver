use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lockfile {
    pub version: String,
    pub modules: HashMap<String, ModuleLock>,
    #[serde(default)]
    pub plugins: HashMap<String, PluginLock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleLock {
    pub source: String,
    pub r#ref: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLock {
    pub version: String,
    pub source: String,
    pub sha256: String,
    pub resolved_at: String,
}
