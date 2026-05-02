use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn dry_run_routes_task_from_flag() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args(["--task", "debug auth flow", "--explain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Effective profile: deep"))
        .stdout(predicate::str::contains("gpt-5.5"))
        .stdout(predicate::str::contains("--reasoning high"))
        .stdout(predicate::str::contains("Execution mode:    dry-run"))
        .stdout(predicate::str::contains("Signals matched:"));
}

#[test]
fn cheap_signal_uses_cheap_policy_mapping() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args(["--task", "small fix in README"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Effective profile: cheap"))
        .stdout(predicate::str::contains("gpt-5.4-mini"))
        .stdout(predicate::str::contains("--reasoning low"));
}

#[test]
fn balanced_signal_uses_balanced_policy_mapping() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args(["--task", "implement config loader"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Effective profile: balanced"))
        .stdout(predicate::str::contains("gpt-5.5"))
        .stdout(predicate::str::contains("--reasoning medium"));
}

#[test]
fn explicit_policy_overrides_default_classifier() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args(["--task", "redesign architecture", "--policy", "cheap"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Requested policy:  cheap"))
        .stdout(predicate::str::contains("Effective profile: cheap"))
        .stdout(predicate::str::contains("gpt-5.4-mini"));
}

#[test]
fn positional_task_works() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args(["implement", "config", "loader"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Effective profile: balanced"));
}

#[test]
fn policy_file_is_loaded() {
    let mut policy = NamedTempFile::new().unwrap();
    write!(
        policy,
        r#"
version: 1
profiles:
  cheap:
    model: local-cheap
    reasoning: low
  balanced:
    model: local-balanced
    reasoning: medium
  deep:
    model: local-deep
    reasoning: high
  extradeep:
    model: local-extra
    reasoning: xhigh
"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args([
        "--task",
        "debug auth flow",
        "--policy-file",
        policy.path().to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Effective profile: deep"))
    .stdout(predicate::str::contains("local-deep"));
}

#[test]
fn invalid_policy_file_fails() {
    let mut policy = NamedTempFile::new().unwrap();
    write!(policy, "version: 999\nprofiles: {{}}\n").unwrap();

    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args([
        "--task",
        "debug auth flow",
        "--policy-file",
        policy.path().to_str().unwrap(),
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("unsupported policy version"));
}

#[test]
fn positional_and_flag_task_conflict() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args(["--task", "one", "two"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "use either --task <TEXT> or positional TASK",
        ));
}

#[test]
fn help_documents_task_argument_without_launching_tui() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("[TASK]"));
}

#[test]
fn unknown_policy_fails() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.args(["--task", "debug auth flow", "--policy", "max"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown policy"));
}
