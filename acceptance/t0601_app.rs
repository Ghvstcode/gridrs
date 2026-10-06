//! Acceptance test for task 0601-app. Outside every task's write scope.

include!("common_sheet.rs");

use gridrs::app::App;
use gridrs::tui::render::strip_ansi;
use gridrs::types::{Command, Key, Key::*, Mode, Outcome, MAX_COLS, MAX_ROWS};
use std::path::PathBuf;

/// A fresh scratch directory under the system temp dir, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("gridrs-t0601-{}-{tag}", std::process::id()));
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

fn app() -> App {
    App::new(Sheet::new(), None)
}

/// Press keys; panics if the app asks to quit.
fn press(app: &mut App, keys: &[Key]) {
    for key in keys {
        assert_eq!(app.handle_key(*key), Outcome::Continue, "unexpected quit on {key:?}");
    }
}

/// Type printable characters.
fn typ(app: &mut App, s: &str) {
    for ch in s.chars() {
        assert_eq!(app.handle_key(Char(ch)), Outcome::Continue, "unexpected quit on {ch:?}");
    }
}

/// Type `:line` and press Enter; returns what the app answered.
fn cmd(app: &mut App, line: &str) -> Outcome {
    assert_eq!(app.handle_key(Char(':')), Outcome::Continue);
    for ch in line.chars() {
        assert_eq!(app.handle_key(Char(ch)), Outcome::Continue);
    }
    app.handle_key(Enter)
}

fn frame(app: &mut App) -> Vec<String> {
    app.frame().iter().map(|l| strip_ansi(l)).collect()
}

// ----------------------------------------------------------------------------
// start-up and movement
// ----------------------------------------------------------------------------

#[test]
fn initial_state() {
    let a = App::new(sheet(&[("A1", "x")]), Some("file.csv".to_string()));
    assert_eq!(a.cursor(), at("A1"));
    assert_eq!(a.mode(), Mode::Normal);
    assert_eq!(a.buffer(), "");
    assert_eq!(a.status(), "");
    assert!(!a.is_dirty());
    assert_eq!(a.path(), Some("file.csv".to_string()));
    assert_eq!(a.sheet().input(at("A1")), "x");
    assert_eq!(app().path(), None);
}

#[test]
fn arrows_and_hjkl_move_the_cursor() {
    let mut a = app();
    press(&mut a, &[Right, Right, Down, Down, Down]);
    assert_eq!(a.cursor(), at("C4"));
    press(&mut a, &[Left, Up]);
    assert_eq!(a.cursor(), at("B3"));
    typ(&mut a, "lljjj");
    assert_eq!(a.cursor(), at("D6"));
    typ(&mut a, "hk");
    assert_eq!(a.cursor(), at("C5"));
    press(&mut a, &[Tab, Tab, BackTab]);
    assert_eq!(a.cursor(), at("D5"));
    assert_eq!(a.mode(), Mode::Normal);
    assert!(!a.is_dirty(), "moving around is not a change");
}

#[test]
fn the_cursor_stops_at_the_edges() {
    let mut a = app();
    press(&mut a, &[Left, Up, BackTab]);
    typ(&mut a, "hk");
    assert_eq!(a.cursor(), at("A1"));
    a.execute(Command::Goto(CellRef::new(MAX_COLS - 1, MAX_ROWS - 1)));
    press(&mut a, &[Right, Down, Tab, PageDown]);
    typ(&mut a, "lj");
    assert_eq!(a.cursor(), CellRef::new(MAX_COLS - 1, MAX_ROWS - 1));
}

