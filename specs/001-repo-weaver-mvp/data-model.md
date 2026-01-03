# Data Model: Repo Weaver MVP

## Entities

### 1. Workspace (Root)
The top-level context defined by `weaver.yaml`.

- **config_version**: Version of the Weaver schema (e.g., "1.0").
- **modules**: List of [Module] dependencies.
- **apps**: List of [App] instances.
- **secrets**: Map of [Secret] definitions.

### 2. Module
A reusable package sourced from Git.

- **name**: Logical name (e.g., "k3s-nebula").
- **source**: Git URL.
- **ref**: Git tag, branch, or commit hash.
- **path**: (Optional) Subdirectory within the repo.

#### Module Content Structure

A module directory MUST contain a manifest and templates:
- **weaver.module.yaml** (Manifest):
  - `inputs`: Variable definitions (name, type, default, required).
  - `outputs`: Values to export after execution.
  - `tasks`: Command definitions (e.g., `install`, `plan`).
- **templates/**: Jinja2 templates (e.g., `main.tf.j2`, `cronjob.yaml.j2`).
- **files/**: Static assets (scripts, pipelines) to be copied as-is.

### 3. App

An instance of a Module applied to a specific path.


- **name**: Instance name (e.g., "prod-cluster").
- **module**: Reference to a [Module].
- **path**: Target filesystem path (relative to workspace root).
- **inputs**: Map of key-value pairs (variables) to pass to templates.

### 4. Secret
A logical secret definition mapped to a concrete provider.

- **name**: Logical name used in templates (e.g., `db_password`).
- **provider**: Provider type (e.g., `aws-ssm`, `env`).
- **key**: Provider-specific key (e.g., SSM path).

### 5. State (Manifest)
Tracks the state of managed files to detect drift.

- **files**: Map of file paths to [FileState].

#### FileState
- **path**: Relative path.
- **checksum**: SHA-256 hash of the content at generation time.
- **last_modified**: Timestamp of generation.
