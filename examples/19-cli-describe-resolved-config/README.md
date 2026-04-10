# 19 - `rw describe` with Includes and Overrides

Demonstrates resolved-config inspection for a split workspace where app policy is
composed across `weaver.yaml` and `weaver.d/*.yaml`.

## What this covers

- `rw describe <app>` resolved config output after include merge (PRD §8)
- Include-driven composition and deterministic overrides (PRD §5.2)
- Non-mutating diagnostics for troubleshooting config inheritance

## How to run

```sh
cd before
rw describe platform
```

## Expected result

`rw describe platform` should show a merged app config where:
- global vars from `weaver.yaml` are present,
- app-level overrides from `weaver.d/policies.yaml` win.

Workspace files remain unchanged, so `before/` and `after/` are identical.
