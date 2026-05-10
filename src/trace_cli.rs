use anyhow::{Context, Result};
use routis_core::{
    DecisionTrace, DecisionTraceInput, PromptMode, RepoFact, RiskZone, RoutingDecision,
};

use crate::trace_store::{append_trace, latest_trace, read_all_trace_summaries};

pub struct CliDecisionTraceInput {
    pub selected_model: String,
    pub selected_reasoning: String,
    pub execution_mode: String,
    pub provider_command: Option<String>,
    pub risk_zones: Vec<RiskZone>,
    pub repo_facts: Vec<RepoFact>,
}

pub fn build_cli_decision_trace(
    task: &str,
    decision: &RoutingDecision,
    input: CliDecisionTraceInput,
) -> DecisionTrace {
    DecisionTrace::from_routing_decision(
        decision,
        DecisionTraceInput {
            session_id: "cli".to_string(),
            task: task.to_string(),
            selected_model: input.selected_model,
            selected_reasoning: input.selected_reasoning,
            prompt_mode: PromptMode::Raw,
            execution_mode: input.execution_mode,
            provider_command: input.provider_command,
            risk_zones: input.risk_zones,
            repo_facts: input.repo_facts,
        },
    )
}

pub fn append_cli_trace(trace: &DecisionTrace) -> Result<()> {
    append_trace(trace).map(|_| ())
}

pub fn print_trace_tree(trace: &DecisionTrace) {
    println!("{}", trace.render_compact_tree());
}

pub fn print_trace_list() -> Result<()> {
    let summaries = read_all_trace_summaries()?;
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
