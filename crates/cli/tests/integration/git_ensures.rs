use crate::common::{TestContext, cmd};
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_git_submodule_ensure() {
    let ctx = TestContext::new();

    // 1. Create Upstream Repo
    let upstream_dir = ctx.temp.path().join("upstream");
    std::fs::create_dir(&upstream_dir).unwrap();

    let git_status = Command::new("git")
        .arg("init")
        .current_dir(&upstream_dir)
        .status()
        .expect("git init upstream failed");
    assert!(git_status.success());

    // config git user
    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&upstream_dir)
        .status()
        .unwrap();
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&upstream_dir)
        .status()
        .unwrap();

    std::fs::write(upstream_dir.join("README.md"), "# Upstream").unwrap();
    Command::new("git")
        .args(&["add", "."])
        .current_dir(&upstream_dir)
        .status()
        .unwrap();
    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&upstream_dir)
        .status()
        .unwrap();

    // Get HEAD ref
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .current_dir(&upstream_dir)
        .output()
        .unwrap();
    let ref_ = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // 2. Create Module Manifest with Ensure
    let module_dir = ctx.temp.path().join("modules/my-module");
    std::fs::create_dir_all(&module_dir).unwrap();

    // Initialize git in module_dir so it can be cloned
    let git_status = Command::new("git")
        .arg("init")
        .current_dir(&module_dir)
        .status()
        .unwrap();
    assert!(git_status.success());
    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&module_dir)
        .status()
        .unwrap();
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&module_dir)
        .status()
        .unwrap();

    // weaver.module.yaml
    let submodule_url = upstream_dir.to_str().unwrap().replace("\\", "/"); // Handle windows path?
    // Using file:// URL for submodule
    let submodule_url = format!("file://{}", submodule_url);

    std::fs::write(
        module_dir.join("weaver.module.yaml"),
        format!(
            r#"
ensures:
  - type: git.submodule
    url: "{}"
    path: "deps/upstream"
    ref: "{}"
"#,
            submodule_url, ref_
        ),
    )
    .unwrap();

    Command::new("git")
        .args(&["add", "."])
        .current_dir(&module_dir)
        .status()
        .unwrap();
    Command::new("git")
        .args(&["commit", "-m", "Module init"])
        .current_dir(&module_dir)
        .status()
        .unwrap();

    let module_src = format!("file://{}", module_dir.to_str().unwrap());

    // 3. Create App Config
    ctx.write_file(
        "weaver.yaml",
        &format!(
            r#"
version: "1"
apps:
  - name: "app1"
    module: "my-module"
    path: "app1"
modules:
  - name: "my-module"
    source: "{}"
    ref: "HEAD"
"#,
            module_src
        ),
    );

    // 4. Init git in workspace (required for submodule add)
    let git_status = Command::new("git")
        .arg("init")
        .current_dir(&ctx.root)
        .status()
        .expect("git init workspace failed");
    assert!(git_status.success());
    // config git user
    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&ctx.root)
        .status()
        .unwrap();
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&ctx.root)
        .status()
        .unwrap();
    // Allow file protocol for submodules in test
    Command::new("git")
        .args(&["config", "protocol.file.allow", "always"])
        .current_dir(&ctx.root)
        .status()
        .unwrap();

    // 5. Run rw apply
    let mut cmd = cmd();
    let assert = cmd
        .arg("apply")
        .env("GIT_ALLOW_PROTOCOL", "file")
        .current_dir(&ctx.root)
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("Ensuring: Ensure git submodule"));

    // 6. Verify Submodule
    let submodule_path = ctx.root.join("app1/deps/upstream");
    assert!(submodule_path.exists());
    assert!(submodule_path.join("README.md").exists());
}
