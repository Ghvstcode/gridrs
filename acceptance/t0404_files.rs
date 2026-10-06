//! Acceptance test for task 0404-files. Outside every task's write scope.

include!("common_sheet.rs");

use gridrs::io::files::*;
use gridrs::types::{CsvMode, Document, FileKind};
use std::path::PathBuf;

/// A fresh scratch directory under the system temp dir, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("gridrs-t0404-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
    fn path(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().to_string()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn budget() -> Sheet {
    let mut s = sheet(&[
        ("A1", "item"), ("B1", "amount"),
        ("A2", "rent, monthly"), ("B2", "1000"),
        ("A3", "food"), ("B3", "300.5"),
        ("A5", "total"), ("B5", "=SUM(B2:B3)"), ("C5", "=B3/B2"),
        ("A6", "'007"), ("B6", "say \"hi\""),
    ]);
    s.set_format(at("B5"), NumFormat::Thousands(2));
    s.set_format(at("C5"), NumFormat::Percent(1));
    s.set_format(at("D9"), NumFormat::Fixed(0));
    s.set_col_width(0, 16);
    s.set_col_width(3, 4);
    s
}

#[test]
fn kind_is_chosen_by_extension() {
    assert_eq!(kind_for_path("budget.grid"), FileKind::Native);
    assert_eq!(kind_for_path("/some/dir/Budget.GRID"), FileKind::Native);
    assert_eq!(kind_for_path("a.b.grid"), FileKind::Native);
    assert_eq!(kind_for_path("budget.csv"), FileKind::Csv);
    assert_eq!(kind_for_path("budget.CSV"), FileKind::Csv);
    assert_eq!(kind_for_path("budget.txt"), FileKind::Csv);
    assert_eq!(kind_for_path("budget"), FileKind::Csv);
    assert_eq!(kind_for_path("grid"), FileKind::Csv);
    assert_eq!(kind_for_path("budget.grid.csv"), FileKind::Csv);
    assert_eq!(kind_for_path("dir.grid/budget"), FileKind::Csv, "only the file name counts");
    assert_eq!(kind_for_path(""), FileKind::Csv);
    assert_eq!(kind_for_path("C:\\work\\budget.grid"), FileKind::Native, "backslash is a separator too");
    assert_eq!(kind_for_path("C:\\work.grid\\budget"), FileKind::Csv);
    assert_eq!(kind_for_path(".grid"), FileKind::Native, "the name ends with .grid");
}

#[test]
fn csv_import() {
    let s = sheet_from_csv("item,amount\nrent,1000\nfood,\"300.5\"\n\ntotal,\"=SUM(B2:B3)\"\n").unwrap();
    assert_eq!(dump(&s), pairs(&[("A1", "item"), ("B1", "amount"), ("A2", "rent"), ("B2", "1000"), ("A3", "food"), ("B3", "300.5"), ("A5", "total"), ("B5", "=SUM(B2:B3)")]));
    assert_eq!(val(&s, "B2"), num(1000.0), "fields are interpreted like typed input");
    assert_eq!(val(&s, "B5"), num(1300.5), "formulas in a CSV are live");
    assert_eq!(val(&s, "A1"), text("item"));
}

#[test]
fn csv_import_edge_cases() {
    assert_eq!(sheet_from_csv("").unwrap().cells(), vec![]);
    assert_eq!(sheet_from_csv("\n\n").unwrap().cells(), vec![]);
    let s = sheet_from_csv("a\n,,c\n").unwrap();
    assert_eq!(dump(&s), pairs(&[("A1", "a"), ("C2", "c")]), "ragged rows and empty fields");
    let s = sheet_from_csv("'007,\" padded \"\r\n").unwrap();
    assert_eq!(input(&s, "A1"), "'007");
    assert_eq!(val(&s, "A1"), text("007"));
    assert_eq!(val(&s, "B1"), text(" padded "));
}

#[test]
fn csv_import_reports_syntax_errors() {
    let e = sheet_from_csv("a,b\n\"unterminated\n").unwrap_err();
    assert!(e.contains("line 2"), "{e:?}");
}

#[test]
fn csv_export_inputs() {
    assert_eq!(
        sheet_to_csv(&budget(), CsvMode::Inputs),
        "item,amount,\n\"rent, monthly\",1000,\nfood,300.5,\n,,\ntotal,=SUM(B2:B3),=B3/B2\n'007,\"say \"\"hi\"\"\",\n"
    );
}

#[test]
fn csv_export_values_uses_each_cells_format() {
    assert_eq!(
        sheet_to_csv(&budget(), CsvMode::Values),
        "item,amount,\n\"rent, monthly\",1000,\nfood,300.5,\n,,\ntotal,\"1,300.50\",30.1%\n007,\"say \"\"hi\"\"\",\n"
    );
}

#[test]
fn csv_export_always_starts_at_a1_and_is_rectangular() {
    let s = sheet(&[("C3", "x"), ("B2", "=1/0")]);
    assert_eq!(sheet_to_csv(&s, CsvMode::Inputs), ",,\n,=1/0,\n,,x\n");
    assert_eq!(sheet_to_csv(&s, CsvMode::Values), ",,\n,#DIV/0!,\n,,x\n");
    assert_eq!(sheet_to_csv(&Sheet::new(), CsvMode::Inputs), "");
    assert_eq!(sheet_to_csv(&Sheet::new(), CsvMode::Values), "");
    // a format on an empty cell is not content
    let mut s = Sheet::new();
    s.set_format(at("B2"), NumFormat::Fixed(2));
    assert_eq!(sheet_to_csv(&s, CsvMode::Values), "");
}

#[test]
fn csv_inputs_round_trip() {
    let original = budget();
    let back = sheet_from_csv(&sheet_to_csv(&original, CsvMode::Inputs)).unwrap();
    assert_eq!(dump(&back), dump(&original));
    for (cell, _) in original.cells() {
        assert_eq!(back.value(cell), original.value(cell));
    }
}

#[test]
fn document_conversion() {
    let doc = sheet_to_document(&budget());
    assert_eq!(
        doc,
        Document {
            cells: vec![
                (at("A1"), "item".into()), (at("B1"), "amount".into()),
                (at("A2"), "rent, monthly".into()), (at("B2"), "1000".into()),
                (at("A3"), "food".into()), (at("B3"), "300.5".into()),
                (at("A5"), "total".into()), (at("B5"), "=SUM(B2:B3)".into()), (at("C5"), "=B3/B2".into()),
                (at("A6"), "'007".into()), (at("B6"), "say \"hi\"".into()),
            ],
            formats: vec![(at("B5"), NumFormat::Thousands(2)), (at("C5"), NumFormat::Percent(1)), (at("D9"), NumFormat::Fixed(0))],
            col_widths: vec![(0, 16), (3, 4)],
        }
    );
    let back = sheet_from_document(&doc);
    assert_eq!(sheet_to_document(&back), doc);
    assert_eq!(val(&back, "B5"), num(1300.5));
    assert_eq!(back.format(at("C5")), NumFormat::Percent(1));
    assert_eq!(back.col_width(0), 16);
    assert_eq!(sheet_to_document(&Sheet::new()), Document::default());
}

#[test]
fn document_with_odd_content_is_tolerated() {
    // decode() can produce these; building a sheet from them must not panic
    let doc = Document {
        cells: vec![(at("A1"), "".into()), (at("B1"), "x".into()), (at("B1"), "y".into())],
        formats: vec![(at("A1"), NumFormat::General)],
        col_widths: vec![(0, 9999), (1, 10)],
    };
    let s = sheet_from_document(&doc);
    assert_eq!(dump(&s), pairs(&[("B1", "y")]), "later entries win, empty inputs are nothing");
    assert_eq!(s.formats(), vec![]);
    assert_eq!(s.col_widths(), vec![(0, 60)]);
}

#[test]
fn save_and_load_native() {
    let dir = Scratch::new("native");
    let path = dir.path("budget.grid");
    save(&budget(), &path).unwrap();
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(on_disk.starts_with("gridrs 1\n"), "{on_disk}");
    assert!(on_disk.contains("\ncell B5 =SUM(B2:B3)\n"));
    assert!(on_disk.contains("\nformat C5 percent 1\n"));
    assert!(on_disk.contains("\nwidth A 16\n"));
    let back = load(&path).unwrap();
    assert_eq!(sheet_to_document(&back), sheet_to_document(&budget()));
}

#[test]
fn save_and_load_csv() {
    let dir = Scratch::new("csv");
    let path = dir.path("budget.csv");
    save(&budget(), &path).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), sheet_to_csv(&budget(), CsvMode::Inputs), "CSV files store inputs");
    let back = load(&path).unwrap();
    assert_eq!(dump(&back), dump(&budget()));
    assert_eq!(back.formats(), vec![], "CSV cannot carry formats");
    // any other extension is CSV too
    let other = dir.path("notes.txt");
    std::fs::write(&other, "1,2\n=A1+B1\n").unwrap();
    assert_eq!(val(&load(&other).unwrap(), "A2"), num(3.0));
}

