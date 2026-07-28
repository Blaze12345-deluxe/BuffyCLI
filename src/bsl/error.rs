//! BSL-specific error types for script parsing and execution.
//!
//! These errors are used internally by the BSL interpreter pipeline. The
//! dispatch layer in `cli` converts these into `BuffyError` variants with
//! user-facing tips when presenting errors to the user.

use thiserror::Error;

/// Errors specific to BSL script parsing and execution.
///
/// These are distinct from the top-level `BuffyError` and are used internally
/// by the BSL interpreter pipeline (lexer, parser, interpreter, executor).
#[derive(Error, Debug)]
pub enum BslError {
    /// A syntax error during lexing or parsing with line number context.
    #[error("Syntax error at line {line}: {message}")]
    Syntax { line: usize, message: String },

    /// An unrecognized instruction keyword was encountered.
    #[error("Unknown instruction at line {line}: `{instruction}`")]
    UnknownInstruction { line: usize, instruction: String },

    /// A general runtime error during script execution.
    #[error("Runtime error: {message}")]
    Runtime { message: String },

    /// A shell command failed with a non-zero exit code.
    #[error("Command failed (exit code {exit_code}): {command}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    /// An IO error occurred (wraps std::io::Error).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
