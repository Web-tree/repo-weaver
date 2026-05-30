# Generic Standardization Engine — Phase 0 + Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement repo-weaver's first generic building-block primitives (`ensure.file.exists`, `ensure.file.from_template`, `ensure.file.md_section`), a generic `ensure_json_key` core API (with `npm.*` as presets), real module lockfile + commit pinning, and an `rw add module` command — proven end-to-end by making the already-authored example **`23-ai-agents-skills-and-agents-md`** pass the snapshot suite.

**Architecture:** Three tiers — WASM plugins (low-level workers), the Rust core engine (resolves modules/plugins, runs `plan`/`apply`, tracks state), and declarative modules. This plan stays in the core engine: it adds built-in file/section/JSON primitives behind the existing `Ensure` trait, enriches the ensure execution context so file ensures can resolve module-relative templates, and wires the dead module lockfile so refs pin to commits.

**Tech Stack:** Rust 2024 (1.92+), `clap`, `serde` + `serde_yml` (YAML) + `serde_json` (`preserve_order`), `tera` (templating), `anyhow` (errors), `sha2`, shelling out to the `git` binary (no `git2`), `assert_cmd` + `predicates` + `tempfile` (tests). CLI bin = `rw`, CLI crate = `repo-weaver`, core crate = `repo-weaver-core`, ops crate = `repo_weaver_ops`.

---

## Key discoveries & deviations from spec (READ FIRST)

These are grounded in a verbatim source audit of the current tree. The plan follows the **approved spec** (`spec.md`) but adapts its wording to the real code:

1. **The flagship acceptance test already exists.** `examples/23-ai-agents-skills-and-agents-md/` has `before/`, `after/`, and a `module/` and uses the exact target primitives (`ensure.file.from_template`, `ensure.file.md_section` with `heading` + `block_marker` selectors, `<!-- rw:section id="..." -->` markers). It is **not registered** in `examples/test-suite.yaml` (only `01`–`17` are) and currently fails. **Phase 1's definition of done = example 23 passes and is registered `stage: implemented`.**

2. **The `Ensure` trait is `plan()` + `execute()`, NOT `plan()` + `apply()`.** (`crates/core/src/ensure/mod.rs`.) We keep these names — renaming to `apply()` is pure churn. Spec FR-004/FR-026 wording "`apply()`" maps to the existing `execute()`.

