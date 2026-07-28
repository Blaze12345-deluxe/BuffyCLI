use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use sha2::Digest;

/// Returns the path to `~/.buffy/`.
pub fn buffy_home() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".buffy")
}

/// Returns the commands directory (`~/.buffy/commands/`).
pub fn commands_dir() -> PathBuf {
    buffy_home().join("commands")
}

/// Returns the packages directory (`~/.buffy/packages/`).
pub fn packages_dir() -> PathBuf {
    buffy_home().join("packages")
}

/// Returns the cache directory (`~/.buffy/cache/`).
pub fn cache_dir() -> PathBuf {
    buffy_home().join("cache")
}

/// Returns the logs directory (`~/.buffy/logs/`).
pub fn logs_dir() -> PathBuf {
    buffy_home().join("logs")
}

/// Creates all required directories under `~/.buffy/`.
pub fn ensure_directories() -> std::io::Result<()> {
    fs::create_dir_all(commands_dir())?;
    fs::create_dir_all(packages_dir())?;
    fs::create_dir_all(cache_dir())?;
    fs::create_dir_all(logs_dir())?;
    Ok(())
}

/// On first run, installs bundled packages (like pip-env) that ship with Buffy.
/// Bundled packages are embedded in the binary and placed into ~/.buffy/commands/
/// so they are available immediately without needing to fetch from a repository.
pub fn ensure_bundled_packages() -> crate::error::Result<()> {
    let commands_dir = commands_dir();
    let pip_env_dir = commands_dir.join("pip-env");
    let pip_env_file = pip_env_dir.join("pip-env.bsl");

    // Skip if already installed (both on disk and in registry)
    if pip_env_file.exists() {
        let installed = crate::config::settings::read_installed()?;
        if installed.iter().any(|e| e.name == "pip-env") {
            return Ok(());
        }
    }

    // Get embedded content
    let bsl_content: &str = include_str!("../../pip-env.bsl");

    // Create the directory
    std::fs::create_dir_all(&pip_env_dir)?;

    // Write the .bsl file
    std::fs::write(&pip_env_file, bsl_content)?;

    // Compute SHA-256 hash of the content
    let hash = sha2::Sha256::digest(bsl_content.as_bytes());
    let hash_str = format!("{:x}", hash);

    // Build package manifest
    let mut sha256_map = HashMap::new();
    sha256_map.insert("pip-env.bsl".to_string(), hash_str);

    let manifest = crate::package::manifest::PackageManifest {
        name: "pip-env".to_string(),
        version: chrono::Local::now().format("%Y.%m.%d").to_string(),
        description: "Creates a Python virtual environment in the current directory."
            .to_string(),
        author: "Buffy Community".to_string(),
        sha256: sha256_map,
        tags: vec![
            "python".to_string(),
            "venv".to_string(),
            "virtualenv".to_string(),
        ],
        dependencies: crate::package::manifest::PackageDependencies::default(),
        assets: vec![],
        license: String::new(),
        homepage: String::new(),
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(pip_env_dir.join("package.json"), &manifest_json)?;

    // Register in installed.json
    let mut installed = crate::config::settings::read_installed()?;
    installed.push(crate::config::settings::InstalledEntry {
        name: "pip-env".to_string(),
        version: manifest.version.clone(),
        installed: chrono::Local::now().format("%Y-%m-%d").to_string(),
        source: "bundled".to_string(),
        sha256: manifest.combined_hash(),
        author: manifest.author.clone(),
        dependencies: crate::config::settings::Dependencies::default(),
    });
    crate::config::settings::write_installed(&installed)?;

    Ok(())
}

/// Global mutex for serializing tests that modify the HOME environment variable.
/// Used across multiple test modules to prevent parallel-test races on HOME.
#[cfg(test)]
pub static TEST_HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffy_home_path() {
        let path = buffy_home();
        assert!(path.ends_with(".buffy"));
    }

    #[test]
    fn test_commands_dir() {
        let path = commands_dir();
        assert!(path.ends_with(".buffy/commands"));
    }

    #[test]
    fn test_subdirs() {
        assert!(packages_dir().ends_with(".buffy/packages"));
        assert!(cache_dir().ends_with(".buffy/cache"));
        assert!(logs_dir().ends_with(".buffy/logs"));
    }

    #[test]
    fn test_ensure_directories() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();

        // Use a temp dir as home for testing
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", tmp.path());

        let result = ensure_directories();
        assert!(result.is_ok());

        assert!(commands_dir().exists());
        assert!(packages_dir().exists());
        assert!(cache_dir().exists());
        assert!(logs_dir().exists());
    }

    #[test]
    fn test_ensure_bundled_packages_installs_pip_env() {
        let _lock = TEST_HOME_LOCK.lock().unwrap();

        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", tmp.path());

        // Ensure directories first
        ensure_directories().unwrap();

        // Run bundled install
        let result = ensure_bundled_packages();
        assert!(result.is_ok(), "ensure_bundled_packages failed: {:?}", result);

        // Verify pip-env.bsl was created
        let bsl_file = commands_dir().join("pip-env").join("pip-env.bsl");
        assert!(bsl_file.exists(), "pip-env.bsl should exist");

        // Verify package.json was created
        let manifest_file = commands_dir().join("pip-env").join("package.json");
        assert!(manifest_file.exists(), "package.json should exist");

        // Verify installed.json has the entry
        let installed = crate::config::settings::read_installed().unwrap();
        let has_pip_env = installed.iter().any(|e| e.name == "pip-env");
        assert!(has_pip_env, "pip-env should be in installed.json");
        assert_eq!(installed.iter().find(|e| e.name == "pip-env").unwrap().source, "bundled");

        // Verify idempotency: calling again should not error
        let result2 = ensure_bundled_packages();
        assert!(result2.is_ok(), "second call should succeed: {:?}", result2);
    }
}
