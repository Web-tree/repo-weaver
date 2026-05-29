# Plugin Management API Contracts

**Feature**: 003-plugin-management-system  
**Date**: 2026-01-17

## CLI Commands

### `rw apply` (existing, extended)

Applies configuration and resolves plugins automatically.

**Plugin Resolution Behavior**:
```
rw apply [--verbose]
```

- Reads `weaver.yaml` for explicit plugin configurations
- Auto-discovers plugins from ensure types if not explicitly configured
- Resolves plugins (download/build if needed)
- Updates `weaver.lock` with checksums
- Creates symlinks in `.rw/plugins/`
- Loads plugins and executes ensures

---

### `rw plugins update` (new)

Updates locked plugin versions.

```
rw plugins update [plugin-name] [--all]
```

**Arguments**:
- `plugin-name`: Optional, update specific plugin only
- `--all`: Update all plugins to latest matching version

**Behavior**:
- Without args: Interactive prompt for which plugins to update
- Re-resolves plugins from sources
- Downloads newer versions if available
- Updates `weaver.lock` with new checksums

---

### `rw plugins list` (new)

Lists installed plugins.

```
rw plugins list [--global]
```

**Output**:
```
NAME           VERSION   SOURCE                                   STATUS
npm-script     v1.0.0    git:github.com/web-tree/rw-plugins       cached
custom-plugin  abc123    path:./my-plugin                         local
```

---

### `rw plugins verify` (new)

Verifies cached plugins against lockfile checksums.

```
rw plugins verify
```

**Output**:
- ✓ on match
- ✗ on mismatch with details

---

## Internal APIs (Rust)

### PluginResolver

```rust
pub struct PluginResolver {
    cache_dir: PathBuf,      // ~/.rw/plugins
    project_dir: PathBuf,    // project root
}

impl PluginResolver {
    /// Create resolver for project
    pub fn new(project_dir: PathBuf) -> anyhow::Result<Self>;
    
    /// Resolve a plugin to a cached WASM path
    pub fn resolve(&self, name: &str, config: &PluginConfig) -> anyhow::Result<ResolvedPlugin>;
    
    /// Resolve from ensure type name (auto-discovery)
    pub fn resolve_ensure_type(&self, type_name: &str) -> anyhow::Result<ResolvedPlugin>;
    
    /// Verify plugin against lockfile
    pub fn verify(&self, name: &str, lock: &PluginLock) -> anyhow::Result<bool>;
}
```

### ResolvedPlugin

```rust
pub struct ResolvedPlugin {
    pub name: String,
    pub version: String,
    pub wasm_path: PathBuf,
    pub sha256: String,
    pub source: PluginSource,
}
```

### PluginCache

```rust
pub struct PluginCache {
    root: PathBuf,  // ~/.rw/plugins
}

impl PluginCache {
    /// Check if plugin version is cached
    pub fn has(&self, name: &str, version: &str) -> bool;
    
    /// Get cached plugin path
    pub fn get(&self, name: &str, version: &str) -> Option<PathBuf>;
    
    /// Store plugin in cache
    pub fn store(&self, name: &str, version: &str, wasm_bytes: &[u8]) -> anyhow::Result<PathBuf>;
    
    /// Create symlink in project
    pub fn link(&self, name: &str, version: &str, project_plugins_dir: &Path) -> anyhow::Result<PathBuf>;
}
```

### PluginFetcher

```rust
pub struct PluginFetcher;

impl PluginFetcher {
    /// Fetch from GitHub/GitLab Releases
    pub async fn fetch_release(&self, url: &str, ref_: &str) -> anyhow::Result<Option<Vec<u8>>>;
    
    /// Build from source using container
    pub fn build_source(&self, repo_url: &str, ref_: &str) -> anyhow::Result<Vec<u8>>;
    
    /// Detect container runtime
    pub fn detect_container_runtime() -> Option<ContainerRuntime>;
}

pub enum ContainerRuntime {
    Docker,
    Podman,
}
```

---

## Error Messages

| Code | Message | Remediation |
|------|---------|-------------|
| E001 | Plugin not found: {name} | Check spelling or add to plugins section |
| E002 | Git fetch failed: {url} | Verify URL and network, check credentials |
| E003 | Checksum mismatch for {name} | Run `rw plugins update {name}` or delete cache |
| E004 | No container runtime found | Install Docker or Podman for source builds |
| E005 | Invalid WASM component: {path} | Plugin may be corrupted, try `rw plugins update` |
| E006 | Plugin source build failed | Check container logs, verify plugin builds locally |
