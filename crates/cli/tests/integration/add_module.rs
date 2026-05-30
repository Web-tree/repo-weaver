use crate::common::{TestContext, cmd};
use predicates::prelude::*;

#[test]
fn add_module_appends_entry_and_pins_commit() {
    let ctx = TestContext::new();
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

    let lock = ctx.read_file("weaver.lock");
    assert!(lock.contains("resolved_commit"));
}
