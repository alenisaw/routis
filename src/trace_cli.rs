use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use routis_core::{
    DecisionTrace, DecisionTraceInput, PromptMode, ProviderCommandPreview, RepoFact, RiskZone,
    RoutingDecision,
};
use sha2::Sha256;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::trace_store::{append_trace, latest_trace, read_trace_summaries};
use routis::paths::routis_dir;

type HmacSha256 = Hmac<Sha256>;

pub struct CliDecisionTraceInput {
    pub selected_model: String,
    pub selected_reasoning: String,
    pub execution_mode: String,
    pub provider_command_preview: Option<ProviderCommandPreview>,
    pub policy_source: String,
    pub policy_overrides: Vec<String>,
    pub risk_zones: Vec<RiskZone>,
    pub repo_facts: Vec<RepoFact>,
}

pub fn build_cli_decision_trace(
    task: &str,
    decision: &RoutingDecision,
    input: CliDecisionTraceInput,
) -> Result<DecisionTrace> {
    let secret = load_or_create_trace_secret()?;
    let task_hash = task_hmac_hex(task, &secret)?;
    Ok(DecisionTrace::from_routing_decision(
        decision,
        DecisionTraceInput {
            session_id: new_cli_session_id(),
            task_hash,
            selected_model: input.selected_model,
            selected_reasoning: input.selected_reasoning,
            prompt_mode: PromptMode::Raw,
            execution_mode: input.execution_mode,
            policy_source: sanitize_trace_value(&input.policy_source, task),
            policy_overrides: input
                .policy_overrides
                .iter()
                .map(|value| sanitize_trace_value(value, task))
                .collect(),
            provider_command_preview: input
                .provider_command_preview
                .map(|preview| sanitize_provider_command_preview(preview, task)),
            risk_zones: input.risk_zones,
            repo_facts: input
                .repo_facts
                .into_iter()
                .map(|fact| RepoFact::new(fact.key, sanitize_trace_value(&fact.value, task)))
                .collect(),
        },
    ))
}

pub fn append_cli_trace(trace: &DecisionTrace) -> Result<()> {
    append_trace(trace).map(|_| ())
}

pub fn print_trace_tree(trace: &DecisionTrace) {
    println!("{}", trace.render_compact_tree());
}

pub fn print_trace_list() -> Result<()> {
    let report = read_trace_summaries(30)?;
    let summaries = report.summaries;
    if summaries.is_empty() {
        println!("No decision traces found.");
        return Ok(());
    }

    println!(
        "{:<18} {:<16} {:<10} {:<10} {:<10} {:<8} {:<8}",
        "timestamp", "task", "profile", "intent", "area", "risk", "conf"
    );
    for item in summaries.iter().rev().take(30) {
        println!(
            "{:<18} {:<16} {:<10} {:<10} {:<10} {:<8} {:<8}",
            item.timestamp_unix_ms,
            shorten(&item.task_hash, 16),
            item.selected_profile,
            item.intent,
            item.area,
            item.risk,
            item.confidence
        );
    }
    if report.skipped_lines > 0 {
        println!("skipped corrupt trace lines: {}", report.skipped_lines);
    }
    Ok(())
}

pub fn print_latest_trace() -> Result<()> {
    let trace = latest_trace()?.context("no decision traces found")?;
    print_trace_tree(&trace);
    Ok(())
}

fn shorten(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        value.to_string()
    } else {
        value.chars().take(max_len).collect()
    }
}

pub fn task_hmac_hex(task: &str, secret: &[u8]) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret).context("invalid trace secret")?;
    mac.update(task.as_bytes());
    Ok(hex(mac.finalize().into_bytes().as_slice()))
}

fn load_or_create_trace_secret() -> Result<Vec<u8>> {
    let path = routis_dir().join("secret");
    if path.exists() {
        return fs::read(&path).with_context(|| format!("failed to read `{}`", path.display()));
    }
    if let Some(parent) = path.parent() {
        create_private_dir(parent)?;
    }
    let mut secret = [0_u8; 32];
    getrandom::getrandom(&mut secret)
        .map_err(|error| anyhow::anyhow!("failed to generate trace secret: {error}"))?;
    write_private_file(&path, &secret)?;
    Ok(secret.to_vec())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .with_context(|| format!("failed to create `{}`", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    Ok(())
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

fn new_cli_session_id() -> String {
    let mut random = [0_u8; 6];
    let _ = getrandom::getrandom(&mut random);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    format!("cli-{now}-{}", hex(&random))
}

fn sanitize_provider_command_preview(
    preview: ProviderCommandPreview,
    raw_task: &str,
) -> ProviderCommandPreview {
    ProviderCommandPreview {
        program: sanitize_trace_value(&preview.program, raw_task),
        args: preview
            .args
            .iter()
            .map(|value| sanitize_trace_value(value, raw_task))
            .collect(),
    }
}

pub fn sanitize_trace_value(value: &str, raw_task: &str) -> String {
    let mut sanitized = value.replace(raw_task, "<task-redacted>");
    for marker in [
        "OPENAI_API_KEY=",
        "ANTHROPIC_API_KEY=",
        "Authorization: Bearer ",
        "Bearer ",
    ] {
        if let Some(index) = sanitized.find(marker) {
            let end = sanitized[index..]
                .find(char::is_whitespace)
                .map_or(sanitized.len(), |offset| index + offset);
            sanitized.replace_range(index..end, "<secret-redacted>");
        }
    }
    sanitized = sanitized.replace(".env", "<env-file>");
    if sanitized.len() > 160 {
        sanitized.truncate(157);
        sanitized.push_str("...");
    }
    sanitized
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use routis_core::{route_task, Profile};

    #[test]
    fn hmac_hash_is_stable_for_same_secret_and_differs_for_tasks() {
        let secret = [7_u8; 32];

        let left = task_hmac_hex("debug auth flow", &secret).unwrap();
        let right = task_hmac_hex("debug auth flow", &secret).unwrap();
        let other = task_hmac_hex("fix README", &secret).unwrap();

        assert_eq!(left, right);
        assert_ne!(left, other);
        assert_eq!(left.len(), 64);
    }

    #[test]
    fn trace_json_does_not_contain_raw_task_or_fake_secrets() {
        let home = tempfile::TempDir::new().unwrap();
        std::env::set_var("ROUTIS_HOME", home.path());
        let task = "debug auth flow OPENAI_API_KEY=sk-test";
        let decision = route_task(task, Profile::Default).unwrap();
        let trace = build_cli_decision_trace(
            task,
            &decision,
            CliDecisionTraceInput {
                selected_model: "gpt-5.5".to_string(),
                selected_reasoning: "high".to_string(),
                execution_mode: "preview".to_string(),
                provider_command_preview: Some(ProviderCommandPreview {
                    program: "codex".to_string(),
                    args: vec![
                        "exec".to_string(),
                        "--".to_string(),
                        task.to_string(),
                        "Authorization: Bearer abc".to_string(),
                        ".env".to_string(),
                    ],
                }),
                policy_source: "OPENAI_API_KEY=sk-test".to_string(),
                policy_overrides: Vec::new(),
                risk_zones: Vec::new(),
                repo_facts: vec![RepoFact::new("task", task)],
            },
        )
        .unwrap();
        let json = serde_json::to_string(&trace).unwrap();
        std::env::remove_var("ROUTIS_HOME");

        assert!(!json.contains(task));
        assert!(!json.contains("sk-test"));
        assert!(!json.contains("Bearer abc"));
        assert!(!json.contains(".env"));
        assert!(json.contains("<task-redacted>"));
    }
}