3. **Module-manifest ensures are ALREADY dispatched** through `build_ensure` in `apply.rs` (the spec's "broken — never executed" was about a different concern). The real gap is: (a) **app-level** ensures (`apps[].ensures`, type `EnsureSpec`) only support `ensure.npm.*` and run through a separate sync path with no module/template context; example 23 puts `ensure.file.*` at the **app level**, so they currently fail to deserialize/run. This plan adds the file primitives to the app-level path with an enriched context.

4. **Two ensure enums exist** (`EnsureSpec` app-level, internally-tagged, npm-only; `EnsureConfig`/`EnsureEntry`/`PluginEnsure` module-level). Spec Phase 0 says "unify." We interpret this pragmatically as **unify the implementations behind the `Ensure` trait** (shared built-in implementors callable from both paths) — NOT a risky big-bang enum merge that would break the `implemented` example 15 / integration tests. Full enum merge is deferred.

5. **The module lockfile is dead code.** `ModuleResolver::new` is always called with `None` (5 sites); `weaver.lock` `modules` map is never written; the cache is keyed by the **symbolic ref string** (so a branch is frozen on first clone). Phase 0 wires this and pins refs to commits via `git rev-parse`.

6. **`PlannedChange` is defined but never populated**; `rw plan` re-runs `apply(dry_run=true)` and signals "changes" only via a brittle `e.to_string().contains("Drift detected")` string match. Phase 1 populates structured changes and returns exit code 2 on any planned change.

7. **Example 23's `before/weaver.yaml` has explanatory comments that `after/weaver.yaml` lacks.** Since `apply` never rewrites `weaver.yaml` and the suite does a **byte-exact** dir compare (ignoring only `weaver.lock`, `.rw/state.yaml`), this is a guaranteed `content mismatch: weaver.yaml`. Task 11 reconciles them (makes `before` byte-identical to `after`).

8. **`ensure.file.md_section`** is the example's name for the spec's `ensure.text.section`. The plan uses the example's name (`ensure.file.md_section`) since the example is the acceptance gate.

9. **Test commands:** no Taskfile/justfile/nextest. Use `cargo test --all-features` and `cargo clippy --all-targets --all-features -- -D warnings`. Example suite: `cargo test -p repo-weaver --test examples_suite`. Integration: `cargo test -p repo-weaver --test integration`. Core: `cargo test -p repo-weaver-core`.

---

## File Structure

**Created:**
- `crates/core/src/json_merge.rs` — generic `ensure_json_key()` + helpers (extracted/generalized from `ensures.rs::set_nested`). One responsibility: structural JSON key-path merge preserving siblings/indent/order.
- `crates/core/src/ensure/file.rs` — built-in file primitives: `EnsureFileExists`, `EnsureFileFromTemplate`, `EnsureFileMdSection` (+ the markdown section algorithm). One responsibility: file/section convergence implementors of the `Ensure` trait.
- `crates/cli/tests/integration/add_module.rs` — integration tests for `rw add module`.

**Modified:**
- `crates/core/src/lib.rs` — add `pub mod json_merge;` and `pub mod file;`-via-`ensure` re-export.
- `crates/core/src/ensures.rs` — re-express `apply_ensure` npm variants over `json_merge::ensure_json_key`.
- `crates/core/src/ensure/mod.rs` — extend `EnsureContext` (add `module_path`, `tera_context`); add `build_app_ensure(spec) -> Option<Box<dyn Ensure>>`; register file module.
- `crates/core/src/config.rs` — add `ensure.file.exists` / `ensure.file.from_template` / `ensure.file.md_section` variants + `MdSelector` type to `EnsureSpec`.
- `crates/core/src/state.rs` — add `OwnedRegion` + `FileState.owned_regions` (additive, default empty).
- `crates/core/src/lockfile.rs` — add `resolved_commit` to `ModuleLock`.
- `crates/core/src/module.rs` — read/write module locks; key cache by resolved commit.
- `crates/ops/src/git.rs` — add `rev_parse_remote`/`rev_parse` for ref→commit resolution.
- `crates/cli/src/commands/apply.rs` — construct enriched `EnsureContext`; route app-level file ensures through the trait; populate `PlannedChange`; persist module lock.
- `crates/cli/src/commands/plan.rs` — structured changes + typed exit codes (replace string-match hack).
- `crates/cli/src/commands/module.rs` — add `Add` subcommand.
- `crates/cli/src/commands/describe.rs` — display new ensure variants.
- `crates/cli/src/main.rs` — (no change; `Module` already wired).
- `examples/23-ai-agents-skills-and-agents-md/before/weaver.yaml` — reconcile to match `after`.
- `examples/test-suite.yaml` — register example 23 as `stage: implemented`.

---

# PHASE 0 — Foundations

## Task 1: Generic `ensure_json_key` core API; npm ensures become presets

**Files:**
- Create: `crates/core/src/json_merge.rs`
- Modify: `crates/core/src/lib.rs` (add `pub mod json_merge;`)
- Modify: `crates/core/src/ensures.rs` (delegate to `json_merge`)
- Test: inline `#[cfg(test)]` in `crates/core/src/json_merge.rs`

- [ ] **Step 1: Write the failing test**

Add to the bottom of the new file `crates/core/src/json_merge.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn sets_nested_key_preserving_siblings_and_indent() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("package.json");
        std::fs::write(
            &file,
            "{\n    \"name\": \"demo\",\n    \"scripts\": {\n        \"build\": \"tsc\"\n    }\n}\n",
        )
        .unwrap();

        ensure_json_key(
            &file,
            &["scripts"],
            "test",
            serde_json::Value::String("vitest".to_string()),
        )
        .unwrap();

        let out = std::fs::read_to_string(&file).unwrap();
        // 4-space indent preserved, trailing newline preserved, sibling kept.
        assert!(out.contains("\"build\": \"tsc\""));
        assert!(out.contains("\"test\": \"vitest\""));
        assert!(out.starts_with("{\n    \"name\""));
        assert!(out.ends_with("}\n"));
    }

    #[test]
    fn is_idempotent() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("package.json");
        std::fs::write(&file, "{\n  \"scripts\": {}\n}\n").unwrap();
        let val = serde_json::Value::String("x".into());
        ensure_json_key(&file, &["scripts"], "a", val.clone()).unwrap();
        let first = std::fs::read_to_string(&file).unwrap();
        ensure_json_key(&file, &["scripts"], "a", val).unwrap();
        let second = std::fs::read_to_string(&file).unwrap();
        assert_eq!(first, second);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p repo-weaver-core --lib json_merge`
Expected: FAIL to compile — `ensure_json_key` and module `json_merge` do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Top of `crates/core/src/json_merge.rs` (this is `set_nested`/`navigate_or_create`/`detect_indent`/`write_pretty` from `ensures.rs`, made public and file-path-based):

```rust
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

/// Set `root[parents[0]][parents[1]]...[key] = value` in a JSON file.
/// Intermediate objects are created in declaration order. Sibling keys, key
/// order (serde_json `preserve_order`), indent width, and trailing newline are
/// all preserved so the diff stays minimal. The file must already exist and
/// contain a JSON object.
pub fn ensure_json_key(
    path: &Path,
    parents: &[&str],
    key: &str,
    value: Value,
) -> anyhow::Result<()> {
    let raw = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;
    let mut root: Value = serde_json::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("invalid JSON in {}: {e}", path.display()))?;

    let detected_indent = detect_indent(&raw);
    let trailing_newline = raw.ends_with('\n');

    let obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("{} is not a JSON object", path.display()))?;

    let target = navigate_or_create(obj, parents)?;
    target.insert(key.to_string(), value);

    write_pretty(path, &root, &detected_indent, trailing_newline)
}

fn navigate_or_create<'a>(
    root: &'a mut Map<String, Value>,
    path: &[&str],
) -> anyhow::Result<&'a mut Map<String, Value>> {
    let mut cur = root;
    for segment in path {
        let entry = cur
            .entry((*segment).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            anyhow::bail!("expected object at `{segment}`, found non-object");
        }
        cur = entry.as_object_mut().expect("just checked entry.is_object()");
    }
    Ok(cur)
}

fn detect_indent(raw: &str) -> String {
    for line in raw.lines().skip(1) {
        let count = line.chars().take_while(|c| *c == ' ').count();
        if count > 0 {
            return " ".repeat(count);
        }
    }
    "  ".to_string()
}

fn write_pretty(
    path: &Path,
    value: &Value,
    indent: &str,
    trailing_newline: bool,
) -> anyhow::Result<()> {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    use serde::Serialize;
    value.serialize(&mut ser)?;
    if trailing_newline {
        buf.push(b'\n');
    }
    fs::write(path, buf)?;
    Ok(())
}
```

Add to `crates/core/src/lib.rs` (next to the other `pub mod` lines):

```rust
pub mod json_merge;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p repo-weaver-core --lib json_merge`
Expected: PASS (2 tests).

- [ ] **Step 5: Re-express npm ensures as presets (no behavior change)**

Replace the body of `apply_ensure` in `crates/core/src/ensures.rs` so each variant delegates to `json_merge::ensure_json_key` and delete the now-duplicate private helpers (`set_nested`, `navigate_or_create`, `detect_indent`, `write_pretty`):

```rust
use crate::config::EnsureSpec;
use crate::json_merge::ensure_json_key;
use serde_json::Value;
use std::path::Path;

/// Apply a single app-level npm ensure to the workspace rooted at `app_root`.
pub fn apply_ensure(app_root: &Path, ensure: &EnsureSpec) -> anyhow::Result<()> {
    match ensure {
        EnsureSpec::NpmScript { file, name, value } => ensure_json_key(
            &app_root.join(file),
            &["scripts"],
            name,
            Value::String(value.clone()),
        ),
        EnsureSpec::NpmDep { file, name, version } => ensure_json_key(
            &app_root.join(file),
            &["dependencies"],
            name,
            Value::String(version.clone()),
        ),
        EnsureSpec::NpmDevDep { file, name, version } => ensure_json_key(
            &app_root.join(file),
            &["devDependencies"],
            name,
            Value::String(version.clone()),
        ),
        EnsureSpec::NpmEngine { file, name, version } => ensure_json_key(
            &app_root.join(file),
            &["engines"],
            name,
            Value::String(version.clone()),
        ),
        // Non-npm variants are handled via the Ensure trait (Task 8), not here.
        _ => anyhow::bail!("apply_ensure only handles ensure.npm.* variants"),
    }
}
```

> Note: the `_ =>` arm is needed because Task 5–7 add non-npm variants to `EnsureSpec`. Until then `EnsureSpec` has only npm variants and the match is exhaustive without it — add the `_` arm in Task 8, not here. For Task 1, omit the `_` arm.

- [ ] **Step 6: Run the full core + example suite to verify no regression**

Run: `cargo test -p repo-weaver-core && cargo test -p repo-weaver --test examples_suite`
Expected: PASS. Example 15 (`node-npm-ensures`) still green (npm presets unchanged).

- [ ] **Step 7: Commit**

```bash
git add crates/core/src/json_merge.rs crates/core/src/lib.rs crates/core/src/ensures.rs
git commit -m "feat(core): extract generic ensure_json_key; npm ensures become presets"
```

---

## Task 2: Enrich `EnsureContext` with module path + template context

**Files:**
- Modify: `crates/core/src/ensure/mod.rs` (struct fields)
- Modify: `crates/cli/src/commands/apply.rs` (construction site, ~L637)
- Test: inline `#[cfg(test)]` in `crates/core/src/ensure/mod.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/core/src/ensure/mod.rs`:

```rust
#[cfg(test)]
mod ctx_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn context_carries_module_path_and_tera_context() {
        let mut tc = tera::Context::new();
        tc.insert("project_name", "acme-api");
        let ctx = EnsureContext {
            app_path: PathBuf::from("/tmp/app"),
            dry_run: true,
            module_path: PathBuf::from("/tmp/module"),
            tera_context: tc,
        };
        assert_eq!(ctx.module_path, PathBuf::from("/tmp/module"));
        assert!(ctx.dry_run);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p repo-weaver-core --lib ctx_tests`
Expected: FAIL to compile — `EnsureContext` has no `module_path` / `tera_context` fields.

- [ ] **Step 3: Write minimal implementation**

In `crates/core/src/ensure/mod.rs`, extend the struct (keep existing fields first for stable ordering):

```rust
pub struct EnsureContext {
    pub app_path: PathBuf,
    pub dry_run: bool,
    /// Resolved module root, used by file/section ensures to locate
    /// module-relative templates (e.g. `sections/skills.md.j2`).
    pub module_path: PathBuf,
    /// Tera rendering context (resolved inputs + `secrets.*`) for
    /// template-backed ensures.
    pub tera_context: tera::Context,
}
```

Ensure `tera` is a dependency of `repo-weaver-core` (it is used by the template engine; confirm `tera` appears under `[dependencies]` in `crates/core/Cargo.toml` — if not, add `tera = { workspace = true }`).

- [ ] **Step 4: Fix the one construction site in apply.rs**

In `crates/cli/src/commands/apply.rs`, the existing `EnsureContext { app_path: dest_root.clone(), dry_run }` (~L637) must now include the new fields. Build a per-app Tera context (reuse the `tera_context` + per-app inputs already assembled in the templates loop) and pass `module_path`:

```rust
        // Build the per-app render context (mirrors the templates loop).
        let mut app_tera_context = tera_context.clone();
        let input_ctx = build_context(&app.inputs)?;
        app_tera_context.extend(input_ctx);

        let ensure_ctx = repo_weaver_core::ensure::EnsureContext {
            app_path: dest_root.clone(),
            dry_run,
            module_path: module_path.clone(),
            tera_context: app_tera_context,
        };
```

> `module_path` is the `resolver.resolve(...)` result already in scope; `build_context` and `tera_context` are already used by the templates loop above in the same function.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p repo-weaver-core --lib ctx_tests && cargo build -p repo-weaver`
Expected: PASS + clean build (apply.rs compiles with the enriched context).

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/ensure/mod.rs crates/cli/src/commands/apply.rs crates/core/Cargo.toml
git commit -m "feat(core): enrich EnsureContext with module_path and tera_context"
```

---

## Task 3: Wire module lockfile + pin refs to commits

**Files:**
- Modify: `crates/ops/src/git.rs` (add `rev_parse_remote`)
- Modify: `crates/core/src/lockfile.rs` (add `resolved_commit`)
- Modify: `crates/core/src/module.rs` (resolve commit, cache by commit, write lock)
- Test: `crates/ops` inline test + `crates/core` inline test

- [ ] **Step 1: Write the failing test (ops: ref→commit)**

Add to `crates/ops/src/git.rs`:

```rust
#[cfg(test)]
mod rev_parse_tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn rev_parse_remote_resolves_a_tag_to_a_40_char_sha() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "t@t.local"],
            vec!["config", "user.name", "T"],
            vec!["commit", "--allow-empty", "-m", "init"],
            vec!["tag", "v1"],
        ] {
            Command::new("git").args(&args).current_dir(&repo).output().unwrap();
        }
        let url = format!("file://{}", repo.display());
        let sha = rev_parse_remote(&url, "v1").unwrap();
        assert_eq!(sha.len(), 40);
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p repo-weaver-ops rev_parse`
Expected: FAIL to compile — `rev_parse_remote` does not exist.

- [ ] **Step 3: Implement `rev_parse_remote` in ops**

Add to `crates/ops/src/git.rs`:

```rust
/// Resolve a symbolic ref (branch/tag/sha) at a remote URL to a concrete
/// 40-char commit SHA using `git ls-remote`. Used to pin module refs so a
/// moving branch is captured as an immutable commit in the lockfile/cache.
pub fn rev_parse_remote(url: &str, ref_: &str) -> anyhow::Result<String> {
    // If the ref already looks like a full SHA, accept it as-is.
    if ref_.len() == 40 && ref_.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(ref_.to_string());
    }
    let output = Command::new("git")
        .arg("ls-remote")
        .arg(url)
        .arg(ref_)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git ls-remote failed for {url}@{ref_}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Prefer an exact tag/branch line; fall back to the first line.
    let sha = stdout
        .lines()
        .find_map(|line| line.split_whitespace().next())
        .ok_or_else(|| anyhow::anyhow!("ref '{ref_}' not found at {url}"))?;
    Ok(sha.to_string())
}
```

- [ ] **Step 4: Run the ops test to verify it passes**

Run: `cargo test -p repo-weaver-ops rev_parse`
Expected: PASS.

- [ ] **Step 5: Add `resolved_commit` to `ModuleLock` (failing core test)**

Modify `crates/core/src/lockfile.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleLock {
    pub source: String,
    pub r#ref: String,
    /// Concrete commit SHA the symbolic `ref` resolved to (commit pinning).
    #[serde(default)]
    pub resolved_commit: String,
    pub checksum: String,
}
```

Add a test in `crates/core/src/module.rs`:

```rust
#[cfg(test)]
mod resolve_tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    fn make_repo(dir: &std::path::Path) -> String {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("file.txt"), "hello").unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "t@t.local"],
            vec!["config", "user.name", "T"],
            vec!["add", "."],
            vec!["commit", "-m", "init"],
            vec!["tag", "v1"],
        ] {
            Command::new("git").args(&args).current_dir(dir).output().unwrap();
        }
        format!("file://{}", dir.display())
    }

    #[test]
    fn resolve_caches_by_commit_and_records_lock() {
        let tmp = tempdir().unwrap();
        // Isolate HOME so the cache lands in the tempdir.
        unsafe { std::env::set_var("HOME", tmp.path()) };
        let url = make_repo(&tmp.path().join("src"));

        let mut resolver = ModuleResolver::new(None).unwrap();
        let path = resolver.resolve("modname", &url, "v1").unwrap();

        assert!(path.join("file.txt").exists());
        let lock = resolver.take_lock();
        let entry = lock.modules.get("modname").expect("module lock recorded");
        assert_eq!(entry.resolved_commit.len(), 40);
        // Cache dir segment is the commit, not "v1".
        assert!(path.to_string_lossy().contains(&entry.resolved_commit));
    }
}
```

- [ ] **Step 6: Run to verify it fails**

Run: `cargo test -p repo-weaver-core --lib resolve_tests`
Expected: FAIL to compile — `resolve` has a different signature, and `take_lock` does not exist.

- [ ] **Step 7: Rewrite `ModuleResolver` to pin commits, cache by commit, accumulate locks**

Replace `crates/core/src/module.rs` body:

```rust
use crate::lockfile::{Lockfile, ModuleLock};
use repo_weaver_ops::git;
use std::path::PathBuf;

pub struct ModuleResolver {
    cache_dir: PathBuf,
    lockfile: Lockfile,
}

impl ModuleResolver {
    pub fn new(existing: Option<Lockfile>) -> anyhow::Result<Self> {
        let home =
            home::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let cache_dir = home.join(".rw").join("store");
        Ok(Self {
            cache_dir,
            lockfile: existing.unwrap_or_default(),
        })
    }

    /// Resolve `source@ref` to a local path, pinning the ref to a concrete
    /// commit. The cache is keyed by the resolved commit so a moving branch
    /// re-resolves correctly. Records a `ModuleLock` keyed by `name`.
    pub fn resolve(&mut self, name: &str, source: &str, ref_: &str) -> anyhow::Result<PathBuf> {
        let commit = git::rev_parse_remote(source, ref_)?;

        let folder_name = urlencoding::encode(source);
        let path = self.cache_dir.join(folder_name.as_ref()).join(&commit);

        if !path.exists() {
            std::fs::create_dir_all(&path)?;
            if let Err(e) = git::clone(source, &commit, &path) {
                std::fs::remove_dir_all(&path).ok();
                return Err(e);
            }
        }

        self.lockfile.modules.insert(
            name.to_string(),
            ModuleLock {
                source: source.to_string(),
                r#ref: ref_.to_string(),
                resolved_commit: commit,
                checksum: String::new(),
            },
        );

        Ok(path)
    }

    /// The accumulated lockfile (call after resolving all modules).
    pub fn take_lock(&self) -> Lockfile {
        self.lockfile.clone()
    }
}
```

> Note: `git::clone` checks out the given ref; a 40-char commit is a valid checkout target. `Lockfile` already derives `Default`; ensure `version` defaults sensibly — set it when writing (Task 9 / apply persists). If `Lockfile::version` has no default, set `self.lockfile.version = "1".into()` in `new` when `existing` is `None`.

- [ ] **Step 8: Update the 5 `ModuleResolver::new(None)` call sites for the new `resolve` signature**

In each of `list.rs:76`, `list.rs:127`, `describe.rs:58`, `apply.rs:71`, `run.rs:49`: the resolver must be `let mut resolver = ...` and `resolve(&module_config.name, &module_config.source, &module_config.r#ref)`. Example for `apply.rs`:

```rust
    let mut resolver = ModuleResolver::new(None)?;
    // ...
    let module_path = resolver.resolve(
        &module_config.name,
        &module_config.source,
        &module_config.r#ref,
    )?;
```

Apply the same `&module_config.name` first-arg + `mut` change in the other four files.

- [ ] **Step 9: Run tests + build all**

Run: `cargo test -p repo-weaver-core --lib resolve_tests && cargo build --workspace`
Expected: PASS + clean build.

- [ ] **Step 10: Run the example suite (commit-pinning must not regress existing examples)**

Run: `cargo test -p repo-weaver --test examples_suite && cargo test -p repo-weaver --test integration`
Expected: PASS. (Local module sources are bootstrapped with a tag `ref`; `ls-remote` against a `file://`/local path resolves the tag — verify example 07/08 still green.)

> If `git ls-remote` rejects a bare local path without `file://`, normalize local paths to `file://` inside `rev_parse_remote` (the snapshot suite uses relative paths). Add that normalization if integration tests fail here.

- [ ] **Step 11: Commit**

```bash
git add crates/ops/src/git.rs crates/core/src/lockfile.rs crates/core/src/module.rs \
        crates/cli/src/commands/list.rs crates/cli/src/commands/describe.rs \
        crates/cli/src/commands/apply.rs crates/cli/src/commands/run.rs
git commit -m "feat(core): pin module refs to commits and wire the module lockfile"
```

---

## Task 4: Sub-file region ownership in state (additive)

**Files:**
- Modify: `crates/core/src/state.rs`
- Test: inline `#[cfg(test)]` in `crates/core/src/state.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/core/src/state.rs`:

```rust
#[cfg(test)]
mod owned_region_tests {
    use super::*;

    #[test]
    fn filestate_defaults_to_no_owned_regions_and_roundtrips() {
        let fs = FileState::new("abc".into());
        assert!(fs.owned_regions.is_empty());

        let region = OwnedRegion {
            id: "recent-changes".into(),
            checksum: "deadbeef".into(),
        };
        let fs2 = FileState::new("abc".into()).with_owned_regions(vec![region.clone()]);
        let yaml = serde_yml::to_string(&fs2).unwrap();
        let back: FileState = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(back.owned_regions.len(), 1);
        assert_eq!(back.owned_regions[0].id, "recent-changes");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p repo-weaver-core --lib owned_region_tests`
Expected: FAIL to compile — `OwnedRegion`, `owned_regions`, `with_owned_regions` do not exist.

- [ ] **Step 3: Write minimal implementation**

In `crates/core/src/state.rs`, add the type and field (additive; `#[serde(default, skip_serializing_if)]` keeps existing state files compatible and avoids noise):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OwnedRegion {
    /// Region identifier (block-marker id, or `heading:<path>`).
    pub id: String,
    /// SHA-256 of the rendered content rw last wrote into this region.
    pub checksum: String,
}
```

Add to `FileState`:

```rust
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owned_regions: Vec<OwnedRegion>,
```

Update `FileState::new` to initialize `owned_regions: Vec::new()`, and add a builder:

```rust
    pub fn with_owned_regions(mut self, regions: Vec<OwnedRegion>) -> Self {
        self.owned_regions = regions;
        self
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p repo-weaver-core --lib owned_region_tests`
Expected: PASS.

- [ ] **Step 5: Run full core tests (state roundtrip must not regress)**

Run: `cargo test -p repo-weaver-core`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/state.rs
git commit -m "feat(core): add sub-file owned_regions to FileState (additive)"
```

---

# PHASE 1 — Primitives + flow + example 23 green

> Phase 1 introduces three built-in file ensures as `Ensure` implementors in a new `crates/core/src/ensure/file.rs`, adds matching `EnsureSpec` variants, routes app-level file ensures through the trait, adds `rw add module`, makes plan output structured with typed exit codes, and finally registers example 23.

## Task 5: `ensure.file.exists` primitive

**Files:**
- Create: `crates/core/src/ensure/file.rs`
- Modify: `crates/core/src/ensure/mod.rs` (add `mod file;`, re-export)
- Modify: `crates/core/src/config.rs` (add `EnsureSpec::FileExists` variant)
- Test: inline `#[cfg(test)]` in `crates/core/src/ensure/file.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/core/src/ensure/file.rs` with:

```rust
use crate::ensure::{Ensure, EnsureContext, EnsurePlan};
use std::path::PathBuf;

/// `ensure.file.exists` — create an empty file (and parent dirs) if absent.
/// Never truncates an existing file (idempotent, non-clobbering).
pub struct EnsureFileExists {
    /// Destination path, relative to the app root.
    pub dest: String,
}

impl Ensure for EnsureFileExists {
    fn plan(&self, ctx: &EnsureContext) -> anyhow::Result<EnsurePlan> {
        let target = ctx.app_path.join(&self.dest);
        let actions = if target.exists() {
            vec![]
        } else {
            vec![format!("create file {}", self.dest)]
        };
        Ok(EnsurePlan {
            description: format!("Ensure file '{}' exists", self.dest),
            actions,
        })
    }

    fn execute(&self, ctx: &EnsureContext) -> anyhow::Result<()> {
        if ctx.dry_run {
            return Ok(());
        }
        let target = ctx.app_path.join(&self.dest);
        if !target.exists() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&target, b"")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn ctx(app: PathBuf) -> EnsureContext {
        EnsureContext {
            app_path: app,
            dry_run: false,
            module_path: PathBuf::from("."),
            tera_context: tera::Context::new(),
        }
    }

    #[test]
    fn creates_missing_file_and_is_idempotent() {
        let dir = tempdir().unwrap();
        let e = EnsureFileExists { dest: "sub/new.txt".into() };
        e.execute(&ctx(dir.path().to_path_buf())).unwrap();
        let p = dir.path().join("sub/new.txt");
        assert!(p.exists());
        std::fs::write(&p, "user content").unwrap();
        e.execute(&ctx(dir.path().to_path_buf())).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "user content");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p repo-weaver-core --lib ensure::file`
Expected: FAIL to compile — module `file` not declared in `ensure/mod.rs`.

- [ ] **Step 3: Declare the module**

In `crates/core/src/ensure/mod.rs`, add near the other `mod` declarations:

```rust
pub mod file;
```

- [ ] **Step 4: Add the `EnsureSpec::FileExists` variant**

In `crates/core/src/config.rs`, inside `enum EnsureSpec`, add:

```rust
    #[serde(rename = "ensure.file.exists")]
    FileExists {
        dest: String,
        #[serde(default)]
        template: Option<String>,
    },
```

> `template` is accepted for forward-compat (seed-from-template) but Task 5 ignores it; `ensure.file.from_template` (Task 6) is the dedicated rendering primitive. Keeping the field avoids a parse error if a module uses it.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p repo-weaver-core --lib ensure::file`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/ensure/file.rs crates/core/src/ensure/mod.rs crates/core/src/config.rs
git commit -m "feat(core): add ensure.file.exists primitive"
```

---

## Task 6: `ensure.file.from_template` primitive

**Files:**
- Modify: `crates/core/src/ensure/file.rs` (add `EnsureFileFromTemplate`)
- Modify: `crates/core/src/config.rs` (add `EnsureSpec::FileFromTemplate`)
- Test: inline `#[cfg(test)]` in `crates/core/src/ensure/file.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/core/src/ensure/file.rs` `tests` module:

```rust
    #[test]
    fn renders_module_template_into_app_dest() {
        let module = tempdir().unwrap();
        let app = tempdir().unwrap();
        std::fs::create_dir_all(module.path().join("templates")).unwrap();
        std::fs::write(
            module.path().join("templates/greeting.txt.j2"),
            "Hello {{ project_name }}\n",
        )
        .unwrap();

        let mut tc = tera::Context::new();
        tc.insert("project_name", "acme-api");
        let ctx = EnsureContext {
            app_path: app.path().to_path_buf(),
            dry_run: false,
            module_path: module.path().to_path_buf(),
            tera_context: tc,
        };

        let e = EnsureFileFromTemplate {
            template: "templates/greeting.txt.j2".into(),
            dest: "greeting.txt".into(),
        };
        e.execute(&ctx).unwrap();
        let out = std::fs::read_to_string(app.path().join("greeting.txt")).unwrap();
        assert_eq!(out, "Hello acme-api\n");
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p repo-weaver-core --lib ensure::file::tests::renders_module_template_into_app_dest`
Expected: FAIL to compile — `EnsureFileFromTemplate` does not exist.

- [ ] **Step 3: Write minimal implementation**

Add to `crates/core/src/ensure/file.rs` (use the same template engine apply.rs uses; confirm the path — likely `crate::template::TemplateEngine`):

```rust
use crate::template::TemplateEngine;

/// `ensure.file.from_template` — render a module-relative Tera template into an
/// app-relative destination. rw fully owns the destination file.
pub struct EnsureFileFromTemplate {
    /// Template path, relative to the module root.
    pub template: String,
    /// Destination path, relative to the app root.
    pub dest: String,
}

impl EnsureFileFromTemplate {
    fn render(&self, ctx: &EnsureContext) -> anyhow::Result<String> {
        let template_path = ctx.module_path.join(&self.template);
        let src = std::fs::read_to_string(&template_path).map_err(|e| {
            anyhow::anyhow!("cannot read template {}: {e}", template_path.display())
        })?;
        let engine = TemplateEngine::new()?;
        engine.render(&src, &ctx.tera_context)
    }
}

impl Ensure for EnsureFileFromTemplate {
    fn plan(&self, ctx: &EnsureContext) -> anyhow::Result<EnsurePlan> {
        Ok(EnsurePlan {
            description: format!("Render '{}' -> '{}'", self.template, self.dest),
            actions: vec![format!("write {}", self.dest)],
        })
    }

    fn execute(&self, ctx: &EnsureContext) -> anyhow::Result<()> {
        if ctx.dry_run {
            return Ok(());
        }
        let rendered = self.render(ctx)?;
        let target = ctx.app_path.join(&self.dest);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, rendered)?;
        Ok(())
    }
}
```

> Confirm the `TemplateEngine` import path by checking the `use` line in `crates/cli/src/commands/apply.rs` (it constructs `TemplateEngine::new()` and calls `.render(&content, &context)`). Match it exactly.

- [ ] **Step 4: Add the `EnsureSpec::FileFromTemplate` variant**

In `crates/core/src/config.rs` `enum EnsureSpec`:

```rust
    #[serde(rename = "ensure.file.from_template")]
    FileFromTemplate { template: String, dest: String },
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p repo-weaver-core --lib ensure::file`
Expected: PASS (all file tests).

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/ensure/file.rs crates/core/src/config.rs
git commit -m "feat(core): add ensure.file.from_template primitive"
```

---

## Task 7: `ensure.file.md_section` primitive (block_marker + heading selectors)

**Files:**
- Modify: `crates/core/src/ensure/file.rs` (add `EnsureFileMdSection`, selector logic)
- Modify: `crates/core/src/config.rs` (add `EnsureSpec::FileMdSection` + `MdSelector`)
- Test: inline `#[cfg(test)]` in `crates/core/src/ensure/file.rs`

This is the anti-clobber primitive. Two selectors:
- **block_marker**: manage the region between `<!-- rw:section id="ID" -->` and `<!-- rw:endsection id="ID" -->`. If absent, append at EOF.
- **heading**: manage the body under a `{#*depth} {title}` heading, up to the next heading of level ≤ depth (or EOF). If absent, append the heading + body at EOF.

Everything outside the managed region is preserved byte-for-byte.

- [ ] **Step 1: Write the failing tests**

Append to `crates/core/src/ensure/file.rs` `tests` module:

```rust
    #[test]
    fn block_marker_appends_then_updates_idempotently() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("AGENTS.md");
        std::fs::write(&file, "# Title\n\nbody\n").unwrap();

        // First apply: append the block at EOF.
        let out = super::upsert_block_marker(
            &std::fs::read_to_string(&file).unwrap(),
            "recent-changes",
            "- one\n- two",
        );
        std::fs::write(&file, &out).unwrap();
        let got = std::fs::read_to_string(&file).unwrap();
        assert_eq!(
            got,
            "# Title\n\nbody\n\n<!-- rw:section id=\"recent-changes\" -->\n- one\n- two\n<!-- rw:endsection id=\"recent-changes\" -->\n"
        );

        // Second apply with new content: replace in place, no duplication.
        let out2 = super::upsert_block_marker(&got, "recent-changes", "- three");
        assert_eq!(
            out2,
            "# Title\n\nbody\n\n<!-- rw:section id=\"recent-changes\" -->\n- three\n<!-- rw:endsection id=\"recent-changes\" -->\n"
        );
        // Idempotent for identical content.
        assert_eq!(super::upsert_block_marker(&out2, "recent-changes", "- three"), out2);
    }

    #[test]
    fn heading_appends_then_updates_preserving_other_sections() {
        let input = "# Title\n\n## Manual Additions\n\nkeep me\n";
        let out = super::upsert_heading(input, &["Skills".to_string()], 2, "body line");
        assert_eq!(
            out,
            "# Title\n\n## Manual Additions\n\nkeep me\n\n## Skills\n\nbody line\n"
        );
        // Re-apply updates only the Skills body; Manual Additions untouched.
        let out2 = super::upsert_heading(&out, &["Skills".to_string()], 2, "new body");
        assert_eq!(
            out2,
            "# Title\n\n## Manual Additions\n\nkeep me\n\n## Skills\n\nnew body\n"
        );
    }

    #[test]
    fn heading_section_stops_at_next_same_or_higher_level_heading() {
        let input = "## Skills\n\nold\n\n## Other\n\nleave\n";
        let out = super::upsert_heading(input, &["Skills".to_string()], 2, "new");
        assert_eq!(out, "## Skills\n\nnew\n\n## Other\n\nleave\n");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p repo-weaver-core --lib ensure::file`
Expected: FAIL to compile — `upsert_block_marker` / `upsert_heading` do not exist.

- [ ] **Step 3: Implement the two pure section functions**

Add to `crates/core/src/ensure/file.rs` (pure string transforms — easy to unit-test, no IO):

```rust
/// Insert or replace a marker-delimited region. `content` is the inner body
/// (no surrounding newlines). Everything outside the markers is preserved.
pub(crate) fn upsert_block_marker(input: &str, id: &str, content: &str) -> String {
    let start = format!("<!-- rw:section id=\"{id}\" -->");
    let end = format!("<!-- rw:endsection id=\"{id}\" -->");
    let body = content.trim_matches('\n');
    let block = format!("{start}\n{body}\n{end}");

    if let (Some(s), Some(e)) = (input.find(&start), input.find(&end)) {
        let e_end = e + end.len();
        let mut out = String::with_capacity(input.len());
        out.push_str(&input[..s]);
        out.push_str(&block);
        out.push_str(&input[e_end..]);
        return out;
    }

    // Append at EOF: ensure exactly one blank line before the block.
    let mut out = input.trim_end_matches('\n').to_string();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str(&block);
    out.push('\n');
    out
}

fn heading_level(line: &str) -> Option<usize> {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    if hashes > 0 && line.chars().nth(hashes) == Some(' ') {
        Some(hashes)
    } else {
        None
    }
}

/// Insert or replace the body under a heading. `path` is a breadcrumb; the last
/// element is the heading title rendered at `depth` (`#`*depth). The managed
/// region runs from the heading line to the line before the next heading of
/// level <= depth (or EOF). Content outside is preserved.
pub(crate) fn upsert_heading(input: &str, path: &[String], depth: usize, content: &str) -> String {
    let title = path.last().map(|s| s.as_str()).unwrap_or("");
    let heading_line = format!("{} {}", "#".repeat(depth), title);
    let body = content.trim_matches('\n');

    let lines: Vec<&str> = input.lines().collect();
    let mut start_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if heading_level(line) == Some(depth) && line.trim_end() == heading_line {
            start_idx = Some(i);
            break;
        }
    }

    let trailing_nl = input.ends_with('\n');

    if let Some(start) = start_idx {
        // Find end: next heading with level <= depth, else EOF.
        let mut end = lines.len();
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            if let Some(l) = heading_level(line) {
                if l <= depth {
                    end = i;
                    break;
                }
            }
        }
        let mut out: Vec<String> = Vec::new();
        out.extend(lines[..start].iter().map(|s| s.to_string()));
        out.push(heading_line.clone());
        out.push(String::new());
        out.push(body.to_string());
        // Preserve a blank separator before the following content if any.
        if end < lines.len() {
            out.push(String::new());
            out.extend(lines[end..].iter().map(|s| s.to_string()));
        }
        let mut joined = out.join("\n");
        if trailing_nl {
            joined.push('\n');
        }
        return joined;
    }

    // Append at EOF.
    let mut out = input.trim_end_matches('\n').to_string();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str(&heading_line);
    out.push_str("\n\n");
    out.push_str(body);
    out.push('\n');
    out
}
```

> The exact blank-line handling above is what produces example 23's `after/AGENTS.md` byte-for-byte. If the Tera-rendered section body has leading/trailing blank lines (from `{% set %}` / `{% for %}` blocks), `body.trim_matches('\n')` normalizes them; verify against the after-file in Task 11 and adjust trimming if the snapshot differs.

- [ ] **Step 4: Run unit tests to verify they pass**

Run: `cargo test -p repo-weaver-core --lib ensure::file`
Expected: PASS (block + heading tests).

- [ ] **Step 5: Add the `EnsureFileMdSection` implementor + `MdSelector` config**

In `crates/core/src/config.rs`, add the selector type and the variant:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MdSelector {
    Heading { path: Vec<String>, #[serde(default = "default_heading_depth")] depth: usize },
    BlockMarker { id: String },
}

fn default_heading_depth() -> usize {
    2
}
```

