# feature-tasks: Repo Weaver MVP

> **Source**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Phase 1: Setup

- [x] T001 Initialize Cargo Workspace with `crates/cli`, `crates/core`, `crates/ops`
- [x] T002 [P] Create `crates/cli` skeleton and `main.rs`
- [x] T003 [P] Create `crates/core` lib skeleton
- [x] T004 [P] Create `crates/ops` lib skeleton
- [x] T005 [P] Setup `tracing` logging with JSON support and `Secret<T>` redaction wrapper in `crates/core`
- [x] T006 Setup integration test harness (assert_cmd) in `tests/integration/common.rs`

## Phase 2: Foundational

- [x] T007 [P] Define Weaver configuration structs in `crates/core/src/config.rs`
- [x] T008 [P] Define State manifest structs in `crates/core/src/state.rs`
- [x] T009 [P] Setup Wasmtime host engine in `crates/core/src/plugin/wasm.rs`
- [x] T010 [P] Setup Tera template engine in `crates/core/src/template.rs`
- [x] T011 [P] Implement git operations (clone/checkout) in `crates/ops/src/git.rs`
- [x] T012 [P] Implement fs operations (ensure_dir, copy) in `crates/ops/src/fs.rs`
- [x] T013 [US1] Create integration test for bootstrap scenario in `tests/integration/apply.rs`
- [x] T014 [P] [US1] Implement `ensure.folder.exists` action logic in `crates/core/src/engine.rs`
- [x] T015 [P] [US1] Implement `ensure.file.from_template` action logic in `crates/core/src/engine.rs`
- [x] T016 [P] [US1] Implement `ensure.task.wrapper` action logic in `crates/core/src/engine.rs`
- [x] T017 [US1] Implement Module resolution (Global Cache `~/.rw/store`, Offline Fallback, Lockfile Integrity Check) in `crates/core/src/module.rs`
- [x] T018 [US1] Implement App instantiation and input validation in `crates/core/src/app.rs`
- [x] T019 [P] [US1] Implement `rw apply` command logic in `crates/cli/src/commands/apply.rs`
- [x] T020 [P] [US1] Implement CLI prompts for missing variables in `crates/cli/src/prompts.rs`
- [x] T021 [P] [US1] Implement AWS SSM Secret Provider via Host Exec (`aws` CLI) in `plugins/aws-ssm/`
  - [x] Define `process.exec` WIT interface
  - [x] Implement `process.exec` host logic in `crates/core`
  - [x] Update `aws-ssm` plugin to use `process.exec`
- [x] T022 [US1] Implement Secret resolution with provider loading and Best-Effort Redaction in `crates/core/src/secret.rs`

## Phase 4: Update and Converge (User Story 2 - P1)

**Goal**: As a maintainer, I can update module pins and preserve my local changes.
**Independent Test**: `rw apply` updates managed files but reports drift/stops on user changes.

- [x] T023 [US2] Create integration test for update/drift scenarios in `tests/integration/update.rs`
- [x] T024 [P] [US2] Implement state calculation and validation in `crates/core/src/state.rs`
- [x] T025 [P] [US2] Implement drift detection logic in `crates/core/src/engine.rs`
- [x] T026 [US2] Update `rw apply` to handle drift prompts (Skip/Overwrite) and Strict Failure for AI unavailability in `crates/cli/src/commands/apply.rs`

## Phase 5: Operational Pipeline Execution (User Story 3 - P2)

**Goal**: As an operator, I can run multi-step tasks with data passing.
**Independent Test**: `rw run` fails fast on error and passes outputs between steps.

- [x] T027 [US3] Create integration test for pipeline execution in `tests/integration/run.rs`
- [x] T028 [P] [US3] Implement output capturing and env var injection in `crates/core/src/engine.rs`
- [x] T029 [P] [US3] Implement `rw run` command logic in `crates/cli/src/commands/run.rs`

## Phase 6: Polish & Cross-Cutting

- [x] T030 [P] Verify JSON logging format for CI/CD
- [x] T031 [P] Create `examples/wasm/rust-basic` to verify multi-language plugin support

## Dependencies

- T017 (Module Resolution) must happen before App instantiation (T018)
- T021 (AWS SSM Plugin) verified by T022 (Secret Resolution)
- T012 (FS Ops) required for T014/T015
- Phase 2 tasks are blocking for Phase 3
