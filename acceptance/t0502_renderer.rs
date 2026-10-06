//! Acceptance test for task 0502-renderer. Outside every task's write scope.

use gridrs::tui::render::{layout, render, strip_ansi, to_ansi, BOLD, RESET, REVERSE};
use gridrs::types::{Align, Align::*, CellRef, ColSlot, GridSource, Layout, Ui, MAX_COLS, MAX_ROWS};
use std::collections::HashMap;

#[derive(Default)]
struct Fake {
    cells: HashMap<CellRef, (String, Align)>,
    widths: HashMap<u32, usize>,
}

impl Fake {
    fn put(&mut self, col: u32, row: u32, text: &str, align: Align) {
        self.cells.insert(CellRef::new(col, row), (text.to_string(), align));
    }
}

impl GridSource for Fake {
    fn text(&self, at: CellRef) -> String {
        self.cells.get(&at).map(|(t, _)| t.clone()).unwrap_or_default()
    }
    fn align(&self, at: CellRef) -> Align {
        self.cells.get(&at).map(|(_, a)| *a).unwrap_or(Left)
    }
    fn col_width(&self, col: u32) -> usize {
        self.widths.get(&col).copied().unwrap_or(10)
    }
}

fn ui(width: usize, height: usize) -> Ui {
    Ui { width, height, top: 0, left: 0, cursor: CellRef::new(0, 0), bar: String::new(), status: String::new(), mode: "NORMAL".to_string() }
}

/// The demo sheet used by most tests. Column widths: A=6, B=8, everything else 10.
fn fruit() -> Fake {
    let mut g = Fake::default();
    g.widths.insert(0, 6);
    g.widths.insert(1, 8);
    g.put(0, 0, "item", Left);
    g.put(1, 0, "qty", Left);
    g.put(2, 0, "price", Left);
    g.put(0, 1, "apple", Left);
    g.put(1, 1, "3", Right);
    g.put(2, 1, "0.50", Right);
    g.put(0, 2, "a very long name", Left);
    g.put(1, 2, "123456789", Right);
    g.put(2, 2, "TRUE", Center);
    g.put(1, 3, "#DIV/0!", Center);
    g.put(2, 3, "12345678901", Right);
    g
}

fn fruit_ui() -> Ui {
    Ui { cursor: CellRef::new(1, 1), bar: "=1+2".to_string(), status: "ready".to_string(), ..ui(40, 8) }
}

fn plain(lines: &[String]) -> Vec<String> {
    lines.iter().map(|l| strip_ansi(l)).collect()
}

#[test]
fn layout_of_the_demo_frame() {
    assert_eq!(
        layout(&fruit(), &fruit_ui()),
        Layout {
            gutter: 3,
            rows: 5,
            cols: vec![ColSlot { col: 0, x: 4, width: 6 }, ColSlot { col: 1, x: 11, width: 8 }, ColSlot { col: 2, x: 20, width: 10 }],
        },
        "column D would need x = 31..41 and the frame is 40 wide, so it is not shown"
    );
}

#[test]
fn the_demo_frame_text_is_exact() {
    let lines = render(&fruit(), &fruit_ui());
    let want = [
        "B2 | =1+2                               ",
        "      A       B         C               ",
        "  1 item   qty      price               ",
        "  2 apple         3       0.50          ",
        "  3 a very ########    TRUE             ",
        "  4        #DIV/0!  ##########          ",
        "  5                                     ",
        "ready                             NORMAL",
    ];
    assert_eq!(plain(&lines), want);
}

