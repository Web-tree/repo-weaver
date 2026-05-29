# Specification Quality Checklist: Build & Distribution

**Purpose**: Validate completeness and quality of Build & Distribution requirements (Focus: C)
**Created**: 2026-01-17
**Feature**: [specs/003-plugin-management-system/spec.md](../spec.md)
**Context**: Validating requirements for Container Builds, Release Fetching, and Source Fallbacks.
**Status**: ✅ Addressed (2026-01-17)

## Release Fetching Requirements

- [x] CHK001 - Is the naming convention/pattern for pre-built WASM assets in release artifacts explicitly defined? → `<plugin-name>.wasm` (kebab-case)
- [x] CHK002 - Are requirements defined for verifying the integrity/checksum of fetched release assets? → SHA256 verified against weaver.lock
- [x] CHK003 - Is the fallback behavior specified when multiple assets match or no assets match? → Error if 0 or >1 `.wasm` files
- [x] CHK004 - Are timeout and retry requirements defined for release downloads? → 30s timeout, 3 retries with exponential backoff

## Containerized Build Requirements

- [x] CHK005 - Is the specific build environment (base image, cargo-component version) defined or configurable? → Published to `ghcr.io/web-tree/rw-plugin-builder`
- [x] CHK006 - Are requirements defined for utilizing host cargo caches within the container to speed up builds? → Mount `~/.cargo/registry` if available (optimization)
- [x] CHK007 - Is the container execution context (user ID, rootless mode, env vars) explicitly specified? → Skipped for v1 (run as current user)
- [x] CHK008 - Are requirements defined for cleaning up container resources (images, volumes) after build? → Out of scope for v1
- [x] CHK009 - Is the expected output artifact location and format from the build container explicitly defined? → `target/wasm32-wasip2/release/<name>.wasm`

## Dependency & Runtime Management

- [x] CHK010 - Are the specific "instruction steps" for installing Docker/Podman defined or linked? → Already in spec FR-010b
- [x] CHK011 - Is the detection logic for container runtimes (precedence between Docker and Podman) specified? → Docker first, then Podman
- [x] CHK012 - Are requirements defined for handling network access during the containerized build (e.g. fetching crates)? → Allowed (required for crates.io)

## Error Handling & Edge Cases

- [x] CHK013 - Are requirements specified for handling build timeouts or hung containers? → 10 minute timeout with clear error
- [x] CHK014 - Is the behavior defined when source build succeeds but the resulting WASM is invalid/incompatible? → Validate with wasmtime, error if invalid
- [x] CHK015 - Are failure modes defined for insufficient disk space or memory during build? → Show clear error to user

## Measurability

- [x] CHK016 - Can "reproducibility" of the source build be objectively verified? → Postponed for v1
- [x] CHK017 - Is the success criterion for "valid WASM plugin" defined with specific validation checks? → Load with wasmtime + WIT interface check
