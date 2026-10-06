//! Whole-sheet operations: copy/paste and inserting/deleting rows and columns.
//!
//! Owned by task `0403-sheet-ops`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::sheet::Sheet;
use crate::types::{CellRef, Clip, Range, StructuralEdit};

/// Copy a range (inputs and number formats) into a clip.
pub fn copy(sheet: &Sheet, range: Range) -> Clip {
    todo!("task 0403-sheet-ops")
}

/// Paste a clip with its top-left corner at `dest`, shifting relative references in formulas.
pub fn paste(sheet: &mut Sheet, clip: &Clip, dest: CellRef) {
    todo!("task 0403-sheet-ops")
}

/// Insert or delete rows/columns: move the cells, their formats (and column
/// widths), and rewrite every formula so it still points at the same data.
pub fn apply_structural(sheet: &mut Sheet, edit: StructuralEdit) {
    todo!("task 0403-sheet-ops")
}
