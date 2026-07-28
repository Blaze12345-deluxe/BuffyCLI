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
