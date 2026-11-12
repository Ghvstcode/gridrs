//! A1-style addresses: column letters, `B12`, `$B$12`, `A1:B7`.
//!
//! Owned by task `0001-cell-addresses`.

use crate::types::{CellRef, Range, Reference, MAX_COLS, MAX_ROWS};

/// Zero-based column index to letters: `0 -> "A"`, `25 -> "Z"`, `26 -> "AA"`, `16383 -> "XFD"`.
pub fn col_to_letters(col: u32) -> String {
    let mut n = u64::from(col) + 1;
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push(b'A' + (n % 26) as u8);
        n /= 26;
    }
    out.reverse();
    String::from_utf8_lossy(&out).into_owned()
}

/// Letters to zero-based column index, ASCII case-insensitive.
/// `None` if empty, not all ASCII letters, or beyond the last column (`XFD`).
pub fn letters_to_col(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let mut n: u64 = 0;
    for b in s.bytes() {
        if !b.is_ascii_alphabetic() {
            return None;
        }
        n = n * 26 + u64::from(b.to_ascii_uppercase() - b'A' + 1);
        if n > u64::from(MAX_COLS) {
            return None;
        }
    }
    Some((n - 1) as u32)
}

/// Parse a 1-based row number: digits only, no leading zero, within `MAX_ROWS`.
fn parse_row(s: &str) -> Option<u32> {
    if s.is_empty() || s.starts_with('0') || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if s.len() > 7 {
        return None;
    }
    let n: u32 = s.parse().ok()?;
    if (1..=MAX_ROWS).contains(&n) {
        Some(n - 1)
    } else {
        None
    }
}

/// Split `LETTERS DIGITS` into its two parts (both non-empty).
fn split_cell(s: &str) -> Option<(&str, &str)> {
    let idx = s.bytes().position(|b| !b.is_ascii_alphabetic())?;
    Some(s.split_at(idx))
}

/// Parse a plain cell address such as `B12` (no `$`, no whitespace).
pub fn parse_cell(s: &str) -> Option<CellRef> {
    let (letters, digits) = split_cell(s)?;
    let col = letters_to_col(letters)?;
    let row = parse_row(digits)?;
    Some(CellRef::new(col, row))
}

/// Parse a reference as written in a formula: `B12`, `$B12`, `B$12`, `$B$12`.
pub fn parse_reference(s: &str) -> Option<Reference> {
    let (abs_col, rest) = match s.strip_prefix('$') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let idx = rest.bytes().position(|b| !b.is_ascii_alphabetic())?;
    let (letters, rest) = rest.split_at(idx);
    let (abs_row, digits) = match rest.strip_prefix('$') {
        Some(r) => (true, r),
        None => (false, rest),
    };
    let col = letters_to_col(letters)?;
    let row = parse_row(digits)?;
    Some(Reference { cell: CellRef::new(col, row), abs_col, abs_row })
}

/// Parse `A1:B7` (or a single cell `A1`) into a normalised range. No `$`.
pub fn parse_range(s: &str) -> Option<Range> {
    match s.split_once(':') {
        None => parse_cell(s).map(Range::single),
        Some((a, b)) => Some(Range::new(parse_cell(a)?, parse_cell(b)?)),
    }
}

/// `CellRef::new(1, 11)` -> `"B12"`.
pub fn format_cell(cell: CellRef) -> String {
    format!("{}{}", col_to_letters(cell.col), u64::from(cell.row) + 1)
}

/// Like [`format_cell`], with `$` before each absolute part: `"$B12"`.
pub fn format_reference(r: Reference) -> String {
    format!(
        "{}{}{}{}",
        if r.abs_col { "$" } else { "" },
        col_to_letters(r.cell.col),
        if r.abs_row { "$" } else { "" },
        u64::from(r.cell.row) + 1
    )
}

/// `"A1:B7"`; a one-cell range prints as just `"A1"`.
pub fn format_range(range: Range) -> String {
    if range.start == range.end {
        format_cell(range.start)
    } else {
        format!("{}:{}", format_cell(range.start), format_cell(range.end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns() {
        assert_eq!(col_to_letters(701), "ZZ");
        assert_eq!(col_to_letters(702), "AAA");
        assert_eq!(col_to_letters(16383), "XFD");
        assert_eq!(letters_to_col("aa"), Some(26));
        assert_eq!(letters_to_col("XFE"), None);
        assert_eq!(letters_to_col(&"Z".repeat(100)), None);
    }

    #[test]
    fn cells_and_ranges() {
        assert_eq!(parse_cell("B12"), Some(CellRef::new(1, 11)));
        assert_eq!(parse_cell("A01"), None);
        assert_eq!(parse_cell("A1048577"), None);
        assert_eq!(parse_cell(&format!("A{}", "9".repeat(50))), None);
        assert_eq!(parse_reference("$$A1"), None);
        assert_eq!(parse_reference("$1"), None);
        assert_eq!(parse_reference("A1$"), None);
        assert_eq!(format_reference(parse_reference("$B$12").unwrap()), "$B$12");
        assert_eq!(parse_range("B7:A1"), parse_range("A1:B7"));
        assert_eq!(format_range(parse_range("C3").unwrap()), "C3");
        assert_eq!(parse_range("A1:B2:C3"), None);
    }
}
