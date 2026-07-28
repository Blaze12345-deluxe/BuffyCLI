pub mod ast;
pub mod error;
pub mod executor;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod variable;

pub use interpreter::interpret;
pub use parser::parse;
