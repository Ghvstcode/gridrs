//! The command line: argument parsing and the three ways to run (`--eval`, `--dump`, interactive).
//!
//! Owned by task `0602-cli`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::CliAction;
use std::io::Write;

/// Parse command-line arguments (without the program name).
pub fn parse_args(args: &[String]) -> Result<CliAction, String> {
    todo!("task 0602-cli")
}

/// Run gridrs. Returns the process exit code.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    todo!("task 0602-cli")
}
