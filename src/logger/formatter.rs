use colored::Colorize;

/// Displays a standard message.
pub fn write(message: &str) {
    println!("{}", message);
}

/// Displays a success message with a green checkmark.
pub fn success(message: &str) {
    println!("{} {}", "✔".green(), message);
}

/// Displays an error message with a red X.
pub fn error(message: &str) {
    eprintln!("{} {}", "✘".red().bold(), message);
}

/// Displays a formatted command error.
pub fn command_error(command: &str, exit_code: i32, stderr: &str) {
    eprintln!("{}", "Error".red().bold());
    eprintln!();
    eprintln!("{}", "Command:".yellow());
    eprintln!("{}", command);
    eprintln!();
    eprintln!("{}", "Exit Code:".yellow());
    eprintln!("{}", exit_code);
    if !stderr.is_empty() {
        eprintln!();
        eprintln!("{}", "stderr:".yellow());
        eprintln!("{}", stderr);
    }
}

/// Displays an info label.
pub fn info(message: &str) {
    println!("{} {}", "ℹ".cyan(), message);
}

/// Displays a warning message.
pub fn warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow(), message);
}
