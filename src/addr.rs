//! A1-style addresses: column letters, `B12`, `$B$12`, `A1:B7`.
//!
//! Owned by task `0001-cell-addresses`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{CellRef, Range, Reference};

/// Zero-based column index to letters: `0 -> "A"`, `25 -> "Z"`, `26 -> "AA"`, `16383 -> "XFD"`.
pub fn col_to_letters(col: u32) -> String {
    todo!("task 0001-cell-addresses")
}

/// Letters to zero-based column index, ASCII case-insensitive.
/// `None` if empty, not all ASCII letters, or beyond the last column (`XFD`).
pub fn letters_to_col(s: &str) -> Option<u32> {
    todo!("task 0001-cell-addresses")
}

/// Parse a plain cell address such as `B12` (no `$`, no whitespace).
pub fn parse_cell(s: &str) -> Option<CellRef> {
    todo!("task 0001-cell-addresses")
}

/// Parse a reference as written in a formula: `B12`, `$B12`, `B$12`, `$B$12`.
pub fn parse_reference(s: &str) -> Option<Reference> {
    todo!("task 0001-cell-addresses")
}

/// Parse `A1:B7` (or a single cell `A1`) into a normalised range. No `$`.
pub fn parse_range(s: &str) -> Option<Range> {
    todo!("task 0001-cell-addresses")
}

/// `CellRef::new(1, 11)` -> `"B12"`.
pub fn format_cell(cell: CellRef) -> String {
    todo!("task 0001-cell-addresses")
}

/// Like [`format_cell`], with `$` before each absolute part: `"$B12"`.
pub fn format_reference(r: Reference) -> String {
    todo!("task 0001-cell-addresses")
}

/// `"A1:B7"`; a one-cell range prints as just `"A1"`.
pub fn format_range(range: Range) -> String {
    todo!("task 0001-cell-addresses")
}
