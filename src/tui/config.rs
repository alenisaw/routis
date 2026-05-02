use crate::tui::state::ConfigState;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[must_use]
pub fn routis_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        PathBuf::from(home).join(".routis")
    } else {
        PathBuf::from(".routis")
    }
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
    }
    fs::write(path, serialize_config(config))
        .with_context(|| format!("failed to write config `{}`", path.display()))
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
    ]
    .join("\n")
}