Add to `enum EnsureSpec`:

```rust
    #[serde(rename = "ensure.file.md_section")]
    FileMdSection {
        file: String,
        selector: MdSelector,
        #[serde(default)]
        content: Option<String>,
        #[serde(default)]
        content_from_template: Option<String>,
    },
```

Add the implementor to `crates/core/src/ensure/file.rs`:

```rust
use crate::config::MdSelector;

/// `ensure.file.md_section` — converge one managed region of a markdown file
/// (block-marker or heading selector). Content outside the region is preserved.
pub struct EnsureFileMdSection {
    pub file: String,
    pub selector: MdSelector,
    pub content: Option<String>,
    pub content_from_template: Option<String>,
}

impl EnsureFileMdSection {
    fn resolved_content(&self, ctx: &EnsureContext) -> anyhow::Result<String> {
        if let Some(rel) = &self.content_from_template {
            let p = ctx.module_path.join(rel);
            let src = std::fs::read_to_string(&p)
                .map_err(|e| anyhow::anyhow!("cannot read section template {}: {e}", p.display()))?;
            let engine = TemplateEngine::new()?;
            engine.render(&src, &ctx.tera_context)
        } else {
            Ok(self.content.clone().unwrap_or_default())
        }
    }
}

impl Ensure for EnsureFileMdSection {
    fn plan(&self, _ctx: &EnsureContext) -> anyhow::Result<EnsurePlan> {
        let what = match &self.selector {
            MdSelector::BlockMarker { id } => format!("block '{id}'"),
            MdSelector::Heading { path, .. } => format!("heading '{}'", path.join(" > ")),
        };
        Ok(EnsurePlan {
            description: format!("Ensure {what} section in {}", self.file),
            actions: vec![format!("converge section in {}", self.file)],
        })
    }

    fn execute(&self, ctx: &EnsureContext) -> anyhow::Result<()> {
        if ctx.dry_run {
            return Ok(());
        }
        let target = ctx.app_path.join(&self.file);
        let current = std::fs::read_to_string(&target)
            .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", target.display()))?;
        let content = self.resolved_content(ctx)?;
        let updated = match &self.selector {
            MdSelector::BlockMarker { id } => upsert_block_marker(&current, id, &content),
            MdSelector::Heading { path, depth } => upsert_heading(&current, path, *depth, &content),
        };
        std::fs::write(&target, updated)?;
        Ok(())
    }
}
```