#[test]
fn nothing_panics_at_the_corner_of_the_sheet_or_on_odd_keys() {
    let mut a = app();
    a.execute(Command::Goto(CellRef::new(MAX_COLS - 1, MAX_ROWS - 1)));
    typ(&mut a, "7");
    press(&mut a, &[Enter]); // commit at the last row: the cursor has nowhere to go
    assert_eq!(a.cursor(), CellRef::new(MAX_COLS - 1, MAX_ROWS - 1));
    typ(&mut a, "8");
    press(&mut a, &[Tab]);
    assert_eq!(a.cursor(), CellRef::new(MAX_COLS - 1, MAX_ROWS - 1));
    assert_eq!(a.sheet().input(a.cursor()), "8");
    for (w, h) in [(0, 0), (1, 1), (3, 3), (80, 24)] {
        a.resize(w, h);
        assert_eq!(frame(&mut a).len(), h);
    }
    press(&mut a, &[End, Ctrl('a'), Ctrl('z'), Esc, Char('~')]);
    assert_eq!(a.mode(), Mode::Normal);
    assert_eq!(a.cursor(), CellRef::new(MAX_COLS - 1, MAX_ROWS - 1), "End and stray Ctrl keys are ignored");
}

#[test]
fn home_and_paging() {
    let mut a = app();
    a.resize(80, 24); // 21 body rows
    a.execute(Command::Goto(at("F10")));
    press(&mut a, &[Home]);
    assert_eq!(a.cursor(), at("A10"));
    press(&mut a, &[PageDown]);
    assert_eq!(a.cursor(), at("A31"));
    press(&mut a, &[PageDown, PageUp, PageUp]);
    assert_eq!(a.cursor(), at("A10"));
    press(&mut a, &[PageUp]);
    assert_eq!(a.cursor(), at("A1"), "clamped at the top");
    a.resize(80, 10); // 7 body rows
    press(&mut a, &[PageDown]);
    assert_eq!(a.cursor(), at("A8"));
    a.resize(80, 2); // no body rows at all: a page is still at least one row
    press(&mut a, &[PageDown]);
    assert_eq!(a.cursor(), at("A9"));
}

// ----------------------------------------------------------------------------
// editing
// ----------------------------------------------------------------------------

#[test]
fn typing_a_number_starts_editing_and_enter_commits_and_moves_down() {
    let mut a = app();
    typ(&mut a, "4");
    assert_eq!(a.mode(), Mode::Edit);
    assert_eq!(a.buffer(), "4");
    typ(&mut a, "2");
    assert_eq!(a.buffer(), "42");
    assert_eq!(a.sheet().input(at("A1")), "", "nothing is written until Enter");
    press(&mut a, &[Enter]);
    assert_eq!(a.mode(), Mode::Normal);
    assert_eq!(a.buffer(), "");
    assert_eq!(a.sheet().value(at("A1")), num(42.0));
    assert_eq!(a.cursor(), at("A2"));
    assert!(a.is_dirty());
}

#[test]
fn every_character_that_starts_a_fresh_edit() {
    for ch in "0123456789=-+.'\"".chars() {
        let mut a = App::new(sheet(&[("A1", "old")]), None);
        press(&mut a, &[Char(ch)]);
        assert_eq!(a.mode(), Mode::Edit, "{ch:?}");
        assert_eq!(a.buffer(), ch.to_string(), "{ch:?} replaces the old content");
    }
}

#[test]
fn letters_that_are_not_commands_do_nothing() {
    let mut a = App::new(sheet(&[("A1", "old")]), None);
    typ(&mut a, "zQW#%");
    assert_eq!(a.mode(), Mode::Normal);
    assert_eq!(a.cursor(), at("A1"));
    assert_eq!(a.sheet().input(at("A1")), "old");
    assert!(!a.is_dirty());
}

#[test]
fn i_and_enter_edit_the_existing_content() {
    let mut a = App::new(sheet(&[("A1", "=1+1")]), None);
    typ(&mut a, "i");
    assert_eq!((a.mode(), a.buffer()), (Mode::Edit, "=1+1".to_string()));
    typ(&mut a, "0");
    press(&mut a, &[Enter]);
    assert_eq!(a.sheet().input(at("A1")), "=1+10");
    assert_eq!(a.sheet().value(at("A1")), num(11.0));
    press(&mut a, &[Up, Enter]);
    assert_eq!((a.mode(), a.buffer()), (Mode::Edit, "=1+10".to_string()));
}

