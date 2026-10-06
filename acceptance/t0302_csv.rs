//! Acceptance test for task 0302-csv. Outside every task's write scope.

use gridrs::io::csv::{parse, write};

fn rows(data: &[&[&str]]) -> Vec<Vec<String>> {
    data.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect()
}

fn ok(text: &str) -> Vec<Vec<String>> {
    parse(text).unwrap_or_else(|e| panic!("parse({text:?}) failed: {e}"))
}

#[test]
fn plain_records() {
    assert_eq!(ok("a,b,c\n1,2,3\n"), rows(&[&["a", "b", "c"], &["1", "2", "3"]]));
    assert_eq!(ok("a,b,c\n1,2,3"), rows(&[&["a", "b", "c"], &["1", "2", "3"]]), "the final newline is optional");
    assert_eq!(ok("single"), rows(&[&["single"]]));
    assert_eq!(ok("single\n"), rows(&[&["single"]]));
}

#[test]
fn empty_input_has_no_records() {
    assert_eq!(ok(""), rows(&[]));
}

#[test]
fn crlf_and_lf_line_endings() {
    assert_eq!(ok("a,b\r\nc,d\r\n"), rows(&[&["a", "b"], &["c", "d"]]));
    assert_eq!(ok("a,b\r\nc,d"), rows(&[&["a", "b"], &["c", "d"]]));
    assert_eq!(ok("a,b\nc,d\r\ne,f\n"), rows(&[&["a", "b"], &["c", "d"], &["e", "f"]]), "mixed endings");
    assert_eq!(ok("a\rb,c\n"), rows(&[&["a\rb", "c"]]), "a lone CR is data, not a line ending");
}

#[test]
fn empty_fields_and_blank_lines() {
    assert_eq!(ok(",\n"), rows(&[&["", ""]]));
    assert_eq!(ok("a,,c\n,,\n"), rows(&[&["a", "", "c"], &["", "", ""]]));
    assert_eq!(ok("a,\n"), rows(&[&["a", ""]]));
    assert_eq!(ok("a\n\nb\n"), rows(&[&["a"], &[""], &["b"]]), "a blank line is a record with one empty field");
    assert_eq!(ok("\n"), rows(&[&[""]]));
    assert_eq!(ok("\r\n\r\n"), rows(&[&[""], &[""]]));
    assert_eq!(ok("a\n\n"), rows(&[&["a"], &[""]]));
}

#[test]
fn ragged_rows_are_kept_as_they_are() {
    assert_eq!(ok("a\nb,c,d\ne,f\n"), rows(&[&["a"], &["b", "c", "d"], &["e", "f"]]));
}

#[test]
fn whitespace_is_data() {
    assert_eq!(ok(" a , b \n"), rows(&[&[" a ", " b "]]));
    assert_eq!(ok("a\tb\n"), rows(&[&["a\tb"]]));
}

#[test]
fn quoted_fields() {
    assert_eq!(ok("\"a\",\"b\"\n"), rows(&[&["a", "b"]]));
    assert_eq!(ok("\"a,b\",c\n"), rows(&[&["a,b", "c"]]), "comma inside quotes");
    assert_eq!(ok("\"say \"\"hi\"\"\",x\n"), rows(&[&["say \"hi\"", "x"]]), "doubled quote is one quote");
    assert_eq!(ok("\"\"\n"), rows(&[&[""]]));
    assert_eq!(ok("\"\",\"\"\n"), rows(&[&["", ""]]));
    assert_eq!(ok("\"\"\"\"\n"), rows(&[&["\""]]));
    assert_eq!(ok("\"line1\nline2\",x\n"), rows(&[&["line1\nline2", "x"]]), "newline inside quotes");
    assert_eq!(ok("\"line1\r\nline2\"\r\nnext\r\n"), rows(&[&["line1\r\nline2"], &["next"]]), "CRLF inside quotes is kept verbatim");
    assert_eq!(ok("x,\"last\""), rows(&[&["x", "last"]]), "quoted field at end of input");
    assert_eq!(ok("\" padded \"\n"), rows(&[&[" padded "]]));
    assert_eq!(ok("\"=SUM(A1,B1)\",2\n"), rows(&[&["=SUM(A1,B1)", "2"]]));
}

#[test]
fn a_quote_inside_an_unquoted_field_is_literal() {
    assert_eq!(ok("5\" pipe,x\n"), rows(&[&["5\" pipe", "x"]]));
    assert_eq!(ok("a\"b\"c\n"), rows(&[&["a\"b\"c"]]));
    assert_eq!(ok(" \"x\"\n"), rows(&[&[" \"x\""]]), "a field is only quoted if the quote is its very first character");
}