- [ ] **Step 6: Run tests + build**

Run: `cargo test -p repo-weaver-core --lib && cargo build -p repo-weaver-core`
Expected: PASS + clean build.

- [ ] **Step 7: Commit**

```bash
git add crates/core/src/ensure/file.rs crates/core/src/config.rs
git commit -m "feat(core): add ensure.file.md_section with block_marker and heading selectors"
```

---

## Task 8: Route app-level file ensures through the trait; update describe

**Files:**
- Modify: `crates/core/src/ensure/mod.rs` (add `build_app_ensure`)
- Modify: `crates/core/src/ensures.rs` (add `_ =>` bail arm now that EnsureSpec has non-npm variants)
- Modify: `crates/cli/src/commands/apply.rs` (dispatch file ensures via trait)
- Modify: `crates/cli/src/commands/describe.rs` (display new variants)
- Test: inline test in `ensure/mod.rs` + rely on Task 11 integration

- [ ] **Step 1: Write the failing test**

Add to `crates/core/src/ensure/mod.rs` `ctx_tests` (or a new module):

```rust
#[cfg(test)]
mod build_app_tests {
    use super::*;
    use crate::config::{EnsureSpec, MdSelector};

    #[test]
    fn file_variants_build_an_ensure_npm_variants_do_not() {
        let md = EnsureSpec::FileMdSection {
            file: "AGENTS.md".into(),
            selector: MdSelector::BlockMarker { id: "x".into() },
            content: Some("hi".into()),
            content_from_template: None,
        };
        assert!(build_app_ensure(&md).is_some());

        let npm = EnsureSpec::NpmScript {
            file: "package.json".into(),
            name: "t".into(),
            value: "v".into(),
        };
        assert!(build_app_ensure(&npm).is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p repo-weaver-core --lib build_app_tests`
