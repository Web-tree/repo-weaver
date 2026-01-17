# Tasks: Plugin Management System

**Input**: Design documents from `/specs/003-plugin-management-system/`
**Prerequisites**: plan.md ✓, spec.md ✓, data-model.md ✓, contracts/api.md ✓, research.md ✓, quickstart.md ✓

**Tests**: Not explicitly requested in specification. Test tasks omitted.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story this task belongs to (US1-US4)
- Exact file paths included in all task descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and module structure

- [ ] T001 Create plugin management module structure at `crates/core/src/plugin/` with `mod.rs` updates
- [ ] T002 [P] Add dependencies `reqwest`, `sha2` to `crates/core/Cargo.toml`
- [ ] T003 [P] Create CLI commands module structure at `crates/cli/src/commands/plugins.rs`
- [ ] T004 Register `plugins` command group in `crates/cli/src/main.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures and utilities that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 Add `PluginConfig` struct with `git`, `path`, `ref` fields in `crates/core/src/config.rs`
- [ ] T006 Add `plugins: HashMap<String, PluginConfig>` to `WeaverConfig` in `crates/core/src/config.rs`
- [ ] T007 [P] Create `PluginSource` enum (Local, Git, Registry) in `crates/core/src/plugin/mod.rs`
- [ ] T008 [P] Create `PluginMetadata` struct (sha256, resolved_at, source_url, build_method) in `crates/core/src/plugin/mod.rs`
- [ ] T009 [P] Create `ResolvedPlugin` struct in `crates/core/src/plugin/mod.rs`
- [ ] T010 Add `PluginLock` struct to `crates/core/src/lockfile.rs`
- [ ] T011 Extend `LockFile` struct with `plugins: HashMap<String, PluginLock>` in `crates/core/src/lockfile.rs`
- [ ] T012 [P] Create `PluginCache` struct with `root: PathBuf` in `crates/core/src/plugin/cache.rs`
- [ ] T013 Implement `PluginCache::has()`, `get()`, `store()` methods in `crates/core/src/plugin/cache.rs`
- [ ] T014 Implement `PluginCache::link()` symlink creation (Unix) in `crates/core/src/plugin/cache.rs`
- [ ] T015 [P] Add cache directory accessibility check on startup in `crates/core/src/plugin/cache.rs`
- [ ] T016 [P] Implement broken symlink detection and auto-cleanup in `crates/core/src/plugin/cache.rs`
- [ ] T017 [P] Create error types with remediation suggestions in `crates/core/src/plugin/mod.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Zero-Config Plugin Usage (Priority: P1) 🎯 MVP

**Goal**: Users can use standard ensures (`npm.script`) without explicit plugin configuration

**Independent Test**: Use `npm.script` ensure type in a module manifest without a `plugins` section and verify automatic download/execution

### Implementation for User Story 1

- [ ] T018 [US1] Create `PluginResolver` struct in `crates/core/src/plugin/resolver.rs`
- [ ] T019 [US1] Implement `PluginResolver::new()` constructor in `crates/core/src/plugin/resolver.rs`
- [ ] T020 [US1] Implement `resolve_ensure_type()` auto-discovery logic in `crates/core/src/plugin/resolver.rs`
- [ ] T021 [US1] Implement default registry URL resolution (RW_REGISTRY_URL env var → config → default) in `crates/core/src/plugin/resolver.rs`
- [ ] T022 [P] [US1] Create `PluginFetcher` struct in `crates/core/src/plugin/fetcher.rs`
- [ ] T023 [US1] Implement `fetch_release()` for GitHub Releases download in `crates/core/src/plugin/fetcher.rs`
- [ ] T024 [US1] Add download timeout (30s), retry logic (3 retries with exponential backoff) in `crates/core/src/plugin/fetcher.rs`
- [ ] T025 [US1] Implement SHA256 checksum calculation and verification in `crates/core/src/plugin/resolver.rs`
- [ ] T026 [US1] Integrate plugin resolution into `rw apply` command flow (existing code modification)
- [ ] T027 [US1] Add lockfile generation/update for resolved plugins in `crates/core/src/plugin/resolver.rs`
- [ ] T028 [US1] Implement `--offline` flag handling (error if plugin not cached) in `crates/core/src/plugin/resolver.rs`

**Checkpoint**: Zero-config usage should now work - `rw apply` auto-downloads required plugins

---

## Phase 4: User Story 2 - Explicit Plugin Configuration (Priority: P1)

**Goal**: Platform engineers can explicitly define plugin sources and versions with lockfile integrity

**Independent Test**: Define a plugin in `weaver.yaml` with a specific git ref and verify exact version is locked and reused

### Implementation for User Story 2

- [ ] T029 [US2] Implement `PluginResolver::resolve()` for explicit PluginConfig in `crates/core/src/plugin/resolver.rs`
- [ ] T030 [US2] Implement Git source resolution (git URL + ref → release download) in `crates/core/src/plugin/resolver.rs`
- [ ] T031 [US2] Implement lockfile verification (checksum mismatch detection) in `crates/core/src/plugin/resolver.rs`
- [ ] T032 [US2] Add checksum mismatch error with remediation (require `rw plugins update`) in `crates/core/src/plugin/resolver.rs`
- [ ] T033 [P] [US2] Implement `rw plugins list` command in `crates/cli/src/commands/plugins.rs`
- [ ] T034 [P] [US2] Implement `rw plugins verify` command in `crates/cli/src/commands/plugins.rs`
- [ ] T035 [US2] Implement `rw plugins update` command (single plugin + --all) in `crates/cli/src/commands/plugins.rs`
- [ ] T036 [US2] Add warning for commit-hash-pinned plugins on update in `crates/cli/src/commands/plugins.rs`

