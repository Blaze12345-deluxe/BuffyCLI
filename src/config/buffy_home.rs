use std::path::PathBuf;
use std::fs;

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
}
