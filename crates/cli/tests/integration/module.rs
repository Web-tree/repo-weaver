use crate::common::{TestContext, cmd};
use predicates::prelude::*;

#[test]
fn test_module_list() {
    let ctx = TestContext::new();

    ctx.write_file(
        "weaver.yaml",
        r#"
version: "1"
modules:
  - name: "mod1"
    source: "https://github.com/example/mod1"
    ref: "v1.0.0"
  - name: "mod2"
    source: "https://github.com/example/mod2"
    ref: "main"
config:
  includes: []
apps: []
"#,
    );

    let mut cmd = cmd();
    let assert = cmd
        .arg("module")
        .arg("list")
        .current_dir(&ctx.root)
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("mod1"))
        .stdout(predicate::str::contains("v1.0.0"))
        .stdout(predicate::str::contains("mod2"))
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_module_update() {
    let ctx = TestContext::new();

    ctx.write_file(
        "weaver.yaml",
        r#"
version: "1"
modules:
  - name: "mod1"
    source: "https://github.com/example/mod1"
    ref: "v1.0.0"
config:
  includes: []
apps: []
"#,
    );

    let mut cmd = cmd();
    let assert = cmd
        .arg("module")
        .arg("update")
        .arg("mod1")
        .arg("--ref")
        .arg("v2.0.0")
        .arg("--no-fetch") // Skip cache clearing for test speed/isolation
        .current_dir(&ctx.root)
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("Updated module 'mod1'"))
        .stdout(predicate::str::contains("v2.0.0"));

    // Verify weaver.yaml was updated
    let content = ctx.read_file("weaver.yaml");
    assert!(content.contains("ref: v2.0.0"));
}

#[test]
fn test_module_update_not_found() {
    let ctx = TestContext::new();
    ctx.write_file("weaver.yaml", "version: '1'\nmodules: []\napps: []");

    let mut cmd = cmd();
    let assert = cmd
        .arg("module")
        .arg("update")
        .arg("nonexistent")
        .arg("--ref")
        .arg("v1")
        .current_dir(&ctx.root)
        .assert();

    assert
        .failure()
        .stderr(predicate::str::contains("Module 'nonexistent' not found"));
}