**Checkpoint**: Explicit configuration and lockfile integrity should work

---

## Phase 5: User Story 3 - Local Plugin Development (Priority: P2)

**Goal**: Plugin developers can point to local directories for rapid iteration

**Independent Test**: Configure a plugin with `path: ./my-plugin` and verify local WASM changes are picked up immediately

### Implementation for User Story 3

- [ ] T037 [US3] Add Local path resolution to `PluginResolver::resolve()` in `crates/core/src/plugin/resolver.rs`
- [ ] T038 [US3] Implement path existence validation with clear error message in `crates/core/src/plugin/resolver.rs`
- [ ] T039 [US3] Skip caching and symlinking for local path plugins in `crates/core/src/plugin/resolver.rs`
- [ ] T040 [US3] Add mutual exclusivity validation (path vs git) in config parsing in `crates/core/src/config.rs`

**Checkpoint**: Local path plugin development workflow should work

---

## Phase 6: User Story 4 - Source Build Fallback (Priority: P3)

**Goal**: Users can build plugins from source when pre-built binaries aren't available

**Independent Test**: Configure a plugin without release assets and verify containerized build is initiated

### Implementation for User Story 4

- [ ] T041 [US4] Implement `detect_container_runtime()` (docker → podman) in `crates/core/src/plugin/fetcher.rs`
- [ ] T042 [US4] Implement `build_source()` containerized cargo-component build in `crates/core/src/plugin/fetcher.rs`
- [ ] T043 [US4] Add build timeout handling (10 minute default) in `crates/core/src/plugin/fetcher.rs`
- [ ] T044 [US4] Add host cargo cache mount optimization in `crates/core/src/plugin/fetcher.rs`
- [ ] T045 [US4] Implement WASM validation after build (wasmtime load check) in `crates/core/src/plugin/fetcher.rs`
- [ ] T046 [US4] Add error with Docker/Podman installation instructions when no runtime found in `crates/core/src/plugin/fetcher.rs`
- [ ] T047 [US4] Integrate source build fallback into resolution flow in `crates/core/src/plugin/resolver.rs`

**Checkpoint**: Full source build fallback should work when no pre-built binaries are available

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Observability, documentation, and improvements that affect multiple stories

- [ ] T048 Implement verbose mode (`-v`) structured logging for resolution/loading/timing in `crates/core/src/plugin/resolver.rs`
- [ ] T049 [P] Add progress spinner for downloads/builds in `crates/cli/src/commands/plugins.rs`
- [ ] T050 [P] Implement `rw plugins prune` command to remove unused versions in `crates/cli/src/commands/plugins.rs`
- [ ] T051 [P] Add YAML structure validation with line numbers for weaver.yaml errors in `crates/core/src/config.rs`
- [ ] T052 [P] Add lockfile merge conflict documentation to README or docs/
- [ ] T053 Run all verification scenarios from quickstart.md and validate functionality
- [ ] T054 Run `cargo clippy` and fix any warnings

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 (P1) and US2 (P1) can proceed in parallel
  - US3 (P2) can start after Foundational (no US1/US2 dependency)
  - US4 (P3) can start after Foundational (no prior story dependency)
- **Polish (Phase 7)**: Depends on core user stories (at least US1+US2) being complete

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on other stories - core auto-discovery flow
- **User Story 2 (P1)**: No dependency on US1 - explicit config path (shares resolver infrastructure)
- **User Story 3 (P2)**: No dependency on US1/US2 - local path is independent resolution path
- **User Story 4 (P3)**: Shares fetcher infrastructure but is independent fallback path

### Within Each User Story

- Resolver before fetcher integration
- Core logic before CLI commands
- Base implementation before error handling refinements

### Parallel Opportunities

**Phase 1 (Setup)**:
```
T002, T003 can run in parallel
```

**Phase 2 (Foundational)**:
```
T007, T008, T009 can run in parallel (different structs)
T012, T015, T016, T017 can run in parallel (different concerns)
```

**Phase 3 (US1)**:
```
T022 (create fetcher) can run parallel with resolver work
```

**Phase 4 (US2)**:
```
T033, T034 can run in parallel (different CLI commands)
```

**Phase 7 (Polish)**:
```
T049, T050, T051, T052 can run in parallel
```

---

## Parallel Example: Foundation Phase

```bash
# Launch struct definitions together:
Task T007: "Create PluginSource enum in crates/core/src/plugin/mod.rs"
Task T008: "Create PluginMetadata struct in crates/core/src/plugin/mod.rs"
Task T009: "Create ResolvedPlugin struct in crates/core/src/plugin/mod.rs"

# Launch cache utilities together:
Task T015: "Add cache directory accessibility check"
Task T016: "Implement broken symlink detection"
Task T017: "Create error types with remediation suggestions"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Zero-Config)
4. **STOP and VALIDATE**: Test with `npm.script` ensure type
5. Deploy/demo if ready

### Recommended Incremental Delivery

1. Setup + Foundational → Infrastructure ready
2. Add User Story 1 → Test auto-discovery → Deploy (Core MVP!)
3. Add User Story 2 → Test explicit config + lockfile → Deploy (Team-ready)
4. Add User Story 3 → Test local dev workflow → Deploy (Plugin dev ready)
5. Add User Story 4 → Test source builds → Deploy (Full feature)
6. Polish phase → Production ready

### Single Developer Order

For solo development, execute in strict order:
- Phase 1 → Phase 2 → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US3) → Phase 6 (US4) → Phase 7

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Each user story is independently testable after Foundational phase
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Platform scope: macOS and Linux only (Windows users use WSL)
