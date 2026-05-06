use anyhow::{Context, Result};
use routis_context::RepoContext;
use routis_core::{route_task_with_repo_context, Profile};
use routis_policy::{apply_policy_rules, PolicyFile};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const DEFAULT_POLICY_PATH: &str = "configs/policies/default.yaml";
const DEFAULT_POLICY_YAML: &str = include_str!("../configs/policies/default.yaml");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub profile: String,
    pub model: String,
    pub reasoning: String,
    pub branch: String,
    pub changed_files: usize,
    pub impact_area: String,
    pub context_percent: usize,
    pub saved_percent: usize,
    pub reason: String,
    pub policy_source: String,
}

#[derive(Debug, Clone)]
pub struct LoadedPolicy {
    pub policy: PolicyFile,
    pub source: String,
}

pub fn build_execution_plan(
    task: &str,
    policy_path: &str,
    cwd: impl AsRef<Path>,
) -> Result<ExecutionPlan> {
    let cwd = cwd.as_ref();
    let repo_root = discover_repo_root(cwd);
    let policy = load_policy(policy_path, repo_root.as_deref())?;
    let repo_context = collect_repo_context_for_cwd(cwd)?;
    let mut plan = plan_execution(task, &policy.policy, &repo_context)?;
    plan.policy_source = policy.source;
    Ok(plan)
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

    if let Ok(policy) = PolicyFile::load(&path) {
        return Ok(LoadedPolicy {
            policy,
            source: path.display().to_string(),
        });
    }

    if path.is_relative() {
        if let Some(root) = repo_root {
            let rooted = root.join(&path);
            if let Ok(policy) = PolicyFile::load(&rooted) {
                return Ok(LoadedPolicy {
                    policy,
                    source: rooted.display().to_string(),
                });
            }
        }
    }

    if requested.replace('\\', "/") == DEFAULT_POLICY_PATH {
        let policy = PolicyFile::parse_yaml(DEFAULT_POLICY_YAML, "embedded default policy")?;
        return Ok(LoadedPolicy {
            policy,
            source: "embedded default policy".to_string(),
        });
    }

    let policy = PolicyFile::load(&path)
        .with_context(|| format!("failed to load policy file `{}`", path.display()))?;
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
    let mut decision = route_task_with_repo_context(
        task,
        Profile::Default,
        &repo_context.risk_zone_hints,
        repo_context.changed_files.len(),
    )?;
    decision.effective_profile = apply_policy_rules(
        policy,
        decision.effective_profile,
        &repo_context.risk_zone_hints,
        &repo_context.changed_files,
    );
    let execution = policy
        .execution_config(decision.effective_profile)
        .context("selected profile has no execution config")?;

    let mut impact_area = format_impact_area(repo_context);
    if is_repo_wide_task(task) && impact_area == "repo" {
        impact_area = "repo-wide".to_string();
    }

    Ok(ExecutionPlan {
        profile: decision.effective_profile.as_str().to_string(),
        model: execution.model.clone(),
        reasoning: execution.reasoning.clone(),
        branch: repo_context
            .branch
            .clone()
            .unwrap_or_else(|| "-".to_string()),
        changed_files: repo_context.changed_files.len(),
        impact_area,
        context_percent: repo_context.changed_files.len().saturating_mul(6).min(100),
        saved_percent: 32,
        reason: decision.explain,
        policy_source: String::new(),
    })
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
