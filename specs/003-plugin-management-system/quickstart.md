# Quickstart: Plugin Management System

## Zero-Config (Just Works)

Use built-in ensure types without any configuration:

```yaml
# weaver.module.yaml
ensures:
  - type: npm.script
    name: build
    command: npm run build
```

Run:
```bash
rw apply
```

The `npm-script` plugin is automatically downloaded from the default registry on first use.

---

## Explicit Plugin Configuration

Pin specific versions in `weaver.yaml`:

```yaml
# weaver.yaml
version: "1"
plugins:
  npm-script:
    git: github.com/web-tree/repo-weaver-plugins/npm-script
    ref: v1.0.0
```

---

## Local Plugin Development

Point to a local directory for rapid iteration:

```yaml
# weaver.yaml
version: "1"
plugins:
  my-plugin:
    path: ./plugins/my-plugin
```

The local WASM is loaded directly, bypassing resolution and caching.

---

## Verbose Mode

See plugin resolution details:

```bash
rw apply -v
```

Output:
```
[plugin] Resolving npm-script...
[plugin] Cache hit: ~/.rw/plugins/npm-script/v1.0.0
[plugin] Loaded in 45ms
```

---

## Plugin Commands

```bash
# List installed plugins
rw plugins list

# Update a specific plugin
rw plugins update npm-script

# Update all plugins
rw plugins update --all

# Verify checksums
rw plugins verify
```
