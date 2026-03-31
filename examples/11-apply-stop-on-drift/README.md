# 11 - Apply Stop on Drift

Demonstrates the default `rw apply` drift behavior (`--strategy stop`): detect drift,
fail fast, and preserve local edits.

## What this covers

- `rw apply` default conflict strategy (`stop`) (CLI contract: `rw apply`)
- Drift safety when managed files are manually edited
- No filesystem mutations when apply aborts on drift

## Before state

- `app/deployment.yaml` was manually changed from `replicas: 2` to `replicas: 5`
- `.rw/state.yaml` still tracks the original generated checksum

## How to run

```sh
cd before
rw apply
```

## Expected result

`rw apply` exits with a drift error, and `before/` remains unchanged (matching `after/`):
- `app/deployment.yaml` keeps the user edit (`replicas: 5`)
- `.rw/state.yaml` is not rewritten
