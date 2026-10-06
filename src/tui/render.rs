//! A viewport of the grid -> lines of text with ANSI styling. Pure: no I/O.
//!
//! Owned by task `0502-renderer`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{GridSource, Layout, Ui};

/// Reverse video on.
pub const REVERSE: &str = "\x1b[7m";
/// Bold on.
pub const BOLD: &str = "\x1b[1m";
/// All attributes off.
pub const RESET: &str = "\x1b[0m";

/// Which rows and columns fit in the frame described by `ui`.
pub fn layout(grid: &dyn GridSource, ui: &Ui) -> Layout {
    todo!("task 0502-renderer")
}

/// Render one frame: exactly `ui.height` lines, each exactly `ui.width`
/// characters wide once the styling escapes are removed.
pub fn render(grid: &dyn GridSource, ui: &Ui) -> Vec<String> {
    todo!("task 0502-renderer")
}

/// Turn rendered lines into one string ready to write to the terminal.
pub fn to_ansi(lines: &[String]) -> String {
    todo!("task 0502-renderer")
}

/// Remove every ANSI escape sequence, leaving the visible text.
pub fn strip_ansi(s: &str) -> String {
    todo!("task 0502-renderer")
}
