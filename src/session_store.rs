#![forbid(unsafe_code)]
#![deny(warnings)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub task: String,
    pub branch: String,
    pub policy: String,
    pub effective_profile: String,
    pub model: String,
    pub reasoning: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub routing_count: usize,
}

impl SessionRecord {
    #[must_use]
    pub fn new(
        task: &str,
        branch: &str,
        policy: &str,
        effective_profile: &str,
        model: &str,
        reasoning: &str,
    ) -> Self {
        let now = now_epoch_seconds();
        Self {
            schema_version: 1,
            id: format!("{now}-{}", slug(task)),
            title: slug(task),
            task: task.to_string(),
            branch: branch.to_string(),
            policy: policy.to_string(),
            effective_profile: effective_profile.to_string(),
            model: model.to_string(),
            reasoning: reasoning.to_string(),
            created_at: now,
            updated_at: now,
            routing_count: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionStore {
    path: PathBuf,
}

impl SessionStore {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn save(&self, record: &SessionRecord) -> Result<()> {
        fs::create_dir_all(&self.path)
            .with_context(|| format!("failed to create `{}`", self.path.display()))?;
        fs::write(
            self.path.join(format!("{}.json", record.id)),
            serde_json::to_string_pretty(record)?,
        )
        .with_context(|| format!("failed to write session `{}`", record.id))
    }

    pub fn list(&self) -> Result<Vec<SessionRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let mut records = Vec::new();
        for entry in fs::read_dir(&self.path)
            .with_context(|| format!("failed to read `{}`", self.path.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let extension = path.extension().and_then(|value| value.to_str());
            if !matches!(extension, Some("json" | "session")) {
                continue;
            }
            let raw = fs::read_to_string(path)?;
            if let Some(record) = deserialize_record(&raw) {
                records.push(record);
            }
        }
        records.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| b.id.cmp(&a.id))
        });
        Ok(records)
    }

    pub fn find(&self, query: &str) -> Result<Option<SessionRecord>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(None);
        }
        Ok(self
            .list()?
            .into_iter()
            .find(|record| record.id.starts_with(query) || record.title == query))
    }
}

#[must_use]
pub fn default_session_store_path() -> PathBuf {
    crate::tui::config::routis_dir().join("sessions")
}

fn deserialize_record(raw: &str) -> Option<SessionRecord> {
    serde_json::from_str(raw)
        .ok()
        .or_else(|| deserialize_legacy(raw))
}

fn deserialize_legacy(raw: &str) -> Option<SessionRecord> {
    let value = |key: &str| {
        raw.lines()
            .filter_map(|line| line.split_once('='))
            .find_map(|(current, value)| (current == key).then_some(value))
    };
    Some(SessionRecord {
        schema_version: value("schema_version")?.parse().ok()?,
        id: value("id")?.to_string(),
        title: value("title")?.to_string(),
        task: unescape(value("task")?),
        branch: unescape(value("branch")?),
        policy: value("policy")?.to_string(),
        effective_profile: value("effective_profile")?.to_string(),
        model: value("model")?.to_string(),
        reasoning: value("reasoning")?.to_string(),
        created_at: value("created_at")?.parse().ok()?,
        updated_at: value("updated_at")?.parse().ok()?,
        routing_count: value("routing_count")?.parse().ok()?,
    })
}

fn slug(value: &str) -> String {
    let slug = value
        .split_whitespace()
        .take(4)
        .map(|part| {
            part.chars()
                .filter(|ch| ch.is_alphanumeric() || *ch == '-')
                .collect::<String>()
                .to_ascii_lowercase()
        })
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "new-session".to_string()
    } else {
        slug
    }
}

fn unescape(value: &str) -> String {
    value.replace("\\n", "\n").replace("\\\\", "\\")
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};
    use tempfile::TempDir;

    #[test]
    fn session_store_saves_and_lists_records_newest_first() {
        let dir = TempDir::new().unwrap();
        let store = SessionStore::new(dir.path().join("sessions"));
        let first = SessionRecord::new(
            "first task",
            "main",
            "default",
            "balanced",
            "gpt-5.5",
            "medium",
        );
        thread::sleep(Duration::from_secs(1));
        let second = SessionRecord::new(
            "debug auth flow",
            "feature/auth",
            "default",
            "deep",
            "gpt-5.5",
            "high",
        );

        store.save(&first).unwrap();
        store.save(&second).unwrap();

        let sessions = store.list().unwrap();

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].title, "debug-auth-flow");
        assert_eq!(sessions[0].branch, "feature/auth");
        assert_eq!(sessions[0].effective_profile, "deep");
        assert_eq!(sessions[1].title, "first-task");
    }

    #[test]
    fn session_store_finds_record_by_id_prefix_or_title() {
        let dir = TempDir::new().unwrap();
        let store = SessionStore::new(dir.path().join("sessions"));
        let record = SessionRecord::new(
            "debug auth flow",
            "feature/auth",
            "default",
            "deep",
            "gpt-5.5",
            "high",
        );
        let id_prefix = record.id.chars().take(8).collect::<String>();
        store.save(&record).unwrap();

        assert_eq!(
            store.find("debug-auth-flow").unwrap().unwrap().id,
            record.id
        );
        assert_eq!(store.find(&id_prefix).unwrap().unwrap().title, record.title);
        assert!(store.find("missing").unwrap().is_none());
    }

    #[test]
    fn session_store_preserves_literal_backslash_n() {
        let dir = TempDir::new().unwrap();
        let store = SessionStore::new(dir.path().join("sessions"));
        let record = SessionRecord::new(
            r"keep literal \n in task",
            "main",
            "default",
            "balanced",
            "gpt-5.5",
            "medium",
        );

        store.save(&record).unwrap();

        let stored = store.find(&record.title).unwrap().unwrap();
        assert_eq!(stored.task, r"keep literal \n in task");
    }
}
