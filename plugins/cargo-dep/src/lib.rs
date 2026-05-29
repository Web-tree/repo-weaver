//! `cargo.dep` ensure — adds/pins a crate dependency via `cargo add`,
//! per PRD "native tool first".

use serde::Deserialize;
use wit_bindgen::generate;

generate!({
    world: "ensure-provider",
    path: "../../wit",
});

use exports::weaver::plugin::ensures::{EnsureError, EnsurePlan, EnsureRequest, Guest};
use weaver::plugin::process::{exec, ExecRequest};

struct Component;

#[derive(Deserialize)]
struct CargoDepConfig {
    name: String,
    #[serde(default)]
    version: String,
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

/// `cargo add` spec: `name` or `name@version`.
fn dep_spec(cfg: &CargoDepConfig) -> String {
    if cfg.version.is_empty() {
        cfg.name.clone()
    } else {
        format!("{}@{}", cfg.name, cfg.version)
    }
}

impl Guest for Component {
    fn plan(req: EnsureRequest) -> Result<EnsurePlan, EnsureError> {
        let cfg: CargoDepConfig = serde_json::from_str(&req.config)
            .map_err(|e| EnsureError::ConfigError(format!("Invalid config: {}", e)))?;
        Ok(EnsurePlan {
            description: format!("Ensure Cargo dependency {}", dep_spec(&cfg)),
            actions: vec![format!("cargo add {}", dep_spec(&cfg))],
        })
    }

    fn execute(req: EnsureRequest) -> Result<String, EnsureError> {
        let cfg: CargoDepConfig = serde_json::from_str(&req.config)
            .map_err(|e| EnsureError::ConfigError(format!("Invalid config: {}", e)))?;
        let spec = dep_spec(&cfg);
        if req.dry_run {
            return Ok(format!("Would run cargo add {}", spec));
        }
        let (status, _, stderr) = run("cargo", &["add".into(), spec.clone()], &req.app_path)
            .map_err(EnsureError::ExecutionError)?;
        if status != 0 {
            return Err(EnsureError::ExecutionError(format!(
                "cargo add failed: {}",
                String::from_utf8_lossy(&stderr)
            )));
        }
        Ok(format!("Ensured Cargo dependency {}", spec))
    }
}

export!(Component);
