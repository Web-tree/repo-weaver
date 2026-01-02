# Contract: weaver.yaml Configuration

**Version**: 1.0

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