#[test]
fn editing_keys() {
    let mut a = app();
    typ(&mut a, "=1+23");
    press(&mut a, &[Backspace, Backspace]);
    assert_eq!(a.buffer(), "=1+");
    typ(&mut a, "é日 x");
    assert_eq!(a.buffer(), "=1+é日 x");
    press(&mut a, &[Backspace, Backspace, Backspace, Backspace]);
    typ(&mut a, "4");
    // keys that mean something in Normal mode are just text or ignored here
    press(&mut a, &[Left, Right, Up, Down, Home, PageDown, Delete, Ctrl('r'), BackTab]);
    typ(&mut a, "*h");
    assert_eq!(a.buffer(), "=1+4*h");
    assert_eq!(a.cursor(), at("A1"));
    press(&mut a, &[Backspace, Backspace, Backspace, Backspace, Backspace, Backspace, Backspace, Backspace]);
    assert_eq!((a.mode(), a.buffer()), (Mode::Edit, String::new()), "backspace on an empty buffer stays in Edit mode");
}

#[test]
fn tab_commits_and_moves_right() {
    let mut a = app();
    typ(&mut a, "1");
    press(&mut a, &[Tab]);
    typ(&mut a, "2");
    press(&mut a, &[Tab]);
    typ(&mut a, "=A1+B1");
    press(&mut a, &[Enter]);
    assert_eq!(a.sheet().value(at("C1")), num(3.0));
    assert_eq!(a.cursor(), at("C2"));
}

#[test]
fn escape_cancels_an_edit() {
    let mut a = App::new(sheet(&[("A1", "keep")]), None);
    typ(&mut a, "=999");
    press(&mut a, &[Esc]);
    assert_eq!(a.mode(), Mode::Normal);
    assert_eq!(a.buffer(), "");
    assert_eq!(a.sheet().input(at("A1")), "keep");
    assert_eq!(a.cursor(), at("A1"));
    assert!(!a.is_dirty());
}

#[test]
fn committing_an_empty_buffer_clears_the_cell() {
    let mut a = App::new(sheet(&[("A1", "x")]), None);
    typ(&mut a, "i");
    press(&mut a, &[Backspace, Enter]);
    assert_eq!(a.sheet().input(at("A1")), "");
    assert!(a.is_dirty());
}

#[test]
fn committing_without_a_change_is_not_a_change() {
    let mut a = App::new(sheet(&[("A1", "x")]), None);
    typ(&mut a, "i");
    press(&mut a, &[Enter]);
    assert!(!a.is_dirty());
    assert_eq!(a.cursor(), at("A2"), "but the cursor still moves");
    typ(&mut a, "u");
    assert_eq!(a.sheet().input(at("A1")), "x", "and there is nothing to undo");
}

#[test]
fn x_delete_and_backspace_clear_the_cell() {
    for key in [Char('x'), Delete, Backspace] {
        let mut a = App::new(sheet(&[("A1", "1"), ("B1", "=A1+1")]), None);
        press(&mut a, &[key]);
        assert_eq!(a.sheet().input(at("A1")), "", "{key:?}");
        assert_eq!(a.sheet().value(at("B1")), num(1.0));
        assert_eq!(a.mode(), Mode::Normal);
        assert!(a.is_dirty());
        // clearing an empty cell is not a change
        let mut a = app();
        press(&mut a, &[key]);
        assert!(!a.is_dirty(), "{key:?}");
    }
}

// ----------------------------------------------------------------------------
// copy / paste
// ----------------------------------------------------------------------------

#[test]
fn yank_and_paste_shift_references() {
    let mut a = App::new(sheet(&[("A1", "1"), ("A2", "2"), ("B1", "=A1*10")]), None);
    press(&mut a, &[Right]);
    typ(&mut a, "y");
    assert!(!a.is_dirty(), "copying is not a change");
    press(&mut a, &[Down]);
    typ(&mut a, "p");
    assert_eq!(a.sheet().input(at("B2")), "=A2*10");
    assert_eq!(a.sheet().value(at("B2")), num(20.0));
    assert!(a.is_dirty());
    // the clipboard survives a paste
    press(&mut a, &[Down]);
    typ(&mut a, "p");
    assert_eq!(a.sheet().input(at("B3")), "=A3*10");
}

