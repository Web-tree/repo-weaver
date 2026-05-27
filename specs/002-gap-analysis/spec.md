# Feature Specification: Repo Weaver MVP Gap Analysis

**Feature Branch**: `002-gap-analysis`  
**Created**: 2026-01-03  
**Status**: Draft  
**Input**: Gap analysis between PRD.md Section 12 (MVP definition) and 001-repo-weaver-mvp implementation

## Overview

This specification documents the gaps between the original PRD MVP requirements and what was implemented in `001-repo-weaver-mvp`. The goal is to prioritize and plan implementation of the remaining MVP features.

## Clarifications

### Session 2026-01-03

- Q: How should `rw apply` behave when an existing submodule is at a different ref than configured?
  - A: **Safe Checkout** - Checkout configured ref only if submodule working tree is clean; fail with error if dirty.
- Q: When should users choose `git.submodule` vs `git.clone_pinned`?
  - A: **Clone Default** - Prefer `clone_pinned` for simplicity; use `submodule` only when explicit git history tracking is needed.
- Q: How should the git plugin behave when clone/fetch fails due to network issues?
  - A: **Fallback to Cache** - If target path already exists with content, keep existing and warn; otherwise fail.

## Gap Analysis Summary

### ✅ Implemented in 001-repo-weaver-mvp

| Feature | Status | Notes |
|---------|--------|-------|
| YAML loader with modules/apps | ✅ Done | `weaver.yaml` parsing works |
| Apps with path scoping | ✅ Done | `AppConfig` with path field |
| Modules pinned by git ref | ✅ Done | Global cache `~/.rw/store` |
| Answers storage + prompting | ✅ Done | `.rw/answers.yaml` |
| `ensure.folder.exists` | ✅ Done | Engine executes it |
| `ensure.file.from_template` | ✅ Done | Tera templating |
| `ensure.task.wrapper` | ✅ Done | Taskfile generation |
| `rw plan` | ✅ Done | Dry-run mode |
| `rw apply` | ✅ Done | Full execution |
| `rw run` | ✅ Done | Pipeline execution |
| Pipeline tasks with JSON capture | ✅ Done | Output capturing works |
| AWS SSM WASM plugin | ✅ Done | Host exec via `process.exec` |

### ❌ NOT Implemented (PRD MVP Requirements)

| Feature | PRD Reference | Priority |
|---------|---------------|----------|
| `includes` YAML merging | PRD §5.1-5.2 | P1 |
| `rw list` command | PRD §8 | P1 |
| `rw describe <app>` command | PRD §8 | P2 |
| `rw check [app]` command | PRD §8 | P2 |
| `rw module list` command | PRD §8 | P2 |
| `rw module update` command | PRD §8 | P2 |
| `ensure.git.submodule` (or clone pinned) | PRD §12 | P1 |
| `ensure.npm.script` | PRD §12 | P2 |
| `ensure.ai.patch` | PRD §12 | P3 |
| k3s-nebula validation workflow | PRD §14 | P1 |

---

## User Scenarios & Testing

### User Story 1 - Config Includes & Fragments (Priority: P1)

As a platform engineer managing a monorepo, I want to split my `weaver.yaml` into multiple files so that each app team can own their configuration independently.

**Why this priority**: Large monorepos require modular config management. Without this, all config lives in one massive file.

**Independent Test**: Can be tested by creating `weaver.yaml` with `includes: ["weaver.d/*.yaml"]`, placing app configs in fragments, and running `rw apply` to verify merged config.

**Acceptance Scenarios**:

1. **Given** a `weaver.yaml` with `includes: ["weaver.d/*.yaml"]` and fragment files, **When** I run `rw plan`, **Then** all apps from fragments are discovered and planned.
2. **Given** a fragment with duplicate app names, **When** I run `rw apply`, **Then** the tool fails with a clear conflict error.
3. **Given** an include glob matching no files, **When** I run `rw apply`, **Then** the tool proceeds with a warning (not an error).

---

### User Story 2 - Discovery Commands (Priority: P1)

As a developer joining a project, I want to list all apps and tasks defined in the workspace so I can understand what's available.

**Why this priority**: Essential for discoverability. Users need visibility into what the tool manages.

**Independent Test**: Can be tested by running `rw list` in a workspace and verifying apps/tasks are displayed.

**Acceptance Scenarios**:

