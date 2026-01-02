use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new weaver.yaml
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Plan changes
    Plan {
        /// Path to app (default: current dir)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Apply changes
    Apply {
        /// Path to app (default: current dir)
        #[arg(default_value = ".")]
        path: String,
    },
}
