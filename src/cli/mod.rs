//! CLI argument parsing and command dispatch.
//!
//! The `cli` module handles all user interaction. `args::CliArgs` defines the
//! CLI interface using clap derive macros, and `dispatch::execute()` routes
//! each flag or command to the appropriate subsystem handler.

pub mod args;
pub mod dispatch;
