//! `k8s.apply` ensure — converges a Kubernetes manifest/kustomization with
//! `kubectl apply`, per PRD "native tool first". `kubectl apply` is
//! declarative and idempotent; `kubectl diff` gates whether changes are
//! pending in `plan`.

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
struct K8sApplyConfig {
    /// Path to a manifest file or directory (`-f`), relative to the app path.
    path: String,
    /// Treat `path` as a kustomization directory (`-k`) instead of `-f`.
    #[serde(default)]
    kustomize: bool,
}

fn run(args: &[String], cwd: &str) -> Result<(i32, Vec<u8>, Vec<u8>), String> {
    let req = ExecRequest {
        program: "kubectl".to_string(),
        args: args.to_vec(),
        cwd: Some(cwd.to_string()),
        env: vec![],
        inherit_env: true,
        stdin: None,
    };
    let r = exec(&req)?;
    Ok((r.status as i32, r.stdout, r.stderr))
}

fn flag(cfg: &K8sApplyConfig) -> &'static str {
    if cfg.kustomize {
        "-k"
    } else {
        "-f"
    }
}

impl Guest for Component {
    fn plan(req: EnsureRequest) -> Result<EnsurePlan, EnsureError> {
        let cfg: K8sApplyConfig = serde_json::from_str(&req.config)
            .map_err(|e| EnsureError::ConfigError(format!("Invalid config: {}", e)))?;

        // `kubectl diff` exits 0 when there is no drift, 1 when changes pend.
        match run(
            &["diff".into(), flag(&cfg).into(), cfg.path.clone()],
            &req.app_path,
        ) {
            Ok((0, _, _)) => Ok(EnsurePlan {
                description: format!("k8s: {} already applied (no drift)", cfg.path),
                actions: vec![],
            }),
            _ => Ok(EnsurePlan {
                description: format!("k8s: apply {}", cfg.path),
                actions: vec![format!("kubectl apply {} {}", flag(&cfg), cfg.path)],
            }),
        }
    }

    fn execute(req: EnsureRequest) -> Result<String, EnsureError> {
        let cfg: K8sApplyConfig = serde_json::from_str(&req.config)
            .map_err(|e| EnsureError::ConfigError(format!("Invalid config: {}", e)))?;
        if req.dry_run {
            return Ok(format!("Would kubectl apply {} {}", flag(&cfg), cfg.path));
        }
        let (status, _, stderr) = run(
            &["apply".into(), flag(&cfg).into(), cfg.path.clone()],
            &req.app_path,
        )
        .map_err(EnsureError::ExecutionError)?;
        if status != 0 {
            return Err(EnsureError::ExecutionError(format!(
                "kubectl apply failed: {}",
                String::from_utf8_lossy(&stderr)
            )));
        }
        Ok(format!("Ensured k8s manifest {} applied", cfg.path))
    }
}

export!(Component);
