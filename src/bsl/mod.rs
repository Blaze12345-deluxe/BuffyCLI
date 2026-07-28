//! BSL (Buffy Script Language) interpreter core.
//!
//! This module provides the complete pipeline for parsing and executing BSL
//! scripts: lexer -> parser -> interpreter -> executor. The public API is
//! `parse()` (tokenize + validate) and `interpret()` (execute).

pub mod ast;
pub mod error;
pub mod executor;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod variable;

pub use interpreter::interpret;
pub use parser::parse;
