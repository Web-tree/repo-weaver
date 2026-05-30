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
    /// Concrete object SHA the symbolic `ref` resolved to (ref pinning).
    /// A commit for branches, lightweight tags, and SHAs; a tag object SHA
    /// for annotated tags (clone+checkout still lands on the right commit).
    #[serde(default)]
    pub resolved_commit: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLock {
    pub version: String,
    pub source: String,
    pub sha256: String,
    pub resolved_at: String,
}
