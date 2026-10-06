//! Acceptance test for task 0103-reference-rewriting. Outside every task's write scope.
//!
//! Trees are built by hand so this task does not depend on the parser.

use gridrs::formula::refs::{adjust, precedents, translate};
use gridrs::types::{BinaryOp, CellRef, ErrorKind, Expr, Range, Reference, StructuralEdit::*, UnaryOp, MAX_COLS, MAX_ROWS};

fn c(col: u32, row: u32) -> CellRef {
    CellRef::new(col, row)
}
/// Relative reference.
fn r(col: u32, row: u32) -> Reference {
    Reference::relative(c(col, row))
}
/// Reference with explicit `$` flags.
fn ra(col: u32, row: u32, abs_col: bool, abs_row: bool) -> Reference {
    Reference { cell: c(col, row), abs_col, abs_row }
}
fn cell(col: u32, row: u32) -> Expr {
    Expr::Ref(r(col, row))
}
fn range(a: Reference, b: Reference) -> Expr {
    Expr::Range(a, b)
}
fn add(a: Expr, b: Expr) -> Expr {
    Expr::Binary(BinaryOp::Add, Box::new(a), Box::new(b))
}
fn sum(args: Vec<Expr>) -> Expr {
    Expr::Call("SUM".to_string(), args)
}
const REF_ERR: Expr = Expr::Error(ErrorKind::Ref);

// ----------------------------------------------------------------------------
// precedents
// ----------------------------------------------------------------------------

#[test]
fn precedents_of_literals_is_empty() {
    assert_eq!(precedents(&Expr::Number(1.0)), vec![]);
    assert_eq!(precedents(&add(Expr::Text("A1".into()), Expr::Bool(true))), vec![]);
    assert_eq!(precedents(&sum(vec![Expr::Error(ErrorKind::Ref)])), vec![]);
}

#[test]
fn precedents_in_source_order_with_duplicates() {
    // -B2 + SUM(A1:A3, C1%, B2) & $D$4
    let e = Expr::Binary(
        BinaryOp::Concat,
        Box::new(add(
            Expr::Unary(UnaryOp::Neg, Box::new(cell(1, 1))),
            sum(vec![range(r(0, 0), r(0, 2)), Expr::Unary(UnaryOp::Percent, Box::new(cell(2, 0))), cell(1, 1)]),
        )),
        Box::new(Expr::Ref(ra(3, 3, true, true))),
    );
    assert_eq!(
        precedents(&e),
        vec![
            Range::single(c(1, 1)),
            Range { start: c(0, 0), end: c(0, 2) },
            Range::single(c(2, 0)),
            Range::single(c(1, 1)),
            Range::single(c(3, 3)),
        ]
    );
}

#[test]
fn precedents_normalises_ranges() {
    // B7:A1 and A7:B1 are both A1:B7
    let want = Range { start: c(0, 0), end: c(1, 6) };
    assert_eq!(precedents(&range(r(1, 6), r(0, 0))), vec![want]);
    assert_eq!(precedents(&range(r(0, 6), r(1, 0))), vec![want]);
}

// ----------------------------------------------------------------------------
// translate (copy/paste)
// ----------------------------------------------------------------------------

#[test]
fn translate_moves_relative_parts_only() {
    assert_eq!(translate(&cell(0, 0), 2, 3), cell(2, 3));
    assert_eq!(translate(&cell(5, 5), -2, -3), cell(3, 2));
    assert_eq!(translate(&Expr::Ref(ra(1, 1, true, false)), 2, 3), Expr::Ref(ra(1, 4, true, false)));
    assert_eq!(translate(&Expr::Ref(ra(1, 1, false, true)), 2, 3), Expr::Ref(ra(3, 1, false, true)));
    assert_eq!(translate(&Expr::Ref(ra(1, 1, true, true)), 2, 3), Expr::Ref(ra(1, 1, true, true)));
    assert_eq!(translate(&cell(4, 4), 0, 0), cell(4, 4));
}

