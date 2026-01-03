use crate::config::WeaverConfig;
use anyhow::{Context, Result};
use glob::glob;
use serde_yml::Value;
use std::path::{Path, PathBuf};

/// Expand glob patterns relative to a base path
pub fn expand_includes(patterns: &[String], base: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for pattern in patterns {
        // Construct the full glob pattern safely
        let full_pattern = if base.as_os_str().is_empty() {
            pattern.clone()
        } else {
            // Join base and pattern.
            // Note: glob patterns must be strings. If base contains non-utf8 chars this might be tricky,
            // but for config paths it's usually fine.
            match base.join(pattern).to_str() {
                Some(s) => s.to_string(),
                None => anyhow::bail!("Invalid path encoding for pattern expansion"),
            }
        };

        // Use glob to find matches
        let paths = glob(&full_pattern).context("Failed to read glob pattern")?;

        // Collect valid paths
        let mut matched_any = false;
        for entry in paths {
            match entry {
                Ok(path) => {
                    files.push(path);
                    matched_any = true;
                }
                Err(e) => tracing::warn!("Glob error: {}", e),
            }
        }

        if !matched_any {
            // EC-001: Warn on empty glob matches (not error)
            tracing::warn!(
                "Review the include pattern '{}' in 'weaver.yaml' as it did not match any files.",
                pattern
            );
        }
    }

    // T015: Add deterministic ordering: alphabetical glob expansion.
    // glob crate usually returns sorted generally, but let's sort to be sure.
    files.sort();

    Ok(files)
}

pub fn merge_configs(base: Value, overlay: Value) -> Value {
    // T013: Implement deep merge
    // If both are maps, merge keys. If both are sequences, concat. Otherwise overlay overwrites base.
    match (base, overlay) {
        (Value::Mapping(mut base_map), Value::Mapping(overlay_map)) => {
            for (k, v) in overlay_map {
                let merged = if let Some(base_val) = base_map.remove(&k) {
                    merge_configs(base_val, v)
                } else {
                    v
                };
                base_map.insert(k, merged);
            }
            Value::Mapping(base_map)
        }
        (Value::Sequence(mut base_seq), Value::Sequence(overlay_seq)) => {
            base_seq.extend(overlay_seq);
            Value::Sequence(base_seq)
        }
        (_, overlay) => overlay,
    }
}

