use crate::error::{BuffyError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Represents the aliases.json file content.
/// Can hold either simple string aliases or conflict preference objects.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Aliases {
    #[serde(flatten)]
    pub entries: HashMap<String, AliasValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AliasValue {
    /// A simple alias: "ve" → "pip-env"
    Simple(String),
    /// A conflict preference: for a given package name, record which SHA to prefer
    /// Format: { preferred: "<sha>", source: "github.com/owner/repo" }
    ConflictPreference {
        preferred: String,
        source: String,
    },
}

/// Reads the aliases file from ~/.buffy/aliases.json.
pub fn read_aliases() -> Result<Aliases> {
    let path = crate::config::buffy_home::buffy_home().join("aliases.json");
    if !path.exists() {
        return Ok(Aliases::default());
    }
    let content = fs::read_to_string(&path)?;
    let aliases: Aliases = serde_json::from_str(&content).map_err(|e| BuffyError::ConfigError {
        path: path.to_string_lossy().to_string(),
        detail: e.to_string(),
    })?;
    Ok(aliases)
}

/// Writes the aliases file to ~/.buffy/aliases.json.
pub fn write_aliases(aliases: &Aliases) -> Result<()> {
    let path = crate::config::buffy_home::buffy_home().join("aliases.json");
    let content = serde_json::to_string_pretty(aliases)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Resolves a single alias or command name.
/// If it's a simple alias, returns the expanded form.
/// If the name has no alias, returns it as-is.
pub fn resolve(name: &str) -> Result<String> {
    let aliases = read_aliases()?;
    match aliases.entries.get(name) {
        Some(AliasValue::Simple(expanded)) => Ok(expanded.clone()),
        _ => Ok(name.to_string()),
    }
}

/// Sets a simple string alias.
pub fn set_alias(name: &str, target: &str) -> Result<()> {
    let mut aliases = read_aliases()?;
    aliases.entries.insert(name.to_string(), AliasValue::Simple(target.to_string()));
    write_aliases(&aliases)
}

/// Removes an alias or conflict preference.
pub fn remove_alias(name: &str) -> Result<()> {
    let mut aliases = read_aliases()?;
    aliases.entries.remove(name);
    write_aliases(&aliases)
}

/// Lists all aliases (simple string ones only).
pub fn list_aliases() -> Result<Vec<(String, String)>> {
    let aliases = read_aliases()?;
    let mut result = Vec::new();
    for (name, value) in &aliases.entries {
        if let AliasValue::Simple(target) = value {
            result.push((name.clone(), target.clone()));
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

/// Sets a conflict preference for a package name.
/// Resolves which SHA/source to prefer when multiple packages share the same name.
pub fn set_conflict_preference(name: &str, preferred_sha: &str, source: &str) -> Result<()> {
    let mut aliases = read_aliases()?;
    aliases.entries.insert(
        name.to_string(),
        AliasValue::ConflictPreference {
            preferred: preferred_sha.to_string(),
            source: source.to_string(),
        },
    );
    write_aliases(&aliases)
}

/// Resolves a conflict for a given package name.
/// Returns the preferred SHA and source if a conflict preference exists, or None.
pub fn resolve_conflict(name: &str) -> Result<Option<(String, String)>> {
    let aliases = read_aliases()?;
    match aliases.entries.get(name) {
        Some(AliasValue::ConflictPreference { preferred, source }) => {
            Ok(Some((preferred.clone(), source.clone())))
        }
        _ => Ok(None),
    }
}

/// Lists all conflict preferences.
pub fn list_conflicts() -> Result<Vec<(String, String, String)>> {
    let aliases = read_aliases()?;
    let mut result = Vec::new();
    for (name, value) in &aliases.entries {
        if let AliasValue::ConflictPreference { preferred, source } = value {
            result.push((name.clone(), preferred.clone(), source.clone()));
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

/// Checks if a package conflict exists: whether a package with the same name
/// but a different SHA-256 hash is already installed.
/// Returns the installed entry's details if a conflict is detected.
pub fn detect_conflict(name: &str, new_sha: &str) -> Result<Option<(String, String, String)>> {
    let installed = crate::config::settings::read_installed()?;
    for entry in &installed {
        if entry.name == name && entry.sha256 != new_sha {
            // Conflicting package found
            return Ok(Some((
                entry.sha256.clone(),
                entry.author.clone(),
                entry.source.clone(),
            )));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::buffy_home::TEST_HOME_LOCK;

    // Create a temp home with full ~/.buffy/ structure.
    // Caller MUST hold TEST_HOME_LOCK for the entire test.
    struct TempHome {
        _dir: tempfile::TempDir,
    }

    fn setup_temp_home() -> TempHome {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".buffy").join("commands")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".buffy").join("cache")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".buffy").join("logs")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".buffy").join("packages")).unwrap();
        std::env::set_var("HOME", tmp.path());
        TempHome { _dir: tmp }
    }

    #[test]
    fn test_default_aliases_empty() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();
        let _home = setup_temp_home();
        let aliases = read_aliases().unwrap();
        assert!(aliases.entries.is_empty());
    }

    #[test]
    fn test_set_and_resolve_alias() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();
        let _home = setup_temp_home();
        set_alias("ve", "pip-env").unwrap();
        assert_eq!(resolve("ve").unwrap(), "pip-env");
    }

    #[test]
    fn test_resolve_unknown_returns_itself() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();
        let _home = setup_temp_home();
        assert_eq!(resolve("pip-env").unwrap(), "pip-env");
    }

    #[test]
    fn test_remove_alias() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();
        let _home = setup_temp_home();
        set_alias("ve", "pip-env").unwrap();
        remove_alias("ve").unwrap();
        assert_eq!(resolve("ve").unwrap(), "ve");
    }

    #[test]
    fn test_set_and_resolve_conflict_preference() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();
        let _home = setup_temp_home();
        set_conflict_preference("pip-env", "abc123sha", "github.com/user/repo").unwrap();

        let result = resolve_conflict("pip-env").unwrap();
        assert!(result.is_some());
        let (sha, source) = result.unwrap();
        assert_eq!(sha, "abc123sha");
        assert_eq!(source, "github.com/user/repo");
    }

    #[test]
    fn test_resolve_non_existent_conflict() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();
        let _home = setup_temp_home();
        let result = resolve_conflict("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_conflicts() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();
        let _home = setup_temp_home();
        set_conflict_preference("pkg-a", "sha1", "repo1").unwrap();
        set_conflict_preference("pkg-b", "sha2", "repo2").unwrap();

        let conflicts = list_conflicts().unwrap();
        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0].0, "pkg-a");
        assert_eq!(conflicts[1].0, "pkg-b");
    }

    #[test]
    fn test_detect_conflict_no_conflict() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();
        let _home = setup_temp_home();
        let result = detect_conflict("test-pkg", "newsha").unwrap();
        assert!(result.is_none());
    }
}
