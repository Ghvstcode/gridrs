//! Expression tree -> value.
//!
//! Owned by task `0104-evaluator`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{Context, Expr, Value};

/// Evaluate an expression. Never panics and never returns `Value::Empty`.
pub fn eval(expr: &Expr, ctx: &dyn Context) -> Value {
    todo!("task 0104-evaluator")
}
