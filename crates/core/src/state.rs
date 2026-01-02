use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub files: HashMap<String, FileState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileState {
    pub path: String,
    pub checksum: String,
    pub last_modified: String, // ISO 8601
}
