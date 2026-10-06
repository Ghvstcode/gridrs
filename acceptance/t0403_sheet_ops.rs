//! Acceptance test for task 0403-sheet-ops (copy/paste, insert/delete rows and columns).
//! Outside every task's write scope.

include!("common_sheet.rs");

use gridrs::ops::{apply_structural, copy, paste};
use gridrs::types::{Clip, ClipCell, StructuralEdit::*, DEFAULT_COL_WIDTH, MAX_COLS, MAX_ROWS};

// ----------------------------------------------------------------------------
// copy
// ----------------------------------------------------------------------------

#[test]
fn copy_captures_inputs_and_formats_relative_to_the_corner() {
    let mut s = sheet(&[("B2", "1"), ("C2", "=B2*2"), ("B3", "text"), ("D9", "outside")]);
    s.set_format(at("C2"), NumFormat::Fixed(2));
    s.set_format(at("C3"), NumFormat::Percent(0)); // format on an empty cell
    let clip = copy(&s, range("B2:C3"));
    assert_eq!(
        clip,
        Clip {
            origin: at("B2"),
            rows: 2,
            cols: 2,
            cells: vec![
                ClipCell { drow: 0, dcol: 0, input: "1".into(), format: NumFormat::General },
                ClipCell { drow: 0, dcol: 1, input: "=B2*2".into(), format: NumFormat::Fixed(2) },
                ClipCell { drow: 1, dcol: 0, input: "text".into(), format: NumFormat::General },
                ClipCell { drow: 1, dcol: 1, input: "".into(), format: NumFormat::Percent(0) },
            ],
        }
    );
}

#[test]
fn copy_of_empty_cells() {
    let s = sheet(&[("A1", "x")]);
    assert_eq!(copy(&s, range("C3:D5")), Clip { origin: at("C3"), rows: 3, cols: 2, cells: vec![] });
    assert_eq!(copy(&s, range("A1")), Clip { origin: at("A1"), rows: 1, cols: 1, cells: vec![ClipCell { drow: 0, dcol: 0, input: "x".into(), format: NumFormat::General }] });
}

#[test]
fn copy_of_a_huge_range_only_visits_what_exists() {
    let s = sheet(&[("B2", "x")]);
    let whole = Range::new(CellRef::new(0, 0), CellRef::new(MAX_COLS - 1, MAX_ROWS - 1));
    let clip = copy(&s, whole);
    assert_eq!((clip.rows, clip.cols), (MAX_ROWS, MAX_COLS));
    assert_eq!(clip.cells, vec![ClipCell { drow: 1, dcol: 1, input: "x".into(), format: NumFormat::General }]);
}

// ----------------------------------------------------------------------------
// paste
// ----------------------------------------------------------------------------

#[test]
fn paste_literals_verbatim() {
    let mut s = sheet(&[("A1", " 7 "), ("A2", "'=not a formula"), ("A3", "text")]);
    let clip = copy(&s, range("A1:A3"));
    paste(&mut s, &clip, at("C5"));
    assert_eq!(input(&s, "C5"), " 7 ");
    assert_eq!(input(&s, "C6"), "'=not a formula");
    assert_eq!(input(&s, "C7"), "text");
    assert_eq!(input(&s, "A1"), " 7 ", "the source is untouched");
}

#[test]
fn paste_shifts_relative_references() {
    let mut s = sheet(&[("A1", "1"), ("A2", "2"), ("B1", "10"), ("B2", "20"), ("C1", "=A1+B1")]);
    let clip = copy(&s, range("C1"));
    paste(&mut s, &clip, at("C2"));
    assert_eq!(input(&s, "C2"), "=A2+B2");
    assert_eq!(val(&s, "C2"), num(22.0));
    paste(&mut s, &clip, at("D1"));
    assert_eq!(input(&s, "D1"), "=B1+C1");
    assert_eq!(val(&s, "D1"), num(21.0));
}

