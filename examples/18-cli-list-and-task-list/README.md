# 18 - CLI Discovery (`rw list` and `rw task list`)

Demonstrates workspace/app discovery commands that do not mutate files.

## What this covers

- `rw list` app/module discovery (PRD §8)
- `rw task list [app]` task discovery from app-local task definitions (PRD §8)
- Non-mutating CLI workflows for CI/operator introspection

## How to run

```sh
cd before
rw list
rw task list web
rw task list infra
```

## Expected result

`rw list` and `rw task list` should print the apps/tasks declared in `weaver.yaml`.
Workspace files remain unchanged, so `before/` and `after/` are identical.
