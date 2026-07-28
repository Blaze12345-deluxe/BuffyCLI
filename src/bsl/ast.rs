/// A complete parsed BSL script.
pub struct BslScript {
    pub metadata: Vec<Metadata>,
    pub statements: Vec<Statement>,
}

impl BslScript {
    pub fn get_output_mode(&self) -> bool {
        for m in &self.metadata {
            if let Metadata::Output(val) = m {
                return *val;
            }
        }
        true // default to showing output
    }
}

/// Metadata lines that appear before executable instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum Metadata {
    Version(String),
    Author(String),
    Description(String),
    Output(bool),
}

/// Executable instructions in a BSL script.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Write(String),
    Run(String),
    Wait(WaitTarget),
    Clear,
    Exit,
    /// Toggle command output on/off at runtime (OUTPUT = true/false)
    SetOutput(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaitTarget {
    Duration(u64),
    Prompt(String),
}

/// Tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    StringLit(String),
    Number(u64),
    Equals,
    Newline,
    Eof,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{}", s),
            Token::StringLit(s) => write!(f, "\"{}\"", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::Equals => write!(f, "="),
            Token::Newline => write!(f, "<newline>"),
            Token::Eof => write!(f, "<eof>"),
        }
    }
}