#[test]
fn translate_recurses_and_keeps_everything_else() {
    // IF(A1>0, SUM($B$1:B3), "A1") moved by (+1, +2)
    let before = Expr::Call(
        "IF".into(),
        vec![
            Expr::Binary(BinaryOp::Gt, Box::new(cell(0, 0)), Box::new(Expr::Number(0.0))),
            sum(vec![range(ra(1, 0, true, true), r(1, 2))]),
            Expr::Text("A1".into()),
        ],
    );
    let after = Expr::Call(
        "IF".into(),
        vec![
            Expr::Binary(BinaryOp::Gt, Box::new(cell(1, 2)), Box::new(Expr::Number(0.0))),
            sum(vec![range(ra(1, 0, true, true), r(2, 4))]),
            Expr::Text("A1".into()),
        ],
    );
    assert_eq!(translate(&before, 1, 2), after);
}

#[test]
fn translate_keeps_range_endpoints_as_written() {
    // B7:A1 stays "backwards"
    assert_eq!(translate(&range(r(1, 6), r(0, 0)), 1, 1), range(r(2, 7), r(1, 1)));
}

#[test]
fn translate_off_the_sheet_becomes_ref_error() {
    assert_eq!(translate(&cell(0, 0), -1, 0), REF_ERR);
    assert_eq!(translate(&cell(0, 0), 0, -1), REF_ERR);
    assert_eq!(translate(&cell(MAX_COLS - 1, 0), 1, 0), REF_ERR);
    assert_eq!(translate(&cell(0, MAX_ROWS - 1), 0, 1), REF_ERR);
    assert_eq!(translate(&cell(MAX_COLS - 2, MAX_ROWS - 2), 1, 1), cell(MAX_COLS - 1, MAX_ROWS - 1));
    // absolute parts never move, so they never fall off
    assert_eq!(translate(&Expr::Ref(ra(0, 0, true, true)), -5, -5), Expr::Ref(ra(0, 0, true, true)));
    // a range with either end off the sheet is replaced as a whole
    assert_eq!(translate(&range(r(0, 0), r(3, 3)), -1, 0), REF_ERR);
    assert_eq!(translate(&range(r(2, 2), r(3, MAX_ROWS - 1)), 0, 1), REF_ERR);
    // only the broken reference is replaced, not the whole formula
    assert_eq!(translate(&add(cell(0, 0), cell(5, 5)), -1, -1), add(REF_ERR, cell(4, 4)));
    assert_eq!(translate(&sum(vec![range(r(0, 0), r(1, 1)), cell(2, 2)]), 0, -1), sum(vec![REF_ERR, cell(2, 1)]));
    // huge offsets must not overflow
    assert_eq!(translate(&cell(3, 3), i64::MAX, i64::MIN), REF_ERR);
}

// ----------------------------------------------------------------------------
// adjust (insert/delete rows and columns)
// ----------------------------------------------------------------------------

#[test]
fn insert_rows_shifts_references_at_or_below() {
    let e = InsertRows { at: 2, count: 3 };
    assert_eq!(adjust(&cell(0, 1), e), cell(0, 1), "above the insertion: unchanged");
    assert_eq!(adjust(&cell(0, 2), e), cell(0, 5), "at the insertion point: pushed down");
    assert_eq!(adjust(&cell(7, 9), e), cell(7, 12));
    // `$` makes no difference for structural edits
    assert_eq!(adjust(&Expr::Ref(ra(0, 2, true, true)), e), Expr::Ref(ra(0, 5, true, true)));
}

#[test]
fn insert_cols_shifts_references_at_or_right() {
    let e = InsertCols { at: 1, count: 2 };
    assert_eq!(adjust(&cell(0, 4), e), cell(0, 4));
    assert_eq!(adjust(&cell(1, 4), e), cell(3, 4));
    assert_eq!(adjust(&Expr::Ref(ra(5, 0, true, false)), e), Expr::Ref(ra(7, 0, true, false)));
}

