use crate::common::cmd;

#[test]
fn test_bootstrap_empty_workspace() {
    let assert = cmd()
        .arg("apply")
        .current_dir("../../tests/fixtures/simple")
        .assert();

    // Expect success for empty workspace
    assert.success();
}
