use buffy::config::buffy_home;

#[test]
fn test_buffy_home_path() {
    let home = buffy_home::buffy_home();
    assert!(home.ends_with(".buffy"));
}

#[test]
fn test_commands_dir_path() {
    let dir = buffy_home::commands_dir();
    assert!(dir.ends_with(".buffy/commands"));
}
