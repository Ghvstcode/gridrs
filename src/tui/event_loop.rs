//! The interactive loop: read bytes, decode keys, drive the app, draw frames.
//!
//! Owned by task `0602-cli`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::app::App;
use std::io::{self, Read, Write};

/// Run the editor until it quits or `input` reaches end of file.
pub fn run(app: &mut App, input: &mut dyn Read, output: &mut dyn Write) -> io::Result<()> {
    todo!("task 0602-cli")
}