pub fn load_with_includes(path: &Path) -> Result<WeaverConfig> {
    // Basic load of the main file to get includes
    // We parse as Value to allow generic merging later,
    // but initially we might want to parse partially or just parse as Value and look for "includes"

    let content =
        std::fs::read_to_string(path).context(format!("Failed to read config file {:?}", path))?;
    let mut main_config_value: Value =
        serde_yml::from_str(&content).context("Failed to parse main config YAML")?;

    // Extract includes from the main config (if any)
    // We check if "includes" key exists and is a sequence of strings
    let patterns = if let Value::Mapping(map) = &main_config_value {
        if let Some(Value::Sequence(seq)) = map.get(&Value::String("includes".to_string())) {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if !patterns.is_empty() {
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let include_paths = expand_includes(&patterns, base_dir)?;

        for include_path in include_paths {
            let include_content = std::fs::read_to_string(&include_path)
                .context(format!("Failed to read include file {:?}", include_path))?;
            let include_value: Value = serde_yml::from_str(&include_content)
                .context(format!("Failed to parse include YAML {:?}", include_path))?;

            // T015: deterministic ordering: later overrides earlier.
            // We merge include INTO the main config?
            // Usually includes are applied ON TOP of main config, or main config overrides includes?
            // "later overrides earlier" for includes.
            // But what about main config vs includes?
            // "Enable modular config with `includes` glob patterns merging YAML fragments"
            // Typically, specific config overrides shared config.
            // If weaver.yaml includes "shared/*.yaml", probably shared is base?
            // Or usually: main config defines the structure, includes enable splitting.
            // Let's assume: Main config is the base. Includes are merged ON TOP of it in order.
            // Wait, if I define an app in main, and also in include?
            // Let's assume includes are merged sequentially into the accumulator.

            main_config_value = merge_configs(main_config_value, include_value);
        }
    }

    // Now deserialize the final merged value into WeaverConfig
    let config: WeaverConfig =
        serde_yml::from_value(main_config_value).context("Failed to deserialize merged config")?;

    // T017: Error on duplicate app names checking involves inspecting the config object
    validate_unique_apps(&config)?;

    Ok(config)
}

fn validate_unique_apps(config: &WeaverConfig) -> Result<()> {
    let mut app_names = std::collections::HashSet::new();
    for app in &config.apps {
        if !app_names.insert(&app.name) {
            anyhow::bail!(
                "Duplicate app name found: '{}'. App names must be unique across all included fragments.",
                app.name
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yml::Value;

    #[test]
    fn test_merge_configs_maps() {
        let base_yaml = r#"
        version: "1.0"
        apps:
          - name: app1
            path: apps/app1
        "#;
        let overlay_yaml = r#"
        apps:
          - name: app2
            path: apps/app2
        secrets:
          db:
            provider: aws
        "#;

        let base: Value = serde_yml::from_str(base_yaml).unwrap();
        let overlay: Value = serde_yml::from_str(overlay_yaml).unwrap();
        let merged = merge_configs(base, overlay);

        // Check merged structure
        // apps should be concatenated (sequence)
        // secrets should be added (map merge)
        // version should stay from base (unless overlay overwrites? Overlay overwrites base keys, but here key 'version' is not in overlay)

        // Wait, current merge implementation for Sequence:
        // (Value::Sequence(mut base_seq), Value::Sequence(overlay_seq)) => { base_seq.extend(overlay_seq); ... }
        // So apps should be app1 then app2.

        let merged_yaml = serde_yml::to_string(&merged).unwrap();
        println!("Merged: {}", merged_yaml);

        let apps_seq = merged.get("apps").unwrap().as_sequence().unwrap();
        assert_eq!(apps_seq.len(), 2);
        assert_eq!(apps_seq[0]["name"].as_str().unwrap(), "app1");
        assert_eq!(apps_seq[1]["name"].as_str().unwrap(), "app2");

        let secrets_map = merged.get("secrets").unwrap().as_mapping().unwrap();
        assert!(secrets_map.contains_key(&Value::String("db".to_string())));

        assert_eq!(merged.get("version").unwrap().as_str().unwrap(), "1.0");
    }

    #[test]
    fn test_merge_configs_overwrite() {
        let base_yaml = "version: '1.0'";
        let overlay_yaml = "version: '2.0'";
        let base: Value = serde_yml::from_str(base_yaml).unwrap();
        let overlay: Value = serde_yml::from_str(overlay_yaml).unwrap();
        let merged = merge_configs(base, overlay);

        assert_eq!(merged.get("version").unwrap().as_str().unwrap(), "2.0");
    }

    #[test]
    fn test_merge_nested_maps() {
        let base_yaml = r#"
        nested:
          key1: value1
          key2: value2
        "#;
        let overlay_yaml = r#"
        nested:
          key2: new_value2
          key3: value3
        "#;
        let base: Value = serde_yml::from_str(base_yaml).unwrap();
        let overlay: Value = serde_yml::from_str(overlay_yaml).unwrap();
        let merged = merge_configs(base, overlay);

        let nested = merged.get("nested").unwrap().as_mapping().unwrap();
        assert_eq!(
            nested
                .get(&Value::String("key1".to_string()))
                .unwrap()
                .as_str()
                .unwrap(),
            "value1"
        );
        assert_eq!(
            nested
                .get(&Value::String("key2".to_string()))
                .unwrap()
                .as_str()
                .unwrap(),
            "new_value2"
        );
        assert_eq!(
            nested
                .get(&Value::String("key3".to_string()))
                .unwrap()
                .as_str()
                .unwrap(),
            "value3"
        );
    }

    #[test]
    fn test_merge_empty_base() {
        let base_yaml = "{}";
        let overlay_yaml = "foo: bar";
        let base: Value = serde_yml::from_str(base_yaml).unwrap();
        let overlay: Value = serde_yml::from_str(overlay_yaml).unwrap();
        let merged = merge_configs(base, overlay);

        assert_eq!(merged.get("foo").unwrap().as_str().unwrap(), "bar");
    }
}
