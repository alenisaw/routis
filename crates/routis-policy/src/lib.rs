#![forbid(unsafe_code)]
#![deny(warnings)]

use routis_core::Profile;
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyFile {
    pub version: u32,
    pub profiles: BTreeMap<String, ProfileExecutionConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileExecutionConfig {
    pub model: String,
    pub reasoning: String,
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
}
