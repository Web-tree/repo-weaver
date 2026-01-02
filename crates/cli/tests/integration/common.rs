use assert_cmd::Command;

pub fn cmd() -> Command {
    Command::cargo_bin("repo-weaver").unwrap()
}
