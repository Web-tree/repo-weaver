# Research Notes: Plugin Management System

**Feature**: 003-plugin-management-system  
**Date**: 2026-01-17

## Technical Findings

### 1. Existing WASM Infrastructure

**Decision**: Build on existing wasmtime v40 infrastructure in `crates/core/src/plugin/`

**Rationale**:
- `EnsurePluginEngine` already handles WASM component model loading via `wasmtime::component::Component`
- `LoadedEnsurePlugin` provides `plan()` and `execute()` methods matching the WIT interface
- `EnsurePluginWrapper` implements the `Ensure` trait for seamless integration
- Existing WIT interface at `wit/plugin.wit` defines `ensure-provider` world with `process` import capability

**Alternatives Considered**:
- Extism framework (rejected: introduces new dependency, existing wasmtime works well)
- Raw WASI (rejected: component model already in use, provides better typing)

---

### 2. Plugin Resolution Strategy

**Decision**: Implement a three-tier resolution strategy

1. **Local Path** (`path: ./my-plugin`): Direct file reference, no caching
2. **Git Source** (`git: github.com/org/repo`, `ref: v1.0.0`): Clone/fetch and look for pre-built releases
3. **Auto-Discovery**: Map ensure type name (e.g., `npm.script`) to default registry path

**Rationale**:
- Matches user stories (zero-config, explicit config, local dev)
- Git-based resolution aligns with existing `ModuleConfig.source/ref` pattern in config.rs
- Local path support essential for plugin development workflow (FR-052)

**Alternatives Considered**:
- OCI registry (rejected: over-engineered for MVP, Git releases sufficient)
- npm-style registry (rejected: forces publishing workflow, Git is simpler)

---

### 3. Caching Architecture

**Decision**: Global cache at `~/.rw/plugins/<name>/<version>` with project symlinks

**Layout**:
```
~/.rw/plugins/
├── npm-script/
│   └── v1.0.0/
│       ├── plugin.wasm     # Downloaded binary
│       └── metadata.json   # Source URL, checksum, timestamp
└── custom-plugin/
    └── abc123def/          # Git commit hash for branch refs
        └── plugin.wasm

.rw/plugins/                 # Project directory (gitignored)
├── npm-script.wasm -> ~/.rw/plugins/npm-script/v1.0.0/plugin.wasm
└── custom-plugin.wasm -> ~/.rw/plugins/custom-plugin/abc123def/plugin.wasm
```

**Rationale**:
- Matches FR-004/FR-005 requirements
- Symlinks minimize disk usage (SC-003: <1MB local)
- Global cache enables offline usage after first download
- Per-version directories prevent conflicts

**Alternatives Considered**:
- Content-addressable storage (rejected: added complexity, version-based is sufficient)
- No symlinks, copy files (rejected: wastes disk, violates SC-003)

---

### 4. Pre-built vs Source Build Strategy

**Decision**: Prioritize pre-built, fallback to containerized source build

**Resolution Order**:
1. Check global cache for matching version
2. For git sources: Check GitHub/GitLab Releases for `.wasm` asset
3. If no binary: Clone source and build via `docker run` with cargo-component

**Container Build Command**:
```bash
docker run --rm -v $(pwd):/workspace -w /workspace \
  ghcr.io/bytecodealliance/cargo-component:0.21 \
  cargo component build --release --target wasm32-wasip1
```

**Rationale**:
- Pre-built binaries are fastest path (SC-002: <500ms if cached)
- Container isolation ensures reproducible builds regardless of host toolchain
- cargo-component official image provides consistent WASM component compilation

**Alternatives Considered**:
- Native cargo-component (rejected: requires users to have Rust toolchain)
- WASI-SDK based builds (rejected: cargo-component is the standard for component model)

---

### 5. Lockfile Format

**Decision**: Extend existing lockfile with plugin section

**Schema** (`weaver.lock`):
```yaml
plugins:
  npm-script:
    version: "v1.0.0"
    source: "git:github.com/web-tree/repo-weaver-plugins/npm-script"
    sha256: "a1b2c3d4..."
    resolved_at: "2026-01-17T12:00:00Z"
  custom-plugin:
    version: "abc123def"
    source: "git:github.com/myorg/custom-plugin"
    sha256: "e5f6g7h8..."
```

**Rationale**:
- SHA256 integrity check satisfies FR-006
- `version` field supports both semver tags and commit hashes
- `resolved_at` helps debugging stale locks
- Format mirrors existing lockfile structure

---

### 6. Security Model

**Decision**: Explicit capability grants via plugin metadata + WIT interface

**Implementation**:
- Plugins have `process` import capability by default (execute external commands)
- No implicit filesystem or network access (WASM sandbox)
- Future: Plugin manifest can declare required capabilities (`capabilities.toml`)
- Host can reject `exec` calls based on allowlist policy

**Rationale**:
- Matches FR-011/FR-012 requirements
- Existing WIT interface already restricts capabilities
- Current trust model: plugins from known sources are trusted

---

### 7. Technology Choices

| Aspect | Choice | Rationale |
|--------|--------|-----------|
| HTTP Client | `reqwest` (blocking) | Already used in ecosystem, well-maintained |
| Git Operations | `git2` crate OR shell-out | git2 for performance, shell-out as fallback |
| Container Detection | `which docker/podman` | Simple, cross-platform |
| Checksum | `sha2` crate | Already in dependencies |
| Symlinks | `std::os::unix::fs::symlink` | Cross-platform via conditional compilation |

---

## Resolved Clarifications

All clarifications from spec addressed:

| Question | Resolution |
|----------|------------|
| Resolution failure handling | FR-010: Detailed error + suggestions + exit |
| Security model | FR-011/012: WASM sandbox + explicit capabilities |
| Version updates | FR-006b: Locked until `rw plugins update` |
| Observability | FR-013: Verbose mode with structured logs |
| Missing container runtime | FR-010b: Error with install instructions |
