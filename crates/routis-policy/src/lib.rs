#![forbid(unsafe_code)]
#![deny(warnings)]

use routis_core::{Profile, RiskZone};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("failed to read policy file `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse policy YAML `{path}`: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("unsupported policy version `{0}`; expected version 1")]
    UnsupportedVersion(u32),
    #[error("missing execution config for profile `{0}`")]
    MissingProfile(Profile),
    #[error("profile `{profile}` has an empty `{field}` value")]
    EmptyField {
        profile: String,
        field: &'static str,
    },
    #[error("policy rule #{index} must define `if_risk_zone` or `if_path`")]
    EmptyRuleMatcher { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyFile {
    pub version: u32,
    pub profiles: BTreeMap<String, ProfileExecutionConfig>,
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileExecutionConfig {
    pub model: String,
    pub reasoning: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRule {
    pub if_risk_zone: Option<RiskZone>,
    pub if_path: Option<String>,
    pub min_profile: Option<Profile>,
    pub max_profile: Option<Profile>,
}

impl PolicyFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, PolicyError> {
        let path = path.as_ref();
        let display_path = path.display().to_string();
        let raw = std::fs::read_to_string(path).map_err(|source| PolicyError::Read {
            path: display_path.clone(),
            source,
        })?;
        Self::parse_yaml(&raw, &display_path)
    }

    pub fn parse_yaml(raw: &str, source_name: &str) -> Result<Self, PolicyError> {
        let policy = serde_yaml::from_str::<Self>(raw).map_err(|source| PolicyError::Parse {
            path: source_name.to_string(),
            source,
        })?;
        policy.validate()?;
        Ok(policy)
    }

    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.version != 1 {
            return Err(PolicyError::UnsupportedVersion(self.version));
        }

        for (index, rule) in self.rules.iter().enumerate() {
            if rule.if_risk_zone.is_none() && rule.if_path.is_none() {
                return Err(PolicyError::EmptyRuleMatcher { index: index + 1 });
            }
        }

        for profile in [
            Profile::Cheap,
            Profile::Balanced,
            Profile::Deep,
            Profile::ExtraDeep,
        ] {
            let config = self
                .execution_config(profile)
                .ok_or(PolicyError::MissingProfile(profile))?;
            let key = profile.as_str().to_string();

            if config.model.trim().is_empty() {
                return Err(PolicyError::EmptyField {
                    profile: key,
                    field: "model",
                });
            }

            if config.reasoning.trim().is_empty() {
                return Err(PolicyError::EmptyField {
                    profile: key,
                    field: "reasoning",
                });
            }
        }

        Ok(())
    }

    pub fn execution_config(&self, profile: Profile) -> Option<&ProfileExecutionConfig> {
        self.profiles.get(profile.as_str())
    }
}

pub fn build_codex_command(
    policy: &PolicyFile,
    profile: Profile,
    task: &str,
) -> Result<Vec<String>, PolicyError> {
    let execution = policy
        .execution_config(profile)
        .ok_or(PolicyError::MissingProfile(profile))?;

    Ok(vec![
        "codex".to_string(),
        "exec".to_string(),
        "-m".to_string(),
        execution.model.clone(),
        "--reasoning".to_string(),
        execution.reasoning.clone(),
        "--".to_string(),
        task.to_string(),
    ])
}

#[must_use]
pub fn apply_policy_rules(
    policy: &PolicyFile,
    profile: Profile,
    risk_zones: &[RiskZone],
    changed_files: &[std::path::PathBuf],
    target_hints: &[std::path::PathBuf],
) -> Profile {
    policy.rules.iter().fold(profile, |current, rule| {
        if !policy_rule_matches(rule, risk_zones, changed_files, target_hints) {
            return current;
        }
        let with_min = rule
            .min_profile
            .map_or(current, |min_profile| max_profile(current, min_profile));
        rule.max_profile
            .map_or(with_min, |max_profile| min_profile(with_min, max_profile))
    })
}

fn policy_rule_matches(
    rule: &PolicyRule,
    risk_zones: &[RiskZone],
    changed_files: &[std::path::PathBuf],
    target_hints: &[std::path::PathBuf],
) -> bool {
    let risk_matches = rule
        .if_risk_zone
        .is_none_or(|zone| risk_zones.contains(&zone));
    let path_matches = rule.if_path.as_ref().is_none_or(|pattern| {
        let pattern = normalize_pattern(pattern);
        let paths = changed_files
            .iter()
            .chain(target_hints.iter())
            .collect::<Vec<_>>();
        if rule.max_profile.is_some() && rule.min_profile.is_none() {
            !paths.is_empty()
                && paths
                    .iter()
                    .all(|path| normalize_path(path).contains(&pattern))
        } else {
            paths
                .iter()
                .any(|path| normalize_path(path).contains(&pattern))
        }
    });
    risk_matches && path_matches
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn normalize_pattern(pattern: &str) -> String {
    pattern.replace('\\', "/").to_ascii_lowercase()
}

fn max_profile(left: Profile, right: Profile) -> Profile {
    if profile_rank(left) >= profile_rank(right) {
        left
    } else {
        right
    }
}

fn min_profile(left: Profile, right: Profile) -> Profile {
    if profile_rank(left) <= profile_rank(right) {
        left
    } else {
        right
    }
}

fn profile_rank(profile: Profile) -> u8 {
    match profile {
        Profile::Cheap => 0,
        Profile::Balanced | Profile::Default => 1,
        Profile::Deep => 2,
        Profile::ExtraDeep => 3,
    }
}

#[must_use]
pub fn format_command(command: &[String]) -> String {
    command
        .iter()
        .map(|part| {
            if part.contains(' ') || part.contains('"') {
                format!("\"{}\"", part.replace('"', "\\\""))
            } else {
                part.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: &str = r#"
version: 1
profiles:
  cheap:
    model: gpt-5.4-mini
    reasoning: low
  balanced:
    model: gpt-5.5
    reasoning: medium
  deep:
    model: gpt-5.5
    reasoning: high
  extradeep:
    model: gpt-5.5
    reasoning: xhigh
"#;

    #[test]
    fn parses_policy_yaml() {
        let policy = PolicyFile::parse_yaml(POLICY, "test.yaml").unwrap();
        let deep = policy.execution_config(Profile::Deep).unwrap();
        assert_eq!(deep.model, "gpt-5.5");
        assert_eq!(deep.reasoning, "high");
    }

    #[test]
    fn builds_command_from_policy() {
        let policy = PolicyFile::parse_yaml(POLICY, "test.yaml").unwrap();
        let command = build_codex_command(&policy, Profile::Deep, "debug auth flow").unwrap();
        assert_eq!(
            format_command(&command),
            "codex exec -m gpt-5.5 --reasoning high -- \"debug auth flow\""
        );
    }

    #[test]
    fn parses_optional_routing_rules() {
        let raw = format!(
            r#"
{POLICY}
rules:
  - if_risk_zone: auth
    min_profile: deep
  - if_path: README.md
    max_profile: cheap
"#
        );

        let policy = PolicyFile::parse_yaml(&raw, "test.yaml").unwrap();

        assert_eq!(policy.rules.len(), 2);
        assert_eq!(policy.rules[0].if_risk_zone, Some(RiskZone::Auth));
        assert_eq!(policy.rules[0].min_profile, Some(Profile::Deep));
        assert_eq!(policy.rules[1].if_path.as_deref(), Some("README.md"));
        assert_eq!(policy.rules[1].max_profile, Some(Profile::Cheap));
    }

    #[test]
    fn policy_rules_apply_min_and_max_profiles() {
        let raw = format!(
            r#"
{POLICY}
rules:
  - if_risk_zone: auth
    min_profile: deep
  - if_path: README.md
    max_profile: cheap
"#
        );
        let policy = PolicyFile::parse_yaml(&raw, "test.yaml").unwrap();

        let elevated = apply_policy_rules(
            &policy,
            Profile::Cheap,
            &[RiskZone::Auth],
            &[std::path::PathBuf::from("src/auth/session.rs")],
            &[],
        );
        let lowered = apply_policy_rules(
            &policy,
            Profile::Balanced,
            &[],
            &[std::path::PathBuf::from("README.md")],
            &[],
        );

        assert_eq!(elevated, Profile::Deep);
        assert_eq!(lowered, Profile::Cheap);
    }

    #[test]
    fn policy_rules_match_explicit_target_hints() {
        let raw = format!(
            r#"
{POLICY}
rules:
  - if_path: README.md
    max_profile: cheap
"#
        );
        let policy = PolicyFile::parse_yaml(&raw, "test.yaml").unwrap();

        let profile = apply_policy_rules(
            &policy,
            Profile::Balanced,
            &[],
            &[],
            &[std::path::PathBuf::from("README.md")],
        );

        assert_eq!(profile, Profile::Cheap);
    }

    #[test]
    fn rejects_missing_profile() {
        let raw = r#"
version: 1
profiles:
  cheap:
    model: gpt-5.4-mini
    reasoning: low
"#;
        let err = PolicyFile::parse_yaml(raw, "broken.yaml").unwrap_err();
        assert!(matches!(
            err,
            PolicyError::MissingProfile(Profile::Balanced)
        ));
    }

    #[test]
    fn rejects_unknown_fields() {
        let raw = r#"
version: 1
unknown: true
profiles:
  cheap:
    model: gpt-5.4-mini
    reasoning: low
  balanced:
    model: gpt-5.5
    reasoning: medium
  deep:
    model: gpt-5.5
    reasoning: high
  extradeep:
    model: gpt-5.5
    reasoning: xhigh
"#;
        let err = PolicyFile::parse_yaml(raw, "broken.yaml").unwrap_err();
        assert!(matches!(err, PolicyError::Parse { .. }));
    }

    #[test]
    fn rejects_rule_without_matcher() {
        let raw = format!(
            r#"
{POLICY}
rules:
  - min_profile: deep
"#
        );

        let err = PolicyFile::parse_yaml(&raw, "broken.yaml").unwrap_err();

        assert!(matches!(err, PolicyError::EmptyRuleMatcher { index: 1 }));
    }
}