#[test]
fn the_demo_frame_styling_is_exact() {
    let lines = render(&fruit(), &fruit_ui());
    assert_eq!(lines.len(), 8);
    // formula bar: no styling
    assert_eq!(lines[0], "B2 | =1+2                               ");
    // header: the cursor's column letter slot is bold
    assert_eq!(lines[1], format!("      A    {BOLD}   B    {RESET}     C               "));
    // rows without the cursor: no styling
    assert_eq!(lines[2], "  1 item   qty      price               ");
    // cursor row: bold row number, reverse-video cell (exactly the column's width)
    assert_eq!(lines[3], format!("{BOLD}  2{RESET} apple  {REVERSE}       3{RESET}       0.50          "));
    assert_eq!(lines[4], "  3 a very ########    TRUE             ");
    // status line: reverse video across the full width
    assert_eq!(lines[7], format!("{REVERSE}ready                             NORMAL{RESET}"));
}

#[test]
fn every_line_is_exactly_as_wide_as_the_terminal() {
    let g = fruit();
    for width in [0, 1, 2, 3, 4, 5, 9, 10, 11, 17, 40, 80, 200] {
        for height in [0, 1, 2, 3, 4, 5, 24] {
            let lines = render(&g, &Ui { width, height, ..fruit_ui() });
            assert_eq!(lines.len(), height, "{width}x{height}: number of lines");
            for (i, line) in lines.iter().enumerate() {
                assert_eq!(strip_ansi(line).chars().count(), width, "{width}x{height}: width of line {i}: {line:?}");
            }
        }
    }
}

#[test]
fn alignment_and_overflow() {
    let mut g = Fake::default();
    g.widths.insert(0, 5);
    g.put(0, 0, "ab", Left);
    g.put(0, 1, "ab", Right);
    g.put(0, 2, "ab", Center); // 3 spare: 1 left, 2 right
    g.put(0, 3, "abc", Center); // 2 spare: 1 left, 1 right
    g.put(0, 4, "abcde", Right); // exactly fits
    g.put(0, 5, "abcdef", Right); // too long, right-aligned (a number): hashes
    g.put(0, 6, "abcdef", Left); // too long, text: cut
    g.put(0, 7, "abcdef", Center); // too long, centred: cut
    let lines = plain(&render(&g, &ui(9, 11)));
    let body: Vec<&str> = lines[2..10].iter().map(|l| &l[4..9]).collect();
    assert_eq!(body, ["ab   ", "   ab", " ab  ", " abc ", "abcde", "#####", "abcde", "abcde"]);
}

#[test]
fn column_headers_are_centred_and_cut_if_needed() {
    let mut g = Fake::default();
    // widths: Z=1, AA=1, AB=3, AC=5; AD (10) no longer fits in 20
    // start at column Z so that we get two-letter names: Z, AA, AB, AC
    let mut u = ui(20, 4);
    u.left = 25;
    u.cursor = CellRef::new(0, 500); // far away: nothing highlighted
    g.widths.insert(25, 1);
    g.widths.insert(26, 1);
    g.widths.insert(27, 3);
    g.widths.insert(28, 5);
    let lines = render(&g, &u);
    //                        gutter Z AA  AB    AC
    assert_eq!(lines[1], format!("{:<20}", "    Z A AB   AC  "));
    assert_eq!(layout(&g, &u).cols, [ColSlot { col: 25, x: 4, width: 1 }, ColSlot { col: 26, x: 6, width: 1 }, ColSlot { col: 27, x: 8, width: 3 }, ColSlot { col: 28, x: 12, width: 5 }]);
}

#[test]
fn scrolling_changes_the_origin() {
    let mut u = fruit_ui();
    u.top = 1;
    u.left = 1;
    let lines = plain(&render(&fruit(), &u));
    // columns B (8), C (10), D (10): 3 + 9 + 11 + 11 = 34 <= 40; E would need 45
    assert_eq!(lines[1], "       B         C          D           ");
    assert_eq!(lines[2], "  2        3       0.50                 ");
    assert_eq!(lines[3], "  3 ########    TRUE                    ");
    assert_eq!(lines[6], "  6                                     ");
    assert_eq!(layout(&fruit(), &u).cols.iter().map(|c| c.col).collect::<Vec<_>>(), [1, 2, 3]);
}

