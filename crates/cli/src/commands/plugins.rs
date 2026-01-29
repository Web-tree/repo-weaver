use clap::{Args, Subcommand};
use repo_weaver_core::lockfile::Lockfile;
use repo_weaver_core::plugin::cache::PluginCache;
use repo_weaver_core::plugin::resolver::PluginResolver;
use std::path::{Path, PathBuf};

/// Manage plugins
#[derive(Args)]
pub struct PluginsArgs {
    #[command(subcommand)]
    command: PluginCommands,
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List all configured plugins
    List,
    /// Verify lockfile integrity
    Verify,
    /// Update plugin versions
    Update {
        /// Plugin name to update (omit for --all)
        plugin: Option<String>,
        /// Update all plugins
        #[arg(long)]
        all: bool,
    },
    /// Remove unused plugin versions from cache
    Prune,
}

pub fn execute(args: PluginsArgs) -> anyhow::Result<()> {
    match args.command {
        PluginCommands::List => execute_list(),
        PluginCommands::Verify => execute_verify(),
        PluginCommands::Update { plugin, all } => {
            if all {
                println!("Update all plugins - not yet implemented");
            } else if let Some(name) = plugin {
                println!("Update plugin '{}' - not yet implemented", name);
            } else {
                anyhow::bail!("Specify a plugin name or use --all");
            }
            Ok(())
        }
        PluginCommands::Prune => {
            println!("Plugin prune command - not yet implemented");
            Ok(())
        }
    }
}

fn execute_list() -> anyhow::Result<()> {
    let lockfile_path = Path::new("weaver.lock");

    if !lockfile_path.exists() {
        println!("No plugins found (weaver.lock does not exist)");
        println!("Run 'rw apply' to resolve and lock plugins");
        return Ok(());
    }

    let content = std::fs::read_to_string(lockfile_path)?;
    let lockfile: Lockfile = serde_yml::from_str(&content)?;

    if lockfile.plugins.is_empty() {
        println!("No plugins configured");
        return Ok(());
    }

    let cache = PluginCache::default();

    // Print header
    println!(
        "{:<20} {:<15} {:<50} {:<10}",
        "NAME", "VERSION", "SOURCE", "STATUS"
    );
    println!("{}", "-".repeat(95));

    for (name, lock) in &lockfile.plugins {
        let status = if lock.source.starts_with("path:") {
            "local".to_string()
        } else if cache.has(name, &lock.version) {
            "cached".to_string()
        } else {
            "missing".to_string()
        };

        println!(
            "{:<20} {:<15} {:<50} {:<10}",
            name, lock.version, &lock.source, status
        );
    }

    Ok(())
}

fn execute_verify() -> anyhow::Result<()> {
    let lockfile_path = Path::new("weaver.lock");

    if !lockfile_path.exists() {
        anyhow::bail!("weaver.lock not found. Run 'rw apply' first.");
    }

    let content = std::fs::read_to_string(lockfile_path)?;
    let lockfile: Lockfile = serde_yml::from_str(&content)?;

    if lockfile.plugins.is_empty() {
        println!("✓ No plugins to verify");
        return Ok(());
    }

    let resolver = PluginResolver::new(PathBuf::from("."))?;
    let mut all_valid = true;

    for (name, lock) in &lockfile.plugins {
        // Skip local path plugins
        if lock.source.starts_with("path:") {
            println!("  {} - skipped (local path)", name);
            continue;
        }

        match resolver.verify(name, lock) {
            Ok(true) => {
                println!("✓ {} - checksum valid", name);
            }
            Err(e) => {
                println!("✗ {} - {}", name, e);
                all_valid = false;
            }
            Ok(false) => {
                // This shouldn't happen based on current verify() implementation
                println!("✗ {} - verification failed", name);
                all_valid = false;
            }
        }
    }

    if all_valid {
        println!("\n✓ All plugins verified successfully");
        Ok(())
    } else {
        anyhow::bail!("Some plugins failed verification. Run 'rw plugins update' to fix.");
    }
}
