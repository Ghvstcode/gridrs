//! Acceptance test for task 0602-cli (argument parsing, --eval, --dump, the event loop).
//! Outside every task's write scope.
//!
//! This is the end-to-end test: it runs the real `gridrs` binary.

use gridrs::app::App;
use gridrs::cli::{parse_args, run};
use gridrs::sheet::Sheet;
use gridrs::tui::event_loop;
use gridrs::tui::render::strip_ansi;
use gridrs::types::{CellRef, CliAction, Value};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const DEMO_CSV: &str = "item,qty,price,total\napple,3,0.5,=B2*C2\npear,10,0.25,=B3*C3\n,,,=SUM(D2:D3)\n";

/// A fresh scratch directory under the system temp dir, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("gridrs-t0602-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
    fn path(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().to_string()
    }
    fn file(&self, name: &str, content: &str) -> String {
        let p = self.path(name);
        std::fs::write(&p, content).unwrap();
        p
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// Call `cli::run` in-process: (exit code, stdout, stderr).
fn call(list: &[&str]) -> (i32, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(&args(list), &mut out, &mut err);
    (code, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

/// Run the real binary with no terminal attached: (exit code, stdout, stderr).
fn gridrs(list: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_gridrs")).args(list).stdin(Stdio::null()).output().expect("run gridrs");
    (out.status.code().expect("exit code"), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

// ----------------------------------------------------------------------------
// parse_args
// ----------------------------------------------------------------------------

#[test]
fn parse_open() {
    assert_eq!(parse_args(&args(&[])), Ok(CliAction::Open { file: None }));
    assert_eq!(parse_args(&args(&["budget.csv"])), Ok(CliAction::Open { file: Some("budget.csv".into()) }));
    assert_eq!(parse_args(&args(&["--", "-odd-name.csv"])), Ok(CliAction::Open { file: Some("-odd-name.csv".into()) }));
    assert_eq!(parse_args(&args(&["--"])), Ok(CliAction::Open { file: None }));
}

#[test]
fn parse_help_and_version_win() {
    for flag in ["--help", "-h"] {
        assert_eq!(parse_args(&args(&[flag])), Ok(CliAction::Help));
        assert_eq!(parse_args(&args(&["file.csv", "--eval", "1", flag])), Ok(CliAction::Help));
        assert_eq!(parse_args(&args(&["--version", flag])), Ok(CliAction::Help), "help beats version");
        assert_eq!(parse_args(&args(&["--bogus", flag])), Ok(CliAction::Help), "help beats errors");
    }
    for flag in ["--version", "-V"] {
        assert_eq!(parse_args(&args(&[flag])), Ok(CliAction::Version));
        assert_eq!(parse_args(&args(&["file.csv", flag])), Ok(CliAction::Version));
    }
    // after `--` they are just file names
    assert_eq!(parse_args(&args(&["--", "--help"])), Ok(CliAction::Open { file: Some("--help".into()) }));
}

#[test]
fn parse_eval() {
    assert_eq!(parse_args(&args(&["--eval", "=1+2"])), Ok(CliAction::Eval { file: None, formulas: vec!["=1+2".into()] }));
    assert_eq!(parse_args(&args(&["-e", "1+2"])), Ok(CliAction::Eval { file: None, formulas: vec!["1+2".into()] }));
    assert_eq!(
        parse_args(&args(&["-e", "=A1", "data.csv", "--eval", "=-B2"])),
        Ok(CliAction::Eval { file: Some("data.csv".into()), formulas: vec!["=A1".into(), "=-B2".into()] }),
        "repeatable, in order, file anywhere"
    );
    // the value after --eval is always a formula, even if it looks like an option
    assert_eq!(parse_args(&args(&["--eval", "-1"])), Ok(CliAction::Eval { file: None, formulas: vec!["-1".into()] }));
    assert_eq!(parse_args(&args(&["--eval", "--help"])), Ok(CliAction::Eval { file: None, formulas: vec!["--help".into()] }));
}

#[test]
fn parse_dump() {
    assert_eq!(parse_args(&args(&["--dump", "f.csv"])), Ok(CliAction::Dump { file: "f.csv".into(), width: 80, height: 24 }));
    assert_eq!(parse_args(&args(&["f.csv", "--size", "40x8", "--dump"])), Ok(CliAction::Dump { file: "f.csv".into(), width: 40, height: 8 }));
    assert_eq!(parse_args(&args(&["--dump", "--size", "1x1", "f.csv"])), Ok(CliAction::Dump { file: "f.csv".into(), width: 1, height: 1 }));
    assert_eq!(parse_args(&args(&["--dump", "--size", "1000x1000", "f.csv"])), Ok(CliAction::Dump { file: "f.csv".into(), width: 1000, height: 1000 }));
}

#[test]
fn parse_errors() {
    for bad in [
        &["--bogus"][..],
        &["-z"],
        &["a.csv", "b.csv"],
        &["--eval"],
        &["a.csv", "-e"],
        &["--dump"],
        &["--dump", "--size", "40x8"],
        &["--dump", "f.csv", "--size"],
        &["--dump", "f.csv", "--size", "40"],
        &["--dump", "f.csv", "--size", "40x"],
        &["--dump", "f.csv", "--size", "x8"],
        &["--dump", "f.csv", "--size", "0x8"],
        &["--dump", "f.csv", "--size", "40x0"],
        &["--dump", "f.csv", "--size", "1001x8"],
        &["--dump", "f.csv", "--size", "40X8"],
        &["--dump", "f.csv", "--size", "40x8x2"],
        &["--dump", "f.csv", "--size", "-4x8"],
        &["--dump", "f.csv", "--eval", "1"],
        &["--size", "40x8", "f.csv"],
        &["--size", "40x8", "--eval", "1"],
    ] {
        let e = parse_args(&args(bad)).expect_err(&format!("{bad:?} should be rejected"));
        assert!(!e.is_empty() && !e.contains('\n'), "{bad:?}: {e:?}");
    }
    assert!(parse_args(&args(&["--bogus"])).unwrap_err().contains("--bogus"));
}

// ----------------------------------------------------------------------------
// run: --help, --version, usage errors
// ----------------------------------------------------------------------------

#[test]
fn help_and_version() {
    let (code, out, err) = call(&["--help"]);
    assert_eq!((code, err.as_str()), (0, ""));
    assert!(out.starts_with("Usage:"), "{out}");
    for needle in ["--eval", "--dump", "--size", "--version", "--help"] {
        assert!(out.contains(needle), "help should mention {needle}:\n{out}");
    }
    assert_eq!(call(&["--version"]), (0, format!("gridrs {}\n", env!("CARGO_PKG_VERSION")), String::new()));
}

#[test]
fn usage_errors_exit_2() {
    let (code, out, err) = call(&["--bogus"]);
    assert_eq!(code, 2);
    assert_eq!(out, "");
    assert!(err.starts_with("gridrs: ") && err.contains("--bogus") && err.contains("--help") && err.ends_with('\n'), "{err:?}");
}

// ----------------------------------------------------------------------------
// run: --eval
// ----------------------------------------------------------------------------

#[test]
fn eval_without_a_file() {
    assert_eq!(call(&["--eval", "=SUM(1,2)"]), (0, "3\n".to_string(), String::new()));
    assert_eq!(call(&["--eval", "SUM(1,2)"]).1, "3\n", "the = is optional");
    assert_eq!(call(&["-e", "=1/3"]).1, "0.333333333333333\n");
    assert_eq!(call(&["-e", "=-2^2"]).1, "4\n", "Excel says so");
    assert_eq!(call(&["-e", "=2^3^2"]).1, "64\n", "Excel says so");
    assert_eq!(call(&["-e", "=50%*200"]).1, "100\n");
    assert_eq!(call(&["-e", "=\"a\"&\"b\"&1+1"]).1, "ab2\n");
    assert_eq!(call(&["-e", "=1<2"]).1, "TRUE\n");
    assert_eq!(call(&["-e", "=0.1+0.2=0.3"]).1, "TRUE\n", "as Excel intended: comparisons see 15 significant digits");
    assert_eq!(call(&["-e", "=A1"]).1, "0\n", "an empty sheet");
}

#[test]
fn eval_error_values_are_printed_and_exit_0() {
    assert_eq!(call(&["-e", "=1/0"]), (0, "#DIV/0!\n".to_string(), String::new()));
    assert_eq!(call(&["-e", "=NOPE(1)"]).1, "#NAME?\n");
    assert_eq!(call(&["-e", "=1+"]).1, "#NAME?\n", "a formula that does not parse");
    assert_eq!(call(&["-e", ""]).1, "#NAME?\n");
    assert_eq!(call(&["-e", "=\"x\"*2"]).1, "#VALUE!\n");
    assert_eq!(call(&["-e", "=SQRT(-1)"]).1, "#NUM!\n");
    assert_eq!(call(&["-e", "=VLOOKUP(9,A1:B2,2,FALSE)"]).1, "#N/A\n");
    assert_eq!(call(&["-e", "=A1:B2"]).1, "#VALUE!\n");
}

#[test]
fn eval_every_function_family() {
    let cases = [
        ("=SUM(1,2,3)+AVERAGE(2,4)+MAX(1,5)-MIN(1,5)+COUNT(1,\"a\")", "14"),
        ("=ROUND(2.675,2)", "2.68"),
        ("=MOD(-7,3)&\"|\"&INT(-2.5)&\"|\"&ABS(-3)&\"|\"&POWER(2,10)&\"|\"&SQRT(16)", "2|-3|3|1024|4"),
        ("=IF(1>2,\"yes\",\"no\")", "no"),
        ("=IFERROR(1/0,\"safe\")", "safe"),
        ("=AND(TRUE,1,NOT(FALSE))", "TRUE"),
        ("=OR(FALSE,0)", "FALSE"),
        ("=ISERROR(NA())&ISNUMBER(1)&ISTEXT(\"x\")&ISBLANK(Z99)", "TRUETRUETRUETRUE"),
        ("=UPPER(LEFT(\"gridrs rocks\",6))&LEN(TRIM(\"  a  b  \"))", "GRIDRS3"),
        ("=CONCAT(MID(\"spreadsheet\",7,5),\"-\",RIGHT(\"abc\",2),LOWER(\"OK\"))", "sheet-bcok"),
        ("=WHOPAID()", "Nobody. Strangers' spare tokens wrote this, one signed task at a time."),
    ];
    for (formula, want) in cases {
        assert_eq!(call(&["--eval", formula]), (0, format!("{want}\n"), String::new()), "{formula}");
    }
}

#[test]
fn eval_against_a_file() {
    let dir = Scratch::new("eval");
    let csv = dir.file("demo.csv", DEMO_CSV);
    assert_eq!(call(&[&csv, "--eval", "=D4"]).1, "4\n");
    assert_eq!(call(&["-e", "=SUM(B2:B3)", &csv, "-e", "=A2&\" and \"&A3", "-e", "D2*2"]).1, "13\napple and pear\n3\n", "one line per formula, in order");
    assert_eq!(call(&[&csv, "-e", "=VLOOKUP(\"pear\",A2:D3,4,FALSE)"]).1, "2.5\n");
    assert_eq!(call(&[&csv, "-e", "=INDEX(A1:D4,MATCH(\"pear\",A1:A4,0),3)"]).1, "0.25\n");
    assert_eq!(call(&[&csv, "-e", "=AVERAGE(C2:C3)"]).1, "0.375\n");
    // the native format works the same
    let grid = dir.file("demo.grid", "gridrs 1\ncell A1 20\ncell A2 22\ncell A3 =A1+A2\nformat A3 fixed 2\n");
    assert_eq!(call(&[&grid, "-e", "=A3"]).1, "42\n", "--eval prints General format, whatever the cell's format");
}

#[test]
fn eval_with_a_bad_file_exits_1() {
    let dir = Scratch::new("eval-bad");
    let missing = dir.path("missing.csv");
    let (code, out, err) = call(&[&missing, "-e", "=1"]);
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(err.starts_with("gridrs: ") && err.contains(&missing) && err.ends_with('\n') && err.lines().count() == 1, "{err:?}");
    let broken = dir.file("broken.grid", "this is not a grid file\n");
    let (code, _, err) = call(&[&broken, "-e", "=1"]);
    assert_eq!(code, 1);
    assert!(err.contains(&broken) && err.contains("line 1"), "{err:?}");
}

// ----------------------------------------------------------------------------
// run: --dump
// ----------------------------------------------------------------------------

#[test]
fn dump_prints_one_plain_frame() {
    let dir = Scratch::new("dump");
    let csv = dir.file("demo.csv", DEMO_CSV);
    let (code, out, err) = call(&["--dump", &csv, "--size", "50x8"]);
    assert_eq!((code, err.as_str()), (0, ""));
    assert!(!out.contains('\x1b'), "no escape sequences in a dump");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 8);
    assert_eq!(
        lines[..7],
        [
            "A1 | item",
            "        A          B          C          D",
            "  1 item       qty        price      total",
            "  2 apple               3        0.5        1.5",
            "  3 pear               10       0.25        2.5",
            "  4                                           4",
            "  5",
        ],
        "trailing spaces are trimmed"
    );
    assert!(lines[7].ends_with(" NORMAL") && lines[7].chars().count() == 50, "{:?}", lines[7]);
    assert!(out.ends_with("NORMAL\n"));
}

#[test]
fn dump_defaults_to_80_by_24() {
    let dir = Scratch::new("dump-default");
    let csv = dir.file("d.csv", DEMO_CSV);
    let (code, out, _) = call(&["--dump", &csv]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 24);
    assert_eq!(lines[1], "        A          B          C          D          E          F          G");
    assert_eq!(lines[23].chars().count(), 80);
}

#[test]
fn dump_with_a_bad_file_exits_1() {
    let dir = Scratch::new("dump-bad");
    let missing = dir.path("missing.csv");
    let (code, out, err) = call(&["--dump", &missing]);
    assert_eq!((code, out.as_str()), (1, ""));
    assert!(err.starts_with("gridrs: ") && err.contains(&missing), "{err:?}");
}

// ----------------------------------------------------------------------------
// the event loop
// ----------------------------------------------------------------------------

/// Hands out its input in fixed chunks, one per `read` call.
struct Chunks(Vec<Vec<u8>>);

impl Read for Chunks {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.0.is_empty() {
            return Ok(0);
        }
        let chunk = self.0.remove(0);
        assert!(chunk.len() <= buf.len(), "the event loop must read with a buffer of at least 1024 bytes");
        buf[..chunk.len()].copy_from_slice(&chunk);
        Ok(chunk.len())
    }
}

fn chunks(list: &[&[u8]]) -> Chunks {
    Chunks(list.iter().map(|c| c.to_vec()).collect())
}

fn num(x: f64) -> Value {
    Value::Number(x)
}

#[test]
fn event_loop_types_into_the_sheet_and_quits() {
    let mut app = App::new(Sheet::new(), None);
    app.resize(40, 8);
    // "20" Enter, "22" Enter, "=A1+A2" Enter, then :q! Enter, then keys that must never be read
    let mut input = chunks(&[b"20\r22\r", b"=A1+A2\r", b":q!\r", b"=999\r"]);
    let mut output = Vec::new();
    event_loop::run(&mut app, &mut input, &mut output).unwrap();
    assert_eq!(app.sheet().value(CellRef::new(0, 2)), num(42.0));
    assert_eq!(app.sheet().input(CellRef::new(0, 3)), "", "nothing after the quit is processed");
    assert_eq!(input.0.len(), 1, "and nothing after the quit is even read");

    let output = String::from_utf8(output).unwrap();
    // one frame at start, one after each of the two chunks that did not quit
    assert_eq!(output.matches("\x1b[H").count(), 3, "{output:?}");
    let frames: Vec<String> = output.split("\x1b[H").skip(1).map(strip_ansi).collect();
    let rows = |frame: &str| -> Vec<String> { frame.split("\r\n").map(|l| l.trim_end().to_string()).collect() };
    assert_eq!(rows(&frames[0])[2], "  1");
    assert_eq!(rows(&frames[1])[2..4], ["  1         20", "  2         22"]);
    assert_eq!(rows(&frames[2])[2..5], ["  1         20", "  2         22", "  3         42"]);
    assert_eq!(rows(&frames[2]).len(), 8);
}

#[test]
fn event_loop_stops_at_end_of_input() {
    let mut app = App::new(Sheet::new(), None);
    let mut input = chunks(&[b"7\r"]);
    let mut output = Vec::new();
    event_loop::run(&mut app, &mut input, &mut output).unwrap();
    assert_eq!(app.sheet().value(CellRef::new(0, 0)), num(7.0));
    let mut output = Vec::new();
    event_loop::run(&mut app, &mut chunks(&[]), &mut output).unwrap();
    assert_eq!(String::from_utf8(output).unwrap().matches("\x1b[H").count(), 1, "the first frame is always drawn");
}

#[test]
fn event_loop_understands_escape_sequences_and_a_lone_escape() {
    let mut app = App::new(Sheet::new(), None);
    // Right, Down, "5", Enter; then start typing and cancel with a lone ESC at the end of a chunk
    let mut input = chunks(&[b"\x1b[C\x1b[B5\r", b"=1+1\x1b", b"\x1b[A"]);
    event_loop::run(&mut app, &mut input, &mut Vec::new()).unwrap();
    assert_eq!(app.sheet().value(CellRef::new(1, 1)), num(5.0));
    assert_eq!(app.sheet().cells().len(), 1, "the second edit was cancelled by ESC");
    assert_eq!(app.cursor(), CellRef::new(1, 1), "Enter moved down to B3, then Up came back to B2");
}

#[test]
fn event_loop_reports_write_errors() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "gone"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut app = App::new(Sheet::new(), None);
    assert!(event_loop::run(&mut app, &mut chunks(&[b"1\r"]), &mut Broken).is_err());
}

// ----------------------------------------------------------------------------
// the real binary
// ----------------------------------------------------------------------------

#[test]
fn binary_version_and_help() {
    assert_eq!(gridrs(&["--version"]), (0, format!("gridrs {}\n", env!("CARGO_PKG_VERSION")), String::new()));
    let (code, out, _) = gridrs(&["-h"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("Usage:"));
}

#[test]
fn binary_eval() {
    assert_eq!(gridrs(&["--eval", "=SUM(1,2)"]), (0, "3\n".to_string(), String::new()));
    let dir = Scratch::new("bin-eval");
    let csv = dir.file("demo.csv", DEMO_CSV);
    assert_eq!(gridrs(&[&csv, "-e", "=D4", "-e", "=WHOPAID()"]).1, "4\nNobody. Strangers' spare tokens wrote this, one signed task at a time.\n");
}

#[test]
fn binary_exit_codes() {
    assert_eq!(gridrs(&["--bogus"]).0, 2);
    let dir = Scratch::new("bin-codes");
    assert_eq!(gridrs(&[&dir.path("missing.csv"), "-e", "1"]).0, 1);
}

#[test]
fn binary_dump() {
    let dir = Scratch::new("bin-dump");
    let csv = dir.file("demo.csv", DEMO_CSV);
    let (code, out, _) = gridrs(&["--dump", &csv, "--size", "50x8"]);
    assert_eq!(code, 0);
    assert_eq!(out.lines().nth(4), Some("  3 pear               10       0.25        2.5"));
}

#[test]
fn binary_refuses_to_start_the_editor_without_a_terminal() {
    // stdin is /dev/null here, so raw mode cannot be entered
    let dir = Scratch::new("bin-notty");
    let csv = dir.file("demo.csv", DEMO_CSV);
    for list in [&[][..], &[csv.as_str()][..]] {
        let (code, out, err) = gridrs(list);
        assert_eq!(code, 1, "{list:?}");
        assert!(err.starts_with("gridrs: ") && err.contains("terminal"), "{err:?}");
        assert!(!out.contains("\x1b[?1049h"), "must not switch screens before knowing it has a terminal");
    }
    // a file that cannot be loaded is reported before anything else
    let broken = dir.file("broken.grid", "nope\n");
    let (code, _, err) = gridrs(&[&broken]);
    assert_eq!(code, 1);
    assert!(err.contains(&broken) && err.contains("line 1"), "{err:?}");
}
