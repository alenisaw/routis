use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::NamedTempFile;
use tempfile::TempDir;

#[test]
fn dry_run_routes_task_from_flag() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    let _clean = configure_clean_workspace(&mut cmd);
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
    let _clean = configure_clean_workspace(&mut cmd);
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
    let _clean = configure_clean_workspace(&mut cmd);
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
    let _clean = configure_clean_workspace(&mut cmd);
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

#[test]
fn explain_uses_repo_context_to_elevate_default_profile() {
    let repo = TempDir::new().unwrap();
    git(repo.path(), ["init"]).unwrap();
    git(repo.path(), ["config", "user.email", "routis@example.test"]).unwrap();
    git(repo.path(), ["config", "user.name", "Routis Test"]).unwrap();
    std::fs::write(repo.path().join("README.md"), "initial\n").unwrap();
    git(repo.path(), ["add", "."]).unwrap();
    git(repo.path(), ["commit", "-m", "initial"]).unwrap();
    std::fs::create_dir_all(repo.path().join("src/auth")).unwrap();
    std::fs::write(
        repo.path().join("src/auth/session.rs"),
        "pub fn touch() {}\n",
    )
    .unwrap();

    let policy_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("configs/policies/default.yaml");
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.current_dir(repo.path())
        .args([
            "--task",
            "small fix",
            "--explain",
            "--policy-file",
            policy_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Effective profile: deep"))
        .stdout(predicate::str::contains("Risk zones:       auth"))
        .stdout(predicate::str::contains("Changed files:    1"));
}

#[test]
fn context_command_prints_repo_context() {
    let repo = TempDir::new().unwrap();
    git(repo.path(), ["init"]).unwrap();
    git(repo.path(), ["config", "user.email", "routis@example.test"]).unwrap();
    git(repo.path(), ["config", "user.name", "Routis Test"]).unwrap();
    std::fs::write(repo.path().join("README.md"), "initial\n").unwrap();
    git(repo.path(), ["add", "."]).unwrap();
    git(repo.path(), ["commit", "-m", "initial"]).unwrap();
    git(repo.path(), ["checkout", "-b", "feature/context"]).unwrap();
    std::fs::create_dir_all(repo.path().join("src/auth")).unwrap();
    std::fs::write(
        repo.path().join("src/auth/session.rs"),
        "pub fn touch() {}\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.current_dir(repo.path())
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Branch:           feature/context",
        ))
        .stdout(predicate::str::contains("Changed files:    1"))
        .stdout(predicate::str::contains("Risk zones:       auth"))
        .stdout(predicate::str::contains("src/auth/session.rs"));
}

#[test]
fn policy_path_rule_can_cap_default_profile() {
    let repo = TempDir::new().unwrap();
    git(repo.path(), ["init"]).unwrap();
    git(repo.path(), ["config", "user.email", "routis@example.test"]).unwrap();
    git(repo.path(), ["config", "user.name", "Routis Test"]).unwrap();
    std::fs::write(repo.path().join("README.md"), "initial\n").unwrap();
    git(repo.path(), ["add", "."]).unwrap();
    git(repo.path(), ["commit", "-m", "initial"]).unwrap();
    std::fs::write(repo.path().join("README.md"), "changed\n").unwrap();

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
rules:
  - if_path: README.md
    max_profile: cheap
"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.current_dir(repo.path())
        .args([
            "--task",
            "implement documentation update",
            "--policy-file",
            policy.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Effective profile: cheap"))
        .stdout(predicate::str::contains("local-cheap"));
}

#[test]
fn session_list_prints_saved_cli_routes() {
    let home = TempDir::new().unwrap();
    let mut route = Command::cargo_bin("routis").unwrap();
    let _clean = configure_clean_workspace(&mut route);
    route
        .env("USERPROFILE", home.path())
        .args(["--task", "debug auth flow"])
        .assert()
        .success();

    let mut list = Command::cargo_bin("routis").unwrap();
    list.env("USERPROFILE", home.path())
        .args(["session", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("debug-auth-flow"))
        .stdout(predicate::str::contains("deep"))
        .stdout(predicate::str::contains("gpt-5.5"));
}

#[test]
fn session_resume_prints_saved_route_preview() {
    let home = TempDir::new().unwrap();
    let mut route = Command::cargo_bin("routis").unwrap();
    let _clean = configure_clean_workspace(&mut route);
    route
        .env("USERPROFILE", home.path())
        .args(["--task", "debug auth flow"])
        .assert()
        .success();

    let mut resume = Command::cargo_bin("routis").unwrap();
    resume
        .env("USERPROFILE", home.path())
        .args(["session", "resume", "debug-auth-flow"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Session:          debug-auth-flow",
        ))
        .stdout(predicate::str::contains(
            "Task:             debug auth flow",
        ))
        .stdout(predicate::str::contains("Effective profile: deep"))
        .stdout(predicate::str::contains("Model:            gpt-5.5"));
}

fn git<const N: usize>(cwd: &Path, args: [&str; N]) -> Result<(), Box<dyn std::error::Error>> {
    let output = StdCommand::new("git")
        .args(args)
        .current_dir(cwd)
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string().into())
    }
}

fn configure_clean_workspace(cmd: &mut Command) -> TempDir {
    let dir = TempDir::new().unwrap();
    let policy_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("configs/policies/default.yaml");
    cmd.current_dir(dir.path())
        .args(["--policy-file", policy_path.to_str().unwrap()]);
    dir
}
