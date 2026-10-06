//! Acceptance test for task 0402-sheet. Outside every task's write scope.

use gridrs::sheet::Sheet;
use gridrs::types::{CellRef, Context, ErrorKind, NumFormat, Range, Value, DEFAULT_COL_WIDTH, MAX_COLS, MAX_COL_WIDTH, MAX_ROWS, MIN_COL_WIDTH};

/// "B12" -> CellRef, without depending on the address parser being correct for exotic input.
fn at(name: &str) -> CellRef {
    let letters: String = name.chars().take_while(|ch| ch.is_ascii_alphabetic()).collect();
    let digits = &name[letters.len()..];
    let col = letters.bytes().fold(0u32, |acc, b| acc * 26 + (b.to_ascii_uppercase() - b'A') as u32 + 1) - 1;
    CellRef::new(col, digits.parse::<u32>().unwrap() - 1)
}
fn num(x: f64) -> Value {
    Value::Number(x)
}
fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}
fn err(e: ErrorKind) -> Value {
    Value::Error(e)
}
fn sheet(cells: &[(&str, &str)]) -> Sheet {
    let mut s = Sheet::new();
    for (name, input) in cells {
        s.set(at(name), input);
    }
    s
}
fn val(s: &Sheet, name: &str) -> Value {
    s.value(at(name))
}

#[test]
fn a_new_sheet_is_empty() {
    let s = Sheet::new();
    assert_eq!(s.value(at("A1")), Value::Empty);
    assert_eq!(s.input(at("A1")), "");
    assert_eq!(s.cells(), vec![]);
    assert_eq!(s.used_range(), None);
    assert_eq!(s.format(at("A1")), NumFormat::General);
    assert_eq!(s.formats(), vec![]);
    assert_eq!(s.col_width(0), DEFAULT_COL_WIDTH);
    assert_eq!(s.col_widths(), vec![]);
    let d = Sheet::default();
    assert_eq!(d.cells(), vec![]);
}

#[test]
fn literals_are_classified() {
    let s = sheet(&[("A1", "42"), ("A2", "hello"), ("A3", "TRUE"), ("A4", "12%"), ("A5", "'42"), ("A6", " 7 "), ("A7", "#N/A"), ("A8", "=")]);
    assert_eq!(val(&s, "A1"), num(42.0));
    assert_eq!(val(&s, "A2"), text("hello"));
    assert_eq!(val(&s, "A3"), Value::Bool(true));
    assert_eq!(val(&s, "A4"), num(0.12));
    assert_eq!(val(&s, "A5"), text("42"));
    assert_eq!(val(&s, "A6"), num(7.0));
    assert_eq!(val(&s, "A7"), err(ErrorKind::NA));
    assert_eq!(val(&s, "A8"), text("="), "a lone = is just text");
}

#[test]
fn inputs_are_kept_exactly_as_typed() {
    let s = sheet(&[("A1", " 7 "), ("A2", "'42"), ("A3", "= sum( a1 , 1 )"), ("A4", "=nonsense((")]);
    assert_eq!(s.input(at("A1")), " 7 ");
    assert_eq!(s.input(at("A2")), "'42");
    assert_eq!(s.input(at("A3")), "= sum( a1 , 1 )");
    assert_eq!(s.input(at("A4")), "=nonsense((");
    assert_eq!(val(&s, "A3"), num(8.0));
}

#[test]
fn formulas_are_evaluated() {
    let s = sheet(&[("A1", "2"), ("A2", "3"), ("B1", "=A1+A2"), ("B2", "=A1*A2^2"), ("B3", "=A1&\"-\"&A2"), ("B4", "=A1>A2"), ("B5", "=-A1%"), ("B6", "=1/0"), ("B7", "=Z99")]);
    assert_eq!(val(&s, "B1"), num(5.0));
    assert_eq!(val(&s, "B2"), num(18.0));
    assert_eq!(val(&s, "B3"), text("2-3"));
    assert_eq!(val(&s, "B4"), Value::Bool(false));
    assert_eq!(val(&s, "B5"), num(-0.02));
    assert_eq!(val(&s, "B6"), err(ErrorKind::Div0));
    assert_eq!(val(&s, "B7"), num(0.0), "a reference to an empty cell shows 0");
}

