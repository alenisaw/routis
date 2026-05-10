use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn help_documents_tui_only_surface() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Interactive TUI"))
        .stdout(predicate::str::contains("--task").not())
        .stdout(predicate::str::contains("route"))
        .stdout(predicate::str::contains("context"));
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

#[test]
fn route_command_prints_decision_without_tui() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args(["route", "fix routing classifier for long prompts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("selected:"))
        .stdout(predicate::str::contains("intent:"))
        .stdout(predicate::str::contains("area: routing"));
}

#[test]
fn route_explain_prints_tree_and_writes_trace() {
    let routis_home = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.env("ROUTIS_HOME", routis_home.path())
        .args(["route", "--explain", "debug auth flow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Routis Decision Trace"))
        .stdout(predicate::str::contains("Selected profile:"));

    let trace_count = std::fs::read_dir(routis_home.path().join("traces"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("jsonl"))
        .count();
    assert_eq!(trace_count, 1);
}

#[test]
fn traces_latest_prints_stored_trace_tree() {
    let routis_home = TempDir::new().unwrap();
    Command::cargo_bin("routis")
        .unwrap()
        .env("ROUTIS_HOME", routis_home.path())
        .args(["route", "--explain", "debug auth flow"])
        .assert()
        .success();

    Command::cargo_bin("routis")
        .unwrap()
        .env("ROUTIS_HOME", routis_home.path())
        .args(["traces", "--latest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Routis Decision Trace"))
        .stdout(predicate::str::contains("Selected profile:"));
}
