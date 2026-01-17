// Global plugin cache management

use std::path::PathBuf;

pub struct PluginCache {
    root: PathBuf,
}

impl PluginCache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}