1. **Given** a workspace with 3 apps, **When** I run `rw list`, **Then** I see all 3 app names with their paths.
2. **Given** a workspace with tasks defined, **When** I run `rw list`, **Then** I see available tasks per app.
3. **Given** a workspace with no apps, **When** I run `rw list`, **Then** I see "No apps defined" message.

---

### User Story 3 - App Inspection (Priority: P2)

As a maintainer, I want to inspect the resolved configuration of a specific app to debug issues and understand the final merged state.

**Why this priority**: Debugging complex module inheritance requires seeing the fully resolved config.

**Independent Test**: Can be tested by running `rw describe <app>` and verifying outputs are correctly resolved/merged.

**Acceptance Scenarios**:

1. **Given** an app extending a module, **When** I run `rw describe my-app`, **Then** I see all inputs, outputs, tasks, and ensures merged.
2. **Given** an app with secrets, **When** I run `rw describe my-app`, **Then** secrets are redacted as `***`.
3. **Given** a non-existent app name, **When** I run `rw describe missing-app`, **Then** error message with available apps is shown.

---

### User Story 4 - Git Submodule/Clone Pinned (Priority: P1)

As a platform engineer, I want to vendor upstream modules as git submodules or pinned clones so that I can track exact versions and review changes.

**Why this priority**: Core module distribution mechanism. Required for true "apps from apps" composition.

**Independent Test**: Can be tested by configuring `ensure.git.submodule` and verifying submodule is added at correct path with correct ref.

**Acceptance Scenarios**:

1. **Given** an ensure with `type: git.submodule`, **When** I run `rw apply`, **Then** submodule is initialized at specified path with pinned ref.
2. **Given** an existing submodule at different ref with **clean working tree**, **When** I run `rw apply`, **Then** submodule is updated to new ref.
3. **Given** an existing submodule with **dirty working tree**, **When** I run `rw apply`, **Then** apply fails with "dirty working tree" error (Safe Checkout behavior).
4. **Given** a submodule path conflict, **When** I run `rw plan`, **Then** conflict is reported before any changes.

---

### User Story 5 - Module Management Commands (Priority: P2)

As a maintainer, I want CLI commands to list and update module versions so I can manage dependencies without editing YAML manually.

**Why this priority**: Convenience feature for module lifecycle management.

**Independent Test**: Can be tested by running `rw module list` and `rw module update <name> --ref <new>`.

**Acceptance Scenarios**:

1. **Given** a workspace with 2 modules, **When** I run `rw module list`, **Then** I see module names, sources, and current refs.
2. **Given** a module name and new ref, **When** I run `rw module update foo --ref v2.0.0`, **Then** `weaver.yaml` is updated and lockfile regenerated.
3. **Given** an invalid module name, **When** I run `rw module update missing --ref v1`, **Then** error with available modules is shown.

---

### User Story 6 - Validation Commands (Priority: P2)

As an operator, I want to run validation checks defined by modules so I can verify the workspace is correctly configured before applying changes.

**Why this priority**: Pre-flight checks prevent costly failures during apply.

**Independent Test**: Can be tested by defining `checks` in app config and running `rw check`.

**Acceptance Scenarios**:

1. **Given** an app with 2 check commands defined, **When** I run `rw check my-app`, **Then** both checks execute and results are reported.
2. **Given** a failing check, **When** I run `rw check`, **Then** exit code is non-zero and failure details shown.
3. **Given** no checks defined, **When** I run `rw check`, **Then** message "No checks defined" is shown.

---

### User Story 7 - npm Script Ensure (Priority: P2)

As a TypeScript developer, I want to ensure npm scripts exist in my package.json using native npm tooling so I can standardize project scripts.

**Why this priority**: TypeScript/Node.js is a major ecosystem. NPM integration enables broader adoption.

**Independent Test**: Can be tested by configuring `ensure.npm.script` and verifying script appears in package.json.

**Acceptance Scenarios**:

1. **Given** an ensure with `type: npm.script` for script "build", **When** I run `rw apply`, **Then** `npm pkg set scripts.build=...` is executed.
2. **Given** an existing script with different value, **When** I run `rw plan`, **Then** diff is shown.
3. **Given** no package.json exists, **When** I run `rw apply`, **Then** clear error is shown.

---

### User Story 8 - AI Patch Ensure (Priority: P3)

