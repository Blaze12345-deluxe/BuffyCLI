pub mod cli;
pub mod config;
pub mod diagnostic;
pub mod logger;
pub mod bsl;
pub mod package;
pub mod repository;
pub mod resolver;
pub mod error;

use clap::Parser;
use cli::args::CliArgs;

/// Entry point called from main.rs.
/// Initializes subsystems and dispatches the CLI command.
pub fn run() -> anyhow::Result<()> {
    // Initialize the ~/.buffy/ directory structure
    config::buffy_home::ensure_directories()?;

    // Initialize logging to file
    init_logging()?;

    // Parse CLI args and dispatch
    let args = CliArgs::parse();
    cli::dispatch::execute(args)?;
    Ok(())
}

/// Initialize tracing-based logging to ~/.buffy/logs/.
fn init_logging() -> std::io::Result<()> {
    let logs_dir = config::buffy_home::logs_dir();
    std::fs::create_dir_all(&logs_dir)?;

    let log_file = logs_dir.join(format!(
        "{}.log",
        chrono::Local::now().format("%Y-%m-%d")
    ));

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    // Use tracing with a file writer
    let writer = std::sync::Mutex::new(file);
    let _ = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_target(false)
        .try_init();

    Ok(())
}
