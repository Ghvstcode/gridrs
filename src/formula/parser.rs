//! Tokens -> expression tree, with Excel operator precedence.
//!
//! Owned by task `0102-formula-parser`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{Expr, FormulaError};

/// Parse formula text (with or without the leading `=`) into an expression tree.
pub fn parse(src: &str) -> Result<Expr, FormulaError> {
    todo!("task 0102-formula-parser")
}