As a maintainer, I want to use AI to make complex edits that are verified and rollback-safe so I can automate sophisticated refactoring.

**Why this priority**: Advanced feature. Core functionality must work first.

**Independent Test**: Can be tested by configuring `ensure.ai.patch` with a prompt and verify step.

**Acceptance Scenarios**:

1. **Given** an AI patch ensure with verify command, **When** I run `rw apply`, **Then** AI diff is generated, applied via git, and verify runs.
2. **Given** verify fails after AI patch, **When** error occurs, **Then** patch is rolled back automatically.
3. **Given** AI service unreachable, **When** I run `rw apply`, **Then** strict failure (no fallback).

---

### User Story 9 - k3s-nebula End-to-End Validation (Priority: P1)

As a platform engineer, I want to bootstrap a complete k3s-nebula workspace with `rw apply` and run the install pipeline to validate the full workflow.

**Why this priority**: PRD acceptance criteria requires this specific validation (PRD §14).

**Independent Test**: Can be tested by running acceptance criteria from PRD §14.

**Acceptance Scenarios**:

1. **Given** empty folder and k3s-nebula module config, **When** I run `rw apply`, **Then** Taskfile + tfvars are generated.
2. **Given** working workspace, **When** I run `rw run app install`, **Then** terraform plan, apply, output capture, and kubectl apply execute in order.
3. **Given** pinned ref update, **When** I run `rw apply`, **Then** prompts for new vars and updates managed files.

---

## Requirements

### Functional Requirements

#### Config & Includes

- **FR-001**: System MUST support `includes` in `weaver.yaml` to load and merge YAML fragments via glob patterns.
- **FR-002**: System MUST perform deep merge of maps (vars, secrets) and concatenation of arrays (ensures, checks).
- **FR-003**: System MUST apply deterministic override rules (later files override earlier files).

#### CLI Commands

- **FR-004**: System MUST provide `rw list` command showing apps, tasks, and module dependencies.
- **FR-005**: System MUST provide `rw describe <app>` command showing fully resolved app config.
- **FR-006**: System MUST provide `rw check [app]` command executing defined validation checks.
- **FR-007**: System MUST provide `rw module list` command showing defined modules with source/ref.
- **FR-008**: System MUST provide `rw module update <name> --ref <newRef>` command.

#### Ensure Types

- **FR-009**: System MUST implement `ensure.git.submodule` for vendoring upstream dependencies.
- **FR-010**: System MUST implement `ensure.git.clone_pinned` as alternative to submodules.
- **FR-011**: System MUST implement `ensure.npm.script` using `npm pkg set` native tooling.
- **FR-012**: System MUST implement `ensure.ai.patch` with diff generation, apply, verify, and rollback.

#### Task Enhancements

- **FR-013**: System MUST support task composition via `call` to invoke other tasks.
- **FR-014**: System MUST support importing tasks from modules.

### Edge Cases & Error Handling

- **EC-001**: Include glob matching no files should warn but not fail.
- **EC-002**: Duplicate app names across fragments must fail with conflict error.
- **EC-003**: AI patch without git repo must fail with clear error.
- **EC-004**: npm ensure without package.json must fail with actionable error.
- **EC-005**: Network failure during git clone/fetch: if target path exists with content, warn and keep existing; otherwise fail.

### Key Entities

- **Include**: A glob pattern or path that specifies additional config fragments to merge.
- **Check**: A validation command executed to verify workspace state.

---

## Success Criteria

### Measurable Outcomes

- **SC-001**: User can bootstrap k3s-nebula workspace from zero to working config in under 30 seconds (PRD §14.1).
- **SC-002**: User can run `rw list` and see all apps/tasks within 1 second.
- **SC-003**: Include fragments with 10+ files are merged in under 2 seconds.
- **SC-004**: `rw module update` updates config and regenerates lockfile atomically.
- **SC-005**: AI patch rollback succeeds 100% of the time when verify fails.

---

## Prioritization Summary

| Priority | Features |
|----------|----------|
| **P1** | Includes/merge, `rw list`, git.submodule, k3s-nebula validation |
| **P2** | `rw describe`, `rw check`, `rw module *`, npm.script |
| **P3** | ai.patch |

## Assumptions

- k3s-nebula module will be created/available for validation testing.
- AI patch will use Claude CLI or similar tool configured externally.
- npm tooling available in PATH for npm.script ensures.
