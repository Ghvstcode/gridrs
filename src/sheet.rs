//! The cell grid: inputs, computed values, formats, column widths, recalculation.
//!
//! Owned by task `0402-sheet`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{CellRef, Context, NumFormat, Range, Value};

/// A spreadsheet. Fields are private and yours to design.
/// Must stay `Clone` (undo keeps snapshots) and `Default` (an empty sheet).
#[derive(Debug, Clone, Default)]
pub struct Sheet {
    _todo: (),
}

impl Sheet {
    /// An empty sheet.
    pub fn new() -> Sheet {
        todo!("task 0402-sheet")
    }

    /// Set one cell's input (what the user typed; `""` clears it) and recalculate.
    pub fn set(&mut self, at: CellRef, input: &str) {
        todo!("task 0402-sheet")
    }

    /// Set many inputs, then recalculate once.
    pub fn apply(&mut self, edits: &[(CellRef, String)]) {
        todo!("task 0402-sheet")
    }

    /// The input of a cell exactly as it was set (`""` if empty).
    pub fn input(&self, at: CellRef) -> String {
        todo!("task 0402-sheet")
    }

    /// The current value of a cell (`Value::Empty` if empty).
    pub fn value(&self, at: CellRef) -> Value {
        todo!("task 0402-sheet")
    }

    /// Every non-empty input, row-major.
    pub fn cells(&self) -> Vec<(CellRef, String)> {
        todo!("task 0402-sheet")
    }

    /// The smallest range covering every non-empty input, or `None` for an empty sheet.
    pub fn used_range(&self) -> Option<Range> {
        todo!("task 0402-sheet")
    }

    /// The number format of a cell (`General` unless set).
    pub fn format(&self, at: CellRef) -> NumFormat {
        todo!("task 0402-sheet")
    }

    pub fn set_format(&mut self, at: CellRef, format: NumFormat) {
        todo!("task 0402-sheet")
    }

    /// Every cell whose format is not `General`, row-major.
    pub fn formats(&self) -> Vec<(CellRef, NumFormat)> {
        todo!("task 0402-sheet")
    }

    /// The width of a column (`DEFAULT_COL_WIDTH` unless set).
    pub fn col_width(&self, col: u32) -> usize {
        todo!("task 0402-sheet")
    }

    /// Set a column's width, clamped to `MIN_COL_WIDTH..=MAX_COL_WIDTH`.
    pub fn set_col_width(&mut self, col: u32, width: usize) {
        todo!("task 0402-sheet")
    }

    /// Every column whose width is not the default, ascending by column.
    pub fn col_widths(&self) -> Vec<(u32, usize)> {
        todo!("task 0402-sheet")
    }
}

impl Context for Sheet {
    fn cell(&self, at: CellRef) -> Value {
        self.value(at)
    }
}