#[test]
fn paste_with_an_empty_clipboard_does_nothing() {
    let mut a = App::new(sheet(&[("A1", "1")]), None);
    typ(&mut a, "p");
    assert_eq!(a.sheet().input(at("A1")), "1");
    assert!(!a.is_dirty());
}

// ----------------------------------------------------------------------------
// undo / redo
// ----------------------------------------------------------------------------

#[test]
fn undo_and_redo_edits() {
    let mut a = app();
    typ(&mut a, "1");
    press(&mut a, &[Enter]);
    typ(&mut a, "2");
    press(&mut a, &[Enter]);
    typ(&mut a, "=A1+A2");
    press(&mut a, &[Enter]);
    assert_eq!(a.sheet().value(at("A3")), num(3.0));
    typ(&mut a, "u");
    assert_eq!(a.sheet().input(at("A3")), "");
    typ(&mut a, "u");
    assert_eq!(a.sheet().input(at("A2")), "");
    assert_eq!(a.sheet().input(at("A1")), "1");
    press(&mut a, &[Ctrl('r')]);
    assert_eq!(a.sheet().input(at("A2")), "2");
    press(&mut a, &[Ctrl('r')]);
    assert_eq!(a.sheet().value(at("A3")), num(3.0));
    press(&mut a, &[Ctrl('r')]);
    assert!(a.status().contains("nothing to redo"), "{:?}", a.status());
    typ(&mut a, "uuu");
    assert_eq!(a.sheet().cells(), vec![]);
    typ(&mut a, "u");
    assert!(a.status().contains("nothing to undo"), "{:?}", a.status());
    assert_eq!(a.mode(), Mode::Normal);
}

#[test]
fn undo_covers_everything_that_changes_the_sheet() {
    let mut a = App::new(sheet(&[("A1", "1"), ("A2", "2"), ("B1", "=A1+A2")]), None);
    let before = dump(a.sheet());
    typ(&mut a, "x"); // clear A1
    typ(&mut a, "y"); // (copy: not a change)
    press(&mut a, &[Down]);
    typ(&mut a, "p"); // paste over A2
    cmd(&mut a, "insrow");
    cmd(&mut a, "delcol");
    cmd(&mut a, "fmt fixed 2");
    cmd(&mut a, "width 20");
    assert_eq!(a.sheet().col_width(0), 20);
    assert_eq!(a.sheet().format(a.cursor()), NumFormat::Fixed(2));
    for _ in 0..6 {
        typ(&mut a, "u");
        assert!(a.status().contains("undone"), "{:?}", a.status());
    }
    assert_eq!(dump(a.sheet()), before);
    assert_eq!(a.sheet().col_width(0), 10);
    assert_eq!(a.sheet().formats(), vec![]);
    typ(&mut a, "u");
    assert!(a.status().contains("nothing to undo"));
    for _ in 0..6 {
        cmd(&mut a, "redo");
        assert!(a.status().contains("redone"), "{:?}", a.status());
    }
    assert_eq!(a.sheet().col_width(0), 20);
    cmd(&mut a, "undo");
    assert_eq!(a.sheet().col_width(0), 10);
}

#[test]
fn a_new_edit_after_undo_discards_redo() {
    let mut a = app();
    typ(&mut a, "1");
    press(&mut a, &[Enter]);
    typ(&mut a, "u");
    typ(&mut a, "7");
    press(&mut a, &[Enter]);
    press(&mut a, &[Ctrl('r')]);
    assert!(a.status().contains("nothing to redo"));
    assert_eq!(a.sheet().cells().len(), 1);
}

// ----------------------------------------------------------------------------
// command mode
// ----------------------------------------------------------------------------

