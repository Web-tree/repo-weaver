use crate::common::{TestContext, cmd};
use predicates::prelude::*;

#[test]
fn test_describe_app() {
    let ctx = TestContext::new();
    ctx.setup_module("mod1", "v1", "ensures: []");

    let mod_path = ctx.root.join("remotes").join("mod1");
    let mod_url = format!("file://{}", mod_path.display());

    ctx.write_file(
        "weaver.yaml",
        &format!(
            r#"
version: "1"
modules:
  - name: "mod1"
    source: "{}"
    ref: "v1"
apps:
  - name: "app1"
    module: "mod1"
    path: "apps/app1"
    inputs:
      apiKey: "secret-value-123"
      public: "public-value"
"#,
            mod_url
        ),
    );

    // Initialise .rw
    std::fs::create_dir_all(ctx.root.join(".rw")).unwrap();
    ctx.write_file(".rw/state.yaml", "files: {}");

    // Test Default Output (Redacted)
    let mut cmd1 = cmd();
    let assert1 = cmd1
        .arg("describe")
        .arg("app1")
        .current_dir(&ctx.root)
        .assert();

    assert1
        .success()
        .stdout(predicate::str::contains("App: app1"))
        .stdout(predicate::str::contains("apiKey: ***")) // Redacted
        .stdout(predicate::str::contains("public: public-value"));

    // Test Show Secrets
    let mut cmd2 = cmd();
    let assert2 = cmd2
        .arg("describe")
        .arg("app1")
        .arg("--show-secrets")
        .current_dir(&ctx.root)
        .assert();

    assert2
        .success()
        .stdout(predicate::str::contains("apiKey: secret-value-123")); // Value shown (unquoted string)

    // Test JSON Output
    let mut cmd3 = cmd();
    let assert3 = cmd3
        .arg("describe")
        .arg("app1")
        .arg("--json")
        .current_dir(&ctx.root)
        .assert();

    assert3
        .success()
        .stdout(predicate::str::contains("\"app\": \"app1\""))
        .stdout(predicate::str::contains("\"apiKey\": \"***\""));
}
