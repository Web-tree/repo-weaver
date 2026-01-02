# Implementation Plan: [FEATURE]

**Branch**: `001-repo-weaver-mvp` | **Date**: 2026-01-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-repo-weaver-mvp/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement the MVP of `repo-weaver`, a declarative tool for scaffolding and managing complex repositories using a "Modular Monolith" architecture in Rust. Key features include a `weaver.yaml` configuration, git-based module versioning, Jinja2 (Tera) templating, and an AWS SSM secrets provider implemented as a WASM component.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.92+
**Primary Dependencies**: `clap` (CLI), `serde` (Auth/Config), `tera` (Templates), `copy_dir` (FS), `dialoguer` (Prompts), `wasmtime` (Plugins), `assert_cmd` (Testing)
**Storage**: File System (Managed State), Git (Modules)
**Testing**: `cargo test`, `assert_cmd` (Integration), `tempfile` (Isolation)
**Target Platform**: CLI (mac/linux)
**Project Type**: Modular Monolith (Cargo Workspace)
**Performance Goals**: Bootstrap < 30s
**Constraints**: Single static binary, Cross-platform
**Scale/Scope**: MVP (Core logic + AWS SSM Plugin + Standard Ops Plugins)

## Core Concepts

### Module Repository

A "Module Repo" is the central source of truth for reusable building blocks.

- **Role**: Contains `modules/` directories, each being a versionable unit.
- **Content**: Terraform modules, Kubernetes YAMLs, Helm charts, Taskfiles, Bash scripts, and GitHub pipeline templates.
- **Usage**: Referenced by `weaver.yaml` via Git URL and Ref (tag/commit). The core engine clones this repo (managed cache) to resolve dependencies.


## Constitution Check

*GATE: Passed.*

- **I. Declarative Desired State**: Compliant (driven by `weaver.yaml`).
- **II. Native Tool First**: Compliant (wraps Terraform/Kubectl/Taskfile).
- **III. Composition and Reuse**: Compliant (Modules/Apps schema).
- **IV. Idempotency & Determinism**: Compliant (State tracking/Drift detection).
- **V. Update Safety**: Compliant (User file preservation).
- **VI. Secret Decoupling**: Compliant (Secret abstraction via providers).
- **Scope**: Compliant (MVP single binary).
- **AI Policy**: Compliant (AI Resolve option).

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

#### Source Code (repository root)

```text
src/
├── main.rs                   # Entry point (CLI)
├── core/                     # Business logic
│   ├── mod.rs
│   ├── config.rs             # weaver.yaml structs
│   ├── state.rs              # .rw/state.yaml structs
│   └── engine.rs             # Execution engine
├── ops/                      # Native operations
│   ├── mod.rs
│   ├── git.rs                # Git clone/checkout
│   └── fs.rs                 # File copy/ensure
└── plugin/                   # Plugin system
    ├── mod.rs
    └── wasm.rs               # Wasmtime integration

tests/
├── integration/              # assert_cmd tests
└── fixtures/                 # Test workspaces
```

**Structure Decision**: Modular Monolith with Cargo Workspace (likely `crates/` but simplified to modules for MVP if desired, though spec says "Cargo Workspace", so let's stick to spec).

*Correction based on Spec "Clarifications"*
```text
crates/
├── cli/                      # Binary entrypoint
│   └── src/main.rs
├── core/                     # Business logic (library)
│   └── src/lib.rs
└── ops/                      # Infrastructure adapters
    └── src/lib.rs

plugins/                      # Official Plugin Store (Source)
└── aws-ssm/                  # [MVP] AWS SSM Parameter Store provider
    ├── src/
    ├── wit/
    └── README.md
└── git/                      # [MVP] Git operations (clone, submodule, diff)
    └── README.md
└── terraform/                # [MVP] Terraform wrapper (plan, apply, output)
    └── README.md
└── taskfile/                 # [MVP] Taskfile.dev wrapper & generator
    └── README.md
└── kustomize/                # [MVP] Kustomize build/edit wrapper
    └── README.md
└── helm/                     # [MVP] Helm template/install wrapper
    └── README.md

examples/wasm/                # Multi-language Plugin Examples
├── rust-basic/
│   ├── src/
│   └── README.md             # TODO: Rust build support
├── python-basic/
│   ├── app.py
│   └── README.md             # TODO: Python build support
├── go-basic/
│   ├── main.go
│   └── README.md             # TODO: Go build support
└── js-basic/
    ├── index.js
    └── README.md             # TODO: JS build support

wit/                          # WIT Contracts (Host <-> Guest)
└── plugin.wit

```

**Decision**: Using Cargo Workspace as per Spec Clarification.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
