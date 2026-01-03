use clap::Args;
use repo_weaver_core::app::App;
use repo_weaver_core::config::ModuleManifest;
use repo_weaver_core::module::ModuleResolver;
use repo_weaver_core::state::{
    FileState, State, calculate_checksum, calculate_checksum_from_bytes,
};
use repo_weaver_core::template::TemplateEngine;
use std::path::{Path, PathBuf};
use tracing::info;
use walkdir::WalkDir;

#[derive(Args, Clone)]
pub struct ApplyArgs {
    /// Skip interactive approval
    #[arg(long)]
    pub auto_approve: bool,

    /// Conflict resolution strategy
    #[arg(long, default_value = "stop")]
    pub strategy: String, // Parsing enum later
}

pub async fn run(args: ApplyArgs) -> anyhow::Result<()> {
    execute(args, false).await
}

pub async fn execute(args: ApplyArgs, dry_run: bool) -> anyhow::Result<()> {
    info!(
        "Running {} (strategy: {}, auto-approve: {})...",
        if dry_run { "plan" } else { "apply" },
        args.strategy,
        args.auto_approve
    );

    // 1. Load config
    let config_path = Path::new("weaver.yaml");
    if !config_path.exists() {
        anyhow::bail!("weaver.yaml not found");
    }
    let config = repo_weaver_core::config::load_with_includes(config_path)?;

    // Load State
    let state_path = Path::new(".rw/state.yaml");
    let mut state = State::load(state_path)?;

    // 2. Init components
    let resolver = ModuleResolver::new(None)?;
    let _template_engine = TemplateEngine::new()?;
    let tera_context = tera::Context::new();

    // 3. Process Apps
    for app_config in &config.apps {
        info!("Processing app: {}", app_config.name);

        let module_config = config
            .modules
            .iter()
            .find(|m| m.name == app_config.module)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not found", app_config.module))?;

        let module_path = resolver.resolve(&module_config.source, &module_config.r#ref)?;
        let manifest_path = module_path.join("weaver.module.yaml");
        let manifest = ModuleManifest::load(&manifest_path)?;

        // Resolve missing inputs (Interactive)
        let answers_path = Path::new(".rw/answers.yaml");
        let resolved_inputs = crate::prompts::resolve_missing_inputs(
            &manifest,
            &app_config.inputs,
            !args.auto_approve, // Interactive if not auto-approve?
            // Spec says "Interactive Prompts". Usually implied by TTY access.
            // But auto_approve flag usually refers to confirmation.
            // Let's assume always interactive unless auto-approve is specified?
            // Or maybe separate --no-input flag?
            // "System checks... if interactive... prompt".
            // auto_approve skips "Review Plan? [y/N]".
            // Missing variables should probably block even in auto-approve unless defaults exist.
            // If auto-approve is on, we are likely non-interactive.
            // Let's allow prompting unless auto_approve is true (non-interactive mode).
            &answers_path,
        )?;

        // Merge resolved inputs into config clone
        let mut app_config_resolved = app_config.clone();
        app_config_resolved.inputs.extend(resolved_inputs);

        let app = App::instantiate(&app_config_resolved, &manifest)?;
        let dest_root = PathBuf::from(&app.path);

        // 3a. Execute Ensures
        let ensure_ctx = repo_weaver_core::ensure::EnsureContext {
            app_path: dest_root.clone(),
            dry_run,
            state: state.clone(),
        };

        for ensure_config in &manifest.ensures {
            let ensure = repo_weaver_core::ensure::build_ensure(ensure_config)?;
            let plan = ensure.plan(&ensure_ctx)?;
            if dry_run {
                info!("Would ensure: {}", plan.description);
                // In dry-run, we might want to skip execute if it has side effects
                // But execute also handles dry-run check internally?
                // The trait implementation `execute` checks `ctx.dry_run`.
                ensure.execute(&ensure_ctx)?;
            } else {
                info!("Ensuring: {}", plan.description);
                ensure.execute(&ensure_ctx)?;
            }
        }

        // Files Processing
        let files_src = module_path.join("files");
        if files_src.exists() {
            for entry in WalkDir::new(&files_src) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let rel_path = entry.path().strip_prefix(&files_src)?;
                    let dest_path = dest_root.join(rel_path);

                    // Check Drift
                    if dest_path.exists() {
                        let current_chk = calculate_checksum(&dest_path)?;
                        if let Some(file_state) = state.files.get(&dest_path) {
                            if file_state.checksum != current_chk {
                                // Drift detected
                                if args.strategy == "stop" && !args.auto_approve {
                                    if dry_run {
                                        info!(
                                            "Drift detected for {:?}. Plan would fail.",
                                            dest_path
                                        );
                                    }
                                    anyhow::bail!(
                                        "Drift detected for {:?}. Use --strategy overwrite to force.",
                                        dest_path
                                    );
                                } else {
                                    if dry_run {
                                        info!(
                                            "Drift detected for {:?}. Plan would overwrite.",
                                            dest_path
                                        );
                                    }
                                }
                            }
                        }
                    }

                    // Write File
                    if dry_run {
                        // Just log
                        info!("Would copy {:?} to {:?}", entry.path(), dest_path);
                    } else {
                        if !dest_path.exists() {
                            if let Some(parent) = dest_path.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                        } else if args.strategy == "stop" && !args.auto_approve {
                            // Double check: if it exists, it might be identical.
                            // Or it is unmanaged.
                            // But if we passed drift check above (managed & matched), we are fine.
                            // If unmanaged (not in state), we are taking ownership.
                        }

                        std::fs::copy(entry.path(), &dest_path)?;

                        // Update State
                        let new_chk = calculate_checksum(&dest_path)?;
                        state.files.insert(
                            dest_path.clone(),
                            FileState {
                                checksum: new_chk,
                                last_updated: "now".to_string(),
                            },
                        );
                    }
                }
            }
        }

        // Templates Processing (Similar logic can be added here, omitting for brevity in this step)
        // ... (existing template logic adapted to manual write/state update) ...
        let templates_src = module_path.join("templates");
        if templates_src.exists() {
            for entry in walkdir::WalkDir::new(&templates_src) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let rel_path = entry.path().strip_prefix(&templates_src)?;
                    let content = std::fs::read_to_string(entry.path())?;
                    let mut context = tera_context.clone();
                    for (k, v) in &app.inputs {
                        // TODO: handle types properly
                        context.insert(k, v);
                    }
                    // Basic rendering, should use template_engine properly
                    // For MVP just text replacement logic or tera one-off?
                    // Existing code used manual text/tera context?
                    // Wait, Step 953 showed "let mut context = tera_context.clone()".
                    // And it didn't actually render in the placeholder code I saw.
                    // I will assume simple copy or placeholder for now.
                    // Actually, I should use template_engine.render if available.
                    // But let's stick to simple text for now or just copy.

                    // Destination logic
                    let file_name = entry.file_name().to_string_lossy();
                    let dest_path = if file_name.ends_with(".j2") {
                        dest_root
                            .join(rel_path.parent().unwrap_or(Path::new("")))
                            .join(rel_path.file_stem().unwrap())
                    } else {
                        dest_root.join(rel_path)
                    };

                    // Drift Check
                    if dest_path.exists() {
                        let current_chk = calculate_checksum(&dest_path)?;
                        if let Some(file_state) = state.files.get(&dest_path) {
                            if file_state.checksum != current_chk {
                                if args.strategy == "stop" && !args.auto_approve {
                                    if dry_run {
                                        info!(
                                            "Drift detected for {:?}. Plan would fail.",
                                            dest_path
                                        );
                                    }
                                    anyhow::bail!(
                                        "Drift detected for {:?}. Use --strategy overwrite to force.",
                                        dest_path
                                    );
                                } else if dry_run {
                                    info!(
                                        "Drift detected for {:?}. Plan would overwrite.",
                                        dest_path
                                    );
                                }
                            }
                        }
                    }

                    if dry_run {
                        info!("Would render {:?} to {:?}", entry.path(), dest_path);
                    } else {
                        if let Some(parent) = dest_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        // Write content (rendered)
                        let rendered =
                            tera::Tera::one_off(&content, &context, false).map_err(|e| {
                                anyhow::anyhow!("Template error in {:?}: {}", entry.path(), e)
                            })?;

                        std::fs::write(&dest_path, &rendered)?;

                        let new_chk = calculate_checksum_from_bytes(rendered.as_bytes());
                        state.files.insert(
                            dest_path.clone(),
                            FileState {
                                checksum: new_chk,
                                last_updated: "now".to_string(),
                            },
                        );
                    }
                }
            }
        }
        // Generate terraform.tfvars.json
        if !app_config_resolved.inputs.is_empty() {
            let tfvars_path = dest_root.join("terraform.tfvars.json");
            let content = serde_json::to_string_pretty(&app_config_resolved.inputs)?;

            if dry_run {
                info!("Would generate {:?}", tfvars_path);
            } else {
                std::fs::write(&tfvars_path, &content)?;

                let new_chk = calculate_checksum_from_bytes(content.as_bytes());
                state.files.insert(
                    tfvars_path,
                    FileState {
                        checksum: new_chk,
                        last_updated: "now".to_string(),
                    },
                );
            }
        }
    }

    if !dry_run {
        state.save(state_path)?;
        info!("Apply complete.");
    } else {
        info!("Plan complete. No changes made.");
    }

    Ok(())
}