#[test]
fn command_mode_keys() {
    let mut a = app();
    typ(&mut a, ":");
    assert_eq!((a.mode(), a.buffer()), (Mode::Command, String::new()));
    typ(&mut a, "goto B3x");
    assert_eq!(a.buffer(), "goto B3x");
    press(&mut a, &[Backspace]);
    // keys that mean something elsewhere are ignored here
    press(&mut a, &[Left, Up, Tab, Delete, Ctrl('r')]);
    assert_eq!(a.buffer(), "goto B3");
    press(&mut a, &[Enter]);
    assert_eq!(a.mode(), Mode::Normal);
    assert_eq!(a.buffer(), "");
    assert_eq!(a.cursor(), at("B3"));
    // Esc leaves without running anything
    typ(&mut a, ":goto A1");
    press(&mut a, &[Esc]);
    assert_eq!((a.mode(), a.cursor()), (Mode::Normal, at("B3")));
    // Backspace on an empty command line leaves too
    typ(&mut a, ":q");
    press(&mut a, &[Backspace, Backspace]);
    assert_eq!(a.mode(), Mode::Normal);
    // a bare address is a goto
    cmd(&mut a, "D4");
    assert_eq!(a.cursor(), at("D4"));
}

#[test]
fn command_errors_are_shown_and_harmless() {
    let mut a = app();
    assert_eq!(cmd(&mut a, "frobnicate"), Outcome::Continue);
    assert_eq!(a.mode(), Mode::Normal);
    assert!(a.status().starts_with("error: ") && a.status().contains("frobnicate"), "{:?}", a.status());
    assert!(!a.is_dirty());
    // the next key press clears the message
    press(&mut a, &[Right]);
    assert_eq!(a.status(), "");
    assert_eq!(cmd(&mut a, ""), Outcome::Continue);
    assert!(a.status().starts_with("error: "));
}

#[test]
fn quitting() {
    let mut a = app();
    assert_eq!(cmd(&mut a, "q"), Outcome::Quit);
    let mut a = app();
    typ(&mut a, "1");
    press(&mut a, &[Enter]);
    assert_eq!(cmd(&mut a, "q"), Outcome::Continue, "unsaved changes: refuse");
    assert!(a.status().starts_with("error: ") && a.status().contains("q!"), "{:?}", a.status());
    assert_eq!(cmd(&mut a, "q!"), Outcome::Quit);
    assert_eq!(a.execute(Command::Quit { force: true }), Outcome::Quit);
}

#[test]
fn ctrl_c_does_not_quit_but_says_how() {
    let mut a = app();
    assert_eq!(a.handle_key(Ctrl('c')), Outcome::Continue);
    assert!(a.status().contains(":q"), "{:?}", a.status());
}

#[test]
fn rows_and_columns_at_the_cursor() {
    let mut a = App::new(sheet(&[("A1", "1"), ("A2", "2"), ("A3", "=A1+A2"), ("B1", "b")]), None);
    press(&mut a, &[Down]);
    cmd(&mut a, "ir");
    assert_eq!(dump(a.sheet()), pairs(&[("A1", "1"), ("B1", "b"), ("A3", "2"), ("A4", "=A1+A3")]));
    assert_eq!(a.cursor(), at("A2"), "the cursor stays where it was");
    assert!(a.is_dirty());
    cmd(&mut a, "dr");
    assert_eq!(dump(a.sheet()), pairs(&[("A1", "1"), ("B1", "b"), ("A2", "2"), ("A3", "=A1+A2")]));
    cmd(&mut a, "ic");
    assert_eq!(a.sheet().input(at("B3")), "=B1+B2");
    assert_eq!(a.sheet().input(at("C1")), "b");
    cmd(&mut a, "dc");
    assert_eq!(a.sheet().input(at("A3")), "=A1+A2");
}

#[test]
fn format_and_width_apply_to_the_cursor() {
    let mut a = App::new(sheet(&[("B2", "0.5")]), None);
    a.execute(Command::Goto(at("B2")));
    cmd(&mut a, "fmt percent 0");
    cmd(&mut a, "width 6");
    assert_eq!(a.sheet().format(at("B2")), NumFormat::Percent(0));
    assert_eq!(a.sheet().col_width(1), 6);
    assert!(a.is_dirty());
    // setting the same format again is not a new undo step
    cmd(&mut a, "fmt percent 0");
    typ(&mut a, "u");
    assert_eq!(a.sheet().col_width(1), 10);
    typ(&mut a, "u");
    assert_eq!(a.sheet().format(at("B2")), NumFormat::General);
}

