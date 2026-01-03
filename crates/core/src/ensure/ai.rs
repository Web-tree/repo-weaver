use crate::ensure::{Ensure, EnsureContext, EnsurePlan};
use anyhow::{Context, Result};
use std::process::Command;

pub struct EnsureAiPatch {
    pub prompt: String,
    pub verify_command: String,
}

impl Ensure for EnsureAiPatch {
    fn plan(&self, ctx: &EnsureContext) -> Result<EnsurePlan> {
        // AI Patch constantly attempts to improve/fix if verification fails?
        // Or if it's "ensure code matches spec"?
        // For MVP: We always propose a plan if verification fails (or always run for now to attempt improvement?)
        // Let's check verification command.

        let should_run = if self.verify_command.is_empty() {
            true // No verification = always run? Or unsafe?
        } else {
            let status = Command::new("sh")
                .arg("-c")
                .arg(&self.verify_command)
                .current_dir(&ctx.app_path)
                .status();

            match status {
                Ok(s) => !s.success(), // If verify fails, run patch
                Err(_) => true, // If verify command fails to run, assume we need to patch? or error?
            }
        };

        if should_run {
            Ok(EnsurePlan {
                description: format!(
                    "AI Patch (prompt: '{}') - Verification failed or missing",
                    self.prompt.chars().take(50).collect::<String>()
                ),
                actions: vec![
                    format!("Generate patch via LLM"),
                    format!("Apply patch"),
                    format!("Verify: {}", self.verify_command),
                ],
            })
        } else {
            // Already verified
            // Return plan with no actions? Or check if EnsurePlan requires actions?
            Ok(EnsurePlan {
                description: "AI Patch verified (no action needed)".to_string(),
                actions: vec![],
            })
        }
    }

    fn execute(&self, ctx: &EnsureContext) -> Result<()> {
        let verify_status = Command::new("sh")
            .arg("-c")
            .arg(&self.verify_command)
            .current_dir(&ctx.app_path)
            .status()
            .unwrap_or_else(|_| std::process::ExitStatus::default()); // Dummy exit status?

        if verify_status.success() {
            println!("Verification passed, skipping AI patch.");
            return Ok(());
        }

        println!("Generating patch for prompt: {}", self.prompt);
        // Mock LLM:
        // In integration tests, we can use an env var to simulate a patch.
        // Or just write a file.
        // For MVP, if env "MOCK_LLM_PATCH_FILE" is set, verify its existence and apply it?
        // Or just write a dummy change if MOCK_LLM_RESPONSE is set?

        // Simulating: Write a file "ai_patch.txt" as proof
        let dummy_file = ctx.app_path.join("ai_generated.txt");
        std::fs::write(&dummy_file, "AI was here").context("Failed to write mock AI file")?;

        // Re-verify
        let status = Command::new("sh")
            .arg("-c")
            .arg(&self.verify_command)
            .current_dir(&ctx.app_path)
            .status()?;

        if !status.success() {
            // Rollback (delete file)
            let _ = std::fs::remove_file(dummy_file);
            return Err(anyhow::anyhow!("Verification failed after AI patch"));
        }

        Ok(())
    }
}
