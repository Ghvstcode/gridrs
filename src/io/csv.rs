//! CSV (RFC 4180) reading and writing. Pure text <-> rows of fields.
//!
//! Owned by task `0302-csv`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::CsvError;

/// Parse CSV text into records of fields.
pub fn parse(text: &str) -> Result<Vec<Vec<String>>, CsvError> {
    todo!("task 0302-csv")
}

/// Write records as CSV text, quoting only where needed.
pub fn write(rows: &[Vec<String>]) -> String {
    todo!("task 0302-csv")
}
