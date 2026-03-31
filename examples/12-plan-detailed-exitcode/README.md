# 12 - Plan Detailed Exit Code

Demonstrates `rw plan --detailed-exitcode` when drift is detected in a managed file.

## What this covers

- `rw plan --detailed-exitcode` behavior (exit code `2` on detected changes)
- Drift reporting without mutating workspace files
- CI-oriented plan checks

## Before state

- `app/settings.yaml` was manually edited (`retries: 7`)
- Module source still defines `retries: 3`
- `.rw/state.yaml` tracks the original generated checksum

## How to run

```sh
cd before
rw plan --detailed-exitcode
```

## Expected result

`rw plan` reports drift and exits with code `2`. Workspace contents are unchanged,
so `before/` should still match `after/`.
