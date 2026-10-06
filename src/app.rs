//! The editor: a state machine from key presses to sheet edits and frames. No I/O except file load/save.
//!
//! Owned by task `0601-app`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::sheet::Sheet;
use crate::types::{CellRef, Command, Key, Mode, Outcome};

/// The editor state. Fields are private and yours to design.
pub struct App {
    _todo: (),
}

impl App {
    /// A new editor showing `sheet`. `path` is where `:w` saves.
    /// Starts in `Mode::Normal` at `A1`, not dirty, with an 80x24 screen.
    pub fn new(sheet: Sheet, path: Option<String>) -> App {
        todo!("task 0601-app")
    }

    /// Handle one key press.
    pub fn handle_key(&mut self, key: Key) -> Outcome {
        todo!("task 0601-app")
    }

    /// Run a parsed `:` command.
    pub fn execute(&mut self, command: Command) -> Outcome {
        todo!("task 0601-app")
    }

    /// Tell the editor the terminal size (columns, lines).
    pub fn resize(&mut self, width: usize, height: usize) {
        todo!("task 0601-app")
    }

    /// Render the current frame (scrolling first so the cursor is visible).
    pub fn frame(&mut self) -> Vec<String> {
        todo!("task 0601-app")
    }

    pub fn sheet(&self) -> &Sheet {
        todo!("task 0601-app")
    }

    pub fn cursor(&self) -> CellRef {
        todo!("task 0601-app")
    }

    pub fn mode(&self) -> Mode {
        todo!("task 0601-app")
    }

    /// What is being typed in `Edit` or `Command` mode (without the `:`); empty in `Normal`.
    pub fn buffer(&self) -> String {
        todo!("task 0601-app")
    }

    /// The message from the last action (empty if none).
    pub fn status(&self) -> String {
        todo!("task 0601-app")
    }

    /// Whether there are unsaved changes.
    pub fn is_dirty(&self) -> bool {
        todo!("task 0601-app")
    }

    /// The file `:w` would save to.
    pub fn path(&self) -> Option<String> {
        todo!("task 0601-app")
    }
}
