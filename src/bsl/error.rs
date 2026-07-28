use thiserror::Error;

#[derive(Error, Debug)]
pub enum BslError {
    #[error("Syntax error at line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("Unknown instruction at line {line}: `{instruction}`")]
    UnknownInstruction { line: usize, instruction: String },

    #[error("Runtime error: {message}")]
    Runtime { message: String },

    #[error("Command failed (exit code {exit_code}): {command}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
