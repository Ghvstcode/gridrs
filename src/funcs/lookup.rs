//! Built-in functions: VLOOKUP INDEX MATCH.
//!
//! Owned by task `0204-funcs-lookup`. Stub: the signature is the contract, the body is yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{Arg, Value};

/// Evaluate the function `name` (ASCII upper case) if this family implements it;
/// `None` for every other name, so the dispatcher in `funcs/mod.rs` can try the next family.
///
/// The stub returns `None` rather than panicking so that the rest of the
/// spreadsheet works (with `#NAME?`) before this family is merged.
pub fn call(name: &str, args: &[Arg]) -> Option<Value> {
    None
}
