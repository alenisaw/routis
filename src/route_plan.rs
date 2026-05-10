use anyhow::{Context, Result};
use routis_context::RepoContext;
use routis_core::{route_task_with_repo_context, Confidence, Profile, RoutingDecision};
use routis_policy::{apply_policy_rules, PolicyFile};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub const DEFAULT_POLICY_PATH: &str = ".routis/policies/default.yaml";
const DEFAULT_POLICY_YAML: &str = include_str!("../configs/policies/default.yaml");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub profile: String,
    pub model: String,
    pub reasoning: String,
    pub branch: String,
    pub changed_files: usize,
    pub impact_area: String,
    pub intent: String,
    pub area: String,
    pub scope: String,
    pub risk: String,
    pub confidence: String,
    pub context_percent: usize,
    pub saved_percent: usize,
    pub reason: String,
    pub policy_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoMapSummary {
    pub branch: String,
    pub changed_files: usize,
    pub repo_markers: Vec<String>,
    pub manifests: Vec<String>,
    pub docs: Vec<String>,
    pub tests: Vec<String>,
    pub workflows: Vec<String>,
    pub instruction_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedPolicy {
    pub policy: PolicyFile,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionPlanWithDecision {
    pub plan: ExecutionPlan,
    pub decision: RoutingDecision,
    pub repo_context: RepoContext,
    pub policy_overrides: Vec<String>,
}

pub fn build_execution_plan(
    task: &str,
    policy_path: &str,
    cwd: impl AsRef<Path>,
) -> Result<ExecutionPlan> {
    Ok(build_execution_plan_with_decision(task, policy_path, cwd)?.plan)
}

pub fn build_execution_plan_with_decision(
    task: &str,
    policy_path: &str,
    cwd: impl AsRef<Path>,
) -> Result<ExecutionPlanWithDecision> {
    let cwd = cwd.as_ref();
    let repo_root = discover_repo_root(cwd);
    let policy = load_policy(policy_path, repo_root.as_deref())?;
    let repo_context = collect_repo_context_for_cwd(cwd)?;
    let (mut plan, decision, policy_overrides) =
        plan_execution_with_decision(task, &policy.policy, &repo_context)?;
    plan.policy_source = policy.source;
    Ok(ExecutionPlanWithDecision {
        plan,
        decision,
        repo_context,
        policy_overrides,
    })
}

pub fn collect_repo_context_for_cwd(cwd: impl AsRef<Path>) -> Result<RepoContext> {
    let cwd = cwd.as_ref();
    let repo_root = discover_repo_root(cwd);
    let context_cwd = repo_root.as_deref().unwrap_or(cwd);
    Ok(routis_context::collect_repo_context(context_cwd)?)
}

pub fn load_policy(policy_path: &str, repo_root: Option<&Path>) -> Result<LoadedPolicy> {
    let requested = policy_path.trim();
    let requested = if requested.is_empty() {
        DEFAULT_POLICY_PATH
    } else {
        requested
    };
    let path = PathBuf::from(requested);
    let is_default_policy = requested.replace('\\', "/") == DEFAULT_POLICY_PATH;

    if path.is_absolute() {
        let policy = PolicyFile::load(&path)
            .with_context(|| format!("failed to load policy file `{}`", path.display()))?;
        return Ok(LoadedPolicy {
            policy,
            source: path.display().to_string(),
        });
    }

    if is_default_policy {
        let installed = crate::paths::default_policy_path();
        if installed.exists() {
            let policy = PolicyFile::load(&installed)
                .with_context(|| format!("failed to load policy file `{}`", installed.display()))?;
            return Ok(LoadedPolicy {
                policy,
                source: installed.display().to_string(),
            });
        }
        let policy = PolicyFile::parse_yaml(DEFAULT_POLICY_YAML, "embedded default policy")?;
        return Ok(LoadedPolicy {
            policy,
            source: "embedded default policy".to_string(),
        });
    }

    let mut rooted_error = None;
    if let Some(root) = repo_root {
        let rooted = root.join(&path);
        if rooted.exists() {
            match PolicyFile::load(&rooted) {
                Ok(policy) => {
                    return Ok(LoadedPolicy {
                        policy,
                        source: rooted.display().to_string(),
                    });
                }
                Err(error) => {
                    rooted_error = Some((rooted, error));
                }
            }
        }
    }

    let policy = PolicyFile::load(&path).with_context(|| {
        if let Some((rooted, _)) = rooted_error {
            format!(
                "failed to load policy file `{}` or `{}`",
                rooted.display(),
                path.display()
            )
        } else {
            format!("failed to load policy file `{}`", path.display())
        }
    })?;
    Ok(LoadedPolicy {
        policy,
        source: path.display().to_string(),
    })
}

pub fn plan_execution(
    task: &str,
    policy: &PolicyFile,
    repo_context: &RepoContext,
) -> Result<ExecutionPlan> {
    Ok(plan_execution_with_decision(task, policy, repo_context)?.0)
}

fn plan_execution_with_decision(
    task: &str,
    policy: &PolicyFile,
    repo_context: &RepoContext,
) -> Result<(ExecutionPlan, RoutingDecision, Vec<String>)> {
    let mut decision = route_task_with_repo_context(
        task,
        Profile::Default,
        &repo_context.risk_zone_hints,
        repo_context.changed_files.len(),
    )?;
    let profile_before_policy = decision.effective_profile;
    let target_hints = decision
        .classification
        .targets
        .iter()
        .map(|target| PathBuf::from(&target.value))
        .collect::<Vec<_>>();
    decision.effective_profile = apply_policy_rules(
        policy,
        decision.effective_profile,
        &repo_context.risk_zone_hints,
        &repo_context.changed_files,
        &target_hints,
    );
    let mut policy_overrides = Vec::new();
    if decision.effective_profile != profile_before_policy {
        policy_overrides.push(format!(
            "{}->{}",
            profile_before_policy.as_str(),
            decision.effective_profile.as_str()
        ));
        decision.signals_matched.push(format!(
            "policy-profile:{}",
            decision.effective_profile.as_str()
        ));
        decision.explain = route_reason(&decision);
    }
    let execution = policy
        .execution_config(decision.effective_profile)
        .context("selected profile has no execution config")?;

    let mut impact_area = format_impact_area(repo_context);
    if is_repo_wide_task(task) && impact_area == "repo" {
        impact_area = "repo-wide".to_string();
    }

    let plan = ExecutionPlan {
        profile: decision.effective_profile.as_str().to_string(),
        model: execution.model.clone(),
        reasoning: execution.reasoning.clone(),
        branch: repo_context
            .branch
            .clone()
            .unwrap_or_else(|| "-".to_string()),
        changed_files: repo_context.changed_files.len(),
        impact_area,
        intent: decision.classification.primary_intent.as_str().to_string(),
        area: decision.classification.area.as_str().to_string(),
        scope: decision.classification.scope.as_str().to_string(),
        risk: decision.classification.risk.as_str().to_string(),
        confidence: decision.classification.confidence.as_str().to_string(),
        context_percent: repo_context.changed_files.len().saturating_mul(6).min(100),
        saved_percent: saved_percent(decision.classification.confidence),
        reason: decision.explain.clone(),
        policy_source: String::new(),
    };
    Ok((plan, decision, policy_overrides))
}

fn route_reason(decision: &RoutingDecision) -> String {
    format!(
        "Auto-selected `{}` for {} / {} / scope {} / confidence {} from signals: {}.",
        decision.effective_profile,
        decision.classification.primary_intent.as_str(),
        decision.classification.area.as_str(),
        decision.classification.scope.as_str(),
        decision.classification.confidence.as_str(),
        decision.signals_matched.join(", ")
    )
}

#[must_use]
pub fn repo_map_summary(context: &RepoContext) -> RepoMapSummary {
    RepoMapSummary {
        branch: context.branch.clone().unwrap_or_else(|| "-".to_string()),
        changed_files: context.changed_files.len(),
        repo_markers: context.repo_markers.clone(),
        manifests: paths_to_strings(&context.manifests),
        docs: paths_to_strings(&context.docs),
        tests: paths_to_strings(&context.tests),
        workflows: paths_to_strings(&context.workflows),
        instruction_files: paths_to_strings(&context.instruction_files),
    }
}

fn paths_to_strings(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

fn saved_percent(confidence: Confidence) -> usize {
    match confidence {
        Confidence::High => 38,
        Confidence::Medium => 28,
        Confidence::Low => 12,
    }
}

#[must_use]
pub fn format_impact_area(repo_context: &RepoContext) -> String {
    if repo_context.risk_zone_hints.is_empty() && repo_context.repo_markers.is_empty() {
        return "repo".to_string();
    }
    let mut impact = repo_context
        .risk_zone_hints
        .iter()
        .map(|zone| zone.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    if impact.is_empty() {
        impact = repo_context.repo_markers.join(", ");
    } else if !repo_context.repo_markers.is_empty() {
        impact.push_str(" · ");
        impact.push_str(&repo_context.repo_markers.join(", "));
    }
    impact
}

pub fn discover_repo_root(cwd: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!root.is_empty()).then(|| PathBuf::from(root))
}

fn is_repo_wide_task(task: &str) -> bool {
    let normalized = task.to_ascii_lowercase();
    [
        "repo",
        "repository",
        "whole project",
        "whole repo",
        "entire repo",
        "this repo",
        "репо",
        "репозит",
        "проект",
    ]
    .iter()
    .any(|signal| normalized.contains(signal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn installed_default_policy_error_is_not_masked_by_embedded_fallback() {
        let dir = TempDir::new().unwrap();
        let routis_home = dir.path().join("routis-home");
        let policy_dir = routis_home.join("policies");
        fs::create_dir_all(&policy_dir).unwrap();
        let policy_path = policy_dir.join("default.yaml");
        fs::write(&policy_path, "version: [").unwrap();

        std::env::set_var("ROUTIS_HOME", &routis_home);
        let error = load_policy(DEFAULT_POLICY_PATH, Some(dir.path())).unwrap_err();
        std::env::remove_var("ROUTIS_HOME");
        let message = format!("{error:#}");

        assert!(message.contains("failed to load policy file"));
        assert!(message.contains("default.yaml"));
        assert!(!message.contains("embedded default policy"));
    }

    #[test]
    fn repo_context_is_collected_from_repo_root_for_nested_cwd() {
        let repo = TempDir::new().unwrap();
        run_git(repo.path(), ["init"]).unwrap();
        run_git(repo.path(), ["config", "user.email", "routis@example.test"]).unwrap();
        run_git(repo.path(), ["config", "user.name", "Routis Test"]).unwrap();
        fs::write(
            repo.path().join("Cargo.toml"),
            "[package]\nname='demo'\nversion='0.1.0'\n",
        )
        .unwrap();
        fs::create_dir_all(repo.path().join("src/nested")).unwrap();
        fs::write(repo.path().join("src/nested/mod.rs"), "pub fn demo() {}\n").unwrap();
        run_git(repo.path(), ["add", "."]).unwrap();
        run_git(repo.path(), ["commit", "-m", "initial"]).unwrap();
        fs::write(repo.path().join("README.md"), "changed\n").unwrap();

        let context = collect_repo_context_for_cwd(repo.path().join("src/nested")).unwrap();

        assert!(context.repo_markers.contains(&"rust".to_string()));
        assert!(context.manifests.contains(&PathBuf::from("Cargo.toml")));
        assert!(context.changed_files.contains(&PathBuf::from("README.md")));
    }

    fn run_git<const N: usize>(
        cwd: &Path,
        args: [&str; N],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string().into())
        }
    }
}
