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
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set_path(key: &'static str, value: &std::path::Path) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var_os(key);
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn routis_home_override_wins() {
        let _guard = env_lock();
        let dir = tempfile::TempDir::new().unwrap();
        let _env_guard = EnvVarGuard::set_path("ROUTIS_HOME", dir.path());

        assert_eq!(routis_dir().unwrap(), dir.path());
    }

    #[test]
    fn default_path_uses_user_home() {
        let _guard = env_lock();
        let _env_guard = EnvVarGuard::remove("ROUTIS_HOME");

        assert_eq!(
            routis_dir()
                .unwrap()
                .file_name()
                .and_then(|value| value.to_str()),
            Some(".routis")
        );
    }
}
