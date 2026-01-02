# Contract: CLI Interface

**Command**: `rw`

## Environment Variables (CHK016, CHK017)

- `RW_TIMEOUT`: Global operation timeout in seconds (Default: `300`).
- `RW_CACHE_DIR`: Directory for global module cache (Default: `~/.rw/store`).
- `RW_LOG`: Log level (trace, debug, info, warn, error).
- **Network Proxies**: System honors standard proxy variables:
  - `HTTP_PROXY` / `http_proxy`
  - `HTTPS_PROXY` / `https_proxy`
  - `NO_PROXY` / `no_proxy`

## Global Flags

- `--no-color`: Disable colored output (useful for CI/CD).
- `--verbose`: Enable debug logging.
- `--quiet`: Suppress all non-error output.
- `--json`: Output logs/result in JSON format.

## Commands

### `rw init`
Initialize a new workspace.

- **Usage**: `rw init [path]`
- **Effect**: Creates `weaver.yaml` and `.gitignore`.
- **Behavior**:
  - If `weaver.yaml` exists: Fails (Exit 1).
  - If directory is not empty but no config: Warnings logged, proceeds to create config.

### `rw plan` (CHK002)
Preview changes to managed files.

- **Usage**: `rw plan [flags]`
- **Flags**:
  - `--detailed-exitcode`: Return exit code 2 if changes are detected.
  - `--out <file>`: Save the plan to a file (binary or JSON).
- **Visual Output**:
  - **Add (`+`)**: Green text.
  - **Change (`~`)**: Yellow text.
  - **Remove (`-`)**: Red text.
  - **Unmanaged/Drift (`?`)**: Purple text (if drift detected).
- **Drift Detection**:
  - Reports if managed files have been manually modified.
  - Ignores whitespace unless meaningful (e.g., in YAML values).

### `rw apply`
Apply configuration to the filesystem.

- **Usage**: `rw apply [--auto-approve] [--strategy=overwrite|stop|ai]`
- **Effect**:
  1. Resolves modules (clones to cache).
  2. Resolves secrets.
  3. Renders templates to target paths.
  4. Updates `.rw/state.yaml`.

## Interactive Flows

### Drift Resolution (CHK001, CHK006, CHK012)

When `rw apply` detects drift in a managed file:

1.  **Prompt**:
    ```text
    Drift detected in 'k8s/deployment.yaml':
    [?] Local file has been modified since last generation.
    
    Select action:
    [o] Overwrite (Discard local changes)
    [s] Skip (Keep local changes, do not update)
    [d] Show Diff
    [a] AI Resolve (Attempt to merge changes)
    
    Action [o/s/d/a]: 
    ```
2.  **AI Resolve Flow**:
    - **Step 1**: User selects `[a]`.
    - **Step 2**: CLI shows spinner `Thinking...`.
    - **Step 3**: System generates a patch.
    - **Step 4**: Review Prompt:
        ```text
        Proposed Merge:
        <<<<<<< GENERATED
        replicas: 3
        =======
        replicas: 5  # user change preserved
        >>>>>>> MERGED
        
        Apply this merge? [Y/n]
        ```
    - **Failure/Timeout**: If AI service fails or times out (Network Error), the system **FALLS BACK** to the main menu with an error message: `[ERROR] AI Resolve unavailable: Connection timed out.`

### Standard Prompts (CHK011)

- **Confirmation**: `Do you want to proceed? [y/N]` (Default: No)
- **Input Request**: `Enter value for 'region':` (No default unless specified in module)

## Progress & Feedback (CHK022, CHK023)

- **Spinners**: All operations taking >500ms (git clone, module download, AI resolve) MUST show an indeterminate spinner: `⠋ Downloading module 'k3s'...`
- **Child Processes**: Output from `rw run` steps (e.g. terraform) is streamed directly.
  - **Prefix**: `[terraform]` (Gray) to distinguish from Weaver internal logs.
- **Success/Failure (CHK003)**:
  - **Success**: `[OK] Applied successfully in 2.3s.` (Green)
  - **Failure**: `[FAIL] Pipeline step 'init' failed.` (Red)

### `rw run`
Execute a task defined in a module.

- **Usage**: `rw run <app-name> <task-name> [args...]`
- **Effect**: chained execution of commands (e.g. terraform apply).
- **Output**: Streams underlying command stdout/stderr. If `--json` is passed, wraps output in JSON steps.

## Exit Codes (CHK013)

- `0`: Success.
- `1`: User Error (Invalid resolution, missing inputs, config parsing failed).
- `2`: Drift Detected (only with `--detailed-exitcode`).
- `3`: System Error (IO failure, Permission denied, Network timeout).
- `4`: Plugin Error (WASM trap, Interface violation).

## Error Output & Behavior

### Standard Error (stderr) (CHK014, CHK004)

All log messages and errors are written to `stderr`. `stdout` is reserved for machine-readable output or direct command results (like `rw plan` diffs).

**Format:**
- **Text (Default)**: Human-readable, colored (unless `--no-color`).
  `[ERROR] Failed to load module 'k3s': network timeout`
- **JSON (`--json`)**: NDJSON format for parsing.
  `{"level":"error","msg":"Failed to load module","module":"k3s","error":"network timeout","ts":"..."}`

**Malformed Config**:
- If `weaver.yaml` parsing fails, the error message MUST include the **Line Number** and **Column** of the error, along with a snippet of the invalid YAML.

### Rollback & Recovery (CHK015)

- **Atomic File Writes**: File generation uses the "write to temp, then rename" pattern to prevent partial corruption of individual files.
- **Partial Apply**: If an `apply` operation fails halfway (e.g., 3 of 5 apps updated), the system does **NOT** auto-rollback. The user must fix the error and re-run `rw apply`. State is updated only for successfully written files.

### Plugin Panic Handling (CHK015)

If a WASM plugin traps (panics):
1.  **Isolation**: The host runtime catches the trap.
2.  **Reporting**: The error is logged with details "Plugin Panic: [Component Name] - [Trap Code]".
3.  **Result**: The CLI terminates with Exit Code `4` (Fail Fast). It does **NOT** attempt to recover or retry partial state.

## Common Workflows (CHK018, CHK019, CHK020, CHK021)

### 1. Update Module Version
**Goal**: Upgrade a dependency (e.g., `k3s` v1 -> v2).

1.  **Action**: User manually edits `ref: v1` to `ref: v2` in `weaver.yaml`.
2.  **Command**: User runs `rw plan`.
3.  **Output**:
    ```text
    [+] Module 'k3s-nebula' will be updated (v1 -> v2).
    
    Files to change:
    ~ k8s/deployment.yaml (Update image tag)
    + k8s/new-service.yaml
    ```
4.  **Apply**: User runs `rw apply`.

### 2. Run Operational Pipeline
**Goal**: Deploy app using generated scripts.

1.  **Command**: `rw run my-app deploy`
2.  **Behavior**:
    - Weaver executes `terraform apply` (defined in module).
    - **Visibility**: stdout/stderr are streamed live.
    - **Failure**: If terraform fails (exit 1):
        ```text
        [terraform] Error: Invalid Region.
        [FAIL] Pipeline step 'deploy' failed (Exit 1). Aborting.
        ```