#[test]
fn help_shows_something_useful() {
    let mut a = app();
    cmd(&mut a, "help");
    assert!(a.status().contains(":q") && a.status().contains(":w"), "{:?}", a.status());
    assert!(!a.status().contains('\n'));
}

// ----------------------------------------------------------------------------
// files
// ----------------------------------------------------------------------------

#[test]
fn write_saves_and_clears_dirty() {
    let dir = Scratch::new("write");
    let path = dir.path("out.csv");
    let mut a = App::new(Sheet::new(), Some(path.clone()));
    typ(&mut a, "1");
    press(&mut a, &[Tab]);
    typ(&mut a, "=A1+1");
    press(&mut a, &[Enter]);
    assert_eq!(cmd(&mut a, "w"), Outcome::Continue);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "1,=A1+1\n");
    assert!(!a.is_dirty());
    assert!(a.status().contains("wrote") && a.status().contains(&path), "{:?}", a.status());
    assert_eq!(cmd(&mut a, "q"), Outcome::Quit);
}

#[test]
fn write_without_a_file_name_is_an_error() {
    let mut a = app();
    typ(&mut a, "1");
    press(&mut a, &[Enter]);
    cmd(&mut a, "w");
    assert!(a.status().starts_with("error: "), "{:?}", a.status());
    assert!(a.is_dirty());
    assert_eq!(cmd(&mut a, "wq"), Outcome::Continue, "a failed write must not quit");
}

#[test]
fn write_with_a_name_adopts_it() {
    let dir = Scratch::new("saveas");
    let first = dir.path("first.grid");
    let second = dir.path("second.csv");
    let mut a = app();
    typ(&mut a, "=1+1");
    press(&mut a, &[Enter]);
    cmd(&mut a, &format!("w {first}"));
    assert_eq!(a.path(), Some(first.clone()));
    assert!(std::fs::read_to_string(&first).unwrap().starts_with("gridrs 1\n"), "the extension picks the format");
    cmd(&mut a, &format!("w {second}"));
    assert_eq!(a.path(), Some(second.clone()));
    assert_eq!(std::fs::read_to_string(&second).unwrap(), "=1+1\n");
    // a failed write keeps the old name and stays dirty
    typ(&mut a, "9");
    press(&mut a, &[Enter]);
    let bad = dir.path("no-such-dir/x.csv");
    cmd(&mut a, &format!("w {bad}"));
    assert!(a.status().starts_with("error: "), "{:?}", a.status());
    assert_eq!(a.path(), Some(second));
    assert!(a.is_dirty());
}

#[test]
fn wq_and_x_write_then_quit() {
    let dir = Scratch::new("wq");
    for (i, command) in ["wq", "x"].iter().enumerate() {
        let path = dir.path(&format!("{i}.csv"));
        let mut a = App::new(Sheet::new(), Some(path.clone()));
        typ(&mut a, "5");
        press(&mut a, &[Enter]);
        assert_eq!(cmd(&mut a, command), Outcome::Quit);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "5\n");
    }
    let path = dir.path("named.csv");
    let mut a = app();
    typ(&mut a, "6");
    press(&mut a, &[Enter]);
    assert_eq!(cmd(&mut a, &format!("wq {path}")), Outcome::Quit);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "6\n");
}

