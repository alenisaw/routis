#![forbid(unsafe_code)]
#![deny(warnings)]

pub use routis_core::RiskZone;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    process::Command,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("failed to run git command `{command}` in `{cwd}`: {source}")]
    GitIo {
        command: String,
        cwd: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoContext {
    pub changed_files: Vec<PathBuf>,
    pub file_extension_spread: HashMap<String, usize>,
    pub risk_zone_hints: Vec<RiskZone>,
    pub branch: Option<String>,
    pub commit_count_since_main: Option<usize>,
    pub repo_markers: Vec<String>,
}

pub fn collect_repo_context(cwd: impl AsRef<Path>) -> Result<RepoContext, ContextError> {
    let cwd = cwd.as_ref();
    if !is_git_repository(cwd)? {
        return Ok(RepoContext::default());
    }

    let changed_files = changed_files(cwd)?;
    Ok(RepoContext {
        file_extension_spread: file_extension_spread(changed_files.iter()),
        risk_zone_hints: detect_risk_zones(changed_files.iter()),
        changed_files,
        branch: current_branch(cwd)?,
        commit_count_since_main: commit_count_since_main(cwd)?,
        repo_markers: repo_markers(cwd),
    })
}

pub fn detect_risk_zones<I, P>(paths: I) -> Vec<RiskZone>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut zones = BTreeSet::new();
    for path in paths {
        detect_path_risk_zones(path.as_ref(), &mut zones);
    }
    zones.into_iter().collect()
}

pub fn file_extension_spread<I, P>(paths: I) -> HashMap<String, usize>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut spread = HashMap::new();
    for path in paths {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|value| value.to_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_ascii_lowercase)
            .unwrap_or_else(|| "[none]".to_string());
        *spread.entry(extension).or_insert(0) += 1;
    }
    spread
}

fn detect_path_risk_zones(path: &Path, zones: &mut BTreeSet<RiskZone>) {
    let normalized = normalize_path(path);
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if normalized.starts_with("configs/")
        || normalized.contains("/config/")
        || file_name.ends_with(".toml")
        || file_name.ends_with(".yaml")
        || file_name.ends_with(".yml")
        || file_name.ends_with(".json")
        || file_name.ends_with(".env")
    {
        zones.insert(RiskZone::Config);
    }
    if normalized.contains("/auth/")
        || normalized.starts_with("auth/")
        || normalized.contains("authentication")
        || normalized.contains("authorization")
        || normalized.contains("login")
    {
        zones.insert(RiskZone::Auth);
    }
    if normalized.contains("schema")
        || normalized.contains("migration")
        || normalized.starts_with("migrations/")
        || file_name.ends_with(".sql")
    {
        zones.insert(RiskZone::Schema);
    }
    if normalized.starts_with(".github/workflows/")
        || normalized.starts_with(".gitlab-ci")
        || normalized.contains("/workflows/")
    {
        zones.insert(RiskZone::Workflow);
    }
    if matches!(
        file_name.as_str(),
        "cargo.toml"
            | "cargo.lock"
            | "package.json"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "pyproject.toml"
            | "requirements.txt"
    ) {
        zones.insert(RiskZone::Package);
    }
    if normalized.starts_with("tests/")
        || normalized.contains("/tests/")
        || normalized.contains("_test.")
        || normalized.contains("test_")
    {
        zones.insert(RiskZone::Tests);
    }
    if normalized.starts_with("docs/")
        || file_name == "readme.md"
        || file_name.ends_with(".md")
        || file_name.ends_with(".mdx")
    {
        zones.insert(RiskZone::Docs);
    }
    if normalized.contains("/tui/")
        || normalized.starts_with("src/tui/")
        || normalized.contains("/ui/")
        || normalized.contains("/screens/")
        || normalized.contains("/widgets/")
    {
        zones.insert(RiskZone::Ui);
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_ascii_lowercase()
}

fn is_git_repository(cwd: &Path) -> Result<bool, ContextError> {
    let output = run_git(cwd, ["rev-parse", "--is-inside-work-tree"])?;
    Ok(output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "true")
}

fn changed_files(cwd: &Path) -> Result<Vec<PathBuf>, ContextError> {
    let output = run_git(cwd, ["status", "--porcelain", "--untracked-files=all"])?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    let mut files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(parse_status_path)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    Ok(files)
}

fn parse_status_path(line: &str) -> Option<&str> {
    if line.len() < 4 {
        return None;
    }
    let path = line.get(3..)?.trim();
    if path.is_empty() {
        return None;
    }
    Some(
        path.rsplit_once(" -> ")
            .map_or(path, |(_, after)| after)
            .trim(),
    )
}

fn current_branch(cwd: &Path) -> Result<Option<String>, ContextError> {
    let output = run_git(cwd, ["branch", "--show-current"])?;
    if !output.status.success() {
        return Ok(None);
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok((!branch.is_empty()).then_some(branch))
}

fn commit_count_since_main(cwd: &Path) -> Result<Option<usize>, ContextError> {
    let output = run_git(cwd, ["rev-list", "--count", "main..HEAD"])?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<usize>()
        .ok())
}

fn run_git<const N: usize>(
    cwd: &Path,
    args: [&str; N],
) -> Result<std::process::Output, ContextError> {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|source| ContextError::GitIo {
            command: format!("git {}", args.join(" ")),
            cwd: cwd.display().to_string(),
            source,
        })
}

