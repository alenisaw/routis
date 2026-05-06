use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_documents_tui_only_surface() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Interactive TUI"))
        .stdout(predicate::str::contains("--task").not())
        .stdout(predicate::str::contains("session").not())
        .stdout(predicate::str::contains("context").not());
}

#[test]
fn routis_without_task_enters_tui_path_when_smoke_env_is_set() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.env("ROUTIS_TUI_SMOKE_EXIT", "1")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn removed_cli_task_flags_are_rejected() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args(["--task", "debug auth flow"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}