#[test]
fn utf8_and_bom() {
    assert_eq!(ok("naïve,日本語,✓\n"), rows(&[&["naïve", "日本語", "✓"]]));
    assert_eq!(ok("\u{feff}a,b\n"), rows(&[&["a", "b"]]), "a leading byte-order mark is dropped");
    assert_eq!(ok("\u{feff}\"a\",b\n"), rows(&[&["a", "b"]]));
    assert_eq!(ok("\u{feff}"), rows(&[]));
    assert_eq!(ok("a,\u{feff}b\n"), rows(&[&["a", "\u{feff}b"]]), "only at the very start");
}

#[test]
fn syntax_errors_report_the_line() {
    let e = parse("a,b\n\"unterminated,c\nd\n").unwrap_err();
    assert_eq!(e.line, 2, "the line where the quoted field started");
    assert!(!e.message.is_empty());

    let e = parse("\"closed\"x,y\n").unwrap_err();
    assert_eq!(e.line, 1, "text after a closing quote");

    let e = parse("ok\nok\n\"a\" ,b\n").unwrap_err();
    assert_eq!(e.line, 3);

    let e = parse("\"multi\nline\",ok\n\"bad\"!\n").unwrap_err();
    assert_eq!(e.line, 3, "physical lines, counting newlines inside quoted fields");

    assert!(parse("\"").is_err());
    assert!(parse("a,\"").is_err());
    assert!(parse("\"a\"\"").is_err());
}

#[test]
fn writing() {
    assert_eq!(write(&rows(&[])), "");
    assert_eq!(write(&rows(&[&["a", "b", "c"], &["1", "2", "3"]])), "a,b,c\n1,2,3\n");
    assert_eq!(write(&rows(&[&["a", "", "c"]])), "a,,c\n");
    assert_eq!(write(&rows(&[&["", ""]])), ",\n");
    assert_eq!(write(&rows(&[&["a"], &["b", "c", "d"]])), "a\nb,c,d\n", "ragged rows are not padded");
}

#[test]
fn writing_quotes_only_when_needed() {
    assert_eq!(write(&rows(&[&["a,b", "c"]])), "\"a,b\",c\n");
    assert_eq!(write(&rows(&[&["say \"hi\""]])), "\"say \"\"hi\"\"\"\n");
    assert_eq!(write(&rows(&[&["line1\nline2", "x"]])), "\"line1\nline2\",x\n");
    assert_eq!(write(&rows(&[&["cr\rhere"]])), "\"cr\rhere\"\n");
    assert_eq!(write(&rows(&[&[" padded ", "tab\there", "semi;colon", "'apostrophe'"]])), " padded ,tab\there,semi;colon,'apostrophe'\n");
    assert_eq!(write(&rows(&[&["=SUM(A1,B1)", "=A1+1"]])), "\"=SUM(A1,B1)\",=A1+1\n");
    assert_eq!(write(&rows(&[&["\""]])), "\"\"\"\"\n");
}

#[test]
fn writing_a_record_with_one_empty_field() {
    // must not be an empty line followed by nothing ambiguous: it round-trips as a blank line
    assert_eq!(write(&rows(&[&["a"], &[""], &["b"]])), "a\n\nb\n");
}

#[test]
fn round_trip() {
    let cases: Vec<Vec<Vec<String>>> = vec![
        rows(&[&["a", "b"], &["c", "d"]]),
        rows(&[&["", "", ""], &["x"]]),
        rows(&[&["with,comma", "with\"quote", "with\nnewline", "with\r\ncrlf"], &["\"", "\"\"", ",", "\n"]]),
        rows(&[&[" lead", "trail ", "\ttab"], &["日本語", "naïve ✓", "=IF(A1>0,\"y\",\"n\")"]]),
        rows(&[&[""], &[""], &["end"]]),
        rows(&[&["#N/A", "'quoted", "-1.5e3", "100%"]]),
    ];
    for data in cases {
        let text = write(&data);
        assert_eq!(ok(&text), data, "round trip through {text:?}");
    }
}

#[test]
fn round_trip_random() {
    let pieces = ["a", "b", ",", "\"", "\n", "\r", " ", "é", "", "=1", "\"\"", "x,y"];
    let mut seed: u64 = 42;
    let mut next = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    for _ in 0..500 {
        let mut data = vec![];
        for _ in 0..(1 + next() % 4) {
            let mut record = vec![];
            for _ in 0..(1 + next() % 4) {
                let mut field = String::new();
                for _ in 0..(next() % 4) {
                    field.push_str(pieces[next() % pieces.len()]);
                }
                record.push(field);
            }
            data.push(record);
        }
        let text = write(&data);
        assert_eq!(ok(&text), data, "round trip through {text:?}");
    }
}
