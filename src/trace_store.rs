use anyhow::{Context, Result};
use routis_core::DecisionTrace;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use routis::paths::routis_dir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceSummary {
    pub session_id: String,
    pub task_hash: String,
    pub timestamp_unix_ms: u128,
    pub selected_profile: String,
    pub selected_model: String,
    pub selected_reasoning: String,
    pub intent: String,
    pub area: String,
    pub scope: String,
    pub risk: String,
    pub confidence: String,
}

impl From<&DecisionTrace> for TraceSummary {
    fn from(trace: &DecisionTrace) -> Self {
        Self {
            session_id: trace.session_id.clone(),
            task_hash: trace.task_hash.clone(),
            timestamp_unix_ms: trace.timestamp_unix_ms,
            selected_profile: trace.selected_profile.clone(),
            selected_model: trace.selected_model.clone(),
            selected_reasoning: trace.selected_reasoning.clone(),
            intent: trace.intent.clone(),
            area: trace.area.clone(),
            scope: trace.scope.clone(),
            risk: trace.risk.clone(),
            confidence: trace.confidence.clone(),
        }
    }
}

#[must_use]
pub fn traces_dir() -> PathBuf {
    routis_dir().join("traces")
}

pub fn append_trace(trace: &DecisionTrace) -> Result<PathBuf> {
    let dir = traces_dir();
    fs::create_dir_all(&dir).with_context(|| format!("failed to create `{}`", dir.display()))?;
    let path = trace_file_path(&dir, &trace.session_id);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open trace file `{}`", path.display()))?;
    let line = serde_json::to_string(trace).context("failed to serialize decision trace")?;
    writeln!(file, "{line}").with_context(|| format!("failed to write `{}`", path.display()))?;
    Ok(path)
}

pub fn read_trace_file(path: impl AsRef<Path>) -> Result<Vec<DecisionTrace>> {
    let path = path.as_ref();
    let file =
        fs::File::open(path).with_context(|| format!("failed to open `{}`", path.display()))?;
    let reader = BufReader::new(file);
    let mut traces = Vec::new();
    for line in reader.lines() {
        let line = line.with_context(|| format!("failed to read `{}`", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        traces.push(
            serde_json::from_str::<DecisionTrace>(&line)
                .with_context(|| format!("invalid trace JSONL in `{}`", path.display()))?,
        );
    }
    Ok(traces)
}

pub fn list_trace_files() -> Result<Vec<PathBuf>> {
    let dir = traces_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = fs::read_dir(&dir)
        .with_context(|| format!("failed to read `{}`", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("jsonl"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

pub fn read_all_trace_summaries() -> Result<Vec<TraceSummary>> {
    let mut summaries = Vec::new();
    for path in list_trace_files()? {
        for trace in read_trace_file(path)? {
            summaries.push(TraceSummary::from(&trace));
        }
    }
    summaries.sort_by_key(|summary| summary.timestamp_unix_ms);
    Ok(summaries)
}

pub fn latest_trace() -> Result<Option<DecisionTrace>> {
    let mut latest = None;
    for path in list_trace_files()? {
        for trace in read_trace_file(path)? {
            let replace = latest.as_ref().is_none_or(|current: &DecisionTrace| {
                trace.timestamp_unix_ms > current.timestamp_unix_ms
            });
            if replace {
                latest = Some(trace);
            }
        }
    }
    Ok(latest)
}

fn trace_file_path(dir: &Path, session_id: &str) -> PathBuf {
    let safe_session = session_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    dir.join(format!("{safe_session}.jsonl"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_file_path_sanitizes_session_id() {
        let path = trace_file_path(Path::new("traces"), "session/a:b");
        assert_eq!(path, Path::new("traces").join("session_a_b.jsonl"));
    }
}
