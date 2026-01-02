use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct WeaverConfig {
    pub version: String,
    pub project: ProjectConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub description: Option<String>,
}

impl Default for WeaverConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            project: ProjectConfig {
                name: "unnamed".to_string(),
                description: None,
            },
        }
    }
}
