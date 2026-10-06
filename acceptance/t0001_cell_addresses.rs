//! Acceptance test for task 0001-cell-addresses. Outside every task's write scope.

use gridrs::addr::*;
use gridrs::types::{CellRef, Range, Reference, MAX_COLS, MAX_ROWS};

#[test]
fn column_letters() {
    for (col, s) in [(0, "A"), (1, "B"), (25, "Z"), (26, "AA"), (27, "AB"), (51, "AZ"), (52, "BA"), (701, "ZZ"), (702, "AAA"), (16383, "XFD")] {
        assert_eq!(col_to_letters(col), s, "col_to_letters({col})");
        assert_eq!(letters_to_col(s), Some(col), "letters_to_col({s:?})");
    }
}

#[test]
fn column_letters_round_trip_for_every_column() {
    for col in 0..MAX_COLS {
        let s = col_to_letters(col);
        assert!(s.bytes().all(|b| b.is_ascii_uppercase()), "{s:?}");
        assert_eq!(letters_to_col(&s), Some(col));
    }
}

#[test]
fn letters_to_col_is_case_insensitive_and_strict() {
    assert_eq!(letters_to_col("aa"), Some(26));
    assert_eq!(letters_to_col("xFd"), Some(16383));
    assert_eq!(letters_to_col(""), None);
    assert_eq!(letters_to_col("XFE"), None, "one past the last column");
    assert_eq!(letters_to_col("ZZZZ"), None);
    assert_eq!(letters_to_col("ZZZZZZZZZZZZZZZZZZZZZZZZ"), None, "must not overflow");
    assert_eq!(letters_to_col("A1"), None);
    assert_eq!(letters_to_col("A "), None);
    assert_eq!(letters_to_col("É"), None);
}

#[test]
fn parse_cell_accepts_plain_addresses() {
    assert_eq!(parse_cell("A1"), Some(CellRef::new(0, 0)));
    assert_eq!(parse_cell("B12"), Some(CellRef::new(1, 11)));
    assert_eq!(parse_cell("b12"), Some(CellRef::new(1, 11)));
    assert_eq!(parse_cell("AA100"), Some(CellRef::new(26, 99)));
    assert_eq!(parse_cell("XFD1048576"), Some(CellRef::new(MAX_COLS - 1, MAX_ROWS - 1)));
}

#[test]
fn parse_cell_rejects_everything_else() {
    for bad in ["", "A", "1", "A0", "A01", "1A", "A1B", "A-1", "A 1", " A1", "A1 ", "$A$1", "A$1", "A1:B2", "XFE1", "A1048577", "A99999999999999999999", "A1.0", "Ä1", "A+1"] {
        assert_eq!(parse_cell(bad), None, "parse_cell({bad:?})");
    }
}

#[test]
fn parse_reference_handles_dollars() {
    let b12 = CellRef::new(1, 11);
    assert_eq!(parse_reference("B12"), Some(Reference { cell: b12, abs_col: false, abs_row: false }));
    assert_eq!(parse_reference("$B12"), Some(Reference { cell: b12, abs_col: true, abs_row: false }));
    assert_eq!(parse_reference("B$12"), Some(Reference { cell: b12, abs_col: false, abs_row: true }));
    assert_eq!(parse_reference("$B$12"), Some(Reference { cell: b12, abs_col: true, abs_row: true }));
    assert_eq!(parse_reference("$xfd$1048576").map(|r| r.cell), Some(CellRef::new(MAX_COLS - 1, MAX_ROWS - 1)));
    for bad in ["", "$", "$$A1", "A$$1", "$A$", "A1$", "$1", "B12$", "$B$0", "SUM", "A1:B2", " $A$1"] {
        assert_eq!(parse_reference(bad), None, "parse_reference({bad:?})");
    }
}

#[test]
fn parse_range_normalises() {
    let want = Range { start: CellRef::new(0, 0), end: CellRef::new(1, 6) };
    assert_eq!(parse_range("A1:B7"), Some(want));
    assert_eq!(parse_range("B7:A1"), Some(want));
    assert_eq!(parse_range("B1:A7"), Some(want));
    assert_eq!(parse_range("a7:b1"), Some(want));
    assert_eq!(parse_range("C3"), Some(Range::single(CellRef::new(2, 2))));
    assert_eq!(parse_range("C3:C3"), Some(Range::single(CellRef::new(2, 2))));
    for bad in ["", ":", "A1:", ":B2", "A1:B2:C3", "A1 : B2", "$A$1:B2", "A:B", "1:2", "A1;B2"] {
        assert_eq!(parse_range(bad), None, "parse_range({bad:?})");
    }
}

#[test]
fn formatting() {
    assert_eq!(format_cell(CellRef::new(0, 0)), "A1");
    assert_eq!(format_cell(CellRef::new(1, 11)), "B12");
    assert_eq!(format_cell(CellRef::new(MAX_COLS - 1, MAX_ROWS - 1)), "XFD1048576");
    let b12 = CellRef::new(1, 11);
    assert_eq!(format_reference(Reference::relative(b12)), "B12");
    assert_eq!(format_reference(Reference { cell: b12, abs_col: true, abs_row: false }), "$B12");
    assert_eq!(format_reference(Reference { cell: b12, abs_col: false, abs_row: true }), "B$12");
    assert_eq!(format_reference(Reference { cell: b12, abs_col: true, abs_row: true }), "$B$12");
    assert_eq!(format_range(Range::new(CellRef::new(1, 6), CellRef::new(0, 0))), "A1:B7");
    assert_eq!(format_range(Range::single(b12)), "B12");
}

#[test]
fn parse_and_format_round_trip() {
    for s in ["A1", "Z9", "AA10", "$C$3", "C$3", "$C3", "XFD1048576"] {
        let r = parse_reference(s).unwrap_or_else(|| panic!("{s} should parse"));
        assert_eq!(format_reference(r), s);
    }
    for s in ["A1:B7", "C3:XFD9", "D4"] {
        assert_eq!(format_range(parse_range(s).unwrap()), s);
    }
}
