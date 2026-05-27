# Research: 002-Gap Analysis

**Feature**: Gap Analysis for REpo Weaver MVP  
**Date**: 2026-01-03

## Summary

This document captures research decisions for implementing the remaining MVP features identified in the gap analysis between PRD §12 and 001-repo-weaver-mvp.

---

## 1. YAML Includes & Merging

### Decision
Use **serde_yml** traversal with deep merge algorithm: maps merge recursively, arrays concatenate.

### Rationale
- Serde already parses YAML to `Value` enums - can traverse and merge before deserializing to typed structs
- Deep merge for maps preserves per-team overrides (vars, secrets)
- Array concat for ensures/checks maintains ordering (later files append)
- Deterministic: glob ordering is alphabetical, later overrides earlier

### Alternatives Considered
1. **yaml-merge crate**: External dependency, less control over conflict resolution
2. **Pre-processor approach**: Shell-level merging loses type safety
3. **Re-deserialize per fragment**: Would require complex partial struct handling

### Implementation Notes
- Add `includes: Vec<String>` field to `WeaverConfig`
- Create `config::loader` module with `load_with_includes()` function
- Use `glob` crate for pattern expansion
- Merge order: root config first, then includes alphabetically

---

## 2. Git Submodule/Clone Pinned

### Decision
Implement both `git.submodule` and `git.clone_pinned` as separate ensure types using native `git` commands.

### Rationale
- **Native Tool First** principle (Constitution II): Use `git submodule add`, `git clone`, `git checkout`
- Submodules provide history tracking; clone_pinned is simpler for read-only vendoring
- "Safe Checkout" behavior for existing submodules (spec clarification)

### Alternatives Considered
1. **git2-rs crate**: Adds complexity, libgit2 dependency; native git is more reliable
2. **Single abstraction**: Users have distinct use cases that warrant separate types

### Implementation Notes
- Extend ops/git.rs with `submodule_add()`, `submodule_update()`, `clone_pinned()` functions
- Add ensure type dispatcher in apply.rs
- Implement dirty-tree detection before checkout

---

## 3. CLI Discovery Commands (list, describe)

### Decision
Add `list` and `describe` subcommands to CLI with structured output.

### Rationale
- Essential for onboarding and debugging
- `list` shows apps/tasks at a glance
- `describe` shows fully resolved config (useful for module inheritance debugging)

### Alternatives Considered
1. **JSON-only output**: Less human-readable; add `--json` flag for scripts
2. **Single `info` command**: Splitting provides cleaner UX

### Implementation Notes
- Create `src/commands/list.rs` and `src/commands/describe.rs`
- Output format: table for list, YAML-like for describe
- Secrets redaction via visitor pattern

---

## 4. Module Management Commands

### Decision
Add `module list` and `module update` subcommands for dependency management.

### Rationale
- Convenience for version bumps without manual YAML editing
- `module list` shows current state from weaver.yaml
- `module update` modifies config and triggers cache refresh

### Alternatives Considered
1. **Lockfile-based only**: Less explicit; users prefer seeing pinned refs directly
2. **Separate `rw-modules` binary**: Unnecessary fragmentation

### Implementation Notes
- Add `src/commands/module.rs` with `list` and `update` subcommands
- Use clap subcommand nesting: `rw module list`, `rw module update <name> --ref <ref>`

---

## 5. Validation Checks Command

### Decision
Add `check` command that executes check definitions from app config.

### Rationale
- Pre-flight validation prevents costly failures during apply
- Aligns with "fail fast" principle

### Implementation Notes
- Add `checks: Vec<CheckDef>` to app config schema
- Create `src/commands/check.rs`
- Execute checks via native shell command

---

## 6. npm.script Ensure

### Decision
Use `npm pkg set scripts.<name>="<command>"` for idempotent script management.

### Rationale
- **Native Tool First** principle: `npm pkg` is the canonical way to modify package.json
- Avoids parsing/writing JSON manually

### Implementation Notes
- Add `ensure_npm_script()` in ops crate
- Pre-check: verify package.json exists, fail with actionable error if not

---

## 7. AI Patch Ensure

### Decision
Implement as three-phase operation: generate diff → apply diff → verify → rollback on failure.

### Rationale
- Constitution AI Integration Policy requires verify/rollback safety protocol
- External AI CLI tool (Claude CLI) generates unified diff
- Git handles application and rollback

### Implementation Notes
- Lower priority (P3) - implement after core P1/P2 features
- Requires git repo initialization check
- Stored verification command in ensure definition

---

## 8. Ensure Type Abstraction

### Decision
Create `Ensure` trait with `plan()` and `execute()` methods for polymorphic dispatch.

### Rationale
- Current apply.rs has inline file/template logic only
- Gap analysis adds git.submodule, git.clone_pinned, npm.script, ai.patch
- Trait-based design enables clean extension

### Implementation Notes
- Create `core/src/ensure.rs` with `Ensure` trait
- Move existing file/template logic into `EnsureFileCopy`, `EnsureTemplate` implementations
- Dispatcher in apply.rs matches on ensure type string

---

## Testing Strategy

### Unit Tests
- YAML merge algorithm (crates/core/src/config/loader)
- Ensure type dispatch logic

### Integration Tests
- Extend `TestContext` for include file scenarios
- Git submodule tests with mock remotes (existing pattern)
- CLI output verification with assert_cmd

### End-to-End
- k3s-nebula validation workflow (manual, per PRD §14)
