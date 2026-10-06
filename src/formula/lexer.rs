//! Formula text -> tokens.
//!
//! Owned by task `0101-formula-lexer`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{FormulaError, Token};

/// Split formula text into tokens. Whitespace between tokens is skipped.
pub fn tokenize(src: &str) -> Result<Vec<Token>, FormulaError> {
    todo!("task 0101-formula-lexer")
}
