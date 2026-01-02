mod commands;

use clap::Parser;
use commands::apply;
use repo_weaver_core::setup_tracing;

#[derive(Parser)]
#[command(name = "repo-weaver")]
#[command(about = "Declarative repository management")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Init,
    Apply(apply::ApplyArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing()?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            println!("Init command");
        }
        Some(Commands::Apply(args)) => {
            apply::run(args).await?;
        }
        None => {}
    }

    Ok(())
}
