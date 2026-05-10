use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn routis_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("ROUTIS_HOME") {
        return Ok(PathBuf::from(path));
    }
    home_dir()
        .map(|path| path.join(".routis"))
        .context("could not determine user home; set ROUTIS_HOME")
}

pub fn default_policy_path() -> Result<PathBuf> {
    Ok(routis_dir()?.join("policies").join("default.yaml"))
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .or_else(
                || match (std::env::var_os("HOMEDRIVE"), std::env::var_os("HOMEPATH")) {
                    (Some(drive), Some(path)) => {
                        let mut value = PathBuf::from(drive);
                        value.push(path);
                        Some(value)
                    }
                    _ => None,
                },
            )
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn routis_home_override_wins() {
        let dir = TempDir::new().unwrap();
        std::env::set_var("ROUTIS_HOME", dir.path());

        assert_eq!(routis_dir().unwrap(), dir.path());

        std::env::remove_var("ROUTIS_HOME");
    }

    #[test]
    fn default_path_uses_user_home() {
        std::env::remove_var("ROUTIS_HOME");

        assert_eq!(
            routis_dir()
                .unwrap()
                .file_name()
                .and_then(|value| value.to_str()),
            Some(".routis")
        );
    }
}
