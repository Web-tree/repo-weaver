//! `terraform.fmt` ensure — canonicalises Terraform/OpenTofu files via
//! `terraform fmt`, per PRD "native tool first". Idempotent: `fmt -check`
//! gates whether any action is needed.

use serde::Deserialize;
use wit_bindgen::generate;

generate!({
    world: "ensure-provider",
    path: "../../wit",
});

use exports::weaver::plugin::ensures::{EnsureError, EnsurePlan, EnsureRequest, Guest};
use weaver::plugin::process::{exec, ExecRequest};

struct Component;

#[derive(Deserialize, Default)]
struct TerraformFmtConfig {
    /// CLI to use: "terraform" (default) or "tofu".
    #[serde(default)]
    tool: String,
    /// Format recursively into subdirectories (default true).
    #[serde(default = "default_true")]
    recursive: bool,
}

fn default_true() -> bool {
    true
}

fn tool_name(cfg: &TerraformFmtConfig) -> String {
    if cfg.tool.is_empty() {
        "terraform".to_string()
    } else {
        cfg.tool.clone()
    }
}

fn run(program: &str, args: &[String], cwd: &str) -> Result<(i32, Vec<u8>, Vec<u8>), String> {
    let req = ExecRequest {
        program: program.to_string(),
        args: args.to_vec(),
        cwd: Some(cwd.to_string()),
        env: vec![],
        inherit_env: true,
        stdin: None,
    };
    let r = exec(&req)?;
    Ok((r.status as i32, r.stdout, r.stderr))
}

fn fmt_args(cfg: &TerraformFmtConfig, check: bool) -> Vec<String> {
    let mut args = vec!["fmt".to_string()];
    if check {
        args.push("-check".to_string());
    }
    if cfg.recursive {
        args.push("-recursive".to_string());
    }
    args
}

impl Guest for Component {
    fn plan(req: EnsureRequest) -> Result<EnsurePlan, EnsureError> {
        let cfg: TerraformFmtConfig = serde_json::from_str(&req.config)
            .map_err(|e| EnsureError::ConfigError(format!("Invalid config: {}", e)))?;
        let tool = tool_name(&cfg);

        // `fmt -check` exits non-zero when files need formatting.
        match run(&tool, &fmt_args(&cfg, true), &req.app_path) {
            Ok((0, _, _)) => Ok(EnsurePlan {
                description: format!("{} fmt: already formatted", tool),
                actions: vec![],
            }),
            _ => Ok(EnsurePlan {
                description: format!("{} fmt: files need formatting", tool),
                actions: vec![format!("{} {}", tool, fmt_args(&cfg, false).join(" "))],
            }),
        }
    }

    fn execute(req: EnsureRequest) -> Result<String, EnsureError> {
        let cfg: TerraformFmtConfig = serde_json::from_str(&req.config)
            .map_err(|e| EnsureError::ConfigError(format!("Invalid config: {}", e)))?;
        let tool = tool_name(&cfg);
        if req.dry_run {
            return Ok(format!("Would run {} fmt", tool));
        }
        let (status, _, stderr) = run(&tool, &fmt_args(&cfg, false), &req.app_path)
            .map_err(EnsureError::ExecutionError)?;
        if status != 0 {
            return Err(EnsureError::ExecutionError(format!(
                "{} fmt failed: {}",
                tool,
                String::from_utf8_lossy(&stderr)
            )));
        }
        Ok(format!("Ensured {} formatting", tool))
    }
}

export!(Component);
