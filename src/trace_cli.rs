use anyhow::{Context, Result};
use hmac::{Hmac, KeyInit, Mac};
use routis_core::{
    DecisionTrace, DecisionTraceInput, PromptMode, ProviderCommandPreview, RepoFact, RiskZone,
    RoutingDecision,
};
use sha2::Sha256;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::trace_store::{append_trace, latest_trace, read_trace_summaries};
use routis::paths::routis_dir;
use routis::private_fs;

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
            session_id: new_cli_session_id()?,
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
    let path = routis_dir()?.join("secret");
    if path.exists() {
        let secret =
            fs::read(&path).with_context(|| format!("failed to read `{}`", path.display()))?;
        if secret.len() != 32 {
            anyhow::bail!(
                "invalid trace secret length in `{}`; expected 32 bytes",
                path.display()
            );
        }
        return Ok(secret);
    }
    if let Some(parent) = path.parent() {
        private_fs::create_private_dir(parent)?;
    }
    let mut secret = [0_u8; 32];
    getrandom::fill(&mut secret)
        .map_err(|error| anyhow::anyhow!("failed to generate trace secret: {error}"))?;
    private_fs::write_private_file(&path, &secret)?;
    Ok(secret.to_vec())
}

fn new_cli_session_id() -> Result<String> {
    let mut random = [0_u8; 6];
    getrandom::fill(&mut random)
        .map_err(|error| anyhow::anyhow!("failed to generate CLI session id: {error}"))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    Ok(format!("cli-{now}-{}", hex(&random)))
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
    for marker in ["OPENAI_API_KEY=", "ANTHROPIC_API_KEY=", "x-api-key="] {
        if let Some(index) = sanitized.find(marker) {
            let end = sanitized[index..]
                .find(char::is_whitespace)
                .map_or(sanitized.len(), |offset| index + offset);
            sanitized.replace_range(index..end, "<secret-redacted>");
        }
    }
    for marker in ["Authorization: Bearer ", "Bearer "] {
        if let Some(index) = sanitized.find(marker) {
            let end = sanitized[index..]
                .find(char::is_whitespace)
                .map_or(sanitized.len(), |offset| index + offset);
            sanitized.replace_range(index..end, "<secret-redacted>");
        }
    }
    sanitized = sanitized
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if part.starts_with("sk-")
                || part.starts_with("ghp_")
                || part.starts_with("github_pat_")
                || lower.contains(".env")
                || looks_like_jwt(part)
            {
                "<secret-redacted>"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if sanitized.contains("-----BEGIN") && sanitized.contains("PRIVATE KEY-----") {
        sanitized = "<secret-redacted>".to_string();
    }
    if sanitized.len() > 160 {
        sanitized.truncate(157);
        sanitized.push_str("...");
    }
    sanitized
}

fn looks_like_jwt(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3 && parts.iter().all(|part| part.len() >= 8)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use routis_core::{route_task, Profile};
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

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
        let _guard = env_lock();
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

    #[test]
    fn sanitizer_redacts_common_secret_shapes() {
        let raw = "fix issue";
        let samples = [
            "OPENAI_API_KEY=sk-test",
            "ANTHROPIC_API_KEY=secret",
            "Authorization: Bearer abc",
            "x-api-key=abc",
            "sk-live-test",
            "ghp_abcdefghijklmnop",
            "github_pat_abcdefghijklmnop",
            ".env.local",
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signaturexx",
            "-----BEGIN PRIVATE KEY----- secret -----END PRIVATE KEY-----",
        ];

        for sample in samples {
            let sanitized = sanitize_trace_value(sample, raw);
            assert!(!sanitized.contains("sk-test"));
            assert!(!sanitized.contains(" secret "));
            assert!(!sanitized.contains("Bearer abc"));
            assert!(!sanitized.contains(".env"));
            assert!(!sanitized.contains("ghp_"));
            assert!(!sanitized.contains("github_pat_"));
            assert!(!sanitized.contains("PRIVATE KEY"));
        }
    }

    #[test]
    fn invalid_trace_secret_length_fails() {
        let _guard = env_lock();
        let home = tempfile::TempDir::new().unwrap();
        std::env::set_var("ROUTIS_HOME", home.path());
        std::fs::write(home.path().join("secret"), b"short").unwrap();

        let error = load_or_create_trace_secret().unwrap_err().to_string();

        std::env::remove_var("ROUTIS_HOME");
        assert!(error.contains("invalid trace secret length"));
    }
}
