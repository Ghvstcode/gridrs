//! Finding and rewriting the cell references inside a formula.
//!
//! Owned by task `0103-reference-rewriting`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{Expr, Range, StructuralEdit};

/// Every cell or range the expression reads, as normalised ranges, in source order
/// (duplicates kept; a single reference is a one-cell range).
pub fn precedents(expr: &Expr) -> Vec<Range> {
    todo!("task 0103-reference-rewriting")
}

/// The formula as it should read after being copied `dcol` columns right and
/// `drow` rows down: relative parts move, `$` parts stay.
pub fn translate(expr: &Expr, dcol: i64, drow: i64) -> Expr {
    todo!("task 0103-reference-rewriting")
}

/// The formula as it should read after rows/columns were inserted or deleted.
pub fn adjust(expr: &Expr, edit: StructuralEdit) -> Expr {
    todo!("task 0103-reference-rewriting")
}