#[test]
fn edit_opens_another_file() {
    let dir = Scratch::new("edit");
    let path = dir.path("data.csv");
    std::fs::write(&path, "10,20\n=A1+B1\n").unwrap();
    let mut a = app();
    a.execute(Command::Goto(at("C3")));
    assert_eq!(cmd(&mut a, &format!("e {path}")), Outcome::Continue);
    assert_eq!(a.sheet().value(at("A2")), num(30.0));
    assert_eq!(a.path(), Some(path.clone()));
    assert_eq!(a.cursor(), at("A1"));
    assert!(!a.is_dirty());
    assert!(a.status().contains(&path), "{:?}", a.status());
    typ(&mut a, "u");
    assert_eq!(a.sheet().value(at("A2")), num(30.0), "opening a file is not undoable");
    // refuses while there are unsaved changes
    typ(&mut a, "1");
    press(&mut a, &[Enter]);
    cmd(&mut a, &format!("e {path}"));
    assert!(a.status().starts_with("error: "), "{:?}", a.status());
    assert_eq!(a.sheet().input(at("A1")), "1");
    // a missing file is an error and changes nothing
    let mut a = App::new(sheet(&[("A1", "keep")]), Some("old.csv".to_string()));
    cmd(&mut a, &format!("e {}", dir.path("missing.csv")));
    assert!(a.status().starts_with("error: "), "{:?}", a.status());
    assert_eq!(a.sheet().input(at("A1")), "keep");
    assert_eq!(a.path(), Some("old.csv".to_string()));
}

