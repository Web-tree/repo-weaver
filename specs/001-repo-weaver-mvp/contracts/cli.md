# Contract: CLI Interface

**Command**: `rw`

## Commands

### `rw init`
Initialize a new workspace.

- **Usage**: `rw init [path]`
- **Effect**: Creates `weaver.yaml` and `.gitignore`.

### `rw plan`
Preview changes to managed files.

- **Usage**: `rw plan`
- **Output**: Diff of changes that `apply` will perform.
- **Drift Detection**: Reports if managed files have been manually modified.

### `rw apply`
Apply configuration to the filesystem.

- **Usage**: `rw apply [--auto-approve]`
- **Effect**:
  1. Resolves modules (clones to cache).
  2. Resolves secrets.
  3. Renders templates to target paths.
  4. Updates `.rw/state.yaml`.

### `rw run`
Execute a task defined in a module.

- **Usage**: `rw run <app-name> <task-name> [args...]`
- **Effect**: chained execution of commands (e.g. terraform apply).

## Exit Codes

- `0`: Success.
- `1`: Generic Error.
- `2`: Drift Detected (during plan if --detailed-exitcode).
