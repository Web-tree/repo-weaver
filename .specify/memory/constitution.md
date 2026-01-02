<!--
Sync Impact Report:
- Version change: 1.0.0 -> 1.1.0
- List of modified principles: Added "Secret Decoupling"
- Added sections: Principle VI
- Removed sections: None
- Templates requiring updates: specs/001-repo-weaver-mvp/plan.md (retroactive check)
-->
# Repo Weaver Constitution

## Core Principles

### I. Declarative Desired State
The YAML describes the state the repo should have. Running `rw` converges actual state to desired state. This is the primary interaction model.

### II. Native Tool First
Repo Weaver must not implement custom parsers for external ecosystems when a native tool can be used instead (e.g., `task`, `npm`, `go`, `cargo`, `terraform`, `kubectl`). It may only parse its own configs or JSON outputs from tools.

### III. Composition and Reuse
Support "apps from apps" by building blocks (modules) that can be combined to form more complex apps. Modules must be versionable and pin-able.

### IV. Idempotency & Determinism
Repo Weaver must be able to run repeatedly and be idempotent. Most changes should be done via deterministic actions. AI-based changes are supported but must be controlled options.

### V. Update Safety
Updating a module or template should not require recreating a repo. It should update in place, safe for local customizations. Unmanaged files must not be changed unless explicitly targeted.

### VI. Secret Decoupling
Configuration must never contain literal secrets. Secrets must be referenced by logical names in templates and resolved at runtime via providers (e.g., env vars, vaults). This ensures security and portability.

## Scope & Constraints

Repo Weaver manages scaffolding, folder structure, wrapper taskfiles, and config generation. It does NOT aim to become a full CI system, replace Terraform/Helm, or implement a full programming language.
MVP constraints: Single static binary (Go/Rust), cross-platform, versioned configuration schema.

## AI Integration Policy

AI patches must be controlled steps:
1. Use an external AI CLI tool.
2. Output a unified diff.
3. Apply diff using git/patch.
4. Run verify checks.
5. Rollback on failure.
This ensures AI actions are verifiable and safe.

## Governance

This Constitution supersedes all other practices and documentation. Amendments require documentation, approval, and a migration plan.

### Governance Rules
1. All modules and features must align with the "Native Tool First" principle.
2. Breaking changes to the specific YAML schema require a version bump.
3. All AI integration must adhere to the Verify/Rollback safety protocol.
4. Versioning follows Semantic Versioning 2.0.0 (MAJOR.MINOR.PATCH).

**Version**: 1.1.0 | **Ratified**: 2026-01-01 | **Last Amended**: 2026-01-02
