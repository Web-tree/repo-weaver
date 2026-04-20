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

## Conditional ensures: `when:`

Any ensure entry may carry a `when:` expression. If `when` evaluates to a falsy value, the ensure is skipped during both `plan` and `apply` (and the state entry for that ensure is cleared, so a previously-managed file falls back to "unmanaged" rather than showing drift).

```yaml
ensures:
  - type: "ensure.file.from_template"
    template: "CLAUDE.md.j2"
    dest: "CLAUDE.md"
    when: '"claude" in inputs.agents'
```

`when` uses the same Tera expression grammar as templates. It has access to `inputs.*`, `module.*`, and the usual computed values. `when` is generic and applies to every ensure type — it is **not** specific to AI agents, file types, or any particular selector. Use it whenever an entry should only fire for some subset of apps or inputs.

## Ensure: `ensure.file.md_section`

Manages a single named region inside a Markdown file. Non-managed content in the same file is preserved byte-for-byte and drift-detected independently, so multiple modules and the user's own edits can co-exist in one file (e.g. `AGENTS.md`).

### Schema

```yaml
- type: "ensure.file.md_section"
  file: "AGENTS.md"                    # path relative to the app path
  selector:                            # discriminated-union; see "Selector types"
    type: "heading"
    path: ["Skills"]
  content: "string literal content"    # either `content` OR `content_from_template` is required
  content_from_template: "sections/skills.md.j2"
  create_file_if_missing: true         # default true
```

### Selector types (v1)

The selector is extensible. Core ships two types in v1; additional types may be registered by plugins.

**`type: "heading"`** — CommonMark heading path.

```yaml
selector:
  type: "heading"
  path: ["Active Technologies"]        # hierarchical; last element = target heading
  depth: 2                             # optional; constrains to #/##/### level
  match: "normalized"                  # exact | normalized (default) | regex
  body_ends_at: "next_sibling_or_higher"  # default; alt: "next_any_heading" | "eof"
  preserve_children: false             # if true, nested headings under target are left untouched
```

Body spans from the line after the heading to the line before the next heading at the same or higher level (or per `body_ends_at`).

**`type: "block_marker"`** — HTML-comment delimited region. Survives structural rearrangement; matches the `<!-- MANUAL ADDITIONS START/END -->` convention already used by the Specify templates.

```yaml
selector:
  type: "block_marker"
  id: "recent-changes"
  begin: "<!-- rw:section id=\"{id}\" -->"     # default template
  end:   "<!-- rw:endsection id=\"{id}\" -->"  # default template
```

If the marker pair is missing, it is appended (with a surrounding blank line). If only one half exists, `apply` fails with an error — intent is ambiguous.

### Reserved selector types (not yet shipped)

Names are reserved to prevent plugin collisions:

- `mdast` — CSS-selector grammar over the CommonMark AST, per `unist-util-select`.
- `mdq` — jq-style queries, per `mdq`.
- `regex` — begin/end regex delimiter pair.
- `frontmatter` — YAML/TOML frontmatter key path.
- `line_range` — hard coordinate fallback.

### Behavior

- **Drift detection**: the `(file, selector-fingerprint)` key is stored in `.rw/state.yaml` with the rendered content hash. If the on-disk region differs, `rw plan` reports drift on that region only; `rw apply` stops per the usual drift-safety rules.
- **Idempotency**: identical inputs → no-op.
- **Creation**: file created if missing (when `create_file_if_missing: true`); region appended when the selector matches nothing.
- **Ordering**: multiple `ensure.file.md_section` entries on the same file are applied in declaration order; "first-seen wins on creation, preserved on update".
- **Plugin extension**: selector type dispatch happens on `selector.type`. New types may be registered via the plugin interface (see `contracts/wit.md`).