#[test]
fn functions_and_ranges_work_through_the_sheet() {
    let s = sheet(&[("A1", "1"), ("A2", "2"), ("A3", "3"), ("B1", "x"), ("C1", "=SUM(A1:A3)"), ("C2", "=sum(A1:B3, 10)"), ("C3", "=MAX(A1:A3)-MIN(A1:A3)"), ("C4", "=NOSUCHFUNCTION(1)")]);
    assert_eq!(val(&s, "C1"), num(6.0));
    assert_eq!(val(&s, "C2"), num(16.0));
    assert_eq!(val(&s, "C3"), num(2.0));
    assert_eq!(val(&s, "C4"), err(ErrorKind::Name));
}

#[test]
fn unparseable_formulas_are_name_errors() {
    let s = sheet(&[("A1", "=1+"), ("A2", "=foo"), ("A3", "=A0"), ("A4", "=\"open"), ("A5", "=A1+1")]);
    for name in ["A1", "A2", "A3", "A4"] {
        assert_eq!(val(&s, name), err(ErrorKind::Name), "{name}");
    }
    assert_eq!(val(&s, "A5"), err(ErrorKind::Name), "and the error propagates to readers");
}

#[test]
fn dependents_recalculate_when_a_cell_changes() {
    let mut s = sheet(&[("A1", "1"), ("B1", "=A1*10"), ("C1", "=B1+1"), ("D1", "=SUM(A1:C1)")]);
    assert_eq!(val(&s, "D1"), num(22.0));
    s.set(at("A1"), "2");
    assert_eq!(val(&s, "B1"), num(20.0));
    assert_eq!(val(&s, "C1"), num(21.0));
    assert_eq!(val(&s, "D1"), num(43.0));
    s.set(at("A1"), "");
    assert_eq!(val(&s, "A1"), Value::Empty);
    assert_eq!(val(&s, "B1"), num(0.0));
    assert_eq!(val(&s, "D1"), num(1.0));
    s.set(at("A1"), "text");
    assert_eq!(val(&s, "B1"), err(ErrorKind::Value));
    assert_eq!(val(&s, "C1"), err(ErrorKind::Value));
    assert_eq!(val(&s, "D1"), err(ErrorKind::Value));
}

#[test]
fn formulas_can_be_entered_before_their_inputs() {
    let mut s = sheet(&[("C1", "=B1+1"), ("B1", "=A1+1")]);
    assert_eq!(val(&s, "C1"), num(2.0));
    s.set(at("A1"), "10");
    assert_eq!(val(&s, "C1"), num(12.0));
}

#[test]
fn replacing_a_formula_rewires_dependencies() {
    let mut s = sheet(&[("A1", "1"), ("A2", "100"), ("B1", "=A1")]);
    s.set(at("B1"), "=A2");
    assert_eq!(val(&s, "B1"), num(100.0));
    s.set(at("A1"), "5");
    assert_eq!(val(&s, "B1"), num(100.0));
    s.set(at("A2"), "200");
    assert_eq!(val(&s, "B1"), num(200.0));
    s.set(at("B1"), "7"); // formula replaced by a literal
    s.set(at("A2"), "300");
    assert_eq!(val(&s, "B1"), num(7.0));
    assert_eq!(s.input(at("B1")), "7");
}

#[test]
fn cycles_are_detected_and_healed() {
    let mut s = sheet(&[("A1", "=B1"), ("B1", "=C1+1"), ("C1", "=A1"), ("D1", "=A1*2"), ("E1", "5"), ("F1", "=E1+1")]);
    for name in ["A1", "B1", "C1", "D1"] {
        assert_eq!(val(&s, name), err(ErrorKind::Cycle), "{name}");
    }
    assert_eq!(val(&s, "F1"), num(6.0), "unrelated cells are fine");
    // break the cycle
    s.set(at("C1"), "10");
    assert_eq!(val(&s, "C1"), num(10.0));
    assert_eq!(val(&s, "B1"), num(11.0));
    assert_eq!(val(&s, "A1"), num(11.0));
    assert_eq!(val(&s, "D1"), num(22.0));
    // and make a new one
    s.set(at("C1"), "=D1");
    for name in ["A1", "B1", "C1", "D1"] {
        assert_eq!(val(&s, name), err(ErrorKind::Cycle), "{name}");
    }
    s.set(at("C1"), "");
    assert_eq!(val(&s, "D1"), num(2.0));
}

