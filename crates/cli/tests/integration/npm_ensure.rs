use crate::common::{TestContext, cmd, weaver_config};
use predicates::prelude::*;

#[test]
fn test_npm_script_ensure() {
    let ctx = TestContext::new();

    // Create app dir and package.json
    let app_dir = ctx.root.join("app1");
    std::fs::create_dir(&app_dir).unwrap();
    ctx.write_file("app1/package.json", r#"{ "name": "app1", "scripts": {} }"#);

    // Setup module with ensure config
    let manifest = r#"
ensures:
  - type: npm.script
    name: "test"
    command: "echo 'test'"
"#;
    ctx.setup_module_with_manifest("base", "master", "dummy content", manifest);

    // Initial config
    // Construct weaver.yaml manually to match our app setup
    let remote_path = ctx.root.join("remotes").join("base");
    let remote_url = format!("file://{}", remote_path.display());

    ctx.write_file(
        "weaver.yaml",
        &format!(
            r#"
version: "1"
modules:
  - name: "base"
    source: "{}"
    ref: "master"
apps:
  - name: "app1"
    module: "base"
    path: "app1"
"#,
            remote_url
        ),
    );

    let mut cmd = cmd();
    let assert = cmd.arg("apply").current_dir(&ctx.root).assert();

    assert
        .success()
        .stdout(predicate::str::contains("Ensured npm script 'test'"));

    // Verify package.json content by parsing JSON
    let content = ctx.read_file("app1/package.json");
    let v: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse package.json");

    let script = v["scripts"]["test"]
        .as_str()
        .expect("script test not found or not string");
    assert_eq!(script, "echo 'test'");
}

#[test]
fn test_npm_script_ensure_missing_package_json() {
    let ctx = TestContext::new();

    // App dir but no package.json
    let app_dir = ctx.root.join("app1");
    std::fs::create_dir(&app_dir).unwrap();

    // Setup module with ensure config
    let manifest = r#"
ensures:
  - type: npm.script
    name: "test"
    command: "echo 'test'"
"#;
    ctx.setup_module_with_manifest("base", "master", "dummy content", manifest);

    let remote_path = ctx.root.join("remotes").join("base");
    let remote_url = format!("file://{}", remote_path.display());

    ctx.write_file(
        "weaver.yaml",
        &format!(
            r#"
version: "1"
modules:
  - name: "base"
    source: "{}"
    ref: "master"
apps:
  - name: "app1"
    module: "base"
    path: "app1"
"#,
            remote_url
        ),
    );

    let mut cmd = cmd();
    let assert = cmd.arg("apply").current_dir(&ctx.root).assert();

    assert
        .failure()
        .stderr(predicate::str::contains("package.json not found"));
}
