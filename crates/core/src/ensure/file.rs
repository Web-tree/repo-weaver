use crate::ensure::{Ensure, EnsureContext, EnsurePlan};
use crate::template::TemplateEngine;

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

/// `ensure.file.from_template` — render a module-relative Tera template into an
/// app-relative destination. rw fully owns the destination file.
pub struct EnsureFileFromTemplate {
    /// Template path, relative to the module root.
    pub template: String,
    /// Destination path, relative to the app root.
    pub dest: String,
}

impl EnsureFileFromTemplate {
    fn render(&self, ctx: &EnsureContext) -> anyhow::Result<String> {
        let template_path = ctx.module_path.join(&self.template);
        let src = std::fs::read_to_string(&template_path).map_err(|e| {
            anyhow::anyhow!("cannot read template {}: {e}", template_path.display())
        })?;
        let engine = TemplateEngine::new()?;
        engine.render(&src, &ctx.tera_context)
    }
}

impl Ensure for EnsureFileFromTemplate {
    fn plan(&self, _ctx: &EnsureContext) -> anyhow::Result<EnsurePlan> {
        Ok(EnsurePlan {
            description: format!("Render '{}' -> '{}'", self.template, self.dest),
            actions: vec![format!("write {}", self.dest)],
        })
    }

    fn execute(&self, ctx: &EnsureContext) -> anyhow::Result<()> {
        if ctx.dry_run {
            return Ok(());
        }
        let rendered = self.render(ctx)?;
        let target = ctx.app_path.join(&self.dest);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, rendered)?;
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

    #[test]
    fn renders_module_template_into_app_dest() {
        let module = tempdir().unwrap();
        let app = tempdir().unwrap();
        std::fs::create_dir_all(module.path().join("templates")).unwrap();
        std::fs::write(
            module.path().join("templates/greeting.txt.j2"),
            "Hello {{ project_name }}\n",
        )
        .unwrap();

        let mut tc = tera::Context::new();
        tc.insert("project_name", "acme-api");
        let ctx = EnsureContext {
            app_path: app.path().to_path_buf(),
            dry_run: false,
            module_path: module.path().to_path_buf(),
            tera_context: tc,
        };

        let e = EnsureFileFromTemplate {
            template: "templates/greeting.txt.j2".into(),
            dest: "greeting.txt".into(),
        };
        e.execute(&ctx).unwrap();
        let out = std::fs::read_to_string(app.path().join("greeting.txt")).unwrap();
        assert_eq!(out, "Hello acme-api\n");
    }
}
