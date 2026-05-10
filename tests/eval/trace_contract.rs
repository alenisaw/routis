use routis::trace_cli::{build_cli_decision_trace, CliDecisionTraceInput};
use routis_core::{ProviderCommandPreview, RepoFact};

fn isolate_routis_home() {
    let home = std::env::temp_dir().join(format!(
        "routis-trace-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&home).unwrap();
    std::env::set_var("ROUTIS_HOME", home);
}

#[test]
fn decision_trace_tree_contains_required_sections() {
    isolate_routis_home();
    let task = "fix TUI layout without changing README";
    let route = routis::route_plan::build_execution_plan_with_decision(
        task,
        routis::route_plan::DEFAULT_POLICY_PATH,
        std::env::current_dir().unwrap(),
    )
    .unwrap();
    let trace = build_cli_decision_trace(
        task,
        &route.decision,
        CliDecisionTraceInput {
            selected_model: route.plan.model.clone(),
            selected_reasoning: route.plan.reasoning.clone(),
            execution_mode: "preview".to_string(),
            provider_command_preview: Some(ProviderCommandPreview {
                program: "codex".to_string(),
                args: vec!["exec".to_string(), "<task-redacted>".to_string()],
            }),
            policy_source: route.plan.policy_source.clone(),
            policy_overrides: route.policy_overrides,
            risk_zones: route.repo_context.risk_zone_hints,
            repo_facts: vec![RepoFact::new("policy-source", route.plan.policy_source)],
        },
    )
    .unwrap();

    let tree = trace.render_compact_tree();
    assert!(tree.contains("Input Analysis"));
    assert!(tree.contains("Repo Context"));
    assert!(tree.contains("Route Decision"));
}

#[test]
fn decision_trace_json_does_not_store_raw_task_text() {
    isolate_routis_home();
    let task = "fix auth flow with OPENAI_API_KEY=abc123 but never store this raw task";
    let route = routis::route_plan::build_execution_plan_with_decision(
        task,
        routis::route_plan::DEFAULT_POLICY_PATH,
        std::env::current_dir().unwrap(),
    )
    .unwrap();
    let trace = build_cli_decision_trace(
        task,
        &route.decision,
        CliDecisionTraceInput {
            selected_model: route.plan.model.clone(),
            selected_reasoning: route.plan.reasoning.clone(),
            execution_mode: "preview".to_string(),
            provider_command_preview: Some(ProviderCommandPreview {
                program: "codex".to_string(),
                args: vec!["exec".to_string(), task.to_string()],
            }),
            policy_source: route.plan.policy_source.clone(),
            policy_overrides: route.policy_overrides,
            risk_zones: route.repo_context.risk_zone_hints,
            repo_facts: vec![RepoFact::new("policy-source", route.plan.policy_source)],
        },
    )
    .unwrap();

    let json = serde_json::to_string(&trace).unwrap();
    assert!(!json.contains(task));
    assert!(!json.contains("OPENAI_API_KEY=abc123"));
    assert!(json.contains("<task-redacted>") || json.contains("<secret-redacted>"));
}