#[test]
fn insert_inside_a_range_grows_it() {
    // SUM(A1:A5), insert 2 rows before row 3 -> SUM(A1:A7)
    assert_eq!(adjust(&range(r(0, 0), r(0, 4)), InsertRows { at: 2, count: 2 }), range(r(0, 0), r(0, 6)));
    // insert before the range -> whole range moves
    assert_eq!(adjust(&range(r(0, 2), r(0, 4)), InsertRows { at: 0, count: 1 }), range(r(0, 3), r(0, 5)));
    // insert after the range -> unchanged
    assert_eq!(adjust(&range(r(0, 0), r(0, 4)), InsertRows { at: 5, count: 1 }), range(r(0, 0), r(0, 4)));
    assert_eq!(adjust(&range(r(0, 0), r(3, 0)), InsertCols { at: 2, count: 1 }), range(r(0, 0), r(4, 0)));
    // a row edit never changes columns and vice versa
    assert_eq!(adjust(&range(r(2, 0), r(4, 9)), InsertCols { at: 0, count: 1 }), range(r(3, 0), r(5, 9)));
}

#[test]
fn insert_pushing_a_reference_off_the_sheet_is_ref_error() {
    assert_eq!(adjust(&cell(0, MAX_ROWS - 1), InsertRows { at: 0, count: 1 }), REF_ERR);
    assert_eq!(adjust(&cell(MAX_COLS - 1, 0), InsertCols { at: 0, count: 1 }), REF_ERR);
    assert_eq!(adjust(&range(r(0, 0), r(0, MAX_ROWS - 1)), InsertRows { at: 5, count: 1 }), REF_ERR);
    assert_eq!(adjust(&cell(0, 10), InsertRows { at: 0, count: u32::MAX }), REF_ERR, "must not overflow");
    assert_eq!(adjust(&cell(0, MAX_ROWS - 2), InsertRows { at: 0, count: 1 }), cell(0, MAX_ROWS - 1));
}

#[test]
fn delete_rows_single_references() {
    let e = DeleteRows { at: 2, count: 3 }; // deletes rows 3, 4, 5 (indices 2, 3, 4)
    assert_eq!(adjust(&cell(0, 1), e), cell(0, 1));
    assert_eq!(adjust(&cell(0, 2), e), REF_ERR);
    assert_eq!(adjust(&cell(0, 4), e), REF_ERR);
    assert_eq!(adjust(&cell(0, 5), e), cell(0, 2));
    assert_eq!(adjust(&cell(9, 20), e), cell(9, 17));
    assert_eq!(adjust(&Expr::Ref(ra(0, 3, true, true)), e), REF_ERR);
    assert_eq!(adjust(&Expr::Ref(ra(0, 6, false, true)), e), Expr::Ref(ra(0, 3, false, true)));
}

#[test]
fn delete_cols_single_references() {
    let e = DeleteCols { at: 1, count: 1 };
    assert_eq!(adjust(&cell(0, 0), e), cell(0, 0));
    assert_eq!(adjust(&cell(1, 0), e), REF_ERR);
    assert_eq!(adjust(&cell(2, 0), e), cell(1, 0));
}

