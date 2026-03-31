use crate::config::{AppConfig, ModuleManifest};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct App {
    pub name: String,
    pub path: PathBuf,
    pub inputs: HashMap<String, serde_yml::Value>,
}

fn validate_input_type(
    key: &str,
    value: &serde_yml::Value,
    expected_type: &str,
) -> anyhow::Result<()> {
    let ok = match expected_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "bool" => value.is_bool(),
        other => anyhow::bail!("Unknown input type '{}' for key '{}'", other, key),
    };
    if !ok {
        anyhow::bail!(
            "Input '{}': expected type '{}', got {:?}",
            key,
            expected_type,
            value
        );
    }
    Ok(())
}

impl App {
    pub fn instantiate(config: &AppConfig, manifest: &ModuleManifest) -> anyhow::Result<Self> {
        let mut final_inputs = HashMap::new();

        for (key, def) in &manifest.inputs {
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
                validate_input_type(key, v, &def.r#type)?;
                final_inputs.insert(key.clone(), v.clone());
            }
        }

        Ok(Self {
            name: config.name.clone(),
            path: PathBuf::from(&config.path),
            inputs: final_inputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yml::Value;

    #[test]
    fn test_validate_string_ok() {
        assert!(validate_input_type("k", &Value::String("hi".into()), "string").is_ok());
    }

    #[test]
    fn test_validate_number_ok() {
        assert!(validate_input_type("k", &Value::Number(42.into()), "number").is_ok());
    }

    #[test]
    fn test_validate_bool_ok() {
        assert!(validate_input_type("k", &Value::Bool(true), "bool").is_ok());
    }

    #[test]
    fn test_validate_type_mismatch() {
        let err = validate_input_type("port", &Value::String("not_a_number".into()), "number");
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("expected type 'number'"));
    }

    #[test]
    fn test_validate_unknown_type() {
        let err = validate_input_type("k", &Value::String("x".into()), "object");
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Unknown input type"));
    }
}
