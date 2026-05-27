# CLI Contract: rw list

## Command

```
rw list [OPTIONS]
```

## Options

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON for scripting |
| `--apps-only` | Show only apps, no tasks |
| `--tasks-only` | Show only tasks, no apps |

## Output Format (Default)

```
APPS:
  <app-name>      <path>          (module: <module-name>@<ref>)
  ...

TASKS:
  <app-name>:<task-name>   <description>
  ...
```

## Output Format (JSON)

```json
{
  "apps": [
    {
      "name": "my-app",
      "path": "./apps/my-app",
      "module": "k3s-nebula",
      "ref": "v1.0.0"
    }
  ],
  "tasks": [
    {
      "app": "my-app",
      "name": "install",
      "description": "Run installation pipeline"
    }
  ]
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Config load error |
| 2 | No apps defined (with message) |
