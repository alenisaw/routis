#![forbid(unsafe_code)]
#![deny(warnings)]

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoutingError {
    #[error("unknown policy `{0}`; expected cheap, balanced, deep, extradeep, or default")]
    UnknownPolicy(String),
    #[error("task must not be empty")]
    EmptyTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    Cheap,
    Balanced,
    Deep,
    ExtraDeep,
    Default,
}

impl Profile {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cheap => "cheap",
            Self::Balanced => "balanced",
            Self::Deep => "deep",
            Self::ExtraDeep => "extradeep",
            Self::Default => "default",
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Profile {
    type Err = RoutingError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "cheap" => Ok(Self::Cheap),
            "balanced" => Ok(Self::Balanced),
            "deep" => Ok(Self::Deep),
            "extradeep" | "extra-deep" | "extra_deep" => Ok(Self::ExtraDeep),
            "default" | "auto" => Ok(Self::Default),
            other => Err(RoutingError::UnknownPolicy(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub requested_profile: Profile,
    pub effective_profile: Profile,
    pub signals_matched: Vec<String>,
    pub explain: String,
}

pub fn route_task(task: &str, requested_profile: Profile) -> Result<RoutingDecision, RoutingError> {
    let task = task.trim();
    if task.is_empty() {
        return Err(RoutingError::EmptyTask);
    }

    let classification = classify_task(task);
    let effective_profile = match requested_profile {
        Profile::Default => classification.profile,
        fixed => fixed,
    };

    let explain = explain_decision(
        requested_profile,
        effective_profile,
        &classification.signals_matched,
    );

    Ok(RoutingDecision {
        requested_profile,
        effective_profile,
        signals_matched: classification.signals_matched,
        explain,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Classification {
    profile: Profile,
    signals_matched: Vec<String>,
}

fn classify_task(task: &str) -> Classification {
    let normalized = task.to_ascii_lowercase();
    let mut matched = Vec::new();
    let mut score: Option<i32> = None;

    apply_signals(
        &normalized,
        &mut matched,
        &mut score,
        &[
            "typo",
            "rename",
            "format",
            "comment",
            "small fix",
            "update readme",
            "docs",
        ],
        0,
    );
    apply_signals(
        &normalized,
        &mut matched,
        &mut score,
        &[
            "implement",
            "update",
            "review",
            "add test",
            "refactor small",
            "feature",
        ],
        1,
    );
    apply_signals(
        &normalized,
        &mut matched,
        &mut score,
        &[
            "debug",
            "investigate",
            "trace",
            "security",
            "migration",
            "edge case",
            "bug",
        ],
        2,
    );
    apply_signals(
        &normalized,
        &mut matched,
        &mut score,
        &[
            "redesign",
            "overhaul",
            "architecture",
            "large refactor",
            "rewrite",
            "rework everything",
        ],
        3,
    );

    let mut resolved_score = score.unwrap_or(1);

    if contains_any(
        &normalized,
        &["quickly", "quick", "just this file", "only this file"],
    ) {
        matched.push("down-modifier".to_string());
        resolved_score -= 1;
    }

    if contains_any(
        &normalized,
        &[
            "carefully",
            "across all files",
            "entire repo",
            "whole repo",
            "all files",
        ],
    ) {
        matched.push("up-modifier".to_string());
        resolved_score += 1;
    }

    let profile = match resolved_score.clamp(0, 3) {
        0 => Profile::Cheap,
        1 => Profile::Balanced,
        2 => Profile::Deep,
        _ => Profile::ExtraDeep,
    };

    if matched.is_empty() {
        matched.push("balanced-baseline".to_string());
    }

    Classification {
        profile,
        signals_matched: matched,
    }
}

fn apply_signals(
    text: &str,
    matched: &mut Vec<String>,
    score: &mut Option<i32>,
    signals: &[&str],
    signal_score: i32,
) {
    for signal in signals {
        if text.contains(signal) {
            matched.push((*signal).to_string());
            *score = Some(score.map_or(signal_score, |current| current.max(signal_score)));
        }
    }
}

fn contains_any(text: &str, signals: &[&str]) -> bool {
    signals.iter().any(|signal| text.contains(signal))
}

fn explain_decision(requested: Profile, effective: Profile, signals: &[String]) -> String {
    if requested == Profile::Default {
        format!(
            "Auto-selected `{}` from signals: {}.",
            effective,
            signals.join(", ")
        )
    } else {
        format!(
            "Using explicit `{}` policy; classifier signals were: {}.",
            effective,
            signals.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheap_signal_routes_to_cheap() {
        let decision = route_task("small fix in README", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::Cheap);
    }

    #[test]
    fn balanced_signal_routes_to_balanced() {
        let decision = route_task("implement a new parser option", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::Balanced);
    }

    #[test]
    fn deep_signal_routes_to_deep() {
        let decision = route_task("debug auth flow", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::Deep);
    }

    #[test]
    fn up_modifier_elevates_deep_to_extradeep() {
        let decision = route_task("debug auth edge case carefully", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::ExtraDeep);
    }

    #[test]
    fn extradeep_signal_routes_to_extradeep() {
        let decision = route_task("redesign the architecture", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::ExtraDeep);
    }

    #[test]
    fn down_modifier_can_lower_routing_profile() {
        let decision = route_task("debug just this file", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::Balanced);
    }

    #[test]
    fn explicit_policy_overrides_classifier() {
        let decision = route_task("redesign the architecture", Profile::Cheap).unwrap();
        assert_eq!(decision.effective_profile, Profile::Cheap);
    }

    #[test]
    fn parses_policy_aliases() {
        assert_eq!("extra-deep".parse::<Profile>().unwrap(), Profile::ExtraDeep);
        assert_eq!("auto".parse::<Profile>().unwrap(), Profile::Default);
    }
}
