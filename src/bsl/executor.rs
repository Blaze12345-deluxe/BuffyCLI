use std::process::{Command, Stdio, Output};
use crate::bsl::error::BslError;

/// Result of executing a shell command.
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

/// Executes a shell command using the detected user shell.
pub fn execute(command: &str, output_enabled: bool) -> Result<ExecResult, BslError> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut cmd = Command::new(&shell);
    cmd.arg("-c").arg(command);

    if output_enabled {
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::piped());
    } else {
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());
    }

    let output: Output = cmd.output().map_err(BslError::Io)?;

    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();

    Ok(ExecResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code,
        success,
    })
}

/// Clears the terminal screen.
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command_success() {
        let result = execute("echo hello", false).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_command_failure() {
        let result = execute("false", false).unwrap();
        assert!(!result.success);
        assert_ne!(result.exit_code, 0);
    }

    #[test]
    fn test_nonexistent_command() {
        let result = execute("nonexistent_command_xyz123", false);
        assert!(result.is_ok()); // shell returns non-zero, but no panic
        if let Ok(res) = result {
            assert!(!res.success);
        }
    }
}
