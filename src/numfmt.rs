//! Turning values into the text shown in a cell.
//!
//! Owned by task `0301-number-format`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{Align, NumFormat, Value};

/// The text displayed for a value under a number format.
pub fn format_value(v: &Value, fmt: NumFormat) -> String {
    todo!("task 0301-number-format")
}

/// Default alignment: numbers right, text left, booleans and errors centred.
pub fn align_of(v: &Value) -> Align {
    todo!("task 0301-number-format")
}
