use clap::Args;
use repo_weaver_core::config::{ModuleManifest, WeaverConfig};
use repo_weaver_core::module::ModuleResolver;
use serde::Serialize;
use serde_yml::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Args)]
pub struct DescribeArgs {
    /// Name of app to describe
    pub app_name: String,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Show secret values
    #[arg(long)]
    pub show_secrets: bool,
}

pub async fn run(args: DescribeArgs) -> anyhow::Result<()> {
    // 1. Load config
    let config_path = Path::new("weaver.yaml");
    if !config_path.exists() {
        anyhow::bail!("weaver.yaml not found");
    }
    let config = repo_weaver_core::config::load_with_includes(config_path)?;

    // 2. Find App
    let app_config = config
        .apps
        .iter()
        .find(|a| a.name == args.app_name)
        .ok_or_else(|| {
            // Available apps
            let names: Vec<_> = config.apps.iter().map(|a| a.name.as_str()).collect();
            anyhow::anyhow!(
                "App '{}' not found. Available apps: {}",
                args.app_name,
                names.join(", ")
            )
        })?;

    // 3. Resolve Module (for Tasks and Ensures)
    let module_config = config
        .modules
        .iter()
        .find(|m| m.name == app_config.module)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Module '{}' not found for app '{}'",
                app_config.module,
                args.app_name
            )
        })?;

    let resolver = ModuleResolver::new(None)?;
    let module_path = resolver.resolve(&module_config.source, &module_config.r#ref)?;
    let manifest_path = module_path.join("weaver.module.yaml");
    let manifest = ModuleManifest::load(&manifest_path)?;

    // 4. Output
    if args.json {
        let mut inputs = app_config.inputs.clone();
        if !args.show_secrets {
            redact_secrets(&mut inputs);
        }

        let desc = serde_json::json!({
            "app": app_config.name,
            "path": app_config.path,
            "module": format!("{}@{}", module_config.name, module_config.r#ref),
            "inputs": inputs,
            "tasks": manifest.tasks,
            "ensures": manifest.ensures,
            "checks": app_config.checks,
        });
        println!("{}", serde_json::to_string_pretty(&desc)?);
    } else {
        println!("App: {}", app_config.name);
        println!("Path: {}", app_config.path);
        println!("Module: {}@{}", module_config.name, module_config.r#ref);
        println!();

        if !app_config.inputs.is_empty() {
            println!("Inputs:");
            let mut inputs = app_config.inputs.clone();
            if !args.show_secrets {
                redact_secrets(&mut inputs);
            }
            for (k, v) in inputs {
                println!("  {}: {}", k, value_to_string(&v));
            }
            println!();
        }

        println!("Tasks:");
        for (task_name, task_def) in &manifest.tasks {
            println!(
                "  {}: {}",
                task_name,
                task_def.description.as_deref().unwrap_or("")
            );
        }
        println!();

        println!("Ensures:");
        for ensure_config in &manifest.ensures {
            // Debug print ensure config?
            // EnsureConfig is enum.
            use repo_weaver_core::config::EnsureConfig;
            match ensure_config {
                EnsureConfig::GitSubmodule { url, path, r#ref } => {
                    println!("  - git.submodule: {} -> {} ({})", url, path, r#ref);
                }
                EnsureConfig::GitClonePinned { url, path, r#ref } => {
                    println!("  - git.clone_pinned: {} -> {} ({})", url, path, r#ref);
                }
                EnsureConfig::NpmScript { name, command } => {
                    println!("  - npm.script: {} -> {}", name, command);
                }
                EnsureConfig::AiPatch {
                    prompt,
                    verify_command: _,
                } => {
                    println!("  - ai.patch: {} (Ensures AI patch is applied)", prompt);
                }
            }
        }
        println!();

        if !app_config.checks.is_empty() {
            println!("Checks:");
            for check in &app_config.checks {
                println!("  - {}: {}", check.name, check.command);
            }
        }
    }

    Ok(())
}

fn redact_secrets(inputs: &mut HashMap<String, Value>) {
    for (k, v) in inputs.iter_mut() {
        let key_lower = k.to_lowercase();
        // Heuristic redaction
        if key_lower.contains("secret")
            || key_lower.contains("key")
            || key_lower.contains("token")
            || key_lower.contains("password")
        {
            *v = Value::String("***".to_string());
        }
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string(v).unwrap_or_default(),
    }
}