#[test]
fn self_reference_and_range_cycles() {
    let mut s = sheet(&[("A1", "=A1+1"), ("B2", "=SUM(B1:B3)"), ("C1", "1"), ("C2", "=SUM(C1:C1)+C3"), ("C3", "=C2")]);
    assert_eq!(val(&s, "A1"), err(ErrorKind::Cycle));
    assert_eq!(val(&s, "B2"), err(ErrorKind::Cycle));
    assert_eq!(val(&s, "C2"), err(ErrorKind::Cycle));
    assert_eq!(val(&s, "C3"), err(ErrorKind::Cycle));
    s.set(at("C3"), "4");
    assert_eq!(val(&s, "C2"), num(5.0));
}

#[test]
fn cycle_errors_are_not_caught_by_iferror_like_logic() {
    // A cyclic cell is #CYCLE! even if evaluating it would not "need" the cycle.
    let s = sheet(&[("A1", "=A1*0")]);
    assert_eq!(val(&s, "A1"), err(ErrorKind::Cycle));
}

#[test]
fn apply_sets_many_cells_and_recalculates_once_consistently() {
    let mut s = sheet(&[("A1", "1"), ("B1", "=A1+A2+A3")]);
    s.apply(&[(at("A2"), "10".to_string()), (at("A3"), "100".to_string()), (at("C1"), "=B1*2".to_string()), (at("A1"), "".to_string())]);
    assert_eq!(val(&s, "B1"), num(110.0));
    assert_eq!(val(&s, "C1"), num(220.0));
    assert_eq!(s.input(at("A1")), "");
    // later edits to the same cell win
    s.apply(&[(at("A1"), "5".to_string()), (at("A1"), "6".to_string())]);
    assert_eq!(val(&s, "B1"), num(116.0));
    // an empty batch is fine
    s.apply(&[]);
    assert_eq!(val(&s, "C1"), num(232.0));
}

#[test]
fn apply_can_swap_two_formulas_without_a_phantom_cycle() {
    let mut s = sheet(&[("A1", "=B1"), ("B1", "1")]);
    s.apply(&[(at("A1"), "2".to_string()), (at("B1"), "=A1".to_string())]);
    assert_eq!(val(&s, "A1"), num(2.0));
    assert_eq!(val(&s, "B1"), num(2.0));
}

#[test]
fn cells_and_used_range() {
    let mut s = sheet(&[("C3", "x"), ("A2", "1"), ("B2", "=A2"), ("E1", "y")]);
    assert_eq!(s.cells(), vec![(at("E1"), "y".to_string()), (at("A2"), "1".to_string()), (at("B2"), "=A2".to_string()), (at("C3"), "x".to_string())], "row-major");
    assert_eq!(s.used_range(), Some(Range { start: at("A1"), end: at("E3") }));
    s.set(at("E1"), "");
    s.set(at("A2"), "");
    assert_eq!(s.cells(), vec![(at("B2"), "=A2".to_string()), (at("C3"), "x".to_string())]);
    assert_eq!(s.used_range(), Some(Range { start: at("B2"), end: at("C3") }));
    s.set(at("B2"), "");
    s.set(at("C3"), "");
    assert_eq!(s.used_range(), None);
    assert_eq!(s.cells(), vec![]);
}

#[test]
fn the_corners_of_the_sheet_work() {
    let last = CellRef::new(MAX_COLS - 1, MAX_ROWS - 1);
    let mut s = Sheet::new();
    s.set(last, "9");
    s.set(at("A1"), "=XFD1048576+1");
    assert_eq!(s.value(at("A1")), num(10.0));
    assert_eq!(s.used_range(), Some(Range { start: at("A1"), end: last }));
}

#[test]
fn a_sum_over_a_whole_column_is_usable() {
    let mut s = sheet(&[("A1", "1"), ("A500", "2"), ("B1", "=SUM(A1:A1000)")]);
    assert_eq!(val(&s, "B1"), num(3.0));
    s.set(at("A999"), "4");
    assert_eq!(val(&s, "B1"), num(7.0));
    s.set(at("A1001"), "100");
    assert_eq!(val(&s, "B1"), num(7.0));
}

