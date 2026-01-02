mod cli;
mod config;
mod engine;
mod ops;
mod utils;

use clap::Parser;
use cli::Cli;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    info!("Repo Weaver started with command: {:?}", cli);

    match cli.command {
        cli::Commands::Init { .. } => {
            println!("Init command executed");
        }
        cli::Commands::Plan { .. } => {
            println!("Plan command executed");
        }
        cli::Commands::Apply { .. } => {
            println!("Apply command executed");
        }
    }

    Ok(())
}