#[test]
fn delete_rows_shrinks_ranges() {
    let a1_a10 = range(r(0, 0), r(0, 9));
    // rows strictly inside
    assert_eq!(adjust(&a1_a10, DeleteRows { at: 3, count: 2 }), range(r(0, 0), r(0, 7)));
    // rows after the range
    assert_eq!(adjust(&a1_a10, DeleteRows { at: 10, count: 5 }), a1_a10);
    // rows before a range move it up
    assert_eq!(adjust(&range(r(0, 5), r(0, 9)), DeleteRows { at: 0, count: 2 }), range(r(0, 3), r(0, 7)));
    // deleting the top of the range: start clamps to the first surviving row
    assert_eq!(adjust(&range(r(0, 2), r(0, 9)), DeleteRows { at: 1, count: 3 }), range(r(0, 1), r(0, 6)));
    // deleting the bottom of the range: end clamps to the last surviving row
    assert_eq!(adjust(&range(r(0, 2), r(0, 9)), DeleteRows { at: 8, count: 5 }), range(r(0, 2), r(0, 7)));
    // deleting exactly the range, or more: nothing left
    assert_eq!(adjust(&range(r(0, 2), r(0, 4)), DeleteRows { at: 2, count: 3 }), REF_ERR);
    assert_eq!(adjust(&range(r(0, 2), r(0, 4)), DeleteRows { at: 0, count: 100 }), REF_ERR);
    // one row survives
    assert_eq!(adjust(&range(r(0, 2), r(0, 4)), DeleteRows { at: 3, count: 50 }), range(r(0, 2), r(0, 2)));
    assert_eq!(adjust(&range(r(0, 2), r(0, 4)), DeleteRows { at: 0, count: 4 }), range(r(0, 0), r(0, 0)));
    // huge counts must not overflow
    assert_eq!(adjust(&a1_a10, DeleteRows { at: 5, count: u32::MAX }), range(r(0, 0), r(0, 4)));
}

#[test]
fn delete_cols_shrinks_ranges() {
    let b2_e2 = range(r(1, 1), r(4, 1));
    assert_eq!(adjust(&b2_e2, DeleteCols { at: 2, count: 1 }), range(r(1, 1), r(3, 1)));
    assert_eq!(adjust(&b2_e2, DeleteCols { at: 0, count: 2 }), range(r(0, 1), r(2, 1)));
    assert_eq!(adjust(&b2_e2, DeleteCols { at: 1, count: 4 }), REF_ERR);
    assert_eq!(adjust(&b2_e2, DeleteRows { at: 1, count: 1 }), REF_ERR, "its only row is gone");
    assert_eq!(adjust(&b2_e2, DeleteRows { at: 0, count: 1 }), range(r(1, 0), r(4, 0)));
}

#[test]
fn ranges_written_backwards_keep_their_shape_and_flags() {
    // A10:A1 (start is the *larger* row) with rows 3..=4 inserted before index 2
    assert_eq!(adjust(&range(r(0, 9), r(0, 0)), InsertRows { at: 2, count: 2 }), range(r(0, 11), r(0, 0)));
    // A10:A1, delete indices 8..=20: the endpoint with the larger row clamps to index 7
    assert_eq!(adjust(&range(r(0, 9), r(0, 0)), DeleteRows { at: 8, count: 13 }), range(r(0, 7), r(0, 0)));
    // flags travel with their endpoint
    let before = range(ra(0, 9, true, false), ra(3, 0, false, true));
    let after = range(ra(0, 7, true, false), ra(3, 0, false, true));
    assert_eq!(adjust(&before, DeleteRows { at: 8, count: 13 }), after);
}

#[test]
fn adjust_recurses_and_replaces_only_what_broke() {
    // A1 + SUM(B2:B4, C3) * D9, delete row index 2 (row 3)
    let before = add(cell(0, 0), Expr::Binary(BinaryOp::Mul, Box::new(sum(vec![range(r(1, 1), r(1, 3)), cell(2, 2)])), Box::new(cell(3, 8))));
    let after = add(cell(0, 0), Expr::Binary(BinaryOp::Mul, Box::new(sum(vec![range(r(1, 1), r(1, 2)), REF_ERR])), Box::new(cell(3, 7))));
    assert_eq!(adjust(&before, DeleteRows { at: 2, count: 1 }), after);
    let lit = add(Expr::Number(1.0), Expr::Text("A1".into()));
    assert_eq!(adjust(&lit, DeleteRows { at: 0, count: 9 }), lit);
}

#[test]
fn zero_count_edits_change_nothing() {
    let e = add(cell(0, 0), range(r(1, 1), r(2, 2)));
    for edit in [InsertRows { at: 0, count: 0 }, DeleteRows { at: 0, count: 0 }, InsertCols { at: 1, count: 0 }, DeleteCols { at: 1, count: 0 }] {
        assert_eq!(adjust(&e, edit), e);
    }
}