#[test]
fn paste_respects_dollar_signs() {
    let mut s = sheet(&[("A1", "=$B$1+B$1+$B1+B1+SUM($B$1:B1)")]);
    let clip = copy(&s, range("A1"));
    paste(&mut s, &clip, at("C3")); // +2 columns, +2 rows
    assert_eq!(input(&s, "C3"), "=$B$1+D$1+$B3+D3+SUM($B$1:D3)");
}

#[test]
fn paste_off_the_edge_turns_references_into_ref_errors() {
    let mut s = sheet(&[("B2", "=A1+B1+A2"), ("C2", "=SUM(A1:B2)")]);
    let clip = copy(&s, range("B2:C2"));
    paste(&mut s, &clip, at("A1")); // -1 column, -1 row
    assert_eq!(input(&s, "A1"), "=#REF!+#REF!+#REF!");
    assert_eq!(input(&s, "B1"), "=SUM(#REF!)");
    assert_eq!(val(&s, "A1"), err(ErrorKind::Ref));
}

#[test]
fn paste_prints_shifted_formulas_canonically_but_keeps_unshifted_ones_verbatim() {
    let mut s = sheet(&[("A1", "= sum( a2:a3 ) * ( 1 + 2 )"), ("B1", "= $a$2 +  1"), ("C1", "=not valid (("), ("D1", "= 1 + 2")]);
    let clip = copy(&s, range("A1:D1"));
    paste(&mut s, &clip, at("A5"));
    assert_eq!(input(&s, "A5"), "=SUM(A6:A7)*(1+2)", "shifted: reprinted");
    assert_eq!(input(&s, "B5"), "= $a$2 +  1", "nothing moved: exactly as typed");
    assert_eq!(input(&s, "C5"), "=not valid ((", "does not parse: exactly as typed");
    assert_eq!(input(&s, "D5"), "= 1 + 2", "no references: exactly as typed");
}

#[test]
fn paste_in_place_changes_nothing() {
    let mut s = sheet(&[("A1", "= a2 + 1"), ("A2", "5")]);
    let clip = copy(&s, range("A1:A2"));
    paste(&mut s, &clip, at("A1"));
    assert_eq!(dump(&s), pairs(&[("A1", "= a2 + 1"), ("A2", "5")]));
}

#[test]
fn paste_overwrites_the_whole_destination_rectangle() {
    let mut s = sheet(&[("A1", "a"), ("B2", "d"), ("D1", "old1"), ("E1", "old2"), ("D2", "old3"), ("E2", "old4"), ("F1", "keep")]);
    s.set_format(at("E1"), NumFormat::Fixed(3));
    s.set_format(at("B2"), NumFormat::Percent(1));
    let clip = copy(&s, range("A1:B2")); // B1 and A2 are empty
    paste(&mut s, &clip, at("D1"));
    assert_eq!(input(&s, "D1"), "a");
    assert_eq!(input(&s, "E1"), "", "an empty source cell clears the destination");
    assert_eq!(input(&s, "D2"), "");
    assert_eq!(input(&s, "E2"), "d");
    assert_eq!(input(&s, "F1"), "keep");
    assert_eq!(s.format(at("E1")), NumFormat::General, "and resets its format");
    assert_eq!(s.format(at("E2")), NumFormat::Percent(1), "formats travel with the cells");
}

#[test]
fn paste_overlapping_its_own_source() {
    // fill-down by one: A1:A3 pasted at A2
    let mut s = sheet(&[("A1", "1"), ("A2", "=A1+1"), ("A3", "=A2+1")]);
    let clip = copy(&s, range("A1:A3"));
    paste(&mut s, &clip, at("A2"));
    assert_eq!(dump(&s), pairs(&[("A1", "1"), ("A2", "1"), ("A3", "=A2+1"), ("A4", "=A3+1")]));
    assert_eq!(val(&s, "A4"), num(3.0));
}

