# Research Findings: Repo Weaver MVP

**Date**: 2026-01-02
**Status**: Complete

## 1. Architecture: Modular Monolith

**Decision**: The project will be structured as a **Modular Monolith** using a Cargo Workspace.
**Rationale**: Clarification Session 2026-01-01.
- **Separation of Concerns**: `crates/core` (logic), `crates/cli` (interface), `crates/ops` (native adapters).
- **Extensibility**: Allows internal modules to be decoupled, paving the way for future splitting if needed, while keeping the MVP build simple (single binary).

**Alternatives Considered**:
- *Microservices*: Too complex for a CLI tool.
- *Single Crate*: Too coupled; hard to test core logic in isolation from CLI concerns.

## 2. Plugin System: WASM + Component Model

**Decision**: Plugins (providers) will be implemented as **WASM Components** using `wasmtime` and WIT interfaces.
**Rationale**: Clarification Session 2026-01-01.
- **Sandboxing**: Secure execution of third-party code.
- **Polyglot**: Writers can use Rust, Python, Go, etc.
- **Distribution**: Portable binaries.

**Implementation**:
- Host: Rust CLI embedding `wasmtime`.
- Interface: WIT files defining the contract (e.g., `secret-store`).

## 3. Secrets Management (MVP)

**Decision**: **AWS SSM Parameter Store** via WASM Component.
**Rationale**: FR-012 & Clarification.
- **Validation**: Proves the plugin architecture works for the most complex case (external network/auth).
- **Auth**: Credentials passed from Host to Guest (avoiding complex WASM-side auth chains).

## 4. State Management & Drift

**Decision**: **State Manifest (`.rw/state.yaml`)**.
**Rationale**: FR-010 & Clarification.
- **Mechanism**: specific manifest file listing managed files and their sha256 checksums.
- **Benefit**: Deterministic drift detection without modifying user files (no "DO NOT EDIT" headers needed).
- **Recovery**: "AI Resolve" to patch user edits back into templates.

## 5. Templating Engine

**Decision**: **Tera**.
**Rationale**: Clarification.
- **Standard**: Jinja2-compatible, widely known by DevOps engineers (Ansible/Salt).
- **Safety**: Logic-less(ish), strict separation of data and view.