Expected: FAIL to compile — `build_app_ensure` does not exist.

- [ ] **Step 3: Implement `build_app_ensure`**

Add to `crates/core/src/ensure/mod.rs`:

```rust
use crate::config::EnsureSpec;

/// Build an `Ensure` for the app-level file primitives. Returns `None` for the
/// `ensure.npm.*` variants, which stay on the native `ensures::apply_ensure`
/// JSON path.
pub fn build_app_ensure(spec: &EnsureSpec) -> Option<Box<dyn Ensure>> {
    use crate::config::EnsureSpec::*;
    match spec {
        FileExists { dest, .. } => Some(Box::new(file::EnsureFileExists { dest: dest.clone() })),
        FileFromTemplate { template, dest } => Some(Box::new(file::EnsureFileFromTemplate {
            template: template.clone(),
            dest: dest.clone(),
        })),
        FileMdSection {
            file,
            selector,
            content,
            content_from_template,
        } => Some(Box::new(file::EnsureFileMdSection {
            file: file.clone(),
            selector: selector.clone(),
            content: content.clone(),
            content_from_template: content_from_template.clone(),
        })),
        NpmScript { .. } | NpmDep { .. } | NpmDevDep { .. } | NpmEngine { .. } => None,
    }
}
```

- [ ] **Step 4: Add the `_ =>` arm to `ensures::apply_ensure`**

