# Feature Specification: Repo Weaver MVP

**Feature Branch**: `001-repo-weaver-mvp`
**Created**: 2026-01-01
**Status**: Draft
**Input**: User description: "Implement Repo Weaver MVP based on PRD"

## User Scenarios & Testing

### User Story 1 - Bootstrap New Workspace (Priority: P1)

As a platform engineer, I want to generate a fully working `k3s-nebula` workspace from a wrapper module so that I can quickly start a new environment with standard conventions.

**Why this priority**: Core value proposition of the tool is bootstrapping complex configs quickly.

**Independent Test**: Can be tested by running `rw apply` in an empty folder with a `weaver.yaml` pointing to a module, and verifying all expected files (Taskfile, tfvars) are generated.

**Acceptance Scenarios**:

1. **Given** an empty folder and a `weaver.yaml` defining a `k3s-nebula` module dependency, **When** I run `rw apply`, **Then** the tool constructs the full folder structure, generates `Taskfile.yml`, and renders `terraform.tfvars`.
2. **Given** missing required variables in `weaver.yaml`, **When** I run `rw apply`, **Then** the tool prompts me for values and saves them to `.rw/answers.yaml`.

---

### User Story 2 - Update and Converge (Priority: P1)

As a maintainer, I want to update the pinned version of an upstream module and have my local specific configuration preserved while platform defaults are updated.

**Why this priority**: Solves the "drift" and "update hell" problem of boilerplate templating.

**Independent Test**: Can be tested by changing the `ref` in `weaver.yaml`, running `rw apply`, and verifying that managed files change while unmanaged files stay touched.

**Acceptance Scenarios**:

1. **Given** an existing workspace with a module pinned to v1, **When** I update the pin to v2 in `weaver.yaml` and run `rw apply`, **Then** the generated `Taskfile` and config files are updated to match v2 patterns.
2. **Given** I have locally modified an unmanaged file, **When** I run `rw apply` with a module update, **Then** my local changes are preserving.

---

### User Story 3 - Operational Pipeline Execution (Priority: P2)

As an operator, I want to run complex multi-step tasks (like "install") that chain together native tools (Terraform, kubectl) and pass data between them.

**Why this priority**: Needed to make the bootstrapped environment actually useful/deployable.

**Independent Test**: specific `rw run` command execution that parses output from one dummy step and uses it in another.

**Acceptance Scenarios**:

1. **Given** a defined `install` task that runs terraform plan, apply, and then kubectl apply, **When** I run `rw run my-app install`, **Then** all steps execute in order, and failure in one stops the pipeline (fail-fast).
2. **Given** a step that captures `terraform output -json`, **When** the next step runs, **Then** it has access to those outputs as template variables.

## Requirements

### Functional Requirements

- **FR-001**: System MUST support a `weaver.yaml` root config with `includes`, `modules`, and `apps` sections.
- **FR-002**: System MUST support loading modules from git repositories pinned to specific refs (tags/commits).
- **FR-003**: System MUST implement the `ensure.folder.exists` action to create directories.
- FR-004: System MUST implement the ensures.file.from_template action to render files using Tera (Jinja2-compatible).
- **FR-004a**: System MUST support a `--yes` or `--non-interactive` flag to bypass all confirmation prompts (e.g., for CI/CD).
  - **Behavior**: When set, "AI Resolve" prompts MUST default to "Fail" (security safety) unless explicitly configured otherwise via flag (e.g., `--strategy=the-ours`).
  - **Missing Input**: If required variables are missing and no interactive prompt is possible, the system MUST fail with a clear error code (exit 1).
- **FR-005**: System MUST implement `ensure.git.submodule` (or pinned clone) to vendor upstream dependencies.
  - **Strategy**: Internal Vendoring. Clone to hidden cache (`.rw/cache`), then render/copy to workspace. No git submodules in user repo.
- **FR-006**: System MUST implement `ensure.task.wrapper` to generate a `Taskfile.yml` including upstream targets.
- **FR-007**: System MUST provide a CLI with `rw plan` to preview changes and `rw apply` to execute them.
- **FR-008**: System MUST support persistent storage of user variables in `.rw/answers.yaml` to avoid repetitive prompting.
  - **Security**: The `.rw/` directory MUST be added to `.gitignore` by default. All answers are treated as local-only environment configuration.
- **FR-009**: System MUST support pipeline tasks where steps can capture stdout (JSON/Regex) and pass it to subsequent steps via environment variables or arguments.
- **FR-010**: System MUST only overwrite files explicitly marked as managed/generated, preserving user files.
  - **Tracking**: System MUST maintain a `.rw/state.yaml` manifest listing all generated files and checksums to detect external modifications and enable safe updates.
  - **Drift Resolution**: When modification is detected, the system MUST halt and prompt the user. Options MUST include: Overwrite, Skip, and **AI Resolve** (generate a patch to merge user changes).
- **FR-011**: System MUST support **Logical Secret Abstraction**.
  - **Behavior**: Templates reference secrets by logical name (e.g., `db_password`).
  - **Mapping**: Logical names are mapped to concrete values or providers (e.g., AWS SSM) in `weaver.yaml`.
  - **Overrides**: Users MUST be able to define default providers and specific overrides for individual secrets.
- **FR-012**: System MUST implement an **AWS SSM Parameter Store** provider for the MVP.
  - **Implementation**: **WASM Component**. The provider logic MUST be compiled to a WASM component that implements the secrets interface.
  - **Authentication**: **Host-Passed Credentials**. The host application must resolve AWS credentials (env vars, profile) and pass them explicitly to the plugin upon initialization.
  - **Scope**: Read-only access to fetch secrets during `rw apply`.
