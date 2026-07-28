use crate::error::{BuffyError, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_repositories: Vec<String>,
    #[serde(default)]
    pub output_preferences: OutputPrefs,
    #[serde(default)]
    pub update_settings: UpdateSettings,
    #[serde(default = "default_true")]
    pub package_verification: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_repositories: Vec::new(),
            output_preferences: OutputPrefs::default(),
            update_settings: UpdateSettings::default(),
            package_verification: true,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OutputPrefs {
    #[serde(default = "default_true")]
    pub color: bool,
    #[serde(default = "default_true")]
    pub show_progress: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSettings {
    #[serde(default)]
    pub check_on_startup: bool,
    #[serde(default)]
    pub auto_update: bool,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            check_on_startup: true,
            auto_update: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledEntry {
    pub name: String,
    pub version: String,
    pub installed: String,
    pub source: String,
    pub sha256: String,
    pub author: String,
    #[serde(default)]
    pub dependencies: Dependencies,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Dependencies {
    #[serde(default)]
    pub system: Vec<String>,
    #[serde(default)]
    pub packages: Vec<String>,
}

/// Reads the config file from ~/.buffy/config.json.
pub fn read_config() -> Result<Config> {
    let path = crate::config::buffy_home::buffy_home().join("config.json");
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(&path)?;
    let config: Config = serde_json::from_str(&content).map_err(|e| BuffyError::ConfigError {
        path: path.to_string_lossy().to_string(),
        detail: e.to_string(),
    })?;
    Ok(config)
}

/// Writes the config file to ~/.buffy/config.json.
pub fn write_config(config: &Config) -> Result<()> {
    let path = crate::config::buffy_home::buffy_home().join("config.json");
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Reads the installed packages database.
pub fn read_installed() -> Result<Vec<InstalledEntry>> {
    let path = crate::config::buffy_home::buffy_home().join("installed.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)?;
    let entries: Vec<InstalledEntry> = serde_json::from_str(&content).map_err(|e| BuffyError::ConfigError {
        path: path.to_string_lossy().to_string(),
        detail: e.to_string(),
    })?;
    Ok(entries)
}

/// Writes the installed packages database.
pub fn write_installed(entries: &[InstalledEntry]) -> Result<()> {
    let path = crate::config::buffy_home::buffy_home().join("installed.json");
    let content = serde_json::to_string_pretty(entries)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Reads the repository list.
pub fn read_repositories() -> Result<Vec<String>> {
    let path = crate::config::buffy_home::buffy_home().join("repositories.json");
    if !path.exists() {
        return Ok(vec!["https://github.com/Blaze12345-deluxe/Buffy-Plugins".to_string()]);
    }
    let content = fs::read_to_string(&path)?;
    let repos: Vec<String> = serde_json::from_str(&content).map_err(|e| BuffyError::ConfigError {
        path: path.to_string_lossy().to_string(),
        detail: e.to_string(),
    })?;
    Ok(repos)
}

/// Writes the repository list.
pub fn write_repositories(repos: &[String]) -> Result<()> {
    let path = crate::config::buffy_home::buffy_home().join("repositories.json");
    let content = serde_json::to_string_pretty(repos)?;
    fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.package_verification);
    }


}
