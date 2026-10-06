//! Acceptance test for task 0303-native-format. Outside every task's write scope.

use gridrs::io::native::{decode, encode};
use gridrs::types::{CellRef, Document, NumFormat};

fn c(col: u32, row: u32) -> CellRef {
    CellRef::new(col, row)
}

fn sample() -> Document {
    Document {
        cells: vec![
            (c(0, 0), "Item".to_string()),
            (c(1, 0), "Price".to_string()),
            (c(0, 1), "Widget, large".to_string()),
            (c(1, 1), "1234.5".to_string()),
            (c(1, 2), "=SUM(B2:B2)".to_string()),
            (c(27, 99), "'007".to_string()),
        ],
        formats: vec![(c(1, 1), NumFormat::Thousands(2)), (c(1, 2), NumFormat::Fixed(0)), (c(2, 2), NumFormat::Percent(1))],
        col_widths: vec![(0, 18), (27, 4)],
    }
}

const SAMPLE_TEXT: &str = "\
gridrs 1
width A 18
width AB 4
cell A1 Item
cell B1 Price
cell A2 Widget, large
cell B2 1234.5
cell B3 =SUM(B2:B2)
cell AB100 '007
format B2 thousands 2
format B3 fixed 0
format C3 percent 1
";

#[test]
fn encode_empty_document() {
    assert_eq!(encode(&Document::default()), "gridrs 1\n");
}

#[test]
fn encode_sample_exactly() {
    assert_eq!(encode(&sample()), SAMPLE_TEXT);
}

#[test]
fn decode_sample_exactly() {
    assert_eq!(decode(SAMPLE_TEXT), Ok(sample()));
    assert_eq!(decode("gridrs 1\n"), Ok(Document::default()));
    assert_eq!(decode("gridrs 1"), Ok(Document::default()), "the final newline is optional");
}

#[test]
fn encode_keeps_the_given_order_within_each_section() {
    let doc = Document {
        cells: vec![(c(1, 1), "second".into()), (c(0, 0), "first".into())],
        formats: vec![(c(5, 5), NumFormat::Fixed(1)), (c(0, 0), NumFormat::Fixed(2))],
        col_widths: vec![(3, 5), (1, 7)],
    };
    assert_eq!(encode(&doc), "gridrs 1\nwidth D 5\nwidth B 7\ncell B2 second\ncell A1 first\nformat F6 fixed 1\nformat A1 fixed 2\n");
    assert_eq!(decode(&encode(&doc)), Ok(doc));
}

#[test]
fn cell_text_is_escaped() {
    let doc = Document {
        cells: vec![
            (c(0, 0), "two\nlines".into()),
            (c(0, 1), "back\\slash".into()),
            (c(0, 2), "cr\rhere".into()),
            (c(0, 3), "  leading and trailing  ".into()),
            (c(0, 4), "\\n is not a newline".into()),
            (c(0, 5), "tab\tstays".into()),
            (c(0, 6), "# not a comment".into()),
            (c(0, 7), "ünïcödé ✓".into()),
        ],
        ..Document::default()
    };
    let text = encode(&doc);
    assert_eq!(
        text,
        "gridrs 1\n\
         cell A1 two\\nlines\n\
         cell A2 back\\\\slash\n\
         cell A3 cr\\rhere\n\
         cell A4   leading and trailing  \n\
         cell A5 \\\\n is not a newline\n\
         cell A6 tab\tstays\n\
         cell A7 # not a comment\n\
         cell A8 ünïcödé ✓\n"
    );
    assert_eq!(decode(&text), Ok(doc));
}

#[test]
fn decode_accepts_any_order_blank_lines_comments_and_crlf() {
    let text = "gridrs 1\r\n\r\n# a comment\r\nformat A1 percent 0\r\ncell A1 0.5\r\n\r\nwidth A 12\r\ncell B1 x\r\n";
    let want = Document { cells: vec![(c(0, 0), "0.5".into()), (c(1, 0), "x".into())], formats: vec![(c(0, 0), NumFormat::Percent(0))], col_widths: vec![(0, 12)] };
    assert_eq!(decode(text), Ok(want));
}

#[test]
fn decode_is_lenient_about_case_of_addresses_only() {
    assert_eq!(decode("gridrs 1\ncell b2 x\nwidth c 5\n"), Ok(Document { cells: vec![(c(1, 1), "x".into())], formats: vec![], col_widths: vec![(2, 5)] }));
    assert!(decode("gridrs 1\nCELL A1 x\n").is_err());
    assert!(decode("gridrs 1\nformat A1 Fixed 2\n").is_err());
}

#[test]
fn decode_general_format_is_allowed() {
    assert_eq!(decode("gridrs 1\nformat A1 general\n"), Ok(Document { cells: vec![], formats: vec![(c(0, 0), NumFormat::General)], col_widths: vec![] }));
}

#[test]
fn a_cell_line_with_nothing_after_the_address_is_an_empty_input() {
    assert_eq!(decode("gridrs 1\ncell A1 \n"), Ok(Document { cells: vec![(c(0, 0), String::new())], ..Document::default() }));
    assert_eq!(decode("gridrs 1\ncell A1\n"), Ok(Document { cells: vec![(c(0, 0), String::new())], ..Document::default() }));
}

#[test]
fn header_errors() {
    for bad in ["", "\n", "gridrs", "gridrs 2\n", "gridrs 1 extra\n", "GRIDRS 1\n", " gridrs 1\n", "cell A1 x\n", "# comment\ngridrs 1\n", "a,b,c\n1,2,3\n"] {
        let e = decode(bad).expect_err(&format!("decode({bad:?}) should fail"));
        assert!(e.starts_with("line 1:"), "{bad:?} -> {e:?}");
    }
}

#[test]
fn body_errors_name_the_line() {
    for (bad_line, why) in [
        ("bogus A1 x", "unknown directive"),
        ("cell", "missing address"),
        ("cell 11 x", "bad address"),
        ("cell A0 x", "bad address"),
        ("cell $A$1 x", "bad address"),
        ("cell A1:B2 x", "bad address"),
        ("cell A1 bad\\escape", "unknown escape"),
        ("cell A1 trailing backslash\\", "dangling backslash"),
        ("width A", "missing width"),
        ("width A 0", "width out of range"),
        ("width A 61", "width out of range"),
        ("width A ten", "bad width"),
        ("width A +5", "bad width: digits only"),
        ("width A 99999999999999999999999", "width out of range"),
        ("width A1 10", "bad column"),
        ("width A 10 20", "extra field"),
        ("format A1", "missing format"),
        ("format A1 fixed", "bad format"),
        ("format A1 fixed 16", "bad format"),
        ("format A1 currency 2", "bad format"),
        ("format A 1 fixed 2", "bad address"),
        (" cell A1 x", "leading space"),
    ] {
        let text = format!("gridrs 1\ncell A1 fine\n\n{bad_line}\ncell A2 fine\n");
        let e = decode(&text).expect_err(&format!("{bad_line:?} should fail: {why}"));
        assert!(e.starts_with("line 4:"), "{bad_line:?} ({why}) -> {e:?}");
    }
}

#[test]
fn round_trip() {
    let mut doc = sample();
    doc.cells.push((c(16383, 1048575), "last cell".into()));
    doc.cells.push((c(3, 3), "\\\\\n\r\\n".into()));
    doc.cells.push((c(3, 4), " ".into()));
    doc.formats.push((c(16383, 1048575), NumFormat::Percent(15)));
    doc.col_widths.push((16383, 60));
    doc.col_widths.push((5, 1));
    assert_eq!(decode(&encode(&doc)), Ok(doc));
}
