use crate::config::{AppConfig, ModuleManifest};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct App {
    pub name: String,
    pub path: PathBuf,
    pub inputs: HashMap<String, serde_yml::Value>,
}

impl App {
    pub fn instantiate(config: &AppConfig, manifest: &ModuleManifest) -> anyhow::Result<Self> {
        let mut final_inputs = HashMap::new();

        for (key, def) in &manifest.inputs {
            // Check provided input OR default
            let val = config.inputs.get(key).or(def.default.as_ref());

            if val.is_none() && def.required {
                anyhow::bail!(
                    "App '{}': Missing required input for module '{}': {}",
                    config.name,
                    config.module,
                    key
                );
            }

            if let Some(v) = val {
                // TODO: Type checking against def.type
                final_inputs.insert(key.clone(), v.clone());
            }
        }

        // Also include inputs that might be passed but not strictly in manifest?
        // Usually strict validation warnings. For now, allow extra inputs?
        // Spec says "Validation rules defined for inputs". Strict is safer.
        // For MVP, just taking what matches manifest is safer or checking mismatch.
        // I'll stick to manifest-driven collection.

        Ok(Self {
            name: config.name.clone(),
            path: PathBuf::from(&config.path),
            inputs: final_inputs,
        })
    }
}