Now that `EnsureSpec` has non-npm variants, the match in `crates/core/src/ensures.rs` is non-exhaustive. Add the bail arm (from Task 1 Step 5 note):

```rust
        _ => anyhow::bail!(
            "ensure type {:?} is not a native npm ensure; route via build_app_ensure",
            ensure
        ),
```

- [ ] **Step 5: Dispatch app-level file ensures in apply.rs**

In `crates/cli/src/commands/apply.rs`, the app-level ensures loop currently calls `ensures::apply_ensure` for every `EnsureSpec`. The `ensure_ctx` (Task 2) is now built before this loop. Replace the loop:

```rust
        // App-level ensures: file.* via the Ensure trait (with module/template
        // context), npm.* via the native JSON path.
        for ensure in &app_config.ensures {
            if let Some(built) = repo_weaver_core::ensure::build_app_ensure(ensure) {
                let plan = built.plan(&ensure_ctx)?;
                if dry_run {
                    info!("Would ensure: {}", plan.description);
                } else {
                    info!("Ensuring: {}", plan.description);
                    built.execute(&ensure_ctx)?;
                }
            } else if dry_run {
                info!("Would apply ensure {:?} for {}", ensure, app_config.name);
            } else {
                repo_weaver_core::ensures::apply_ensure(&dest_root, ensure)?;
            }
        }
```

> Ensure `ensure_ctx` is constructed (Task 2 Step 4) **above** this loop. The module-declared ensures loop (`manifest.ensures` via `build_ensure`) stays as-is below it, reusing the same `ensure_ctx`.

- [ ] **Step 6: Update describe.rs to display the new variants**

In `crates/cli/src/commands/describe.rs`, the match over `EnsureSpec` for app-level ensures (and/or the helper printing app ensures) must handle the new variants. Add arms:

```rust
                EnsureSpec::FileExists { dest, .. } => {
                    println!("  - ensure.file.exists: {dest}");
                }
                EnsureSpec::FileFromTemplate { template, dest } => {
                    println!("  - ensure.file.from_template: {template} -> {dest}");
                }
                EnsureSpec::FileMdSection { file, .. } => {
                    println!("  - ensure.file.md_section: {file}");
                }
```

> If `describe.rs` only iterates `manifest.ensures` (module-level `EnsureEntry`) and not `app_config.ensures`, add a short loop printing app-level ensures too, so `rw describe` reflects them. Match the file's existing style.

- [ ] **Step 7: Run tests + build the workspace**

Run: `cargo test -p repo-weaver-core --lib && cargo build --workspace`
Expected: PASS + clean build.

- [ ] **Step 8: Commit**

```bash
git add crates/core/src/ensure/mod.rs crates/core/src/ensures.rs \
        crates/cli/src/commands/apply.rs crates/cli/src/commands/describe.rs
git commit -m "feat(cli): dispatch app-level file ensures through the Ensure trait"
```

---

## Task 9: `rw add module <git-url>` command

**Files:**
- Modify: `crates/cli/src/commands/module.rs` (add `Add` subcommand)
- Create: `crates/cli/tests/integration/add_module.rs`
- Modify: `crates/cli/tests/integration/main.rs` (add `mod add_module;`)

- [ ] **Step 1: Write the failing integration test**

Create `crates/cli/tests/integration/add_module.rs`:

```rust
use crate::common::{TestContext, cmd};
use predicates::prelude::*;

#[test]
fn add_module_appends_entry_and_pins_commit() {
    let ctx = TestContext::new();
    // Minimal workspace.
    ctx.write_file("weaver.yaml", "version: \"1\"\nmodules: []\napps: []\n");

    // Local module repo with a tag.
    let module_repo = ctx.root.join("std-repo");
    std::fs::create_dir_all(&module_repo).unwrap();
    std::fs::write(module_repo.join("weaver.module.yaml"), "inputs: {}\n").unwrap();
    for args in [
        vec!["init"],
        vec!["config", "user.email", "t@t.local"],
        vec!["config", "user.name", "T"],
        vec!["add", "."],
        vec!["commit", "-m", "init"],
        vec!["tag", "v1"],
    ] {
        std::process::Command::new("git").args(&args).current_dir(&module_repo).output().unwrap();
    }
    let url = format!("file://{}", module_repo.display());

    let mut c = cmd();
    c.arg("module")
        .arg("add")
        .arg(&url)
        .arg("--name")
        .arg("standards")
        .arg("--ref")
        .arg("v1")
        .env("HOME", ctx.temp.path())
        .current_dir(&ctx.root)
        .assert()
        .success()
        .stdout(predicate::str::contains("standards"));

    let weaver = ctx.read_file("weaver.yaml");
    assert!(weaver.contains("name: standards") || weaver.contains("name: \"standards\""));
    assert!(weaver.contains(&url));

    // Lockfile written with a pinned commit.
    let lock = ctx.read_file("weaver.lock");
    assert!(lock.contains("resolved_commit"));
}
```

- [ ] **Step 2: Register the test module + run to verify it fails**