fn repo_markers(cwd: &Path) -> Vec<String> {
    let mut markers = Vec::new();
    if cwd.join("Cargo.toml").exists() {
        markers.push("rust".to_string());
    }
    if cwd.join("src/tui").exists() {
        markers.push("tui".to_string());
    }
    if cwd.join("crates").exists() {
        markers.push("workspace".to_string());
    }
    if cwd.join("tests").exists() {
        markers.push("tests".to_string());
    }
    if cwd.join(".github/workflows").exists() {
        markers.push("workflow".to_string());
    }
    if cwd.join("README.md").exists() {
        markers.push("docs".to_string());
    }
    markers
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn detects_risk_zones_from_repository_paths() {
        let zones = detect_risk_zones([
            ".github/workflows/ci.yml",
            "Cargo.toml",
            "src/tui/screens/home.rs",
            "tests/cli_smoke.rs",
            "README.md",
            "migrations/001_create_users.sql",
            "src/auth/session.rs",
            "configs/policies/default.yaml",
        ]);

        assert_eq!(
            zones,
            vec![
                RiskZone::Config,
                RiskZone::Auth,
                RiskZone::Schema,
                RiskZone::Workflow,
                RiskZone::Package,
                RiskZone::Tests,
                RiskZone::Docs,
                RiskZone::Ui,
            ]
        );
    }

    #[test]
    fn counts_file_extensions_case_insensitively() {
        let spread = file_extension_spread(["README.md", "src/main.RS", "Cargo.toml", "LICENSE"]);

        assert_eq!(spread.get("md"), Some(&1));
        assert_eq!(spread.get("rs"), Some(&1));
        assert_eq!(spread.get("toml"), Some(&1));
        assert_eq!(spread.get("[none]"), Some(&1));
    }

    #[test]
    fn collects_branch_changed_files_extensions_and_risk_zones() {
        let repo = TempDir::new().unwrap();
        git(repo.path(), ["init"]).unwrap();
        git(repo.path(), ["config", "user.email", "routis@example.test"]).unwrap();
        git(repo.path(), ["config", "user.name", "Routis Test"]).unwrap();
        std::fs::write(repo.path().join("README.md"), "initial\n").unwrap();
        git(repo.path(), ["add", "."]).unwrap();
        git(repo.path(), ["commit", "-m", "initial"]).unwrap();
        git(repo.path(), ["checkout", "-b", "feature/context"]).unwrap();

        std::fs::create_dir_all(repo.path().join(".github/workflows")).unwrap();
        std::fs::create_dir_all(repo.path().join("src/auth")).unwrap();
        std::fs::write(repo.path().join(".github/workflows/ci.yml"), "name: ci\n").unwrap();
        std::fs::write(
            repo.path().join("src/auth/session.rs"),
            "pub fn touch() {}\n",
        )
        .unwrap();

        let context = collect_repo_context(repo.path()).unwrap();

        assert_eq!(context.branch.as_deref(), Some("feature/context"));
        assert_eq!(
            context.changed_files,
            vec![
                std::path::PathBuf::from(".github/workflows/ci.yml"),
                std::path::PathBuf::from("src/auth/session.rs"),
            ]
        );
        assert_eq!(context.file_extension_spread.get("rs"), Some(&1));
        assert_eq!(context.file_extension_spread.get("yml"), Some(&1));
        assert_eq!(
            context.risk_zone_hints,
            vec![RiskZone::Config, RiskZone::Auth, RiskZone::Workflow]
        );
        assert!(context.repo_markers.contains(&"docs".to_string()));
    }

    #[test]
    fn non_git_directory_returns_default_context() {
        let dir = TempDir::new().unwrap();

        let context = collect_repo_context(dir.path()).unwrap();

        assert_eq!(context.branch, None);
        assert!(context.changed_files.is_empty());
        assert!(context.file_extension_spread.is_empty());
        assert!(context.risk_zone_hints.is_empty());
    }

    fn git<const N: usize>(
        cwd: &std::path::Path,
        args: [&str; N],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git").args(args).current_dir(cwd).output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string().into())
        }
    }
}
