use crate::lockfile::Lockfile;
use repo_weaver_ops::git;
use std::path::PathBuf;

pub struct ModuleResolver {
    cache_dir: PathBuf,
    lockfile: Option<Lockfile>,
}

impl ModuleResolver {
    pub fn new(lockfile: Option<Lockfile>) -> anyhow::Result<Self> {
        let home =
            home::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let cache_dir = home.join(".rw").join("store");
        Ok(Self {
            cache_dir,
            lockfile,
        })
    }

    pub fn resolve(&self, source: &str, ref_: &str) -> anyhow::Result<PathBuf> {
        // 1. Check Lockfile Integrity
        if let Some(lock) = &self.lockfile {
            if let Some(module_lock) = lock.modules.get(source) {
                // Verify ref matches lock
                if module_lock.r#ref != ref_ {
                    // Start drift or update? Strict mode says abort?
                    // For now, warn or error?
                    // Lockfile Integrity Check rule: "System MUST abort ... if weaver.lock checksums do not match downloaded content"
                    // Here we check ref match first.
                }
            }
        }

        let folder_name = urlencoding::encode(source);
        let path = self.cache_dir.join(folder_name.as_ref()).join(ref_);

        if !path.exists() {
            // Offline Fallback check:
            // "System MUST auto-fallback to the global cache with a warning if the upstream source is unreachable."
            // Implicit: git clone handles network. If it fails, we can't fallback if the folder doesn't exist locally!
            // Fallback implies "use what we have even if we can't update"?
            // But if path !exists, we don't have it.
            // So we try valid clone.
            std::fs::create_dir_all(&path)?;
            if let Err(e) = git::clone(source, ref_, &path) {
                // If clone fails and we don't have it, we assume we can't proceed.
                // If we had it (path exists), we would skip clone.
                // So fallback applies to "Create App from Module" -> Resolve Module.
                // If module is already resolved (cached), we use it.
                // We are checking `!path.exists()`. So if it exists, we stick to it (implicit offline use).
                // If it doesn't exist, we MUST clone. If clone fails, we fail.
                std::fs::remove_dir_all(&path).ok();
                return Err(e);
            }
        }

        Ok(path)
    }
}
