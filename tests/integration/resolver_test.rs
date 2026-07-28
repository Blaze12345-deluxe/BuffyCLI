use std::path::PathBuf;
use std::sync::Mutex;

/// Serializes access to HOME env var across parallel resolver tests.
static HOME_LOCK: Mutex<()> = Mutex::new(());

/// Create a temp commands directory and set HOME to it.
/// Returns the TempDir (keeps it alive) and the commands path.
fn setup() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let commands_dir = tmp.path().join(".buffy").join("commands");
    std::fs::create_dir_all(&commands_dir).unwrap();
    std::env::set_var("HOME", tmp.path());
    (tmp, commands_dir)
}

#[test]
fn test_resolve_exact_file() {
    let _lock = HOME_LOCK.lock().unwrap();
    let (tmp, cmds) = setup();
    let cmd_dir = cmds.join("pip-env");
    std::fs::create_dir_all(&cmd_dir).unwrap();
    std::fs::write(cmd_dir.join("create.bsl"), "WRITE \"hello\"\nEXIT").unwrap();

    let args = vec!["pip-env".to_string(), "create".to_string()];
    let result = buffy::resolver::tree::resolve(&args);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.ends_with("create.bsl"));
}

#[test]
fn test_resolve_nested_path() {
    let _lock = HOME_LOCK.lock().unwrap();
    let (tmp, cmds) = setup();
    let cmd_dir = cmds.join("docker").join("compose");
    std::fs::create_dir_all(&cmd_dir).unwrap();
    std::fs::write(cmd_dir.join("up.bsl"), "WRITE \"up\"\nEXIT").unwrap();

    let args = vec!["docker".to_string(), "compose".to_string(), "up".to_string()];
    let result = buffy::resolver::tree::resolve(&args);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.ends_with("up.bsl"));
}

#[test]
fn test_resolve_index_bsl_default() {
    let _lock = HOME_LOCK.lock().unwrap();
    let (tmp, cmds) = setup();
    let cmd_dir = cmds.join("pip-env");
    std::fs::create_dir_all(&cmd_dir).unwrap();
    std::fs::write(cmd_dir.join("index.bsl"), "WRITE \"default\"\nEXIT").unwrap();
    std::fs::write(cmd_dir.join("create.bsl"), "WRITE \"create\"\nEXIT").unwrap();

    let args = vec!["pip-env".to_string()];
    let result = buffy::resolver::tree::resolve(&args);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.ends_with("index.bsl"), "Should resolve to index.bsl, got {:?}", path);
}

#[test]
fn test_resolve_dir_name_match() {
    let _lock = HOME_LOCK.lock().unwrap();
    let (tmp, cmds) = setup();
    let cmd_dir = cmds.join("pip-env");
    std::fs::create_dir_all(&cmd_dir).unwrap();
    std::fs::write(cmd_dir.join("pip-env.bsl"), "WRITE \"self\"\nEXIT").unwrap();

    let args = vec!["pip-env".to_string()];
    let result = buffy::resolver::tree::resolve(&args);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.ends_with("pip-env.bsl"), "Should resolve to pip-env.bsl, got {:?}", path);
}

#[test]
fn test_resolve_first_alphabetically() {
    let _lock = HOME_LOCK.lock().unwrap();
    let (tmp, cmds) = setup();
    let cmd_dir = cmds.join("tools");
    std::fs::create_dir_all(&cmd_dir).unwrap();
    std::fs::write(cmd_dir.join("z-final.bsl"), "").unwrap();
    std::fs::write(cmd_dir.join("a-first.bsl"), "").unwrap();

    let args = vec!["tools".to_string()];
    let result = buffy::resolver::tree::resolve(&args);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.ends_with("a-first.bsl"), "Should resolve to first alphabetically, got {:?}", path);
}

#[test]
fn test_resolve_flat_lookup_fallback() {
    let _lock = HOME_LOCK.lock().unwrap();
    let (tmp, cmds) = setup();
    // Create a package directory with a uniquely-named .bsl file
    let pkg_dir = cmds.join("file-system");
    std::fs::create_dir_all(&pkg_dir).unwrap();
    std::fs::write(pkg_dir.join("cd-down.bsl"), "WRITE \"down\"\nEXIT").unwrap();

    // Resolve by flat name (not by package path)
    let args = vec!["cd-down".to_string()];
    let result = buffy::resolver::tree::resolve(&args);
    assert!(result.is_ok(), "Flat lookup should find cd-down.bsl: {:?}", result);
    let path = result.unwrap();
    assert!(path.ends_with("cd-down.bsl"), "Should resolve to cd-down.bsl, got {:?}", path);
}

#[test]
fn test_resolve_flat_lookup_under_package() {
    let _lock = HOME_LOCK.lock().unwrap();
    let (tmp, cmds) = setup();
    // Create two packages with different .bsl files
    let pkg1 = cmds.join("file-system");
    std::fs::create_dir_all(&pkg1).unwrap();
    std::fs::write(pkg1.join("cd-down.bsl"), "WRITE \"down\"\nEXIT").unwrap();

    let pkg2 = cmds.join("git-flow");
    std::fs::create_dir_all(&pkg2).unwrap();
    std::fs::write(pkg2.join("git-tag.bsl"), "WRITE \"tag\"\nEXIT").unwrap();

    // Full path should still work
    let args = vec!["file-system".to_string(), "cd-down".to_string()];
    let result = buffy::resolver::tree::resolve(&args);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.ends_with("cd-down.bsl"), "Full path should still resolve, got {:?}", path);
}

#[test]
fn test_resolve_command_not_found() {
    let _lock = HOME_LOCK.lock().unwrap();
    let (tmp, cmds) = setup();

    let args = vec!["nonexistent".to_string()];
    let result = buffy::resolver::tree::resolve(&args);
    assert!(result.is_err());
    match result {
        Err(buffy::error::BuffyError::CommandNotFound { command }) => {
            assert_eq!(command, "nonexistent");
        }
        _ => panic!("Expected CommandNotFound error"),
    }
}
