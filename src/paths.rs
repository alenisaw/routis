use std::path::PathBuf;

#[must_use]
pub fn routis_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("ROUTIS_HOME") {
        return PathBuf::from(path);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.join(".routis");
        }
    }
    PathBuf::from(".routis")
}

#[must_use]
pub fn default_policy_path() -> PathBuf {
    routis_dir().join("policies").join("default.yaml")
}
