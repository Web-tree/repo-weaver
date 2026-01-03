use clap::Args;
use repo_weaver_core::config::WeaverConfig;
// use repo_weaver_core::state::State; // Might need state for tasks later?
use std::path::Path;
// use tracing::info;

#[derive(Args)]
pub struct ListArgs {
    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Show only apps
    #[arg(long, conflicts_with = "tasks_only")]
    pub apps_only: bool,

    /// Show only tasks
    #[arg(long, conflicts_with = "apps_only")]
    pub tasks_only: bool,
}

pub async fn run(args: ListArgs) -> anyhow::Result<()> {
    // Load config
    let config_path = Path::new("weaver.yaml");
    if !config_path.exists() {
        // T027: Handle empty workspace
        if args.json {
            println!("{{ \"apps\": [], \"tasks\": [] }}");
        } else {
            eprintln!("No apps defined (weaver.yaml not found)");
        }
        std::process::exit(2);
    }

    let config = repo_weaver_core::config::load_with_includes(config_path)?;

    if args.json {
        print_json(&config, &args)?;
    } else {
        print_table(&config, &args)?;
    }

    Ok(())
}

fn print_json(config: &WeaverConfig, args: &ListArgs) -> anyhow::Result<()> {
    // T023: JSON output
    // Struct for JSON output
    use serde::Serialize;

    // Actually, let's look at spec requirements.
    // "APPS section with name, path, module".
    #[derive(Serialize)]
    struct AppInfo<'a> {
        name: &'a str,
        path: &'a str,
        module: &'a str,
    }

    let apps: Vec<AppInfo> = config
        .apps
        .iter()
        .map(|a| AppInfo {
            name: &a.name,
            path: &a.path,
            module: &a.module,
        })
        .collect();

    // Tasks: app:task format.
    // We need to resolve modules to list tasks?
    // "TASKS section output (app:task format with description)".
    // To list tasks, we need to load module manifests.
    // For now, let's implement apps first.

    let include_apps = !args.tasks_only;
    // let include_tasks = !args.apps_only;

    // For simplicity in first pass, just output apps.
    // Tasks listing requires module resolution which implies network or cache.
    // Does `rw list` require fetching modules?
    // "Discovery Commands... showing apps and tasks".
    // If modules are not present, we can't show tasks.
    // We should probably check if modules are cached.

    // For now, let's just dump apps.

    let json_output = serde_json::json!({
        "apps": if include_apps { Some(apps) } else { None },
        "tasks": [] // Placeholder
    });

    println!("{}", serde_json::to_string_pretty(&json_output)?);
    Ok(())
}

fn print_table(config: &WeaverConfig, args: &ListArgs) -> anyhow::Result<()> {
    // T025: Table format
    // Use simple println for MVP or a table crate?
    // "default table output format".
    // Let's use simple formatting.

    if !args.tasks_only {
        if config.apps.is_empty() {
            println!("No apps defined.");
        } else {
            println!("{:<20} {:<30} {:<20}", "APP", "PATH", "MODULE");
            println!("{:<20} {:<30} {:<20}", "---", "----", "------");
            for app in &config.apps {
                println!("{:<20} {:<30} {:<20}", app.name, app.path, app.module);
            }
        }
    }

    if !args.apps_only {
        // Tasks placeholder
        // println!("\nTASKS");
        // ...
    }

    Ok(())
}
