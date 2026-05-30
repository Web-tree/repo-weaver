use crate::ensure::{Ensure, EnsureContext, EnsurePlan};

/// `ensure.file.exists` — create an empty file (and parent dirs) if absent.
/// Never truncates an existing file (idempotent, non-clobbering).
pub struct EnsureFileExists {
    /// Destination path, relative to the app root.
    pub dest: String,
}

impl Ensure for EnsureFileExists {
    fn plan(&self, ctx: &EnsureContext) -> anyhow::Result<EnsurePlan> {
        let target = ctx.app_path.join(&self.dest);
        let actions = if target.exists() {
            vec![]
        } else {
            vec![format!("create file {}", self.dest)]
        };
        Ok(EnsurePlan {
            description: format!("Ensure file '{}' exists", self.dest),
            actions,
        })
    }

    fn execute(&self, ctx: &EnsureContext) -> anyhow::Result<()> {
        if ctx.dry_run {
            return Ok(());
        }
        let target = ctx.app_path.join(&self.dest);
        if !target.exists() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&target, b"")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn ctx(app: PathBuf) -> EnsureContext {
        EnsureContext {
            app_path: app,
            dry_run: false,
            module_path: PathBuf::from("."),
            tera_context: tera::Context::new(),
        }
    }

    #[test]
    fn creates_missing_file_and_is_idempotent() {
        let dir = tempdir().unwrap();
        let e = EnsureFileExists { dest: "sub/new.txt".into() };
        e.execute(&ctx(dir.path().to_path_buf())).unwrap();
        let p = dir.path().join("sub/new.txt");
        assert!(p.exists());
        std::fs::write(&p, "user content").unwrap();
        e.execute(&ctx(dir.path().to_path_buf())).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "user content");
    }
}
