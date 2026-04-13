# 22 - Module Ref Bump (`rw module update`)

Demonstrates the CLI workflow for bumping a pinned module reference in
workspace config.

## What this covers

- `rw module list` to inspect current module refs (PRD §8)
- `rw module update <name> --ref <newRef>` to update a pinned ref (PRD §8)
- Update intent captured in config before running `rw apply`

## How to run

```sh
cd before
rw module list
rw module update platform-standards --ref v2
```

## Expected result

After `rw module update`, `before/weaver.yaml` should match `after/weaver.yaml`
with the module ref changed from `v1` to `v2`.