#[test]
fn paste_clips_at_the_sheet_edge() {
    let mut s = sheet(&[("A1", "a"), ("B1", "b"), ("A2", "c"), ("B2", "d")]);
    let clip = copy(&s, range("A1:B2"));
    let corner = CellRef::new(MAX_COLS - 1, MAX_ROWS - 1);
    paste(&mut s, &clip, corner);
    assert_eq!(s.input(corner), "a", "only the part that fits is pasted");
    assert_eq!(s.cells().len(), 5);
    // on the last row but not the last column: the top row of the clip fits, the bottom row does not
    let mut s = sheet(&[("A1", "a"), ("B1", "b"), ("A2", "c"), ("B2", "d")]);
    let clip = copy(&s, range("A1:B2"));
    paste(&mut s, &clip, CellRef::new(5, MAX_ROWS - 1));
    assert_eq!(s.input(CellRef::new(5, MAX_ROWS - 1)), "a");
    assert_eq!(s.input(CellRef::new(6, MAX_ROWS - 1)), "b");
    assert_eq!(s.cells().len(), 6);
}

#[test]
fn paste_recalculates_dependents() {
    let mut s = sheet(&[("A1", "1"), ("B1", "100"), ("C1", "=B1*2")]);
    let clip = copy(&s, range("A1"));
    paste(&mut s, &clip, at("B1"));
    assert_eq!(val(&s, "C1"), num(2.0));
}

// ----------------------------------------------------------------------------
// insert rows / columns
// ----------------------------------------------------------------------------

fn ledger() -> Sheet {
    // A        B
    // 1 item   amount
    // 2 rent   1000
    // 3 food   300
    // 4 fun    50
    // 5 total  =SUM(B2:B4)      C5 =B5/B2   D1 =$B$3   E1 = b2 + 0
    sheet(&[
        ("A1", "item"), ("B1", "amount"), ("D1", "=$B$3"), ("E1", "= b2 + 0"),
        ("A2", "rent"), ("B2", "1000"),
        ("A3", "food"), ("B3", "300"),
        ("A4", "fun"), ("B4", "50"),
        ("A5", "total"), ("B5", "=SUM(B2:B4)"), ("C5", "=B5/B2"),
    ])
}

#[test]
fn insert_rows_moves_cells_and_rewrites_formulas() {
    let mut s = ledger();
    apply_structural(&mut s, InsertRows { at: 2, count: 2 }); // two new rows above "food"
    assert_eq!(
        dump(&s),
        pairs(&[
            ("A1", "item"), ("B1", "amount"), ("D1", "=$B$5"), ("E1", "= b2 + 0"),
            ("A2", "rent"), ("B2", "1000"),
            ("A5", "food"), ("B5", "300"),
            ("A6", "fun"), ("B6", "50"),
            ("A7", "total"), ("B7", "=SUM(B2:B6)"), ("C7", "=B7/B2"),
        ])
    );
    assert_eq!(val(&s, "B7"), num(1350.0));
    assert_eq!(val(&s, "D1"), num(300.0));
    // the new rows are usable and inside the SUM range
    s.set(at("B3"), "25");
    assert_eq!(val(&s, "B7"), num(1375.0));
}

#[test]
fn a_formula_that_did_not_change_keeps_its_exact_text() {
    let mut s = ledger();
    apply_structural(&mut s, InsertRows { at: 2, count: 1 });
    assert_eq!(input(&s, "E1"), "= b2 + 0", "B2 is above the insertion: untouched, not reprinted");
    apply_structural(&mut s, InsertRows { at: 0, count: 1 });
    assert_eq!(input(&s, "E2"), "=B3+0", "now it moved: reprinted canonically");
}

