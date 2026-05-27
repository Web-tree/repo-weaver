# CLI Contract: rw describe

## Command

```
rw describe <app-name> [OPTIONS]
```

## Arguments

| Arg | Required | Description |
|-----|----------|-------------|
| `app-name` | Yes | Name of app to describe |

## Options

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |
| `--show-secrets` | Show secret values (requires confirmation) |

## Output Format (Default - YAML-like)

```
App: my-app
Path: ./apps/my-app
Module: k3s-nebula@v1.0.0

Inputs:
  cluster_name: "prod-cluster"
  region: "us-east-1"
  node_count: 3

Secrets:
  aws_access_key: ***
  aws_secret_key: ***

Tasks:
  install: Run terraform apply and kubectl setup
  plan: Preview terraform changes

Ensures:
  - folder.exists: ./apps/my-app
  - file.from_template: Taskfile.yml
  - git.submodule: vendor/k8s-manifests@v2.0.0

Checks:
  - terraform-validate: terraform validate
  - kubectl-lint: kubectl apply --dry-run=client
```

## Error Cases

| Condition | Message | Exit Code |
|-----------|---------|-----------|
| App not found | `Error: App 'foo' not found. Available apps: my-app, other-app` | 1 |
| Config error | `Error: Failed to load config: <details>` | 1 |
