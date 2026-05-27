# Quickstart: 002-Gap Analysis

## Feature Overview

This feature implements remaining MVP requirements from the PRD gap analysis.

## New Commands

```bash
# List apps and tasks
rw list

# Inspect resolved app config
rw describe <app-name>

# Run validation checks
rw check [app-name]

# Module management
rw module list
rw module update <name> --ref <new-ref>
```

## New Ensure Types

```yaml
ensures:
  # Git submodule (history tracking)
  - type: git.submodule
    url: https://github.com/org/module
    ref: v1.0.0
    path: vendor/module

  # Git clone pinned (simpler)
  - type: git.clone_pinned
    url: https://github.com/org/deps
    ref: main
    path: deps/external

  # npm script management
  - type: npm.script
    name: build
    command: tsc && vite build

  # AI patch (P3)
  - type: ai.patch
    prompt: "Add error handling to all API calls"
    verify: cargo test
```

## Config Includes

```yaml
# weaver.yaml
version: "1"
includes:
  - "weaver.d/*.yaml"    # Load all fragments
  - "apps/*/weaver.yaml" # Per-app configs

modules:
  - name: base
    source: https://github.com/org/base
    ref: v2.0.0
```

## Validation Checks

```yaml
apps:
  - name: my-app
    module: base
    path: apps/my-app
    checks:
      - name: terraform-validate
        command: terraform validate
      - name: lint
        command: cargo clippy
```
