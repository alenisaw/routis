use routis_core::{route_task, Profile};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RoutingCase {
    task: String,
    expected_profile: String,
    expected_reasoning: String,
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

fn reasoning_for_profile(profile: &str) -> &'static str {
    match profile {
        "cheap" => "low",
        "balanced" => "medium",
        "deep" => "high",
        "extradeep" => "xhigh",
        _ => "medium",
    }
}
