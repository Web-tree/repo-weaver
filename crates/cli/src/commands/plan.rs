use clap::Args;
use std::path::PathBuf;
use tracing::info;

#[derive(Args)]
pub struct PlanArgs {
    /// Return exit code 2 if changes are detected
    #[arg(long)]
    pub detailed_exitcode: bool,

    /// Save the plan to a file
    #[arg(long)]
    pub out: Option<PathBuf>,
}

pub async fn run(args: PlanArgs) -> anyhow::Result<()> {
    info!("Running plan...");

    // Map PlanArgs to ApplyArgs
    let apply_args = crate::commands::apply::ApplyArgs {
        auto_approve: false,          // Plan is interactive for inputs
        strategy: "stop".to_string(), // Default strategy to detect drift
        offline: false,               // Plan command doesn't support offline mode
    };

    let result = crate::commands::apply::execute(apply_args, true).await;

    match result {
        Ok(_) => {
            // No drift detected (or strategy allowed it, but we use stop)
        }
        Err(e) => {
            // If error is drift related and detailed_exitcode is set...
            // For now just error out.
            // If detailed_exitcode is required, we need structured error handling.
            // For MVP, just printing error is fine, but exit code 2 is requested.
            // Anyhow error propagates as exit code 1.
            // To support exit code 2, we might need a custom error type or check string.
            if args.detailed_exitcode {
                // If drift, return specific code.
                // Anyhow doesn't support custom exit codes easily in main?
                // We can just exit::process(2) here.
                if e.to_string().contains("Drift detected") {
                    eprintln!("Error: {}", e);
                    std::process::exit(2);
                }
            }
            return Err(e);
        }
    }

    Ok(())
}
