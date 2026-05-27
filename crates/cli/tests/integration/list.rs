use crate::common::{TestContext, cmd};

use predicates::prelude::*;

#[test]
fn test_list_apps() {
    let ctx = TestContext::new();

    ctx.write_file(
        "weaver.yaml",
        r#"
version: "1"
apps:
  - name: "app1"
    module: "mod1"
    path: "app1"
"#,
    );

    let mut cmd = cmd();
    let assert = cmd.arg("list").current_dir(&ctx.root).assert();

    assert
        .success()
        .stdout(predicate::str::contains("app1"))
        .stdout(predicate::str::contains("mod1"))
        .stdout(predicate::str::contains("APP"));
}

#[test]
fn test_list_json() {
    let ctx = TestContext::new();

    ctx.write_file(
        "weaver.yaml",
        r#"
version: "1"
apps:
  - name: "app1"
    module: "mod1"
    path: "app1"
"#,
    );

    let mut cmd = cmd();
    let assert = cmd
        .arg("list")
        .arg("--json")
        .current_dir(&ctx.root)
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("\"name\": \"app1\""));
}

#[test]
fn test_list_empty_workspace() {
    let ctx = TestContext::new();
    // No weaver.yaml

    let mut cmd = cmd();
    let assert = cmd.arg("list").current_dir(&ctx.root).assert();

    assert
        .failure()
        .code(2)
        .stderr(predicate::str::contains("No apps defined"));
}
