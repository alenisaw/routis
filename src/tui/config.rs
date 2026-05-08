use crate::tui::state::ConfigState;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const DEFAULT_POLICY_YAML: &str = include_str!("../../configs/policies/default.yaml");

#[must_use]
pub fn routis_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("ROUTIS_HOME") {
        return PathBuf::from(path);
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(root) = discover_repo_root(&cwd) {
            return root.join(".routis");
        }
        return cwd.join(".routis");
    }
    PathBuf::from(".routis")
}

#[must_use]
pub fn default_config_path() -> PathBuf {
    routis_dir().join("config.toml")
}

pub fn load_config(path: &Path) -> Result<Option<ConfigState>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read config `{}`", path.display()))?;
    Ok(Some(parse_config(&raw)))
}

pub fn save_config(path: &Path, config: &ConfigState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
        ensure_default_policy(parent)?;
    }
    fs::write(path, serialize_config(config))
        .with_context(|| format!("failed to write config `{}`", path.display()))
}

fn ensure_default_policy(routis_dir: &Path) -> Result<()> {
    let path = routis_dir.join("policies").join("default.yaml");
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    fs::write(&path, DEFAULT_POLICY_YAML)
        .with_context(|| format!("failed to write default policy `{}`", path.display()))
}

fn discover_repo_root(cwd: &Path) -> Option<PathBuf> {
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

fn parse_config(raw: &str) -> ConfigState {
    let mut config = ConfigState::default();
    for line in raw.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "display_name" => config.display_name = value.to_string(),
            "provider" => config.provider = value.to_string(),
            "model" => config.model = value.to_string(),
            "reasoning" => config.reasoning = value.to_string(),
            "theme" => config.theme = value.to_string(),
            "policy_file" => config.policy_file = value.to_string(),
            _ => {}
        }
    }
    config
}

fn serialize_config(config: &ConfigState) -> String {
    [
        format!("display_name={}", config.display_name),
        format!("provider={}", config.provider),
        format!("model={}", config.model),
        format!("reasoning={}", config.reasoning),
        format!("theme={}", config.theme),
        format!("policy_file={}", config.policy_file),
    ]
    .join("\n")
}
