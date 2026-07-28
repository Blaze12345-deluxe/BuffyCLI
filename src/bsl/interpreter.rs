use crate::bsl::ast::{BslScript, Statement, WaitTarget};
use crate::bsl::error::BslError;
use crate::bsl::executor;
use crate::bsl::variable::{self, ExecutionContext};
use std::io::{self, BufRead, Write};
use std::time::Duration;

/// Executes a parsed BSL script.
pub fn interpret(script: &BslScript, args: &[String]) -> Result<(), BslError> {
    let output_enabled = script.get_output_mode();
    let ctx = ExecutionContext {
        args: args.to_vec(),
    };

    for statement in &script.statements {
        match statement {
            Statement::Write(text) => {
                let resolved = variable::resolve(text, &ctx);
                println!("{}", resolved);
            }
            Statement::Run(cmd) => {
                let resolved = variable::resolve(cmd, &ctx);
                let result = executor::execute(&resolved, output_enabled)?;
                if !result.success {
                    return Err(BslError::CommandFailed {
                        command: resolved,
                        exit_code: result.exit_code,
                        stderr: result.stderr,
                    });
                }
            }
            Statement::Wait(target) => {
                handle_wait(target)?;
            }
            Statement::Clear => {
                executor::clear_screen();
            }
            Statement::Exit => {
                break;
            }
        }
    }

    Ok(())
}

fn handle_wait(target: &WaitTarget) -> Result<(), BslError> {
    match target {
        WaitTarget::Duration(secs) => {
            std::thread::sleep(Duration::from_secs(*secs));
            Ok(())
        }
        WaitTarget::Prompt(message) => {
            print!("{}", message);
            io::stdout().flush().map_err(BslError::Io)?;
            let mut input = String::new();
            io::stdin().lock().read_line(&mut input).map_err(BslError::Io)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bsl::ast::{BslScript, Metadata, Statement, WaitTarget};

    fn make_script(statements: Vec<Statement>) -> BslScript {
        BslScript {
            metadata: vec![],
            statements,
        }
    }

    #[test]
    fn test_write_statement() {
        let script = make_script(vec![Statement::Write("hello".to_string()), Statement::Exit]);
        interpret(&script, &[]).unwrap();
    }

    #[test]
    fn test_exit_stops_execution() {
        let script = make_script(vec![Statement::Exit, Statement::Write("should not appear".to_string())]);
        interpret(&script, &[]).unwrap();
    }
}