#[test]
fn insert_cols_moves_cells_and_rewrites_formulas() {
    let mut s = ledger();
    apply_structural(&mut s, InsertCols { at: 1, count: 1 }); // new column B
    assert_eq!(input(&s, "C5"), "=SUM(C2:C4)");
    assert_eq!(input(&s, "D5"), "=C5/C2");
    assert_eq!(input(&s, "E1"), "=$C$3");
    assert_eq!(input(&s, "F1"), "=C2+0");
    assert_eq!(input(&s, "A3"), "food");
    assert_eq!(input(&s, "B3"), "");
    assert_eq!(input(&s, "C3"), "300");
    assert_eq!(val(&s, "C5"), num(1350.0));
    assert_eq!(s.cells().len(), 13);
}

#[test]
fn delete_rows_removes_cells_shrinks_ranges_and_breaks_direct_references() {
    let mut s = ledger();
    apply_structural(&mut s, DeleteRows { at: 2, count: 1 }); // delete "food"
    assert_eq!(
        dump(&s),
        pairs(&[
            ("A1", "item"), ("B1", "amount"), ("D1", "=#REF!"), ("E1", "= b2 + 0"),
            ("A2", "rent"), ("B2", "1000"),
            ("A3", "fun"), ("B3", "50"),
            ("A4", "total"), ("B4", "=SUM(B2:B3)"), ("C4", "=B4/B2"),
        ])
    );
    assert_eq!(val(&s, "B4"), num(1050.0));
    assert_eq!(val(&s, "D1"), err(ErrorKind::Ref));
}

#[test]
fn delete_cols() {
    let mut s = ledger();
    apply_structural(&mut s, DeleteCols { at: 0, count: 1 }); // delete the labels
    assert_eq!(
        dump(&s),
        pairs(&[("A1", "amount"), ("C1", "=$A$3"), ("D1", "=A2+0"), ("A2", "1000"), ("A3", "300"), ("A4", "50"), ("A5", "=SUM(A2:A4)"), ("B5", "=A5/A2")])
    );
    assert_eq!(val(&s, "B5"), num(1.35));
    let mut s = ledger();
    apply_structural(&mut s, DeleteCols { at: 1, count: 1 }); // delete the amounts
    assert_eq!(input(&s, "B5"), "=#REF!/#REF!");
    assert_eq!(input(&s, "C1"), "=#REF!");
    assert_eq!(input(&s, "A5"), "total");
}

#[test]
fn insert_then_delete_is_the_identity_on_values() {
    let mut s = ledger();
    apply_structural(&mut s, InsertRows { at: 3, count: 4 });
    apply_structural(&mut s, DeleteRows { at: 3, count: 4 });
    apply_structural(&mut s, InsertCols { at: 0, count: 2 });
    apply_structural(&mut s, DeleteCols { at: 0, count: 2 });
    let fresh = ledger();
    for (cell, _) in fresh.cells() {
        assert_eq!(s.value(cell), fresh.value(cell), "{}", name_of(cell));
    }
    assert_eq!(s.cells().len(), fresh.cells().len());
}

#[test]
fn unparseable_formulas_and_text_are_moved_untouched() {
    let mut s = sheet(&[("A2", "=oops (("), ("A3", "'=A1"), ("A4", "A1")]);
    apply_structural(&mut s, InsertRows { at: 0, count: 1 });
    assert_eq!(dump(&s), pairs(&[("A3", "=oops (("), ("A4", "'=A1"), ("A5", "A1")]));
}

#[test]
fn formats_move_with_their_cells() {
    let mut s = sheet(&[("A1", "1"), ("A3", "3")]);
    s.set_format(at("A1"), NumFormat::Fixed(1));
    s.set_format(at("A3"), NumFormat::Fixed(3));
    s.set_format(at("C4"), NumFormat::Percent(0));
    apply_structural(&mut s, InsertRows { at: 1, count: 2 });
    assert_eq!(s.formats(), vec![(at("A1"), NumFormat::Fixed(1)), (at("A5"), NumFormat::Fixed(3)), (at("C6"), NumFormat::Percent(0))]);
    apply_structural(&mut s, DeleteRows { at: 4, count: 1 });
    assert_eq!(s.formats(), vec![(at("A1"), NumFormat::Fixed(1)), (at("C5"), NumFormat::Percent(0))], "a deleted row takes its formats with it");
    apply_structural(&mut s, InsertCols { at: 0, count: 1 });
    assert_eq!(s.formats(), vec![(at("B1"), NumFormat::Fixed(1)), (at("D5"), NumFormat::Percent(0))]);
    apply_structural(&mut s, DeleteCols { at: 1, count: 2 });
    assert_eq!(s.formats(), vec![(at("B5"), NumFormat::Percent(0))]);
}

