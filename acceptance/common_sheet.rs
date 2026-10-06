// Shared helpers for acceptance tests that work on whole sheets (included with `include!`).
// Outside every task's write scope.

#[allow(unused_imports)]
use gridrs::sheet::Sheet;
#[allow(unused_imports)]
use gridrs::types::{CellRef, ErrorKind, NumFormat, Range, Value};

/// "B12" -> CellRef.
#[allow(dead_code)]
fn at(name: &str) -> CellRef {
    let letters: String = name.chars().take_while(|ch| ch.is_ascii_alphabetic()).collect();
    let digits = &name[letters.len()..];
    let col = letters.bytes().fold(0u32, |acc, b| acc * 26 + (b.to_ascii_uppercase() - b'A') as u32 + 1) - 1;
    CellRef::new(col, digits.parse::<u32>().unwrap() - 1)
}
/// "A1:B7" -> Range.
#[allow(dead_code)]
fn range(name: &str) -> Range {
    let (a, b) = name.split_once(':').unwrap_or((name, name));
    Range::new(at(a), at(b))
}
#[allow(dead_code)]
fn num(x: f64) -> Value {
    Value::Number(x)
}
#[allow(dead_code)]
fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}
#[allow(dead_code)]
fn err(e: ErrorKind) -> Value {
    Value::Error(e)
}
/// Build a sheet from (address, input) pairs.
#[allow(dead_code)]
fn sheet(cells: &[(&str, &str)]) -> Sheet {
    let mut s = Sheet::new();
    for (name, input) in cells {
        s.set(at(name), input);
    }
    s
}
#[allow(dead_code)]
fn val(s: &Sheet, name: &str) -> Value {
    s.value(at(name))
}
#[allow(dead_code)]
fn input(s: &Sheet, name: &str) -> String {
    s.input(at(name))
}
/// All non-empty inputs as ("A1", "input") pairs, row-major.
#[allow(dead_code)]
fn dump(s: &Sheet) -> Vec<(String, String)> {
    s.cells().into_iter().map(|(c, i)| (name_of(c), i)).collect()
}
#[allow(dead_code)]
fn name_of(c: CellRef) -> String {
    let mut col = c.col + 1;
    let mut letters = String::new();
    while col > 0 {
        letters.insert(0, (b'A' + ((col - 1) % 26) as u8) as char);
        col = (col - 1) / 26;
    }
    format!("{letters}{}", c.row + 1)
}
#[allow(dead_code)]
fn pairs(data: &[(&str, &str)]) -> Vec<(String, String)> {
    data.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}
