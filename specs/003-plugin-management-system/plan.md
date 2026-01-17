# Implementation Plan: Plugin Management System

**Branch**: `003-plugin-management-system` | **Date**: 2026-01-17 | **Spec**: [spec.md](file:///Users/max/git/webtree/repo-weaver/specs/003-plugin-management-system/spec.md)  
**Input**: Feature specification from `/specs/003-plugin-management-system/spec.md`

## Summary

Implement a comprehensive plugin management system for Repo Weaver that enables:
1. **Zero-config usage** - Built-in ensure types work without explicit plugin configuration
2. **Explicit versioning** - Pin plugins to specific git refs with lockfile integrity
3. **Local development** - Point to local directories for rapid plugin iteration
4. **Source builds** - Containerized fallback when pre-built binaries aren't available

Built on existing wasmtime v40 infrastructure in `crates/core/src/plugin/`.

## Technical Context

**Language/Version**: Rust 2024 edition  
**Primary Dependencies**: wasmtime 40.0.0, sha2, reqwest (new), git2 or shell-out  
**Storage**: File-based global cache (`~/.rw/plugins/`) with symlinks  
**Testing**: cargo test, integration tests in `tests/`  
**Target Platform**: Cross-platform (macOS, Linux, Windows)  
**Project Type**: Single crate extension (`crates/core`)  
**Performance Goals**: <500ms resolution for cached plugins (SC-002)  
**Constraints**: <1MB local disk usage via symlinks (SC-003), offline-capable after first download  
**Scale/Scope**: Support for 10+ plugins per project

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Declarative Desired State | ✅ PASS | plugins section in weaver.yaml describes desired state |
| II. Native Tool First | ✅ PASS | Uses git for fetching, docker/podman for builds |
| III. Composition and Reuse | ✅ PASS | Plugins are versionable, pin-able modules |
| IV. Idempotency & Determinism | ✅ PASS | Lockfile ensures exact versions, checksums verified |
| V. Update Safety | ✅ PASS | Explicit `rw plugins update` required, no auto-upgrade |
| VI. Secret Decoupling | ✅ PASS | No secrets in plugin config, resolved at runtime |

**Post-Design Re-check**: All gates still pass. Design aligns with constitution.

## Project Structure

### Documentation (this feature)

```text
specs/003-plugin-management-system/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: Technical decisions
├── data-model.md        # Phase 1: Entity definitions
├── quickstart.md        # Phase 1: Usage guide
├── contracts/           # Phase 1: API contracts
│   └── api.md
└── checklists/          # Requirements checklist
    └── requirements.md
```

### Source Code (repository root)

```text
crates/core/src/
├── plugin/
│   ├── mod.rs              # [MODIFY] Add new modules
│   ├── ensure_wasm.rs      # Existing - no changes
│   ├── wasm.rs             # Existing - no changes
│   ├── resolver.rs         # [NEW] Plugin resolution logic
│   ├── cache.rs            # [NEW] Global cache management
│   ├── fetcher.rs          # [NEW] Download/build plugins
│   └── lockfile.rs         # [NEW] Plugin lockfile handling
├── config.rs               # [MODIFY] Add PluginConfig struct
└── lockfile.rs             # [MODIFY] Add plugins section

crates/cli/src/
├── commands/
│   └── plugins.rs          # [NEW] Plugin subcommands
└── main.rs                 # [MODIFY] Register plugins command

tests/
├── fixtures/
│   └── plugin_resolution/  # [NEW] Test fixtures
└── integration/
    └── plugin_tests.rs     # [NEW] Integration tests
```

**Structure Decision**: Single project extension. All plugin management logic in `crates/core/src/plugin/` with CLI commands in `crates/cli/src/commands/plugins.rs`.

---

## Proposed Changes

### Core Library (`crates/core`)

#### [MODIFY] [config.rs](file:///Users/max/git/webtree/repo-weaver/crates/core/src/config.rs)

- Add `PluginConfig` struct with `git`, `path`, `ref` fields
- Add `plugins: HashMap<String, PluginConfig>` to `WeaverConfig`

#### [NEW] [resolver.rs](file:///Users/max/git/webtree/repo-weaver/crates/core/src/plugin/resolver.rs)

- `PluginResolver` struct: coordinates resolution logic
- `resolve()`: resolve config to cached WASM path
- `resolve_ensure_type()`: auto-discovery from type name
- `verify()`: check against lockfile

#### [NEW] [cache.rs](file:///Users/max/git/webtree/repo-weaver/crates/core/src/plugin/cache.rs)

- `PluginCache` struct: manage `~/.rw/plugins/` directory
- `has()`, `get()`, `store()`, `link()` methods
- Cross-platform symlink creation

#### [NEW] [fetcher.rs](file:///Users/max/git/webtree/repo-weaver/crates/core/src/plugin/fetcher.rs)

- `PluginFetcher` struct: download and build plugins
- `fetch_release()`: GitHub/GitLab release download
- `build_source()`: containerized cargo-component build
- `detect_container_runtime()`: check docker/podman

#### [MODIFY] [lockfile.rs](file:///Users/max/git/webtree/repo-weaver/crates/core/src/lockfile.rs)

- Add `PluginLock` struct with `version`, `source`, `sha256`, `resolved_at`
- Extend `LockFile` to include `plugins` section

---

### CLI (`crates/cli`)

#### [NEW] [plugins.rs](file:///Users/max/git/webtree/repo-weaver/crates/cli/src/commands/plugins.rs)

- `PluginsCommand` with subcommands: `list`, `update`, `verify`
- Integrate with `PluginResolver` from core

#### [MODIFY] [main.rs](file:///Users/max/git/webtree/repo-weaver/crates/cli/src/main.rs)

- Register `plugins` command group

---

## Verification Plan

### Automated Tests

**Existing test infrastructure**:
```bash
# Run all tests
cargo test --workspace
```

**New tests to add**:

1. **Unit tests** in `crates/core/src/plugin/`:
   - `resolver.rs` tests: resolution logic for each source type
   - `cache.rs` tests: storage, retrieval, symlink creation
   - `fetcher.rs` tests: mock HTTP responses, build detection

2. **Integration tests** in `tests/integration/plugin_tests.rs`:
   - Test plugin resolution with actual git repos (using test fixtures)
   - Test lockfile generation and verification
   - Test checksum validation

**Commands to run**:
```bash
# Unit tests
cargo test -p repo-weaver-core

# Integration tests  
cargo test --test plugin_tests

# All tests
cargo test --workspace
```

### Manual Verification

1. **Zero-config test**:
   - Create minimal `weaver.module.yaml` with `type: npm.script`
   - Run `rw apply`
   - Verify plugin downloads automatically and ensure executes

2. **Explicit config test**:
   - Add plugins section to `weaver.yaml` with specific git ref
   - Run `rw apply`
   - Verify `weaver.lock` contains correct SHA256
   - Run again on new machine, verify exact version used

3. **Local path test**:
   - Configure plugin with `path: ./my-plugin`
   - Modify the local WASM
   - Run `rw apply`
   - Verify changes are picked up immediately

4. **Verbose mode test**:
   - Run `rw apply -v`
   - Verify structured logs show resolution, loading, timing

## Complexity Tracking

No constitution violations requiring justification.