#[test]
fn export_writes_displayed_values() {
    let dir = Scratch::new("export");
    let path = dir.path("report.csv");
    let mut s = sheet(&[("A1", "0.5"), ("B1", "=A1*3")]);
    s.set_format(at("A1"), NumFormat::Percent(0));
    s.set_format(at("B1"), NumFormat::Fixed(2));
    let mut a = App::new(s, Some("source.grid".to_string()));
    cmd(&mut a, &format!("export {path}"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "50%,1.50\n");
    assert_eq!(a.path(), Some("source.grid".to_string()), "exporting does not rename the document");
    assert!(a.status().contains(&path), "{:?}", a.status());
    cmd(&mut a, &format!("export {}", dir.path("no-such-dir/x.csv")));
    assert!(a.status().starts_with("error: "), "{:?}", a.status());
}

// ----------------------------------------------------------------------------
// frames
// ----------------------------------------------------------------------------

#[test]
fn frame_shows_formatted_values_bar_and_status() {
    let mut s = sheet(&[("A1", "item"), ("B1", "price"), ("A2", "tea"), ("B2", "1234.5"), ("B3", "=B2/0"), ("B4", "=B2>1")]);
    s.set_format(at("B2"), NumFormat::Thousands(2));
    let mut a = App::new(s, Some("shop.csv".to_string()));
    a.resize(30, 8);
    a.execute(Command::Goto(at("B2")));
    assert_eq!(
        frame(&mut a),
        [
            "B2 | 1234.5                   ",
            "        A          B          ",
            "  1 item       price          ",
            "  2 tea          1,234.50     ",
            "  3             #DIV/0!       ",
            "  4               TRUE        ",
            "  5                           ",
            "shop.csv                NORMAL",
        ]
    );
}

#[test]
fn frame_reflects_the_mode() {
    let mut a = app();
    a.resize(30, 4);
    assert_eq!(frame(&mut a)[3], "[no file]               NORMAL");
    typ(&mut a, "=1+");
    let f = frame(&mut a);
    assert_eq!(f[0], "A1 | =1+                      ");
    assert_eq!(f[3], "[no file]                 EDIT");
    press(&mut a, &[Esc]);
    typ(&mut a, ":go");
    let f = frame(&mut a);
    assert_eq!(f[0], "A1 | :go                      ");
    assert_eq!(f[3], "[no file]              COMMAND");
    press(&mut a, &[Esc]);
    typ(&mut a, "5");
    press(&mut a, &[Enter]);
    assert_eq!(frame(&mut a)[3], "[no file] [+]           NORMAL", "unsaved changes are flagged");
    cmd(&mut a, "nonsense");
    assert!(frame(&mut a)[3].starts_with("error: "), "a message replaces the file name until the next key");
    press(&mut a, &[Esc]);
    assert_eq!(frame(&mut a)[3], "[no file] [+]           NORMAL");
}

#[test]
fn frame_uses_the_sheets_column_widths() {
    let mut s = sheet(&[("A1", "abcdefgh"), ("B1", "x")]);
    s.set_col_width(0, 3);
    let mut a = App::new(s, None);
    a.resize(20, 4);
    assert_eq!(frame(&mut a)[2], "  1 abc x           ");
}

#[test]
fn the_view_scrolls_to_keep_the_cursor_visible() {
    let mut a = app();
    a.resize(40, 8); // 5 body rows; gutter 3; columns of 10 -> A, B, C visible
    let first_rows = |a: &mut App| -> (String, String) {
        let f = frame(a);
        (f[1].split_whitespace().collect::<Vec<_>>().join(" "), f[2].trim().to_string())
    };
    assert_eq!(first_rows(&mut a), ("A B C".to_string(), "1".to_string()));
    // moving down past the last visible row scrolls by exactly enough
    press(&mut a, &[Down, Down, Down, Down]);
    assert_eq!(first_rows(&mut a), ("A B C".to_string(), "1".to_string()), "row 5 is still visible");
    press(&mut a, &[Down]);
    assert_eq!(first_rows(&mut a), ("A B C".to_string(), "2".to_string()), "row 6: scrolled by one");
    // moving right past the last visible column scrolls by exactly enough
    press(&mut a, &[Right, Right]);
    assert_eq!(first_rows(&mut a).0, "A B C");
    press(&mut a, &[Right]);
    assert_eq!(first_rows(&mut a).0, "B C D");
    // moving back up/left scrolls back only when the cursor leaves the view
    press(&mut a, &[Left, Left]);
    assert_eq!(first_rows(&mut a).0, "B C D");
    press(&mut a, &[Left]);
    assert_eq!(first_rows(&mut a).0, "A B C");
    press(&mut a, &[Up, Up, Up, Up]);
    assert_eq!(first_rows(&mut a).1, "2");
    press(&mut a, &[Up]);
    assert_eq!(first_rows(&mut a).1, "1");
    // a far jump lands with the cursor on the last visible row / column
    cmd(&mut a, "goto Z100");
    let f = frame(&mut a);
    assert_eq!(f[1].split_whitespace().collect::<Vec<_>>(), ["X", "Y", "Z"]);
    assert_eq!(f[2].trim(), "96");
    assert_eq!(f[6].trim(), "100");
    // and jumping back to the top-left resets the view
    cmd(&mut a, "goto A1");
    assert_eq!(first_rows(&mut a), ("A B C".to_string(), "1".to_string()));
}

#[test]
fn frames_have_the_requested_size() {
    let mut a = App::new(sheet(&[("A1", "x")]), None);
    for (w, h) in [(80, 24), (20, 5), (1, 1), (0, 0), (200, 60), (5, 3)] {
        a.resize(w, h);
        let f = frame(&mut a);
        assert_eq!(f.len(), h, "{w}x{h}");
        for line in &f {
            assert_eq!(line.chars().count(), w, "{w}x{h}: {line:?}");
        }
    }
}

#[test]
fn a_whole_session() {
    let dir = Scratch::new("session");
    let path = dir.path("budget.csv");
    let mut a = App::new(Sheet::new(), Some(path.clone()));
    // type a small table: Enter moves down, so this fills column A, then column B
    for word in ["'rent", "'food", "'total"] {
        typ(&mut a, word);
        press(&mut a, &[Enter]);
    }
    cmd(&mut a, "goto B1");
    for word in ["1000", "300.5", "=SUM(B1:B2)"] {
        typ(&mut a, word);
        press(&mut a, &[Enter]);
    }
    assert_eq!(a.sheet().value(at("B3")), num(1300.5));
    cmd(&mut a, "goto B3");
    cmd(&mut a, "fmt thousands 2");
    a.resize(30, 6);
    let f = frame(&mut a);
    assert_eq!(f[4], "  3 total        1,300.50     ");
    assert_eq!(cmd(&mut a, "wq"), Outcome::Quit);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "'rent,1000\n'food,300.5\n'total,=SUM(B1:B2)\n");
}
