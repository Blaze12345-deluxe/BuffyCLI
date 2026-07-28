use std::path::PathBuf;

/// Returns the user's home directory.
pub fn home_dir() -> PathBuf {
    dirs::home_dir().expect("Could not determine home directory")
}

/// Returns the Buffy configuration directory (~/.buffy).
pub fn buffy_dir() -> PathBuf {
    home_dir().join(".buffy")
}
