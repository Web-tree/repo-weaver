use clap::Args;
use repo_weaver_core::app::App;
use repo_weaver_core::config::{ModuleManifest, WeaverConfig};
use repo_weaver_core::engine::Engine;
use repo_weaver_core::module::ModuleResolver;
use repo_weaver_core::state::{
    FileState, State, calculate_checksum, calculate_checksum_from_bytes,
};
use repo_weaver_core::template::{TemplateEngine, build_context};
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
    let config = WeaverConfig::load_with_includes(config_path)?;

    // Load State
    let state_path = Path::new(".rw/state.yaml");
    let mut state = State::load(state_path)?;

    // 2. Init components
    let resolver = ModuleResolver::new(None)?;
    let template_engine = TemplateEngine::new()?;
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
        let interactive = !args.auto_approve;
        let resolved_inputs = crate::prompts::resolve_missing_inputs(
            &manifest,
            &app_config.inputs,
            interactive,
            &answers_path,
            &app_config.name,
        )?;

        // Clean up stale answers for removed inputs
        let current_keys: std::collections::HashSet<String> =
            manifest.inputs.keys().cloned().collect();
        crate::prompts::cleanup_stale_answers(
            &answers_path,
            &app_config.name,
            &current_keys,
            interactive,
        )?;

        // Merge resolved inputs into config clone
        let mut app_config_resolved = app_config.clone();
        app_config_resolved.inputs.extend(resolved_inputs);

        let app = App::instantiate(&app_config_resolved, &manifest)?;
        let dest_root = PathBuf::from(&app.path);

        // Files Processing
        let files_src = module_path.join("files");
        if files_src.exists() {
            for entry in WalkDir::new(&files_src) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let rel_path = entry.path().strip_prefix(&files_src)?;
                    let dest_path = dest_root.join(rel_path);

                    // Check Drift
                    if dest_path.exists()
                        && let Some(file_state) = state.files.get(&dest_path)
                    {
                        let current_chk = calculate_checksum(&dest_path)?;
                        if file_state.checksum != current_chk {
                            if args.strategy == "stop" {
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

                    // Write File
                    if dry_run {
                        info!("Would copy {:?} to {:?}", entry.path(), dest_path);
                    } else {
                        Engine::ensure_file_copy(entry.path(), &dest_path)?;

                        let new_chk = calculate_checksum(&dest_path)?;
                        state
                            .files
                            .insert(dest_path.clone(), FileState::new(new_chk));
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
                    let input_ctx = build_context(&app.inputs)?;
                    context.extend(input_ctx);

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
                    if dest_path.exists()
                        && let Some(file_state) = state.files.get(&dest_path)
                    {
                        let current_chk = calculate_checksum(&dest_path)?;
                        if file_state.checksum != current_chk {
                            if args.strategy == "stop" {
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

                    if dry_run {
                        info!("Would render {:?} to {:?}", entry.path(), dest_path);
                    } else {
                        let rendered = template_engine.render(&content, &context)?;
                        if let Some(parent) = dest_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&dest_path, &rendered)?;

                        let new_chk = calculate_checksum_from_bytes(rendered.as_bytes());
                        state
                            .files
                            .insert(dest_path.clone(), FileState::new(new_chk));
                    }
                }
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
