//! The native `.grid` format: a line-oriented, diff-friendly text file.
//!
//! Owned by task `0303-native-format`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::Document;

/// Serialise a document to `.grid` text.
pub fn encode(doc: &Document) -> String {
    todo!("task 0303-native-format")
}

/// Parse `.grid` text. The error message names the offending line (`line 3: ...`).
pub fn decode(text: &str) -> Result<Document, String> {
    todo!("task 0303-native-format")
}