#[test]
fn column_widths_move_with_their_columns() {
    let mut s = Sheet::new();
    s.set_col_width(0, 5);
    s.set_col_width(2, 20);
    s.set_col_width(3, 30);
    apply_structural(&mut s, InsertCols { at: 1, count: 2 });
    assert_eq!(s.col_widths(), vec![(0, 5), (4, 20), (5, 30)]);
    assert_eq!(s.col_width(1), DEFAULT_COL_WIDTH, "new columns have the default width");
    apply_structural(&mut s, DeleteCols { at: 4, count: 1 });
    assert_eq!(s.col_widths(), vec![(0, 5), (4, 30)]);
    // row edits never touch widths
    apply_structural(&mut s, InsertRows { at: 0, count: 3 });
    apply_structural(&mut s, DeleteRows { at: 0, count: 1 });
    assert_eq!(s.col_widths(), vec![(0, 5), (4, 30)]);
}

#[test]
fn cells_pushed_off_the_sheet_are_dropped() {
    let last_row = MAX_ROWS - 1;
    let last_col = MAX_COLS - 1;
    let mut s = Sheet::new();
    s.set(CellRef::new(0, last_row), "bottom");
    s.set(CellRef::new(last_col, 0), "right");
    s.set(at("A1"), "=A1048576&\"!\"");
    apply_structural(&mut s, InsertRows { at: 5, count: 1 });
    assert_eq!(s.input(CellRef::new(0, last_row)), "");
    assert_eq!(input(&s, "A1"), "=#REF!&\"!\"");
    apply_structural(&mut s, InsertCols { at: 5, count: 1 });
    assert_eq!(s.input(CellRef::new(last_col, 0)), "");
    assert_eq!(s.cells().len(), 1);
}

#[test]
fn zero_count_and_out_of_range_edits_are_no_ops() {
    let fresh = ledger();
    for edit in [
        InsertRows { at: 2, count: 0 },
        DeleteRows { at: 2, count: 0 },
        InsertCols { at: 1, count: 0 },
        DeleteCols { at: 1, count: 0 },
        InsertRows { at: MAX_ROWS, count: 3 },
        DeleteRows { at: MAX_ROWS, count: 3 },
        InsertCols { at: MAX_COLS + 7, count: 3 },
        DeleteCols { at: MAX_COLS, count: 3 },
    ] {
        let mut s = ledger();
        apply_structural(&mut s, edit);
        assert_eq!(dump(&s), dump(&fresh), "{edit:?}");
    }
}

#[test]
fn deleting_everything_leaves_an_empty_sheet() {
    let mut s = ledger();
    s.set_format(at("B2"), NumFormat::Fixed(2));
    apply_structural(&mut s, DeleteRows { at: 0, count: u32::MAX });
    assert_eq!(s.cells(), vec![]);
    assert_eq!(s.formats(), vec![]);
    assert_eq!(s.used_range(), None);
}

#[test]
fn structural_edits_recalculate() {
    let mut s = sheet(&[("A1", "1"), ("A2", "2"), ("A3", "=A1+A2"), ("B1", "=A3*10")]);
    apply_structural(&mut s, DeleteRows { at: 1, count: 1 });
    assert_eq!(input(&s, "A2"), "=A1+#REF!");
    assert_eq!(val(&s, "A2"), err(ErrorKind::Ref));
    assert_eq!(input(&s, "B1"), "=A2*10");
    assert_eq!(val(&s, "B1"), err(ErrorKind::Ref));
}
