use crate::config::EnsureSpec;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

/// Apply a single ensure to the workspace rooted at `app_root`.
///
/// All `ensure.npm.*` variants converge `package.json` by parsing it as JSON,
/// inserting or replacing the relevant key, and re-serializing with the same
/// pretty formatting. JSON key order is preserved via `serde_json`'s
/// `preserve_order` feature so the file diff stays minimal.
pub fn apply_ensure(app_root: &Path, ensure: &EnsureSpec) -> anyhow::Result<()> {
    match ensure {
        EnsureSpec::NpmScript { file, name, value } => {
            set_nested(app_root, file, &["scripts"], name, Value::String(value.clone()))
        }
        EnsureSpec::NpmDep { file, name, version } => set_nested(
            app_root,
            file,
            &["dependencies"],
            name,
            Value::String(version.clone()),
        ),
        EnsureSpec::NpmDevDep {
            file,
            name,
            version,
        } => set_nested(
            app_root,
            file,
            &["devDependencies"],
            name,
            Value::String(version.clone()),
        ),
        EnsureSpec::NpmEngine {
            file,
            name,
            version,
        } => set_nested(
            app_root,
            file,
            &["engines"],
            name,
            Value::String(version.clone()),
        ),
    }
}

/// Set `root[parent[0]][parent[1]]...[key] = value` in a JSON file rooted at
/// `app_root.join(file)`. Intermediate objects are created in declaration
/// order. The file must exist and contain a JSON object.
fn set_nested(
    app_root: &Path,
    file: &str,
    parents: &[&str],
    key: &str,
    value: Value,
) -> anyhow::Result<()> {
    let path = app_root.join(file);
    let raw = fs::read_to_string(&path)
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

    write_pretty(&path, &root, &detected_indent, trailing_newline)
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
        cur = entry
            .as_object_mut()
            .expect("just checked entry.is_object()");
    }
    Ok(cur)
}

/// Detect the indent used in a pretty-printed JSON file by looking at the
/// first indented line. Falls back to two spaces. Mirrors what `npm pkg set`
/// does so reformatting diffs stay quiet.
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_pkg(dir: &Path, body: &str) {
        fs::write(dir.join("package.json"), body).unwrap();
    }

    fn read_pkg(dir: &Path) -> String {
        fs::read_to_string(dir.join("package.json")).unwrap()
    }

    #[test]
    fn ensure_npm_script_appends_to_existing_scripts() {
        let dir = TempDir::new().unwrap();
        write_pkg(
            dir.path(),
            "{\n  \"name\": \"x\",\n  \"scripts\": {\n    \"start\": \"node .\"\n  }\n}\n",
        );

        apply_ensure(
            dir.path(),
            &EnsureSpec::NpmScript {
                file: "package.json".into(),
                name: "build".into(),
                value: "tsc".into(),
            },
        )
        .unwrap();

        let after = read_pkg(dir.path());
        assert!(after.contains("\"start\": \"node .\""));
        assert!(after.contains("\"build\": \"tsc\""));
        assert!(after.ends_with("\n"));
        // start must come before build (preserve_order).
        let start_idx = after.find("\"start\"").unwrap();
        let build_idx = after.find("\"build\"").unwrap();
        assert!(start_idx < build_idx);
    }

    #[test]
    fn ensure_npm_script_replaces_existing_key() {
        let dir = TempDir::new().unwrap();
        write_pkg(
            dir.path(),
            "{\n  \"scripts\": {\n    \"build\": \"old\"\n  }\n}\n",
        );

        apply_ensure(
            dir.path(),
            &EnsureSpec::NpmScript {
                file: "package.json".into(),
                name: "build".into(),
                value: "tsc -p .".into(),
            },
        )
        .unwrap();

        let after = read_pkg(dir.path());
        assert!(after.contains("\"build\": \"tsc -p .\""));
        assert!(!after.contains("\"old\""));
    }

    #[test]
    fn ensure_npm_engine_creates_engines_object() {
        let dir = TempDir::new().unwrap();
        write_pkg(dir.path(), "{\n  \"name\": \"x\"\n}\n");

        apply_ensure(
            dir.path(),
            &EnsureSpec::NpmEngine {
                file: "package.json".into(),
                name: "node".into(),
                version: ">=20 <21".into(),
            },
        )
        .unwrap();

        let after = read_pkg(dir.path());
        assert!(after.contains("\"engines\": {"));
        assert!(after.contains("\"node\": \">=20 <21\""));
    }

    #[test]
    fn ensure_npm_dep_and_devdep_are_separate() {
        let dir = TempDir::new().unwrap();
        write_pkg(dir.path(), "{}\n");

        apply_ensure(
            dir.path(),
            &EnsureSpec::NpmDep {
                file: "package.json".into(),
                name: "express".into(),
                version: "^4".into(),
            },
        )
        .unwrap();
        apply_ensure(
            dir.path(),
            &EnsureSpec::NpmDevDep {
                file: "package.json".into(),
                name: "typescript".into(),
                version: "^5".into(),
            },
        )
        .unwrap();

        let json: Value = serde_json::from_str(&read_pkg(dir.path())).unwrap();
        assert_eq!(json["dependencies"]["express"], "^4");
        assert_eq!(json["devDependencies"]["typescript"], "^5");
    }

    #[test]
    fn errors_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let err = apply_ensure(
            dir.path(),
            &EnsureSpec::NpmScript {
                file: "package.json".into(),
                name: "x".into(),
                value: "y".into(),
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("cannot read"));
    }

    #[test]
    fn errors_when_json_invalid() {
        let dir = TempDir::new().unwrap();
        write_pkg(dir.path(), "not json");
        let err = apply_ensure(
            dir.path(),
            &EnsureSpec::NpmScript {
                file: "package.json".into(),
                name: "x".into(),
                value: "y".into(),
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid JSON"));
    }

    #[test]
    fn detect_indent_two_spaces() {
        assert_eq!(detect_indent("{\n  \"a\": 1\n}\n"), "  ");
    }

    #[test]
    fn detect_indent_four_spaces() {
        assert_eq!(detect_indent("{\n    \"a\": 1\n}\n"), "    ");
    }

    #[test]
    fn detect_indent_default_when_flat() {
        assert_eq!(detect_indent("{\"a\":1}"), "  ");
    }
}
