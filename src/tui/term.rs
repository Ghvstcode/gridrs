//! Raw mode and terminal size, by shelling out to `stty` (no libc, no dependencies).
//!
//! Owned by task `0501-terminal-input`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::Stty;
use std::io;

/// Switch to the alternate screen and hide the cursor.
pub const ENTER_SCREEN: &str = "\x1b[?1049h\x1b[?25l";
/// Show the cursor and leave the alternate screen.
pub const LEAVE_SCREEN: &str = "\x1b[?25h\x1b[?1049l";

/// The real `stty`, run with this process's standard input.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemStty;

impl Stty for SystemStty {
    fn run(&mut self, args: &[&str]) -> io::Result<String> {
        todo!("task 0501-terminal-input")
    }
}

/// Holds the terminal in raw mode; restores the saved settings when dropped.
/// Fields are private and yours to design.
pub struct RawMode<S: Stty> {
    _todo: std::marker::PhantomData<S>,
}

impl<S: Stty> RawMode<S> {
    /// Save the current settings and switch to raw mode without echo.
    pub fn enter(stty: S) -> io::Result<RawMode<S>> {
        todo!("task 0501-terminal-input")
    }

    /// Restore the saved settings. Idempotent.
    pub fn leave(&mut self) -> io::Result<()> {
        todo!("task 0501-terminal-input")
    }
}

impl<S: Stty> Drop for RawMode<S> {
    fn drop(&mut self) {
        // Implemented by task 0501-terminal-input: must call `leave` and ignore its error.
    }
}

/// Parse the output of `stty size` (`"24 80\n"`) into `(rows, cols)`.
pub fn parse_size(output: &str) -> Option<(usize, usize)> {
    todo!("task 0501-terminal-input")
}

/// `(rows, cols)` of the terminal, or `(24, 80)` when it cannot be determined.
pub fn terminal_size(stty: &mut dyn Stty) -> (usize, usize) {
    todo!("task 0501-terminal-input")
}
