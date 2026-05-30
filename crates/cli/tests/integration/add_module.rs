use crate::common::{TestContext, cmd};
use predicates::prelude::*;
use std::path::Path;

/// Create a local git repo at `dir` with a `weaver.module.yaml`, an initial
/// commit tagged `v1`, then a second commit tagged `v2`. Returns the
/// `file://` URL for the repo.
fn make_module_repo(dir: &Path) -> String {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("weaver.module.yaml"), "inputs: {}\n").unwrap();
    for args in [
        vec!["init"],
        vec!["config", "user.email", "t@t.local"],
        vec!["config", "user.name", "T"],
        vec!["add", "."],
        vec!["commit", "-m", "init"],
        vec!["tag", "v1"],
    ] {
        std::process::Command::new("git").args(&args).current_dir(dir).output().unwrap();
    }
    // Second commit + tag so update-to-a-different-ref can be exercised.
    std::fs::write(dir.join("README.md"), "v2\n").unwrap();
    for args in [
        vec!["add", "."],
        vec!["commit", "-m", "second"],
        vec!["tag", "v2"],
    ] {
        std::process::Command::new("git").args(&args).current_dir(dir).output().unwrap();
    }
    format!("file://{}", dir.display())
}

#[test]
fn add_module_appends_entry_and_pins_commit() {
    let ctx = TestContext::new();
    ctx.write_file("weaver.yaml", "version: \"1\"\nmodules: []\napps: []\n");

    let url = make_module_repo(&ctx.root.join("std-repo"));

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

    let lock = ctx.read_file("weaver.lock");
    assert!(lock.contains("resolved_commit"));
}

#[test]
fn add_module_is_idempotent_updates_existing() {
    let ctx = TestContext::new();
    ctx.write_file("weaver.yaml", "version: \"1\"\nmodules: []\napps: []\n");

    let url = make_module_repo(&ctx.root.join("std-repo"));

    let run_add = |ref_: &str| {
        let mut c = cmd();
        c.arg("module")
            .arg("add")
            .arg(&url)
            .arg("--name")
            .arg("standards")
            .arg("--ref")
            .arg(ref_)
            .env("HOME", ctx.temp.path())
            .current_dir(&ctx.root)
            .assert()
            .success();
    };

    // First add.
    run_add("v1");
    // Second add of the SAME module must succeed (idempotent), not bail/duplicate.
    run_add("v1");

    let weaver = ctx.read_file("weaver.yaml");
    // Exactly one entry for the module, regardless of serializer quoting.
    let count = weaver.matches("name: standards").count() + weaver.matches("name: \"standards\"").count();
    assert_eq!(count, 1, "expected exactly one 'standards' module entry, got {count}\n{weaver}");

    // Re-running with a different ref updates the existing entry's ref.
    run_add("v2");
    let weaver = ctx.read_file("weaver.yaml");
    let count = weaver.matches("name: standards").count() + weaver.matches("name: \"standards\"").count();
    assert_eq!(count, 1, "expected still exactly one entry after update, got {count}\n{weaver}");
    assert!(
        weaver.contains("ref: v2") || weaver.contains("ref: \"v2\""),
        "expected ref updated to v2\n{weaver}"
    );
    assert!(
        !weaver.contains("ref: v1") && !weaver.contains("ref: \"v1\""),
        "expected old ref v1 to be gone\n{weaver}"
    );
}
