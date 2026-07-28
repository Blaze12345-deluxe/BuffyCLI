use crate::bsl::ast::{BslScript, Metadata, Statement, Token, WaitTarget};
use crate::bsl::error::BslError;

/// Parses a token stream into a BslScript AST.
pub fn parse(tokens: Vec<(usize, Token)>) -> Result<BslScript, BslError> {
    let tokens: Vec<_> = tokens.into_iter().filter(|(_, t)| !matches!(t, Token::Newline)).collect();
    let mut pos = 0;
    let mut metadata = Vec::new();
    let mut statements = Vec::new();
    let mut in_metadata = true;

    while pos < tokens.len() {
        let (line, token) = &tokens[pos];

        match token {
            Token::Eof => break,
            Token::Ident(ident) => {
                if in_metadata && is_metadata_keyword(ident) {
                    let meta = parse_metadata(&tokens, &mut pos, *line)?;
                    metadata.push(meta);
                } else {
                    in_metadata = false;
                    let stmt = parse_statement(&tokens, &mut pos, *line)?;
                    statements.push(stmt);
                }
            }
            _ => {
                return Err(BslError::Syntax {
                    line: *line,
                    message: format!("Unexpected token: {}", token),
                });
            }
        }
    }

    Ok(BslScript { metadata, statements })
}

fn is_metadata_keyword(ident: &str) -> bool {
    matches!(ident, "VERSION" | "AUTHOR" | "DESCRIPTION" | "OUTPUT")
}

fn parse_metadata(tokens: &[(usize, Token)], pos: &mut usize, line: usize) -> Result<Metadata, BslError> {
    let ident = match &tokens[*pos].1 {
        Token::Ident(s) => s.clone(),
        _ => unreachable!(),
    };
    *pos += 1;

    // Expect equals sign
    if *pos >= tokens.len() || !matches!(tokens[*pos].1, Token::Equals) {
        return Err(BslError::Syntax {
            line,
            message: format!("Expected `=` after {}", ident),
        });
    }
    *pos += 1;

    // Expect value
    if *pos >= tokens.len() {
        return Err(BslError::Syntax {
            line,
            message: format!("Expected value after {} =", ident),
        });
    }

    match ident.as_str() {
        "VERSION" => {
            if let Token::StringLit(v) = &tokens[*pos].1 {
                *pos += 1;
                Ok(Metadata::Version(v.clone()))
            } else {
                Err(BslError::Syntax {
                    line,
                    message: "VERSION requires a string value".to_string(),
                })
            }
        }
        "AUTHOR" => {
            if let Token::StringLit(v) = &tokens[*pos].1 {
                *pos += 1;
                Ok(Metadata::Author(v.clone()))
            } else {
                Err(BslError::Syntax {
                    line,
                    message: "AUTHOR requires a string value".to_string(),
                })
            }
        }
        "DESCRIPTION" => {
            if let Token::StringLit(v) = &tokens[*pos].1 {
                *pos += 1;
                Ok(Metadata::Description(v.clone()))
            } else {
                Err(BslError::Syntax {
                    line,
                    message: "DESCRIPTION requires a string value".to_string(),
                })
            }
        }
        "OUTPUT" => {
            let val = match &tokens[*pos].1 {
                Token::StringLit(s) => s.clone(),
                _ => {
                    return Err(BslError::Syntax {
                        line,
                        message: "OUTPUT requires true or false".to_string(),
                    });
                }
            };
            *pos += 1;
            match val.as_str() {
                "true" => Ok(Metadata::Output(true)),
                "false" => Ok(Metadata::Output(false)),
                _ => Err(BslError::Syntax {
                    line,
                    message: format!("OUTPUT must be true or false, got `{}`", val),
                }),
            }
        }
        _ => unreachable!(),
    }
}