- **FR-013**: System MUST implement a standardized **Secrets Interface (WIT)** for all secret providers.
  - **Interface**: `get-secret: func(name: string) -> result<string, error>;`
  - **Contract**: Providers MUST implement this world to be loadable by the host.
- **FR-014**: System MUST implement structured logging using `tracing` and support user-configurable verbosity.
  - **Flags**: Support `--verbose` (debug logs), `--quiet` (errors only), and `--json` (machine-readable structured logs).


## Clarifications

### Session 2026-01-01

- Q: What architectural pattern should be used?
  - A: **Modular Monolith**. The system will use a Cargo Workspace with strict separation between the `Core` (engine, traits), `CLI` (user interface), and `Modules` (git loading, resolution). This ensures "modules implementation" is decoupled, enabling support for private/external modules via Git.
- Q: How should the codebase be structured?
  - A: **Cargo Workspace**. Organize as `crates/core` (business logic), `crates/cli` (binary entrypoint), and `crates/ops` (native operations adapters). This physical separation enforces the modular monolith pattern.
- Q: How will the plugin system be implemented?
  - A: **Wasmtime + Component Model**. The host will be Rust, embedding Wasmtime. Plugins will be WASM components (via WIT interfaces), enabling secure, polyglot extensibility (Rust, Python, Go, JS). See `docs/wasmtime.md`.
- Q: How are plugins distributed and stored?
  - A: **Global Content-Addressable Cache**. Plugins are distributed within Modules (Git) but stored centrally in the user's home directory (e.g., `~/.rw/store`), deduplicated by hash (pnpm-style). This saves disk space and enables efficient versioning across projects.
- Q: How are plugins activated and dependencies managed?
  - A: **Explicit Activation & Go-Style Resolution**. Plugins must be explicitly listed in `weaver.yaml`. Dependencies are resolved via Git (finding the root definition) and exact versions are locked in a `weaver.lock` file (content hashes), similar to Go modules.

- Q: How should the system track managed vs. unmanaged files?
  - A: **State Manifest**. The system will maintain a hidden manifest (e.g., `.rw/state.yaml`) to track generated files and their checksums, enabling drift detection and safe updates without brittle magic headers.
- Q: What is the dependency management strategy?
  - A: **Internal Vendoring**. Modules are cloned to a hidden cache (e.g., `.rw/cache`) and rendered into the workspace. This avoids rigid git submodules and allows "weaving" content from multiple sources.
- Q: Which template engine should be used?
  - A: **Tera**. We will use the Tera crate (Jinja2-compatible) for standard, powerful, and safe logic-less templating in Rust.
- Q: How are user variables/secrets handled?
  - A: **GitIgnore by Default**. All user answers are stored in `.rw/answers.yaml`, and the tool ensures `.rw/` is listed in `.gitignore` to prevent accidental secret leakage.
- Q: How should drift (modification of managed files) be handled?
  - A: **Prompt with AI Option**. The system detects drift via checksums and prompts the user. The prompt includes an "AI Resolve" option to intelligently merge changes, which can also be pre-configured as a default strategy.

  - **Prompt with AI Option**. The system detects drift via checksums and prompts the user. The prompt includes an "AI Resolve" option to intelligently merge changes, which can also be pre-configured as a default strategy.
- Q: How should secrets be defined and resolved?
  - A: **Logical Abstraction with Provider Mapping**. Secrets are referenced by logical names (e.g., `db_password`) in modules/templates. These are mapped to concrete providers (e.g., `aws-ssm`) in the root `weaver.yaml`, supporting hierarchy (Global Default -> Module Default -> Explicit Override). This decouples consumption from storage.
- Q: How will the AWS SSM provider be implemented for MVP?
  - A: **WASM Component**. To validate the plugin architecture early, the AWS SSM logic will be built as a WASM component, even for the MVP.
- Q: How will the AWS SSM plugin authenticate?
  - A: **Host-Passed Credentials**. To avoid complex WASM networking/auth logic, the host (Rust CLI) will resolve standard AWS credentials and pass them explicitly to the plugin via the WIT interface.

### Session 2026-01-02

- Q: What ensures high coverage for CLI commands?
  - A: **AssertCmd + Tempfile**. Use `assert_cmd` for reliable black-box CLI testing and `tempfile` to create isolated, disposable environments for every test case.
- Q: How should Core Logic IO be tested?
  - A: **Real IO with Ephemeral Resources**. Avoid excessive mocking. Use real filesystem and git operations against temporary directories to ensure realistic behavior and catch integration issues early.
- Q: How are WASM plugins tested?
  - A: **Host Integration Tests**. Compile WASM components and load them into a test host instance to verify the actual WASM runtime execution and WIT interface boundary.
- Q: What about non-interactive usage?
  - A: **CI/CD Friendly**. System MUST support `--yes` or `--non-interactive`. In this mode, prompts are disabled. Missing answers = Error. Drift detection = Error (unless auto-fix strategy is pre-configured).
- Q: What are the logging standards?
  - A: **Tracing + JSON**. Use `tracing` crate. Support `--json` flag to emit NDJSON logs for machine parsing (e.g., by CI systems or IDEs).

### Key Entities

- **Workspace**: The root context defined by `weaver.yaml`.
- **App**: A configuration unit bound to a filesystem path.
- **Module**: A reusable package of templates and configuration blocks, versioned via Git.
- **Ensure**: A declarative assertion of state (file exists, folder exists).
- **Task**: An imperative pipeline of commands.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A user can bootstrap a new `k3s-nebula` compatible workspace from zero to full config files in under 30 seconds (excluding git clone time).
- **SC-002**: The tool detects 100% of drift in managed files when running `rw plan` against a modified state.
- **SC-003**: 100% of required variables not present in config triggers a user prompt.
- **SC-004**: Pipeline steps successfully capture and verify JSON output from a mock command in integration tests.
