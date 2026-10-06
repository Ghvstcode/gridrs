//! Value semantics: literal parsing, coercion, comparison, decimal rounding, number-to-text.
//!
//! Owned by task `0002-values`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{ErrorKind, Value};
use std::cmp::Ordering;

/// Parse a number the way a cell or a text-to-number coercion accepts it
/// (`12`, `-1.5`, `.5`, `1e3`, `12%`). Surrounding ASCII whitespace is ignored.
pub fn parse_number(s: &str) -> Option<f64> {
    todo!("task 0002-values")
}

/// Turn what a user typed into a cell (not starting with `=`) into a value.
pub fn parse_literal(input: &str) -> Value {
    todo!("task 0002-values")
}

/// Round to `digits` decimal places, half away from zero, the way Excel does
/// (`round_to(2.675, 2) == 2.68`). `digits` may be negative.
pub fn round_to(x: f64, digits: i32) -> f64 {
    todo!("task 0002-values")
}

/// `x` rounded like [`round_to`] and written with exactly `decimals` decimals:
/// `format_fixed(2.675, 2) == "2.68"`, `format_fixed(3.0, 2) == "3.00"`. Never prints `-0`.
pub fn format_fixed(x: f64, decimals: usize) -> String {
    todo!("task 0002-values")
}

/// A number as text in "General" format: at most 15 significant digits,
/// no trailing zeros, scientific notation only for very large or very small magnitudes.
pub fn number_to_text(n: f64) -> String {
    todo!("task 0002-values")
}

/// Coerce to a number (`Err` carries the error value the formula should produce).
pub fn to_number(v: &Value) -> Result<f64, ErrorKind> {
    todo!("task 0002-values")
}

/// Coerce to text.
pub fn to_text(v: &Value) -> Result<String, ErrorKind> {
    todo!("task 0002-values")
}

/// Coerce to a boolean.
pub fn to_bool(v: &Value) -> Result<bool, ErrorKind> {
    todo!("task 0002-values")
}

/// Total order used by `=`, `<`, `>` and by lookups: numbers < text < booleans,
/// text compared case-insensitively, an empty cell adapting to the other side.
pub fn compare(a: &Value, b: &Value) -> Ordering {
    todo!("task 0002-values")
}