fn parse_statement(tokens: &[(usize, Token)], pos: &mut usize, line: usize) -> Result<Statement, BslError> {
    let ident = match &tokens[*pos].1 {
        Token::Ident(s) => s.clone(),
        _ => unreachable!(),
    };
    *pos += 1;

    match ident.as_str() {
        "WRITE" => {
            let arg = expect_string_arg(tokens, pos, line, "WRITE")?;
            Ok(Statement::Write(arg))
        }
        "RUN" => {
            let arg = expect_string_arg(tokens, pos, line, "RUN")?;
            Ok(Statement::Run(arg))
        }
        "WAIT" => {
            if *pos < tokens.len() {
                match &tokens[*pos].1 {
                    Token::Number(n) => {
                        *pos += 1;
                        Ok(Statement::Wait(WaitTarget::Duration(*n)))
                    }
                    Token::StringLit(s) => {
                        *pos += 1;
                        Ok(Statement::Wait(WaitTarget::Prompt(s.clone())))
                    }
                    _ => Err(BslError::Syntax {
                        line,
                        message: "WAIT requires a number or string".to_string(),
                    }),
                }
            } else {
                Err(BslError::Syntax {
                    line,
                    message: "WAIT requires a number or string".to_string(),
                })
            }
        }
        "CLEAR" => Ok(Statement::Clear),
        "EXIT" => Ok(Statement::Exit),
        "OUTPUT" => {
            // Runtime output toggle: expect = true/false
            if *pos >= tokens.len() || !matches!(tokens[*pos].1, Token::Equals) {
                return Err(BslError::Syntax {
                    line,
                    message: "OUTPUT requires = true or = false".to_string(),
                });
            }
            *pos += 1;
            if *pos >= tokens.len() {
                return Err(BslError::Syntax {
                    line,
                    message: "OUTPUT requires a value after =".to_string(),
                });
            }
            match &tokens[*pos].1 {
                Token::StringLit(s) if s == "true" => {
                    *pos += 1;
                    Ok(Statement::SetOutput(true))
                }
                Token::StringLit(s) if s == "false" => {
                    *pos += 1;
                    Ok(Statement::SetOutput(false))
                }
                _ => Err(BslError::Syntax {
                    line,
                    message: "OUTPUT must be true or false".to_string(),
                }),
            }
        }
        _ => Err(BslError::UnknownInstruction {
            line,
            instruction: ident,
        }),
    }
}

fn expect_string_arg(tokens: &[(usize, Token)], pos: &mut usize, line: usize, instruction: &str) -> Result<String, BslError> {
    if *pos >= tokens.len() {
        return Err(BslError::Syntax {
            line,
            message: format!("{} requires a string argument", instruction),
        });
    }
    match &tokens[*pos].1 {
        Token::StringLit(s) => {
            *pos += 1;
            Ok(s.clone())
        }
        _ => Err(BslError::Syntax {
            line,
            message: format!("{} requires a string argument", instruction),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bsl::lexer;

    fn parse_source(source: &str) -> Result<BslScript, BslError> {
        let tokens = lexer::tokenize(source)?;
        parse(tokens)
    }

    #[test]
    fn test_minimal_script() {
        let script = parse_source(r#"
VERSION = "1.0"
DESCRIPTION = "Test"
AUTHOR = "Test"
OUTPUT = false

WRITE "hello"
EXIT
"#).unwrap();

        assert_eq!(script.metadata.len(), 4);
        assert_eq!(script.statements.len(), 2);
        assert!(!script.get_output_mode());
    }

    #[test]
    fn test_metadata_after_statement_is_error() {
        // Per BSL spec: "Metadata must appear before executable instructions"
        // VERSION after a WRITE is an unknown instruction → parse error
        let result = parse_source(r#"
WRITE "hello"
VERSION = "1.0"
"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_instruction() {
        let result = parse_source(r#"FOOBAR "baz""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_command() {
        let script = parse_source(r#"RUN "ls -la""#).unwrap();
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0] {
            Statement::Run(cmd) => assert_eq!(cmd, "ls -la"),
            _ => panic!("Expected Run statement"),
        }
    }

    #[test]
    fn test_wait_duration() {
        let script = parse_source("WAIT 5").unwrap();
        match &script.statements[0] {
            Statement::Wait(WaitTarget::Duration(n)) => assert_eq!(*n, 5),
            _ => panic!("Expected Wait duration"),
        }
    }

    #[test]
    fn test_wait_prompt() {
        let script = parse_source(r#"WAIT "Press enter""#).unwrap();
        match &script.statements[0] {
            Statement::Wait(WaitTarget::Prompt(s)) => assert_eq!(s, "Press enter"),
            _ => panic!("Expected Wait prompt"),
        }
    }

    #[test]
    fn test_run_with_spaces_in_command() {
        let source = r#"RUN "python3 -m venv .venv""#;
        let tokens = lexer::tokenize(source).unwrap();
        let script = parse(tokens).unwrap();
        match &script.statements[0] {
            Statement::Run(cmd) => assert_eq!(cmd, "python3 -m venv .venv"),
            _ => panic!("Expected Run statement"),
        }
    }
}