#[test]
fn formats() {
    let mut s = sheet(&[("A1", "1.5")]);
    s.set_format(at("A1"), NumFormat::Fixed(2));
    s.set_format(at("C3"), NumFormat::Percent(0)); // formatting an empty cell is allowed
    s.set_format(at("B1"), NumFormat::Thousands(1));
    assert_eq!(s.format(at("A1")), NumFormat::Fixed(2));
    assert_eq!(s.format(at("A2")), NumFormat::General);
    assert_eq!(s.formats(), vec![(at("A1"), NumFormat::Fixed(2)), (at("B1"), NumFormat::Thousands(1)), (at("C3"), NumFormat::Percent(0))], "row-major");
    assert_eq!(s.value(at("A1")), num(1.5), "a format never changes the value");
    // a format is not content
    assert_eq!(s.cells(), vec![(at("A1"), "1.5".to_string())]);
    assert_eq!(s.used_range(), Some(Range::single(at("A1"))));
    // clearing the input keeps the format; setting General removes it
    s.set(at("A1"), "");
    assert_eq!(s.format(at("A1")), NumFormat::Fixed(2));
    s.set_format(at("A1"), NumFormat::General);
    s.set_format(at("B1"), NumFormat::General);
    assert_eq!(s.formats(), vec![(at("C3"), NumFormat::Percent(0))]);
}

#[test]
fn column_widths() {
    let mut s = Sheet::new();
    s.set_col_width(2, 20);
    s.set_col_width(0, 4);
    assert_eq!(s.col_width(2), 20);
    assert_eq!(s.col_width(1), DEFAULT_COL_WIDTH);
    assert_eq!(s.col_widths(), vec![(0, 4), (2, 20)]);
    s.set_col_width(0, 0);
    assert_eq!(s.col_width(0), MIN_COL_WIDTH, "clamped up");
    s.set_col_width(2, 9999);
    assert_eq!(s.col_width(2), MAX_COL_WIDTH, "clamped down");
    s.set_col_width(2, DEFAULT_COL_WIDTH);
    assert_eq!(s.col_widths(), vec![(0, MIN_COL_WIDTH)], "a default width is not listed");
}

#[test]
fn clone_is_a_deep_independent_snapshot() {
    let mut s = sheet(&[("A1", "1"), ("B1", "=A1+1")]);
    s.set_format(at("A1"), NumFormat::Fixed(1));
    s.set_col_width(0, 5);
    let snapshot = s.clone();
    s.set(at("A1"), "10");
    s.set(at("C1"), "new");
    s.set_format(at("A1"), NumFormat::General);
    s.set_col_width(0, 50);
    assert_eq!(snapshot.value(at("A1")), num(1.0));
    assert_eq!(snapshot.value(at("B1")), num(2.0));
    assert_eq!(snapshot.input(at("C1")), "");
    assert_eq!(snapshot.format(at("A1")), NumFormat::Fixed(1));
    assert_eq!(snapshot.col_width(0), 5);
    // and the snapshot is still a working sheet
    let mut restored = snapshot.clone();
    restored.set(at("A1"), "5");
    assert_eq!(restored.value(at("B1")), num(6.0));
    assert_eq!(s.value(at("B1")), num(11.0));
}

#[test]
fn the_sheet_is_an_evaluation_context() {
    let s = sheet(&[("A1", "3"), ("A2", "4")]);
    let ctx: &dyn Context = &s;
    assert_eq!(ctx.cell(at("A1")), num(3.0));
    assert_eq!(ctx.cell(at("Z9")), Value::Empty);
    let expr = gridrs::formula::parser::parse("SUM(A1:A2)*2").unwrap();
    assert_eq!(gridrs::formula::eval::eval(&expr, ctx), num(14.0));
}

#[test]
fn many_cells_and_a_long_dependency_chain() {
    // A1 = 1, A(n) = A(n-1) + 1 for 3000 rows, entered top-down in one batch
    let mut edits = vec![(at("A1"), "1".to_string())];
    for row in 1..3000u32 {
        edits.push((CellRef::new(0, row), format!("=A{row}+1")));
    }
    let mut s = Sheet::new();
    s.apply(&edits);
    assert_eq!(s.value(CellRef::new(0, 2999)), num(3000.0));
    s.set(at("A1"), "101");
    assert_eq!(s.value(CellRef::new(0, 2999)), num(3100.0));
    assert_eq!(s.cells().len(), 3000);
}
