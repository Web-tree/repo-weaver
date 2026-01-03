use crate::common::{TestContext, cmd};
use predicates::prelude::*;

#[test]
fn test_ai_patch_ensure() {
    let ctx = TestContext::new();

    // App dir and initial file
    let app_dir = ctx.root.join("app1");
    std::fs::create_dir(&app_dir).unwrap();
    // Verification command will check if file contains "AI was here"
    // Initially it doesn't.
    ctx.write_file("app1/target.txt", "Original content");

    let manifest = r#"
ensures:
  - type: ai.patch
    prompt: "Add 'AI was here' to target.txt"
    verify_command: "grep 'AI was here' ai_generated.txt"
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
        .success()
        .stdout(predicate::str::contains("Generating patch for prompt"));

    // Verify file content logic in `ai.rs` creates `ai_generated.txt` containing "AI was here"
    // Wait, the `verify_command` checks `target.txt`.
    // The dummy implementation in `ai.rs` writes to `ai_generated.txt`.
    // So verification will FAIL even after patch application (execute runs verify again).
    // And `execute` will assume verification failure implies rollback/error.

    // I need to align the test and `ai.rs` (mock behavior).
    // `ai.rs`:
    // let dummy_file = ctx.app_path.join("ai_generated.txt");
    // std::fs::write(&dummy_file, "AI was here")...

    // If I change verification command to check `ai_generated.txt`, it should pass *after* execute.
    // verify_command: "ls ai_generated.txt"
    // Or "grep 'AI was here' ai_generated.txt" if content matches.
}

#[test]
fn test_ai_patch_ensure_success_with_mock() {
    let ctx = TestContext::new();

    // App dir
    let app_dir = ctx.root.join("app1");
    std::fs::create_dir(&app_dir).unwrap();

    // Use verification command that passes IF `ai.rs` did its job.
    // ai.rs writes "AI was here" to "ai_generated.txt".

    let manifest = r#"
ensures:
  - type: ai.patch
    prompt: "Create AI file"
    verify_command: "grep 'AI was here' ai_generated.txt"
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
        .success()
        .stdout(predicate::str::contains("Generating patch"));

    // Verify file exists
    assert!(ctx.root.join("app1/ai_generated.txt").exists());
}
