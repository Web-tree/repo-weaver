mod commands;
mod prompts;

use clap::{CommandFactory, Parser};
use commands::{apply, init, plan};
use repo_weaver_core::setup_tracing;

#[derive(Parser)]
#[command(name = "repo-weaver")]
#[command(about = "Declarative repository management")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Enable debug logging
    #[arg(long, global = true)]
    verbose: bool,

    /// Suppress all non-error output
    #[arg(long, global = true)]
    quiet: bool,

    /// Output logs/result in JSON format
    #[arg(long, global = true)]
    json: bool,
}

#[derive(clap::Subcommand)]
enum Commands {
    Init(init::InitArgs),
    Plan(plan::PlanArgs),
    Apply(apply::ApplyArgs),
    Run(crate::commands::run::RunArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing()?;

    let cli = Cli::parse();

    // Global flags handling (stub for now, tracing setup logic could be refined here)
    if cli.verbose {
        // Adjust logging level
    }

    match cli.command {
        Some(Commands::Init(args)) => {
            init::run(args)?;
        }
        Some(Commands::Plan(args)) => {
            plan::run(args).await?;
        }
        Some(Commands::Apply(args)) => {
            apply::run(args).await?;
        }
        Some(Commands::Run(args)) => {
            crate::commands::run::run(args).await?;
        }
        None => {
            Cli::command().print_help()?;
        }
    }

    Ok(())
}
