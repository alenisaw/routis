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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TraceReadReport {
    pub summaries: Vec<TraceSummary>,
    pub skipped_lines: usize,
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
    create_private_dir(&dir)?;
    let path = trace_file_path(&dir, &trace.session_id);
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&path)
        .with_context(|| format!("failed to open trace file `{}`", path.display()))?;
    let line = serde_json::to_string(trace).context("failed to serialize decision trace")?;
    writeln!(file, "{line}").with_context(|| format!("failed to write `{}`", path.display()))?;
    Ok(path)
}

pub fn read_trace_file_lossy(path: impl AsRef<Path>) -> Result<(Vec<DecisionTrace>, usize)> {
    let path = path.as_ref();
    let file =
        fs::File::open(path).with_context(|| format!("failed to open `{}`", path.display()))?;
    let reader = BufReader::new(file);
    let mut traces = Vec::new();
    let mut skipped = 0;
    for line in reader.lines() {
        let line = line.with_context(|| format!("failed to read `{}`", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<DecisionTrace>(&line) {
            Ok(trace) => traces.push(trace),
            Err(_) => skipped += 1,
        }
    }
    Ok((traces, skipped))
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

pub fn read_trace_summaries(limit: usize) -> Result<TraceReadReport> {
    let mut summaries = Vec::new();
    let mut skipped_lines = 0;
    for path in list_trace_files()? {
        let (traces, skipped) = read_trace_file_lossy(path)?;
        skipped_lines += skipped;
        for trace in traces {
            summaries.push(TraceSummary::from(&trace));
        }
    }
    summaries.sort_by_key(|summary| summary.timestamp_unix_ms);
    if summaries.len() > limit {
        summaries = summaries.split_off(summaries.len() - limit);
    }
    Ok(TraceReadReport {
        summaries,
        skipped_lines,
    })
}

pub fn latest_trace() -> Result<Option<DecisionTrace>> {
    let mut latest = None;
    for path in list_trace_files()? {
        for trace in read_trace_file_lossy(path)?.0 {
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
        .take(80)
        .collect::<String>();
    dir.join(format!("{safe_session}.jsonl"))
}

fn create_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("failed to create `{}`", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("failed to set permissions on `{}`", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_file_path_sanitizes_session_id() {
        let path = trace_file_path(Path::new("traces"), "session/a:b");
        assert_eq!(path, Path::new("traces").join("session_a_b.jsonl"));
    }

    #[test]
    fn lossy_trace_reader_skips_corrupt_lines() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("trace.jsonl");
        std::fs::write(&path, "{bad json}\n\n").unwrap();

        let (traces, skipped) = read_trace_file_lossy(path).unwrap();

        assert!(traces.is_empty());
        assert_eq!(skipped, 1);
    }
}
