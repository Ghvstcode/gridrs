//! Raw terminal bytes -> key presses.
//!
//! Owned by task `0501-terminal-input`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::Key;

/// Decode as many complete keys as possible from `bytes`.
///
/// Returns the keys and the number of bytes consumed. With `flush == false` an
/// incomplete trailing sequence is left unconsumed (so the caller can append
/// more bytes and try again); with `flush == true` everything is consumed.
pub fn decode(bytes: &[u8], flush: bool) -> (Vec<Key>, usize) {
    todo!("task 0501-terminal-input")
}
