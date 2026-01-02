# API Requirements Quality Checklist: Repo Weaver MVP

**Feature**: `001-repo-weaver-mvp`
**Created**: 2026-01-02
**Purpose**: Validation of CLI, WIT, and Schema contracts.

## 1. CLI Contract Completeness

- [x] CHK001 - Are all supported flags explicitly listed for `rw plan` (e.g., `--detailed-exitcode`, `--json`)? [Completeness, CLI Contract]
- [x] CHK002 - Is the `--no-color` flag defined for CI/CD environments? [Completeness, Gap]
- [x] CHK003 - Are output formats (Text vs JSON) defined for *every* command (init, plan, apply, run)? [Completeness, Spec FR-014]
- [x] CHK004 - Is the behavior of `rw init` defined when the target directory is not empty? [Edge Case, CLI Contract]

## 2. WIT Interface (Plugin Contract)

- [x] CHK005 - Is the specific WIT signature for `get-secret` defined? (Note: `docs/wasmtime.md` only shows generic `run`). [Clarity, Spec FR-013]
- [x] CHK006 - Are specific error variants defined for the Secret interface (e.g., `AccessDenied`, `NotFound`) vs generic strings? [Clarity, Type Safety]
- [x] CHK007 - Is the mechanism for passing AWS credentials to the WASM plugin defined in WIT or Host ABI? [Completeness, FR-012]
- [x] CHK008 - Are strict types used for secrets to prevent accidental logging (e.g., `secret<string>` vs `string`)? [Safety, FR-015]

## 3. Configuration Schema (`weaver.yaml`)

- [x] CHK009 - Are validation rules defined for `inputs` (e.g., required vs optional, default values)? [Completeness, Data Model]
- [x] CHK010 - Is the `ref` field in Modules pinned to specific Git semantics (tag vs branch vs hash)? [Clarity, FR-002]
- [x] CHK011 - Are "reserved names" defined for inputs to prevent collisions with system internal variables? [Edge Case, Gap]
- [x] CHK012 - Is the precedence order defined for `env` vars vs `weaver.yaml` inputs? [Consistency, Spec FR-011]

## 4. API Error Handling

- [x] CHK013 - Are exit codes explicitly mapped to error categories (e.g., 1=UserError, 2=Drift, 3=SystemError)? [Consistency, CLI Contract]
- [x] CHK014 - Is the format of standard error (stderr) output defined (e.g., structured JSON lines vs plain text)? [Clarity, Observability]
- [x] CHK015 - Is the behavior defined when a plugin returns a panic/trap (WASM trap)? [Recovery, Resilience]

## 5. Dependency & Network APIs

- [x] CHK016 - Is the timeout behavior defined for Git operations and Module downloads? [Resilience, Gap]
- [x] CHK017 - Are proxy configurations supported/defined for corporate environments? [Completeness, Gap]
