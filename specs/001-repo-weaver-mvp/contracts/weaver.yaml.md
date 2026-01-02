# Contract: weaver.yaml Configuration

**Version**: 1.0

## Input Resolution & Precedence (CHK012)

The system resolves input values for modules in the following order (highest to lowest):

1.  **CLI Overrides**: Flags passed at runtime (e.g., `--set key=value`).
2.  **Environment Variables**: Variables matching `RW_<APP>_<INPUT>`.
3.  **App Configuration**: `inputs` stanza in `weaver.yaml`.
4.  **Module Defaults**: `default` values defined in `weaver.module.yaml`.

## Reserved Names (CHK011)

To prevent collision with system internals, the following input names are **RESERVED** and validation will fail if used:

- `rw_*` (Any prefix starting with rw_)
- `module_path`
- `workspace_root`
- `output`

## Input Validation (CHK009)

- **Required**: If an input is marked `required: true` in the module and no value is resolved, the operation fails.
- **Types**:
  - `string`: standard text.
  - `bool`: true/false (native yaml or "true"/"false" string).
  - `number`: integers or floats.
  - `list(string)`: arrays of strings.

## Schema

```yaml
# Version of the manifest format
version: "1.0"

# Module Dependencies (Upstream)
modules:
  - name: "k3s-nebula"
    source: "https://github.com/webtree/modules.git"
    ref: "v1.0.0"
    path: "modules/k3s-nebula" # Optional subdir

# Application Instances
apps:
  - name: "prod-cluster"
    module: "k3s-nebula"
    path: "./clusters/prod"
    inputs:
      node_count: 3
      region: "us-east-1"
      # Logical secret reference
      db_password: "${secrets.db_password}"

# Secret Definitions (Logical to Concrete)
secrets:
  # Concrete value (for dev/testing)
  api_key:
    provider: "env"
    key: "API_KEY"
  
  # AWS SSM Provider (WASM)
  db_password:
    provider: "aws-ssm"
    key: "/prod/db/password"
```

## Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | string | Yes | Schema version. |
| `modules` | list | Yes | List of upstream module sources. |
| `apps` | list | Yes | List of app instances to generate. |
| `secrets` | map | No | Mapping of logical secret names to providers. |
