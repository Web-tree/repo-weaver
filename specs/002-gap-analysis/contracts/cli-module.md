# CLI Contract: rw module

## Subcommands

### rw module list

```
rw module list [OPTIONS]
```

#### Options

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

#### Output Format

```
MODULES:
  <name>    <source>                              <ref>
  k3s       https://github.com/org/k3s-nebula     v1.2.0
  base      https://github.com/org/base-module   main
```

---

### rw module update

```
rw module update <name> --ref <new-ref> [OPTIONS]
```

#### Arguments

| Arg | Required | Description |
|-----|----------|-------------|
| `name` | Yes | Module name to update |

#### Options

| Flag | Required | Description |
|------|----------|-------------|
| `--ref <ref>` | Yes | New git ref (tag, branch, commit) |
| `--no-fetch` | No | Skip fetching module cache |

#### Behavior

1. Locate module in `weaver.yaml`
2. Update `ref` field to new value
3. Clear module from cache (unless `--no-fetch`)
4. Print confirmation

#### Output

```
Updated module 'k3s' from v1.2.0 to v2.0.0
Module cache cleared. Run 'rw apply' to fetch new version.
```

---

## Error Cases

| Condition | Message | Exit Code |
|-----------|---------|-----------|
| Module not found | `Error: Module 'foo' not found. Available: k3s, base` | 1 |
| Invalid ref | `Error: Invalid ref format: <ref>` | 1 |
| No modules defined | `No modules defined in weaver.yaml` | 0 |
