# 30 - Init Workspace

Demonstrates `rw init` — bootstrapping a brand new workspace from an empty
directory.

## What this covers

- `rw init` command (PRD §8)
- Default `weaver.yaml` skeleton creation
- `.gitignore` creation with `.rw/` entry

## Before state

- An empty workspace (a `.gitkeep` placeholder keeps the dir tracked in git)
- No `weaver.yaml`, no `.gitignore`

## How to run

```sh
cd before
rw init
```

## Expected result

After `rw init`, `before/` should match `after/`:
- `weaver.yaml` created with default skeleton (`version: "1"`, empty `modules`, empty `apps`)
- `.gitignore` created with `.rw/` entry
- The pre-existing `.gitkeep` is preserved
