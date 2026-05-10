use anyhow::{Context, Result};
use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellHistory {
    max_entries: usize,
    entries: VecDeque<String>,
    timestamps: VecDeque<Option<u64>>,
}

impl ShellHistory {
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: VecDeque::new(),
            timestamps: VecDeque::new(),
        }
    }

    #[must_use]
    pub fn entries(&self) -> &[String] {
        self.entries.as_slices().0
    }

    pub fn push(&mut self, value: &str) {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return;
        }
        self.push_entry(trimmed.to_string(), Some(now_epoch_seconds()));
    }

    fn push_entry(&mut self, value: String, timestamp: Option<u64>) {
        self.entries.push_back(value);
        self.timestamps.push_back(timestamp);
        while self.entries.len() > self.max_entries {
            let _ = self.entries.pop_front();
            let _ = self.timestamps.pop_front();
        }
    }

    pub fn load(path: &Path, max_entries: usize) -> Result<Self> {
        let mut history = Self::new(max_entries);
        if !path.exists() {
            return Ok(history);
        }
        let legacy_timestamp = fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(system_time_to_epoch_seconds);
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read history `{}`", path.display()))?;
        for line in raw.lines() {
            if let Some((timestamp, value)) = parse_history_line(line) {
                if !value.trim().is_empty() {
                    history.push_entry(value.trim().to_string(), timestamp.or(legacy_timestamp));
                }
            }
        }
        Ok(history)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            crate::private_fs::create_private_dir(parent)?;
        }
        let body = self
            .entries
            .iter()
            .zip(self.timestamps.iter())
            .map(|(entry, timestamp)| match timestamp {
                Some(value) => format!("{value}\t{entry}"),
                None => entry.clone(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        crate::private_fs::write_private_file(path, body.as_bytes())
            .with_context(|| format!("failed to write history `{}`", path.display()))
    }
}

#[must_use]
pub fn recent_sessions(limit: usize) -> Vec<(String, String)> {
    default_history_path()
        .and_then(|path| ShellHistory::load(&path, 1000))
        .map(|history| history.recent(limit))
        .unwrap_or_default()
}

impl ShellHistory {
    #[must_use]
    pub fn recent(&self, limit: usize) -> Vec<(String, String)> {
        self.recent_detailed(limit)
            .into_iter()
            .map(|item| (item.title, item.updated))
            .collect()
    }

    #[must_use]
    pub fn recent_detailed(&self, limit: usize) -> Vec<HistorySessionItem> {
        let now = now_epoch_seconds();
        self.entries
            .iter()
            .zip(self.timestamps.iter())
            .rev()
            .filter(|(entry, _)| !entry.trim_start().starts_with('/'))
            .take(limit)
            .map(|(entry, timestamp)| {
                let updated = timestamp
                    .map(|value| relative_time(value, now))
                    .unwrap_or_else(|| "earlier".to_string());
                HistorySessionItem {
                    title: crate::tui::session::make_session_title(entry),
                    created: updated.clone(),
                    updated,
                    conversation: entry.to_string(),
                    branch: "-".to_string(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistorySessionItem {
    pub title: String,
    pub created: String,
    pub updated: String,
    pub branch: String,
    pub conversation: String,
}

pub fn default_history_path() -> anyhow::Result<PathBuf> {
    Ok(crate::tui::config::routis_dir()?.join("shell_history"))
}

fn parse_history_line(line: &str) -> Option<(Option<u64>, &str)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let Some((prefix, value)) = trimmed.split_once('\t') else {
        return Some((None, trimmed));
    };
    match prefix.parse::<u64>() {
        Ok(timestamp) => Some((Some(timestamp), value)),
        Err(_) => Some((None, trimmed)),
    }
}

fn now_epoch_seconds() -> u64 {
    system_time_to_epoch_seconds(SystemTime::now()).unwrap_or(0)
}

fn system_time_to_epoch_seconds(value: SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .ok()
}

fn relative_time(timestamp: u64, now: u64) -> String {
    let elapsed = now.saturating_sub(timestamp);
    match elapsed {
        0..=59 => "now".to_string(),
        60..=3_599 => format!("{}m ago", elapsed / 60),
        3_600..=86_399 => format!("{}h ago", elapsed / 3_600),
        _ => format!("{}d ago", elapsed / 86_400),
    }
}