#[test]
fn save_overwrites() {
    let dir = Scratch::new("overwrite");
    let path = dir.path("x.csv");
    save(&sheet(&[("A1", "a long first version"), ("A2", "more")]), &path).unwrap();
    save(&sheet(&[("A1", "v2")]), &path).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "v2\n");
}

#[test]
fn load_errors_are_one_line_and_mention_the_path() {
    let dir = Scratch::new("errors");
    let missing = dir.path("does-not-exist.csv");
    let e = load(&missing).unwrap_err();
    assert!(e.contains(&missing) && !e.contains('\n'), "{e:?}");

    let bad_native = dir.path("bad.grid");
    std::fs::write(&bad_native, "not a grid file\n").unwrap();
    let e = load(&bad_native).unwrap_err();
    assert!(e.contains(&bad_native) && e.contains("line 1") && !e.contains('\n'), "{e:?}");

    let bad_csv = dir.path("bad.csv");
    std::fs::write(&bad_csv, "ok\n\"broken\n").unwrap();
    let e = load(&bad_csv).unwrap_err();
    assert!(e.contains(&bad_csv) && e.contains("line 2") && !e.contains('\n'), "{e:?}");

    let not_utf8 = dir.path("latin1.csv");
    std::fs::write(&not_utf8, b"caf\xe9\n").unwrap();
    let e = load(&not_utf8).unwrap_err();
    assert!(e.contains(&not_utf8) && !e.contains('\n'), "{e:?}");

    let e = load(&dir.path("")).unwrap_err(); // a directory
    assert!(!e.is_empty() && !e.contains('\n'), "{e:?}");
}

#[test]
fn save_errors_mention_the_path() {
    let dir = Scratch::new("save-errors");
    let path = dir.path("no-such-subdir/x.csv");
    let e = save(&budget(), &path).unwrap_err();
    assert!(e.contains(&path) && !e.contains('\n'), "{e:?}");
}
