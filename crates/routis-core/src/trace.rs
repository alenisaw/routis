use crate::{RiskZone, RoutingDecision};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DECISION_TRACE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromptMode {
    Raw,
    Compiled,
    PreviewOnly,
}

impl PromptMode {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Compiled => "compiled",
            Self::PreviewOnly => "preview-only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchedSignal {
    pub kind: String,
    pub value: String,
    pub weight: i32,
}

impl MatchedSignal {
    #[must_use]
    pub fn new(kind: impl Into<String>, value: impl Into<String>, weight: i32) -> Self {
        Self {
            kind: kind.into(),
            value: value.into(),
            weight,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoFact {
    pub key: String,
    pub value: String,
}

impl RepoFact {
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteTreeNode {
    pub label: String,
    pub value: Option<String>,
    pub children: Vec<RouteTreeNode>,
}

impl RouteTreeNode {
    #[must_use]
    pub fn leaf(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: Some(value.into()),
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn branch(label: impl Into<String>, children: Vec<RouteTreeNode>) -> Self {
        Self {
            label: label.into(),
            value: None,
            children,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteTree {
    pub root: RouteTreeNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionTrace {
    pub schema_version: u32,
    pub session_id: String,
    pub task_hash: String,
    pub timestamp_unix_ms: u128,
    pub language: String,
    pub intent: String,
    pub area: String,
    pub scope: String,
    pub risk: String,
    pub confidence: String,
    pub matched_signals: Vec<MatchedSignal>,
    pub risk_zones: Vec<String>,
    pub repo_facts: Vec<RepoFact>,
    pub requested_profile: String,
    pub selected_profile: String,
    pub selected_model: String,
    pub selected_reasoning: String,
    pub prompt_mode: PromptMode,
    pub execution_mode: String,
    pub provider_command: Option<String>,
    pub route_tree: RouteTree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionTraceInput {
    pub session_id: String,
    pub task: String,
    pub selected_model: String,
    pub selected_reasoning: String,
    pub prompt_mode: PromptMode,
    pub execution_mode: String,
    pub provider_command: Option<String>,
    pub risk_zones: Vec<RiskZone>,
    pub repo_facts: Vec<RepoFact>,
}

impl DecisionTrace {
    #[must_use]
    pub fn from_routing_decision(decision: &RoutingDecision, input: DecisionTraceInput) -> Self {
        let classification = &decision.classification;
        let matched_signals = decision
            .signals_matched
            .iter()
            .map(|signal| signal_to_matched_signal(signal))
            .collect::<Vec<_>>();
        let risk_zones = input
            .risk_zones
            .iter()
            .map(|zone| zone.as_str().to_string())
            .collect::<Vec<_>>();
        let route_tree = build_route_tree(
            decision,
            &matched_signals,
            &risk_zones,
            &input.repo_facts,
            &input.selected_model,
            &input.selected_reasoning,
        );

        Self {
            schema_version: DECISION_TRACE_SCHEMA_VERSION,
            session_id: input.session_id,
            task_hash: fnv1a64_hex(&input.task),
            timestamp_unix_ms: unix_timestamp_ms(),
            language: format!("{:?}", classification.language).to_ascii_lowercase(),
            intent: classification.primary_intent.as_str().to_string(),
            area: classification.area.as_str().to_string(),
            scope: classification.scope.as_str().to_string(),
            risk: classification.risk.as_str().to_string(),
            confidence: classification.confidence.as_str().to_string(),
            matched_signals,
            risk_zones,
            repo_facts: input.repo_facts,
            requested_profile: decision.requested_profile.as_str().to_string(),
            selected_profile: decision.effective_profile.as_str().to_string(),
            selected_model: input.selected_model,
            selected_reasoning: input.selected_reasoning,
            prompt_mode: input.prompt_mode,
            execution_mode: input.execution_mode,
            provider_command: input.provider_command,
            route_tree,
        }
    }

    #[must_use]
    pub fn render_compact_tree(&self) -> String {
        render_tree(&self.route_tree.root)
    }
}

#[must_use]
pub fn fnv1a64_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn unix_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

fn signal_to_matched_signal(signal: &str) -> MatchedSignal {
    if let Some((kind, value)) = signal.split_once(':') {
        MatchedSignal::new(kind, value, 1)
    } else {
        MatchedSignal::new("signal", signal, 1)
    }
}

fn build_route_tree(
    decision: &RoutingDecision,
    matched_signals: &[MatchedSignal],
    risk_zones: &[String],
    repo_facts: &[RepoFact],
    selected_model: &str,
    selected_reasoning: &str,
) -> RouteTree {
    let classification = &decision.classification;
    let signal_nodes = matched_signals
        .iter()
        .map(|signal| RouteTreeNode::leaf(&signal.kind, &signal.value))
        .collect::<Vec<_>>();
    let risk_nodes = risk_zones
        .iter()
        .map(|zone| RouteTreeNode::leaf("risk-zone", zone))
        .collect::<Vec<_>>();
    let repo_nodes = repo_facts
        .iter()
        .map(|fact| RouteTreeNode::leaf(&fact.key, &fact.value))
        .collect::<Vec<_>>();

    RouteTree {
        root: RouteTreeNode::branch(
            "Routis Decision Trace",
            vec![
                RouteTreeNode::branch(
                    "Input Analysis",
                    vec![
                        RouteTreeNode::leaf(
                            "Language",
                            format!("{:?}", classification.language).to_ascii_lowercase(),
                        ),
                        RouteTreeNode::leaf("Intent", classification.primary_intent.as_str()),
                        RouteTreeNode::leaf("Area", classification.area.as_str()),
                        RouteTreeNode::leaf("Scope", classification.scope.as_str()),
                        RouteTreeNode::leaf("Risk", classification.risk.as_str()),
                        RouteTreeNode::leaf("Confidence", classification.confidence.as_str()),
                    ],
                ),
                RouteTreeNode::branch("Matched Signals", signal_nodes),
                RouteTreeNode::branch("Repo Context", [risk_nodes, repo_nodes].concat()),
                RouteTreeNode::branch(
                    "Route Decision",
                    vec![
                        RouteTreeNode::leaf(
                            "Requested profile",
                            decision.requested_profile.as_str(),
                        ),
                        RouteTreeNode::leaf(
                            "Selected profile",
                            decision.effective_profile.as_str(),
                        ),
                        RouteTreeNode::leaf("Model", selected_model),
                        RouteTreeNode::leaf("Reasoning", selected_reasoning),
                    ],
                ),
            ],
        ),
    }
}

fn render_tree(root: &RouteTreeNode) -> String {
    let mut output = String::new();
    output.push_str(&format_node(root));
    output.push('\n');
    for (index, child) in root.children.iter().enumerate() {
        render_child(child, "", index + 1 == root.children.len(), &mut output);
    }
    output
}

fn render_child(node: &RouteTreeNode, prefix: &str, is_last: bool, output: &mut String) {
    let connector = if is_last { "└─" } else { "├─" };
    output.push_str(prefix);
    output.push_str(connector);
    output.push(' ');
    output.push_str(&format_node(node));
    output.push('\n');

    let child_prefix = if is_last { "   " } else { "│  " };
    let next_prefix = format!("{prefix}{child_prefix}");
    for (index, child) in node.children.iter().enumerate() {
        render_child(
            child,
            &next_prefix,
            index + 1 == node.children.len(),
            output,
        );
    }
}

fn format_node(node: &RouteTreeNode) -> String {
    match &node.value {
        Some(value) => format!("{}: {}", node.label, value),
        None => node.label.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_hash_is_stable() {
        assert_eq!(
            fnv1a64_hex("debug auth flow"),
            fnv1a64_hex("debug auth flow")
        );
        assert_ne!(fnv1a64_hex("debug auth flow"), fnv1a64_hex("fix readme"));
    }

    #[test]
    fn matched_signal_parser_keeps_kind_and_value() {
        let signal = signal_to_matched_signal("risk-zone:auth");
        assert_eq!(signal.kind, "risk-zone");
        assert_eq!(signal.value, "auth");
    }
}
