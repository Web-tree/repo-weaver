use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct State {
    pub files: HashMap<PathBuf, FileState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileState {
    pub checksum: String,
    pub last_updated: String, // ISO timestamp
}

impl State {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let state = serde_yml::from_str(&content)?;
            Ok(state)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_yml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

pub fn calculate_checksum(path: &Path) -> anyhow::Result<String> {
    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn calculate_checksum_from_bytes(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}