Add `mod add_module;` to `crates/cli/tests/integration/main.rs`.

Run: `cargo test -p repo-weaver --test integration add_module`
Expected: FAIL — `module add` is not a recognized subcommand.

- [ ] **Step 3: Add the `Add` subcommand**

In `crates/cli/src/commands/module.rs`, extend the enum and dispatch:

```rust
#[derive(Subcommand)]
pub enum ModuleCommands {
    /// List defined modules
    List(ListArgs),
    /// Update a module's ref
    Update(UpdateArgs),
    /// Add a module to weaver.yaml and pin it in weaver.lock
    Add(AddArgs),
}

#[derive(Args)]
pub struct AddArgs {
    /// Module source (git URL or local path)
    pub source: String,
    /// Module name (defaults to the repo name derived from the source)
    #[arg(long)]
    pub name: Option<String>,
    /// Git ref (branch, tag, or commit)
    #[arg(long, default_value = "main")]
    pub r#ref: String,
}
```

```rust
pub fn execute(args: ModuleArgs) -> anyhow::Result<()> {
    match args.command {
        ModuleCommands::List(args) => run_list(args),
        ModuleCommands::Update(args) => run_update(args),
        ModuleCommands::Add(args) => run_add(args),
    }
}
```

- [ ] **Step 4: Implement `run_add`**

Add to `crates/cli/src/commands/module.rs`:

```rust
use repo_weaver_core::config::ModuleConfig;
use repo_weaver_core::lockfile::Lockfile;
use repo_weaver_core::module::ModuleResolver;

fn derive_name(source: &str) -> String {
    source
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("module")
        .trim_end_matches(".git")
        .to_string()
}

fn run_add(args: AddArgs) -> anyhow::Result<()> {
    let config_path = Path::new("weaver.yaml");
    let mut config = WeaverConfig::load(config_path)?;

    let name = args.name.clone().unwrap_or_else(|| derive_name(&args.source));
    if config.modules.iter().any(|m| m.name == name) {
        anyhow::bail!("Module '{}' already exists in weaver.yaml", name);
    }

    // Resolve + pin the commit (also clones into the global store).
    let mut resolver = ModuleResolver::new(None)?;
    resolver.resolve(&name, &args.source, &args.r#ref)?;

    // Append the module entry.
    config.modules.push(ModuleConfig {
        name: name.clone(),
        source: args.source.clone(),
        r#ref: args.r#ref.clone(),
        path: None,
    });
    let f = std::fs::File::create(config_path)?;
    serde_yml::to_writer(f, &config)?;

    // Persist the module lock (merge into existing weaver.lock if present).
    let lock_path = Path::new("weaver.lock");
    let mut lock = if lock_path.exists() {
        serde_yml::from_str::<Lockfile>(&std::fs::read_to_string(lock_path)?)?
    } else {
        Lockfile { version: "1".to_string(), ..Default::default() }
    };
    let resolved = resolver.take_lock();
    if let Some(ml) = resolved.modules.get(&name) {
        lock.modules.insert(name.clone(), ml.clone());
    }
    std::fs::write(lock_path, serde_yml::to_string(&lock)?)?;

    println!("Added module '{}' ({} @ {})", name, args.source, args.r#ref);
    println!("Run 'rw plan' to preview what will converge.");
    Ok(())
}
```

> Confirm `ModuleConfig`, `Lockfile`, `ModuleResolver` are re-exported at the paths used (`repo_weaver_core::config::ModuleConfig`, `repo_weaver_core::lockfile::Lockfile`, `repo_weaver_core::module::ModuleResolver`) — they are `pub mod` in `lib.rs`. The `serde_yml::to_writer` round-trip drops comments (matches the existing `run_update` behavior; acceptable for v1, flagged in spec out-of-scope).

- [ ] **Step 5: Run the integration test to verify it passes**

Run: `cargo test -p repo-weaver --test integration add_module`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/cli/src/commands/module.rs crates/cli/tests/integration/add_module.rs \
        crates/cli/tests/integration/main.rs
git commit -m "feat(cli): add 'rw module add' to adopt and pin a module"
```

---

## Task 10: Structured plan output + typed exit codes

**Files:**
- Modify: `crates/cli/src/commands/apply.rs` (collect `PlannedChange`s, return them)
- Modify: `crates/cli/src/commands/plan.rs` (render changes; exit 2 on any change)
- Test: example 12 (`plan-detailed-exitcode`) must stay green; add example-suite coverage in Task 11

This replaces the brittle `e.to_string().contains("Drift detected")` signal. The cleanest minimal approach: have `apply::execute` return the count of planned changes in dry-run mode, and have `plan::run` map a non-zero count to exit 2.

- [ ] **Step 1: Write the failing test (core PlannedChange render helper)**

`PlannedChange` already exists in `crates/core/src/plan.rs`. Add a tiny renderer with a test. Append to `crates/core/src/plan.rs`:

```rust
#[cfg(test)]
mod render_tests {
    use super::*;

