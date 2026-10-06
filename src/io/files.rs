//! Sheets on disk: CSV import/export, the native format, load/save by path.
//!
//! Owned by task `0404-files`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::sheet::Sheet;
use crate::types::{CsvMode, Document, FileKind};

/// `.grid` (ASCII case-insensitive extension) is `Native`; everything else is `Csv`.
pub fn kind_for_path(path: &str) -> FileKind {
    todo!("task 0404-files")
}

/// Build a sheet from CSV text: field at record `r`, position `c` becomes the input of that cell.
pub fn sheet_from_csv(text: &str) -> Result<Sheet, String> {
    todo!("task 0404-files")
}

/// Write the sheet as CSV, from `A1` to the bottom-right of the used range.
pub fn sheet_to_csv(sheet: &Sheet, mode: CsvMode) -> String {
    todo!("task 0404-files")
}

/// Everything the native format stores about a sheet.
pub fn sheet_to_document(sheet: &Sheet) -> Document {
    todo!("task 0404-files")
}

/// Rebuild a sheet from a document.
pub fn sheet_from_document(doc: &Document) -> Sheet {
    todo!("task 0404-files")
}

/// Read a sheet from disk, choosing the format with [`kind_for_path`].
/// Errors are one human-readable line that mentions the path.
pub fn load(path: &str) -> Result<Sheet, String> {
    todo!("task 0404-files")
}

/// Write a sheet to disk, choosing the format with [`kind_for_path`] (CSV uses `CsvMode::Inputs`).
pub fn save(sheet: &Sheet, path: &str) -> Result<(), String> {
    todo!("task 0404-files")
}
