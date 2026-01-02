use clap::Args;
use repo_weaver_core::app::App;
use repo_weaver_core::config::{ModuleManifest, WeaverConfig};
use repo_weaver_core::engine::Engine;
use repo_weaver_core::module::ModuleResolver;
use repo_weaver_core::template::TemplateEngine;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Args)]
pub struct ApplyArgs {
    // Add flags later
}

pub async fn run(_args: ApplyArgs) -> anyhow::Result<()> {
    info!("Running apply...");

    // 1. Load config
    let config_path = Path::new("weaver.yaml");
    if !config_path.exists() {
        anyhow::bail!("weaver.yaml not found");
    }
    let config = WeaverConfig::load(config_path)?;

    // 2. Init components
    let resolver = ModuleResolver::new(None)?; // Todo: Load lockfile
    let template_engine = TemplateEngine::new()?;
    let tera_context = tera::Context::new(); // Global context

    // 3. Process Apps
    for app_config in &config.apps {
        info!("Processing app: {}", app_config.name);

        // Resolve Module
        let module_config = config
            .modules
            .iter()
            .find(|m| m.name == app_config.module)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not found", app_config.module))?;

        let module_path = resolver.resolve(&module_config.source, &module_config.r#ref)?;

        // Load Module Manifest
        let manifest_path = module_path.join("weaver.module.yaml");
        let manifest = ModuleManifest::load(&manifest_path)?;

        // Instantiate App (Validation)
        let app = App::instantiate(app_config, &manifest)?;

        // Execute Logic (Copy files, Templates)
        // For MVP: recursive copy of files/ + template rendering
        // Spec: "Module Content Structure: templates/, files/"

        let dest_root = PathBuf::from(&app.path);

        // Copy static files
        let files_src = module_path.join("files");
        if files_src.exists() {
            repo_weaver_ops::fs::copy(&files_src, &dest_root)?;
        }

        // Render templates
        let templates_src = module_path.join("templates");
        if templates_src.exists() {
            for entry in walkdir::WalkDir::new(&templates_src) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let rel_path = entry.path().strip_prefix(&templates_src)?;
                    // Spec says `main.tf.j2`. So we strip one extension?
                    // Spec says `main.tf.j2`. So we strip one extension?
                    // Or just render everything in templates/ to dest/

                    // Simple logic: read, render, write.
                    let content = std::fs::read_to_string(entry.path())?;

                    // Create app context
                    let mut context = tera_context.clone();
                    for (k, v) in &app.inputs {
                        context.insert(k, v);
                    }

                    // Determine dest path. If .j2, remove it.
                    let file_name = entry.file_name().to_string_lossy();
                    let dest_path = if file_name.ends_with(".j2") {
                        dest_root
                            .join(rel_path.parent().unwrap_or(Path::new("")))
                            .join(rel_path.file_stem().unwrap())
                    } else {
                        dest_root.join(rel_path)
                    };

                    Engine::ensure_file_from_template(
                        &template_engine,
                        &content,
                        &context,
                        &dest_path,
                    )?;
                }
            }
        }
    }

    Ok(())
}
