use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuffyError {
    #[error("BSL syntax error in {path}:{line}: {message}")]
    BslSyntax {
        path: String,
        line: usize,
        message: String,
    },

    #[error("BSL runtime error: command `{command}` failed with exit code {exit_code}{}\nUse OUTPUT = true to see the command's output.", show_stderr(.stderr))]
    BslRuntime {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("Unknown instruction at line {line}: `{instruction}`\n  Tip: Available instructions are: VERSION, AUTHOR, DESCRIPTION, OUTPUT, WRITE, RUN, WAIT, CLEAR, EXIT")]
    UnknownInstruction { line: usize, instruction: String },

    #[error("Command not found: `{command}`\n  Tip: Use `buffy --list` to see installed commands, or `buffy --repo search {command}` to find packages")]
    CommandNotFound { command: String },

    #[error("Multiple packages provide `{command}`: {detail}\n  Tip: Specify the package name, e.g. `buffy <package> {command}`")]
    AmbiguousCommand {
        command: String,
        /// Human-readable list of (package_name, path) pairs
        matches: Vec<(String, String)>,
        /// Human-readable summary for the error message
        detail: String,
    },

    #[error("Package `{name}` not found in any repository\n  Tip: Use `buffy --repo search {name}` across repositories, or `buffy --repo add <url>` to add more repositories")]
    PackageNotFound { name: String },

    #[error("Package `{name}` verification failed: {detail}\n  Tip: Try reinstalling with `buffy --update-package {name}` or `buffy --install {name}`")]
    PackageVerificationFailed { name: String, detail: String },

    #[error("Invalid package manifest in `{path}`: {detail}")]
    InvalidManifest { path: String, detail: String },

    #[error("Failed to connect to repository `{url}`: {detail}\n  Tip: Check your internet connection and verify the URL is correct")]
    RepositoryConnection { url: String, detail: String },

    #[error("Invalid configuration in `{path}`: {detail}\n  Tip: Run `buffy --repair` to fix configuration issues, or `buffy --reset` to reset to defaults")]
    ConfigError { path: String, detail: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] ureq::Error),

    #[error("Dependency error: {message}")]
    DependencyError {
        package: String,
        message: String,
    },

    #[error("Circular dependency detected: {chain}\n  Tip: Check the dependency declarations in your packages to break the cycle")]
    CircularDependency {
        chain: String,
    },

    #[error("Missing system dependency: {dependency}\n  Tip: Install it with your system package manager (apt, brew, etc.)")]
    MissingSystemDependency {
        dependency: String,
        purpose: String,
    },
}

pub type Result<T> = std::result::Result<T, BuffyError>;

/// Helper to show stderr preview for runtime errors
fn show_stderr(stderr: &str) -> String {
    if stderr.is_empty() {
        String::new()
    } else {
        let preview = stderr.trim().lines().next().unwrap_or("").to_string();
        format!("\n  stderr: {}", preview)
    }
}