#[test]
fn the_gutter_grows_with_the_row_numbers() {
    let g = Fake::default();
    let mut u = ui(30, 8); // 5 rows
    for (top, gutter) in [(0, 3), (994, 3), (995, 4), (9_994, 4), (9_995, 5), (999_995, 7)] {
        u.top = top;
        let l = layout(&g, &u);
        assert_eq!(l.gutter, gutter, "top = {top}: last visible row is {}", top + 5);
        assert_eq!(l.cols[0].x, gutter + 1);
    }
    u.top = 995;
    u.cursor = CellRef::new(0, 999);
    let lines = render(&g, &u);
    assert_eq!(strip_ansi(&lines[2]), format!("{:<30}", " 996"));
    assert_eq!(lines[6], format!("{BOLD}1000{RESET} {REVERSE}          {RESET}{}", " ".repeat(15)));
}

#[test]
fn nothing_is_drawn_past_the_edges_of_the_sheet() {
    let mut g = Fake::default();
    g.put(MAX_COLS - 1, MAX_ROWS - 1, "last", Left);
    let mut u = ui(40, 8);
    u.top = MAX_ROWS - 2;
    u.left = MAX_COLS - 2;
    u.cursor = CellRef::new(MAX_COLS - 1, MAX_ROWS - 1);
    let l = layout(&g, &u);
    assert_eq!(l.rows, 5, "`rows` is how many body lines there are, even if the sheet ends first");
    assert_eq!(l.cols.iter().map(|c| c.col).collect::<Vec<_>>(), [MAX_COLS - 2, MAX_COLS - 1]);
    let lines = plain(&render(&g, &u));
    assert_eq!(lines[1], format!("{:<40}", "           XFC        XFD"));
    assert_eq!(lines[2], format!("{:<40}", "1048575"));
    assert_eq!(lines[3], format!("{:<40}", "1048576            last"));
    assert_eq!(lines[4], " ".repeat(40), "past the last row: blank");
    assert_eq!(lines[6], " ".repeat(40));
}

#[test]
fn a_cursor_outside_the_viewport_highlights_nothing() {
    let mut u = fruit_ui();
    u.cursor = CellRef::new(30, 30);
    let lines = render(&fruit(), &u);
    for line in &lines[..7] {
        assert!(!line.contains('\x1b'), "{line:?}");
    }
    // cursor row visible but column not: only the row number is bold
    u.cursor = CellRef::new(30, 1);
    let lines = render(&fruit(), &u);
    assert_eq!(lines[3], format!("{BOLD}  2{RESET} apple         3       0.50          "));
    assert!(!lines[1].contains('\x1b'));
    // cursor column visible but row not: only the column letter is bold
    u.cursor = CellRef::new(0, 30);
    let lines = render(&fruit(), &u);
    assert_eq!(lines[1], format!("    {BOLD}  A   {RESET}    B         C               "));
    assert!(!lines[3].contains('\x1b'));
}

#[test]
fn formula_bar() {
    let g = Fake::default();
    let mut u = ui(20, 4);
    u.cursor = CellRef::new(27, 99);
    u.bar = "hello".into();
    assert_eq!(render(&g, &u)[0], "AB100 | hello       ");
    u.bar = "this is far too long to fit".into();
    assert_eq!(render(&g, &u)[0], "AB100 | this is far ");
    u.bar = String::new();
    assert_eq!(render(&g, &u)[0], "AB100 |             ");
}

#[test]
fn status_line() {
    let g = Fake::default();
    let mut u = ui(20, 4);
    u.status = "saved".into();
    u.mode = "EDIT".into();
    assert_eq!(strip_ansi(&render(&g, &u)[3]), "saved           EDIT");
    u.status = "a status message that is too long".into();
    assert_eq!(strip_ansi(&render(&g, &u)[3]), "a status messag EDIT", "the message is cut; one space always separates it from the mode");
    u.status = String::new();
    u.mode = String::new();
    assert_eq!(render(&g, &u)[3], format!("{REVERSE}{}{RESET}", " ".repeat(20)));
    // narrower than the mode label: the line is still exactly `width` wide
    let mut u = ui(4, 4);
    u.status = "saved".into();
    u.mode = "COMMAND".into();
    assert_eq!(strip_ansi(&render(&g, &u)[3]), " COM");
}

