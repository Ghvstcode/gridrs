//! Acceptance test for task 0503-commands. Outside every task's write scope.

use gridrs::command::parse_command;
use gridrs::types::{CellRef, Command, Command::*, NumFormat};

fn ok(line: &str) -> Command {
    parse_command(line).unwrap_or_else(|e| panic!("parse_command({line:?}) failed: {e}"))
}

fn some(s: &str) -> Option<String> {
    Some(s.to_string())
}

#[test]
fn write() {
    assert_eq!(ok("w"), Write(None));
    assert_eq!(ok("w budget.csv"), Write(some("budget.csv")));
    assert_eq!(ok("w   /tmp/my budget 2026.grid  "), Write(some("/tmp/my budget 2026.grid")), "the file name is the rest of the line, trimmed");
    assert_eq!(ok("w w"), Write(some("w")));
}

#[test]
fn quit() {
    assert_eq!(ok("q"), Quit { force: false });
    assert_eq!(ok("q!"), Quit { force: true });
    assert_eq!(ok("  q  "), Quit { force: false });
}

#[test]
fn write_and_quit() {
    assert_eq!(ok("wq"), WriteQuit(None));
    assert_eq!(ok("x"), WriteQuit(None));
    assert_eq!(ok("wq out.csv"), WriteQuit(some("out.csv")));
    assert_eq!(ok("x out.csv"), WriteQuit(some("out.csv")));
}

#[test]
fn edit_and_export() {
    assert_eq!(ok("e data.csv"), Edit("data.csv".to_string()));
    assert_eq!(ok("e  two words.csv "), Edit("two words.csv".to_string()));
    assert_eq!(ok("export report.csv"), Export("report.csv".to_string()));
}

#[test]
fn goto() {
    assert_eq!(ok("goto B12"), Goto(CellRef::new(1, 11)));
    assert_eq!(ok("goto  aa100"), Goto(CellRef::new(26, 99)));
    assert_eq!(ok("goto XFD1048576"), Goto(CellRef::new(16383, 1048575)));
    // a bare address is a goto
    assert_eq!(ok("B12"), Goto(CellRef::new(1, 11)));
    assert_eq!(ok(" c3 "), Goto(CellRef::new(2, 2)));
}

#[test]
fn rows_and_columns() {
    assert_eq!(ok("insrow"), InsertRow);
    assert_eq!(ok("ir"), InsertRow);
    assert_eq!(ok("delrow"), DeleteRow);
    assert_eq!(ok("dr"), DeleteRow);
    assert_eq!(ok("inscol"), InsertCol);
    assert_eq!(ok("ic"), InsertCol);
    assert_eq!(ok("delcol"), DeleteCol);
    assert_eq!(ok("dc"), DeleteCol);
}

#[test]
fn format() {
    assert_eq!(ok("fmt general"), Format(NumFormat::General));
    assert_eq!(ok("fmt fixed 2"), Format(NumFormat::Fixed(2)));
    assert_eq!(ok("format thousands 0"), Format(NumFormat::Thousands(0)));
    assert_eq!(ok("fmt percent 1"), Format(NumFormat::Percent(1)));
    assert_eq!(ok("fmt   fixed    15 "), Format(NumFormat::Fixed(15)), "extra spaces between words are fine");
}

#[test]
fn width() {
    assert_eq!(ok("width 14"), Width(14));
    assert_eq!(ok("width 1"), Width(1));
    assert_eq!(ok("width 60"), Width(60));
}

#[test]
fn undo_redo_help() {
    assert_eq!(ok("undo"), Undo);
    assert_eq!(ok("u"), Undo);
    assert_eq!(ok("redo"), Redo);
    assert_eq!(ok("help"), Help);
    assert_eq!(ok("h"), Help);
}

#[test]
fn errors() {
    for bad in [
        "", "   ", "nope", "W", "Q", "quit", "write", ":w", "q now", "q! now", "q !", "wq!", "e", "e   ", "export", "goto", "goto 12", "goto A0", "goto A1 B2", "goto $A$1", "goto A1:B2", "A0", "A1:B2", "12", "insrow 3", "ir 2", "dr x", "ic 1", "dc 1", "fmt", "fmt nope", "fmt fixed", "fmt fixed 16", "fmt fixed -1", "fmt Fixed 2", "format", "width", "width 0", "width 61", "width ten", "width 5 5", "width -3", "width 5.0", "undo 2", "u 2", "redo 1", "help me", "h 1",
    ] {
        let e = parse_command(bad).expect_err(&format!("parse_command({bad:?}) should fail"));
        assert!(!e.is_empty() && !e.contains('\n'), "error for {bad:?} should be one non-empty line: {e:?}");
    }
}

#[test]
fn unknown_command_errors_name_the_command() {
    let e = parse_command("frobnicate now").unwrap_err();
    assert!(e.contains("frobnicate"), "{e:?}");
}