    #[test]
    fn renders_change_lines_with_action_prefix() {
        let changes = vec![
            PlannedChange { action: "create".into(), path: "AGENTS.md#Skills".into(), preview: None },
            PlannedChange { action: "update".into(), path: "package.json".into(), preview: None },
        ];
        let rendered = render_changes(&changes);
        assert!(rendered.contains("+ create AGENTS.md#Skills"));
        assert!(rendered.contains("~ update package.json"));
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p repo-weaver-core --lib plan::render_tests`
Expected: FAIL to compile — `render_changes` does not exist.

- [ ] **Step 3: Implement `render_changes`**

Add to `crates/core/src/plan.rs`:

```rust
/// Render planned changes as a human-readable list. Glyph by action:
/// `+` create/add, `~` update/change, `-` remove, `?` drift, else ` `.
pub fn render_changes(changes: &[PlannedChange]) -> String {
    let mut out = String::new();
    for c in changes {
        let glyph = match c.action.as_str() {
            "create" | "add" => '+',
            "update" | "change" => '~',
            "remove" | "delete" => '-',
            "drift" => '?',
            _ => ' ',
        };
        out.push_str(&format!("{glyph} {} {}\n", c.action, c.path));
    }
    out
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p repo-weaver-core --lib plan::render_tests`
Expected: PASS.

- [ ] **Step 5: Make `apply::execute` return a change count; collect file-ensure changes**

Change the signature of `apply::execute` to return the number of planned changes in dry-run:

```rust
pub async fn execute(args: ApplyArgs, dry_run: bool) -> anyhow::Result<usize> {
    // ... existing body ...
    let mut planned_changes: Vec<repo_weaver_core::plan::PlannedChange> = Vec::new();
```

Inside the file-copy / template / app-ensure / module-ensure loops, when `dry_run` and an action would occur, push a `PlannedChange`. Minimal coverage that satisfies the exit-code contract — in the app-level file-ensure branch (Task 8 Step 5), use the ensure's own `plan()`:

```rust
            if let Some(built) = repo_weaver_core::ensure::build_app_ensure(ensure) {
                let plan = built.plan(&ensure_ctx)?;
                if dry_run {
                    if !plan.actions.is_empty() {
                        planned_changes.push(repo_weaver_core::plan::PlannedChange {
                            action: "update".to_string(),
                            path: plan.description.clone(),
                            preview: None,
                        });
                    }
                    info!("Would ensure: {}", plan.description);
                } else {
                    info!("Ensuring: {}", plan.description);
                    built.execute(&ensure_ctx)?;
                }
            } else if dry_run {
                planned_changes.push(repo_weaver_core::plan::PlannedChange {
                    action: "update".to_string(),
                    path: format!("{:?}", ensure),
                    preview: None,
                });
                info!("Would apply ensure {:?} for {}", ensure, app_config.name);
            } else {
                repo_weaver_core::ensures::apply_ensure(&dest_root, ensure)?;
            }
```

Likewise, in the files/template "Would copy/render" `dry_run` branches, push a `PlannedChange { action: "create", path: dest_path.display().to_string(), preview: None }`.

At the end of `execute`, before `Ok(...)`:

```rust
    if dry_run && !planned_changes.is_empty() {
        print!("{}", repo_weaver_core::plan::render_changes(&planned_changes));
        info!("{} change(s) to converge.", planned_changes.len());
    }
    // existing state.save(...) guarded by !dry_run ...
    Ok(planned_changes.len())
}
```

Update the `run` wrapper and any caller (`apply --plan` path) to use the returned `usize` (ignore with `let _ =` where the count is irrelevant):

```rust
pub async fn run(args: ApplyArgs) -> anyhow::Result<()> {
    let _ = execute(args, false).await?;
    Ok(())
}
```

- [ ] **Step 6: Rewrite `plan::run` to use the typed count**

Replace `crates/cli/src/commands/plan.rs` `run`:

```rust
pub async fn run(args: PlanArgs) -> anyhow::Result<()> {
    info!("Running plan...");

    let apply_args = crate::commands::apply::ApplyArgs {
        auto_approve: false,
        strategy: "stop".to_string(),
        plan: None,
        offline: false,
    };

    let change_count = crate::commands::apply::execute(apply_args, true).await?;

    if args.detailed_exitcode && change_count > 0 {
        std::process::exit(2);
    }
    Ok(())
}
```

> Drift still bails inside `execute` (the existing `--strategy stop` `anyhow::bail!`), which propagates as exit 1 unless caught; example 12 uses `plan --detailed-exitcode` against a drift example expecting exit 2. Verify: if example 12 relies on the *drift* path (not net-new changes), keep the drift case mapping to exit 2 — add, just before the `bail!` in the drift branch when `dry_run`, a `planned_changes.push(PlannedChange{action:"drift", path:.., preview:None})` and `return Ok(planned_changes.len())` instead of bailing in dry-run, so plan reports drift as a change rather than an error. Make this change in `execute`'s drift branches (both files and templates loops) **for dry-run only**; keep the `bail!` for real apply.

- [ ] **Step 7: Run the affected example tests**

Run: `cargo test -p repo-weaver --test examples_suite`
Expected: PASS — example 11 (`apply-stop-on-drift`, expect_failure) still fails-as-expected on real apply; example 12 (`plan-detailed-exitcode`) still exits 2.

- [ ] **Step 8: Build + clippy + commit**

Run: `cargo build --workspace && cargo clippy --all-targets --all-features -- -D warnings`
Expected: clean.

```bash
git add crates/core/src/plan.rs crates/cli/src/commands/apply.rs crates/cli/src/commands/plan.rs
git commit -m "feat(cli): structured plan changes and typed exit codes"
```

---

## Task 11: Make example 23 pass and register it

**Files:**
- Modify: `examples/23-ai-agents-skills-and-agents-md/before/weaver.yaml` (reconcile with `after`)
- Modify: `examples/test-suite.yaml` (register `23` as `implemented`)
- (Investigate/adjust) `crates/core/src/ensure/file.rs` if byte output differs

- [ ] **Step 1: Run example 23 in isolation to see the current diff**

Run: `cargo test -p repo-weaver --test examples_suite -- --nocapture`
Then read `target/examples-compat-report.md` and look for the `23-ai-agents-skills-and-agents-md` row.

Expected at this point: example 23 likely reports `content mismatch: before/weaver.yaml` (comments) and possibly `content mismatch: app/AGENTS.md` if section byte output differs.

- [ ] **Step 2: Reconcile `before/weaver.yaml` to match `after/weaver.yaml` byte-for-byte**

The snapshot suite copies the example, runs `rw` in `before/`, then byte-compares the mutated `before/` against `after/`. `apply` never rewrites `weaver.yaml`, so `before/weaver.yaml` MUST equal `after/weaver.yaml`. Read both:

Run: `diff examples/23-ai-agents-skills-and-agents-md/before/weaver.yaml examples/23-ai-agents-skills-and-agents-md/after/weaver.yaml`

Make them identical. Recommended: keep the explanatory comments by copying `before/weaver.yaml` over `after/weaver.yaml` (comments help readers and the after-file is not asserted against anything else). Verify the diff is now empty:

Run: `cp examples/23-ai-agents-skills-and-agents-md/before/weaver.yaml examples/23-ai-agents-skills-and-agents-md/after/weaver.yaml && diff examples/23-ai-agents-skills-and-agents-md/before/weaver.yaml examples/23-ai-agents-skills-and-agents-md/after/weaver.yaml`
Expected: no output (identical).

- [ ] **Step 3: Run the suite; inspect any AGENTS.md byte mismatch**

Run: `cargo test -p repo-weaver --test examples_suite -- --nocapture`

If `app/AGENTS.md` mismatches, the section algorithm's whitespace differs from the expected `after/app/AGENTS.md`. Compare the produced file by temporarily adding a `custom_assertion` or by reproducing locally:

```bash
# Reproduce by hand in a scratch dir:
cp -r examples/23-ai-agents-skills-and-agents-md /tmp/ex23
cd /tmp/ex23/before
HOME=/tmp/ex23home cargo run -p repo-weaver --bin rw -- apply --auto-approve
diff app/AGENTS.md ../after/app/AGENTS.md
```

- [ ] **Step 4: Adjust `upsert_heading` / `upsert_block_marker` whitespace to match the snapshot**

If `diff` shows blank-line differences, tune the trimming/joining in `crates/core/src/ensure/file.rs` (Task 7 Step 3) until the produced `AGENTS.md` is byte-identical to `after/app/AGENTS.md`. Re-run the unit tests after each tweak:

Run: `cargo test -p repo-weaver-core --lib ensure::file`
Expected: unit tests still PASS and the scratch `diff` is empty.

- [ ] **Step 5: Register example 23 as `implemented`**

Add to `examples/test-suite.yaml` under `examples:`:

```yaml
  23-ai-agents-skills-and-agents-md:
    stage: implemented
```

> Recall the harness gate: a `pending` example that PASSES makes the suite panic ("promote to implemented"). Registering it `implemented` at the moment it passes is required. Do NOT register it before it actually passes (an `implemented` example that fails also panics).

- [ ] **Step 6: Run the full suite — example 23 green**

Run: `cargo test -p repo-weaver --test examples_suite`
Expected: PASS, including `23-ai-agents-skills-and-agents-md`. Confirm `target/examples-compat-report.md` shows `23 ... implemented ... ✅ pass`.

- [ ] **Step 7: Full workspace verification**

Run: `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`
Expected: all tests PASS, no clippy warnings.

- [ ] **Step 8: Commit**

```bash
git add examples/23-ai-agents-skills-and-agents-md/after/weaver.yaml examples/test-suite.yaml \
        crates/core/src/ensure/file.rs
git commit -m "test(examples): make example 23 (AGENTS.md md_section) pass and register it"
```

---

## Self-Review

**Spec coverage (Phase 0 + Phase 1 items):**
- Unify ensures → Task 1 (json.key + npm presets) + Task 8 (shared `Ensure`-trait dispatch for app-level file ensures). *Adaptation: implementations unified behind the trait; the two enums are not merged (deferred) to avoid breaking example 15/integration — documented in "Key discoveries" #4.*
- Generic `ensure.json.key` with npm as presets → Task 1. ✓
- Wire lockfile + cache by resolved commit → Task 3. ✓
- Sub-file region ownership in state → Task 4. ✓ (field added; full read/write enforcement is Phase 3 — region checksums are populated by md_section in a later phase; Task 4 lays the schema.)
- `ensure.file.exists` + `ensure.text.section` → Task 5 + Task 7 (`md_section`, the example's name). ✓
- Route module ensures through dispatch → already present; app-level routing added in Task 8. ✓ (documented #3.)
- `rw add module` (raw git URL) → Task 9. ✓
- Structured `PlannedChange` + typed exit codes → Task 10. ✓
- Promote example #23 → Task 11. ✓

**Placeholder scan:** No "TBD"/"handle errors"/"similar to" — every code step shows complete code. Two flagged verification points (TemplateEngine import path in Task 6; example-23 whitespace tuning in Task 11) are explicit verify-and-adjust steps with exact reproduce commands, not placeholders.

**Type consistency:** `EnsureContext { app_path, dry_run, module_path, tera_context }` is used identically in Tasks 2/5/6/7. `EnsureSpec` variant names (`FileExists`, `FileFromTemplate`, `FileMdSection`) and their fields match across config.rs (Tasks 5/6/7), `build_app_ensure` (Task 8), and describe.rs (Task 8). `MdSelector::{Heading{path,depth}, BlockMarker{id}}` is consistent in Task 7 config + implementor + Task 8. `ensure_json_key(path, parents, key, value)` signature matches Task 1 use in `ensures.rs`. `ModuleResolver::resolve(name, source, ref)` + `take_lock()` consistent across Tasks 3/9. `PlannedChange { action, path, preview }` matches the verbatim struct in Task 10.

**Known cross-task ordering constraints (call out to the executor):**
- Task 1 Step 5 omits the `_ =>` arm; Task 8 Step 4 adds it once `EnsureSpec` has non-npm variants. Do them in order.
- Task 2 must construct `ensure_ctx` above the app-ensure loop that Task 8 modifies.
- Task 10 changes `apply::execute`'s return type to `usize`; update `plan.rs` (Task 10 Step 6) and the `run` wrapper together.