#[test]
fn control_characters_never_reach_the_terminal() {
    let mut g = Fake::default();
    g.put(0, 0, "a\x1b[2Jb", Left);
    g.put(1, 0, "x\ty\nz", Left);
    let mut u = ui(40, 4);
    u.cursor = CellRef::new(5, 5);
    u.bar = "one\ntwo\x07".into();
    u.status = "st\ratus\x1b[0m".into();
    u.mode = "MO\x1bDE".into();
    let lines = render(&g, &u);
    assert_eq!(lines[0], format!("{:<40}", "F6 | one two "));
    assert_eq!(lines[2], format!("{:<40}", "  1 a [2Jb     x y z"));
    assert_eq!(strip_ansi(&lines[3]), format!("{:<34} MO DE", "st atus [0m"));
    // the only escapes anywhere are the three styling ones
    for line in &lines {
        let rest = line.replace(REVERSE, "").replace(BOLD, "").replace(RESET, "");
        assert!(!rest.chars().any(|c| c.is_control()), "{line:?}");
    }
}

#[test]
fn width_is_counted_in_characters() {
    let mut g = Fake::default();
    g.widths.insert(0, 4);
    g.put(0, 0, "éé", Right);
    g.put(0, 1, "ñandú!", Left);
    let lines = plain(&render(&g, &ui(8, 5)));
    assert_eq!(lines[2], "  1   éé");
    assert_eq!(lines[3], "  2 ñand");
}

#[test]
fn zero_width_columns_are_treated_as_width_one() {
    let mut g = Fake::default();
    g.widths.insert(0, 0);
    g.put(0, 0, "x", Left);
    let l = layout(&g, &ui(20, 4));
    assert_eq!(l.cols[0], ColSlot { col: 0, x: 4, width: 1 });
    assert_eq!(l.cols[1], ColSlot { col: 1, x: 6, width: 10 });
}

#[test]
fn to_ansi_homes_the_cursor_and_clears_each_line() {
    let lines = vec!["one".to_string(), "two".to_string(), "three".to_string()];
    assert_eq!(to_ansi(&lines), "\x1b[Hone\x1b[K\r\ntwo\x1b[K\r\nthree\x1b[K");
    assert_eq!(to_ansi(&[]), "\x1b[H");
    assert_eq!(to_ansi(&["x".to_string()]), "\x1b[Hx\x1b[K");
}

#[test]
fn strip_ansi_removes_escape_sequences() {
    assert_eq!(strip_ansi("plain"), "plain");
    assert_eq!(strip_ansi(""), "");
    assert_eq!(strip_ansi("\x1b[7mhi\x1b[0m"), "hi");
    assert_eq!(strip_ansi("a\x1b[1;31mb\x1b[Kc\x1b[?25hd\x1b[He"), "abcde");
    assert_eq!(strip_ansi("\x1b[Hone\x1b[K\r\ntwo\x1b[K"), "one\r\ntwo", "only escape sequences are removed");
    assert_eq!(strip_ansi("é\x1b[1mñ\x1b[0m"), "éñ");
    assert_eq!(strip_ansi("a\x1bb"), "ab", "a lone ESC is dropped on its own");
    assert_eq!(strip_ansi("end\x1b"), "end");
    assert_eq!(strip_ansi("cut\x1b[12"), "cut", "an unfinished sequence is dropped to the end");
    let frame = render(&fruit(), &fruit_ui());
    assert_eq!(strip_ansi(&to_ansi(&frame)), plain(&frame).join("\r\n"));
}
