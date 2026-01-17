# Feature Specification: Plugin Management System

**Feature Branch**: `003-plugin-management-system`
**Created**: 2026-01-17
**Status**: Draft
**Input**: User description: "Design and implement a comprehensive plugin management system for Repo Weaver..."

## Clarifications

### Session 2026-01-17

- Q: What should happen when plugin resolution fails? → A: Display detailed error with remediation suggestions, then exit.
- Q: What security model should plugins operate under? → A: Sandboxed with explicit capability grants (WIT-defined).
- Q: How should plugin version updates work? → A: Lock to exact version; explicit `rw plugins update` command required.
- Q: What level of observability should the plugin system provide? → A: Verbose mode (`-v`) with structured logs for resolution, loading, timing.
- Q: What should happen when source build is needed but no container runtime is detected? → A: Error with instructions to install Docker or Podman.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Zero-Config Plugin Usage (Priority: P1)

As a developer, I want to use standard ensures (like `npm.script`) without manually configuring plugins, so that I can get started quickly.

**Why this priority**: Essential for UX; users shouldn't need to know about "plugins" to use basic features.

**Independent Test**: Use `npm.script` in a module manifest without a `plugins` section in `weaver.yaml` and verify it works.

**Acceptance Scenarios**:

1. **Given** a `weaver.module.yaml` with `type: npm.script`, **When** I run `rw apply`, **Then** the system automatically downloads the `npm-script` plugin and executes it.
2. **Given** a cached plugin, **When** I go offline and run `rw apply`, **Then** the system uses the cached plugin without network access.

---

### User Story 2 - Explicit Plugin Configuration (Priority: P1)

As a platform engineer, I want to explicitly define plugin sources and versions, so that I can ensure consistency across my team.

**Why this priority**: Required for deterministic builds and using custom/third-party plugins.

**Independent Test**: Define a plugin in `weaver.yaml` with a specific version/git ref and verify that exact version is used.

**Acceptance Scenarios**:

1. **Given** a `weaver.yaml` with a `plugins` section pointing to a specific git tag, **When** I run `rw apply`, **Then** the specific version is fetched and locked in `weaver.lock`.
2. **Given** a `weaver.lock` file, **When** I run `rw apply` on another machine, **Then** the exact same plugin version (checksum match) is used.

---

### User Story 3 - Local Plugin Development (Priority: P2)

As a plugin developer, I want to point `weaver.yaml` to a local directory, so that I can iterate on my plugin without pushing to git.

**Why this priority**: Critical for the plugin development workflow.

**Independent Test**: Point `weaver.yaml` to a local path and verify changes to the WASM binary are picked up immediately.

**Acceptance Scenarios**:

1. **Given** a config with `path: ./my-plugin`, **When** I run `rw apply`, **Then** the local WASM file is loaded directly.

---

### User Story 4 - Source Build Fallback (Priority: P3)

As a user on a unique architecture or security-constrained environment, I want to build plugins from source (using Docker), so that I don't rely on pre-built binaries.

**Why this priority**: Enhances security and compatibility but less common than using pre-builts.

**Independent Test**: Configure a plugin source with no release assets and verify `rw` initiates a containerized build.

**Acceptance Scenarios**:

1. **Given** a plugin source without release assets, **When** I run `rw apply`, **Then** `cargo component build` is executed inside a Docker container to generate the WASM.

---

## Requirements *(mandatory)*

### Functional Requirements

#### Configuration & Discovery
- **FR-001**: System MUST support a `plugins` section in `weaver.yaml` to explicit declare plugins (name, source, ref, path).
- **FR-002**: System MUST support "Auto-Discovery" for built-in ensure types (e.g., `npm.script`), resolving them to a default registry if not explicitly configured.
- **FR-003**: The default registry URL MUST be configurable/overridable, defaulting to `https://github.com/web-tree/repo-weaver-plugins`.

#### Resolution & Caching
- **FR-004**: System MUST store downloaded plugins in a global cache directory (`~/.rw/plugins/<name>/<version/ref>`).
- **FR-005**: System MUST link plugins from the global cache to the project workspace (`.rw/plugins`) using symbolic links to save space.
- **FR-006**: System MUST verify plugin integrity using SHA256 checksums stored in `weaver.lock`.
- **FR-006b**: Plugin versions MUST remain locked until explicitly updated via `rw plugins update`; `rw apply` MUST NOT auto-upgrade plugins.

#### Distribution & Building
- **FR-007**: System MUST attempt to fetch pre-built `.wasm` binaries from Git Provider Releases (e.g., GitHub Releases) matching the requested version.
- **FR-008**: If no pre-built binary is found (and not in local mode), System MUST attempt to build the plugin from source.
- **FR-009**: Source builds MUST be executed within a containerized environment (Docker/Podman) to ensure reproducibility and toolchain availability.

#### Edge Cases & Failure Handling
- **FR-010**: On plugin resolution failure (network error, invalid checksum, missing ref), System MUST display a detailed error message with remediation suggestions and exit with non-zero status.
- **FR-010b**: If source build is required but no container runtime (Docker/Podman) is detected, System MUST error with installation instructions rather than attempting a native build.

#### Security & Sandboxing
- **FR-011**: Plugins MUST execute within the WASM sandbox with no implicit access to filesystem or network.
- **FR-012**: Plugin capabilities (filesystem paths, network hosts) MUST be explicitly granted via the WIT interface and declared in plugin metadata.

#### Observability
- **FR-013**: System MUST support a verbose mode (`-v` flag) that outputs structured logs for plugin resolution, loading, capability grants, and timing metrics.

### Key Entities

- **Plugin**: A WASM component implementing the `ensure-provider` WIT interface.
- **Registry**: A collection of Git repositories or a single repository serving as an index for plugins.
- **Global Cache**: User-level storage for plugin binaries (`~/.rw/plugins`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can use standard ensures (`npm.script`) with ZERO additional configuration in `weaver.yaml`.
- **SC-002**: Plugin resolution adds less than 500ms overhead if the plugin is already cached globally.
- **SC-003**: A project with 5 plugins uses less than 1MB of local disk space (excluding global cache) via symlinking.
- **SC-004**: System successfully builds a valid WASM plugin from source using Docker if no binary is provided.
