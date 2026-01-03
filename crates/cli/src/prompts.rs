use dialoguer::{Input, theme::ColorfulTheme};
use repo_weaver_core::config::ModuleManifest;
use serde_yml::Value;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn load_answers(path: &Path) -> anyhow::Result<HashMap<String, Value>> {
    if path.exists() {
        let content = fs::read_to_string(path)?;
        // Use BTreeMap for stable ordering if we want, but HashMap is fine for internal use.
        // Serde yaml returns a Value or a Map? from_str can parse into HashMap.
        let answers: HashMap<String, Value> = serde_yml::from_str(&content)?;
        Ok(answers)
    } else {
        Ok(HashMap::new())
    }
}

pub fn save_answers(path: &Path, answers: &HashMap<String, Value>) -> anyhow::Result<()> {
    // Merge with existing? Or assumes caller handles merging?
    // Usually we read all, update, save all.
    // Let's implement logical update: read existing, merge new, save.

    let mut current = load_answers(path).unwrap_or_default();
    for (k, v) in answers {
        current.insert(k.clone(), v.clone());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Sort keys for stability? BTreeMap?
    // Let's convert to BTreeMap for saving.
    let sorted: BTreeMap<_, _> = current.into_iter().collect();

    let content = serde_yml::to_string(&sorted)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn resolve_missing_inputs(
    manifest: &ModuleManifest,
    provided_inputs: &HashMap<String, Value>,
    interactive: bool,
    answers_file: &Path,
) -> anyhow::Result<HashMap<String, Value>> {
    let mut resolved = HashMap::new();
    let theme = ColorfulTheme::default();

    // Load previously saved answers
    let saved_answers = load_answers(answers_file).unwrap_or_default();
    let mut new_answers = HashMap::new();

    for (key, def) in &manifest.inputs {
        // 1. Provided explicitly?
        if let Some(val) = provided_inputs.get(key) {
            resolved.insert(key.clone(), val.clone());
            continue;
        }

        // 2. Saved answer?
        if let Some(val) = saved_answers.get(key) {
            resolved.insert(key.clone(), val.clone());
            continue;
        }

        // 3. Default?
        if let Some(default_val) = &def.default {
            // If default exists, use it.
            resolved.insert(key.clone(), default_val.clone());
            continue;
        }

        // 4. Missing and Required -> Prompt
        if !interactive {
            anyhow::bail!(
                "Missing required input '{}' and interactive mode disabled.",
                key
            );
        }

        let prompt_text = if let Some(desc) = &def.description {
            format!("{} ({})", key, desc)
        } else {
            key.clone()
        };

        let input_str: String = Input::with_theme(&theme)
            .with_prompt(&prompt_text)
            .interact_text()?;

        let val = Value::String(input_str);
        resolved.insert(key.clone(), val.clone());
        new_answers.insert(key.clone(), val);
    }

    // Save newly collected answers
    if !new_answers.is_empty() {
        save_answers(answers_file, &new_answers)?;
    }

    Ok(resolved)
}
