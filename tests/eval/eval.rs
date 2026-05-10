use routis_core::{
    route_task, DecisionTrace, DecisionTraceInput, Profile, PromptMode, ProviderCommandPreview,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct RoutingCase {
    task: String,
    expected_profile: String,
    expected_reasoning: String,
    expected_intent: String,
    expected_area: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct RecognitionCase {
    task: String,
    expected_intent: String,
    expected_area: String,
    description: String,
}

#[test]
fn routing_eval_cases_match_expected_profiles() {
    let cases: Vec<RoutingCase> = serde_json::from_str(include_str!("routing_cases.json"))
        .expect("routing eval fixture should be valid JSON");

    for case in cases {
        let decision = route_task(&case.task, Profile::Default)
            .unwrap_or_else(|error| panic!("{}: {error}", case.description));
        assert_eq!(
            decision.effective_profile.as_str(),
            case.expected_profile,
            "{}",
            case.description
        );
        assert_eq!(
            decision.classification.primary_intent.as_str(),
            case.expected_intent,
            "{}",
            case.description
        );
        assert_eq!(
            decision.classification.area.as_str(),
            case.expected_area,
            "{}",
            case.description
        );
        let expected_reasoning = reasoning_for_profile(decision.effective_profile.as_str());
        assert_eq!(
            expected_reasoning, case.expected_reasoning,
            "{}",
            case.description
        );
    }
}

#[test]
fn recognition_eval_cases_match_expected_intent_and_area() {
    let cases: Vec<RecognitionCase> = serde_json::from_str(include_str!("recognition_cases.json"))
        .expect("recognition eval fixture should be valid JSON");

    for case in cases {
        let decision = route_task(&case.task, Profile::Default)
            .unwrap_or_else(|error| panic!("{}: {error}", case.description));

        assert_eq!(
            decision.classification.primary_intent.as_str(),
            case.expected_intent,
            "{}",
            case.description
        );
        assert_eq!(
            decision.classification.area.as_str(),
            case.expected_area,
            "{}",
            case.description
        );
    }
}

#[test]
fn explain_tree_snapshot_fragments_are_stable() {
    let cases = [
        ("docs_cheap.txt", "update README.md installation section"),
        ("auth_deep.txt", "debug auth flow"),
        (
            "architecture_extradeep.txt",
            "review the architecture of the routing engine",
        ),
        (
            "ru_ui_fix.txt",
            "исправь TUI layout и экран с trace панелью",
        ),
        (
            "mixed_release.txt",
            "проверь release notes and changelog for v0.4.0",
        ),
    ];

    for (snapshot, task) in cases {
        let actual = build_eval_trace(task).render_compact_tree();
        let expected = read_snapshot(snapshot);
        for expected_line in expected.lines().filter(|line| !line.trim().is_empty()) {
            assert!(
                actual.contains(expected_line),
                "snapshot fragment `{expected_line}` missing for `{snapshot}`\nactual:\n{actual}"
            );
        }
    }
}

fn build_eval_trace(task: &str) -> DecisionTrace {
    let decision = route_task(task, Profile::Default).expect("eval task should route");
    DecisionTrace::from_routing_decision(
        &decision,
        DecisionTraceInput {
            session_id: "eval-session".to_string(),
            task_hash: "eval-task-hash".to_string(),
            timestamp_unix_ms: Some(1),
            selected_model: "gpt-5.5".to_string(),
            selected_reasoning: reasoning_for_profile(decision.effective_profile.as_str())
                .to_string(),
            prompt_mode: PromptMode::PreviewOnly,
            execution_mode: "eval".to_string(),
            policy_source: "eval fixture".to_string(),
            policy_overrides: Vec::new(),
            provider_command_preview: Some(ProviderCommandPreview {
                program: "codex".to_string(),
                args: vec![
                    "exec".to_string(),
                    "-m".to_string(),
                    "gpt-5.5".to_string(),
                    "--reasoning".to_string(),
                    reasoning_for_profile(decision.effective_profile.as_str()).to_string(),
                    "--".to_string(),
                    "<task-redacted>".to_string(),
                ],
            }),
            risk_zones: Vec::new(),
            repo_facts: Vec::new(),
        },
    )
}

fn read_snapshot(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("eval")
        .join("explain_tree_snapshots")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read `{}`: {error}", path.display()))
}

fn reasoning_for_profile(profile: &str) -> &'static str {
    match profile {
        "cheap" => "low",
        "balanced" => "medium",
        "deep" => "high",
        "extradeep" => "xhigh",
        _ => "medium",
    }
}
