use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lockfile {
    pub version: String,
    pub modules: HashMap<String, ModuleLock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleLock {
    pub source: String,
    pub r#ref: String,
    pub checksum: String,
}
