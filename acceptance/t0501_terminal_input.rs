//! Acceptance test for task 0501-terminal-input (key decoding + raw mode).
//! Outside every task's write scope.

use gridrs::tui::keys::decode;
use gridrs::tui::term::{parse_size, terminal_size, RawMode, SystemStty, ENTER_SCREEN, LEAVE_SCREEN};
use gridrs::types::{Key, Key::*, Stty};
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

/// Decode with flush = true and require that everything was consumed.
fn keys(bytes: &[u8]) -> Vec<Key> {
    let (keys, used) = decode(bytes, true);
    assert_eq!(used, bytes.len(), "flush must consume everything: {bytes:?}");
    keys
}

// ----------------------------------------------------------------------------
// keys
// ----------------------------------------------------------------------------

#[test]
fn printable_ascii() {
    assert_eq!(keys(b"a"), vec![Char('a')]);
    assert_eq!(keys(b"Hi 1+~"), vec![Char('H'), Char('i'), Char(' '), Char('1'), Char('+'), Char('~')]);
    assert_eq!(keys(b""), vec![]);
    assert_eq!(keys(b":="), vec![Char(':'), Char('=')]);
}

#[test]
fn utf8() {
    assert_eq!(keys("é".as_bytes()), vec![Char('é')]);
    assert_eq!(keys("日本".as_bytes()), vec![Char('日'), Char('本')]);
    assert_eq!(keys("a🦀b".as_bytes()), vec![Char('a'), Char('🦀'), Char('b')]);
}

#[test]
fn control_keys() {
    assert_eq!(keys(b"\r"), vec![Enter]);
    assert_eq!(keys(b"\n"), vec![Enter]);
    assert_eq!(keys(b"\t"), vec![Tab]);
    assert_eq!(keys(b"\x7f"), vec![Backspace]);
    assert_eq!(keys(b"\x08"), vec![Backspace]);
    assert_eq!(keys(b"\x01"), vec![Ctrl('a')]);
    assert_eq!(keys(b"\x03"), vec![Ctrl('c')]);
    assert_eq!(keys(b"\x12"), vec![Ctrl('r')]);
    assert_eq!(keys(b"\x1a"), vec![Ctrl('z')]);
    assert_eq!(keys(b"\x00\x1c\x1d\x1e\x1f"), vec![], "other control bytes are ignored");
    assert_eq!(keys(b"a\x00b"), vec![Char('a'), Char('b')]);
}

#[test]
fn every_ctrl_letter() {
    for (i, letter) in ('a'..='z').enumerate() {
        let byte = (i + 1) as u8;
        let want = match byte {
            0x08 => Backspace,
            0x09 => Tab,
            0x0a | 0x0d => Enter,
            _ => Ctrl(letter),
        };
        assert_eq!(keys(&[byte]), vec![want], "byte {byte:#04x}");
    }
}

#[test]
fn arrows_and_navigation() {
    assert_eq!(keys(b"\x1b[A"), vec![Up]);
    assert_eq!(keys(b"\x1b[B"), vec![Down]);
    assert_eq!(keys(b"\x1b[C"), vec![Right]);
    assert_eq!(keys(b"\x1b[D"), vec![Left]);
    assert_eq!(keys(b"\x1b[H"), vec![Home]);
    assert_eq!(keys(b"\x1b[F"), vec![End]);
    assert_eq!(keys(b"\x1b[Z"), vec![BackTab]);
    assert_eq!(keys(b"\x1b[1~"), vec![Home]);
    assert_eq!(keys(b"\x1b[7~"), vec![Home]);
    assert_eq!(keys(b"\x1b[4~"), vec![End]);
    assert_eq!(keys(b"\x1b[8~"), vec![End]);
    assert_eq!(keys(b"\x1b[3~"), vec![Delete]);
    assert_eq!(keys(b"\x1b[5~"), vec![PageUp]);
    assert_eq!(keys(b"\x1b[6~"), vec![PageDown]);
}

#[test]
fn application_mode_arrows() {
    assert_eq!(keys(b"\x1bOA"), vec![Up]);
    assert_eq!(keys(b"\x1bOB"), vec![Down]);
    assert_eq!(keys(b"\x1bOC"), vec![Right]);
    assert_eq!(keys(b"\x1bOD"), vec![Left]);
    assert_eq!(keys(b"\x1bOH"), vec![Home]);
    assert_eq!(keys(b"\x1bOF"), vec![End]);
    assert_eq!(keys(b"\x1bOP"), vec![], "F1 and friends are consumed and ignored");
}

#[test]
fn modifiers_are_ignored() {
    assert_eq!(keys(b"\x1b[1;5C"), vec![Right], "Ctrl-Right");
    assert_eq!(keys(b"\x1b[1;2A"), vec![Up], "Shift-Up");
    assert_eq!(keys(b"\x1b[3;5~"), vec![Delete], "Ctrl-Delete");
    assert_eq!(keys(b"\x1b[5;2~"), vec![PageUp]);
}

#[test]
fn unknown_sequences_are_swallowed_whole() {
    assert_eq!(keys(b"\x1b[15~"), vec![], "F5");
    assert_eq!(keys(b"\x1b[200~"), vec![], "bracketed paste start");
    assert_eq!(keys(b"\x1b[?25h"), vec![]);
    assert_eq!(keys(b"\x1b[M"), vec![]);
    assert_eq!(keys(b"\x1b[I"), vec![], "focus in");
    assert_eq!(keys(b"a\x1b[15~b"), vec![Char('a'), Char('b')], "and decoding resumes right after");
    assert_eq!(keys(b"\x1b[1;5Qx"), vec![Char('x')]);
    assert_eq!(keys(b"\x1b[ qx"), vec![Char('x')], "intermediate bytes belong to the sequence");
}

#[test]
fn escape_itself() {
    assert_eq!(keys(b"\x1b"), vec![Esc]);
    assert_eq!(keys(b"\x1b\x1b"), vec![Esc, Esc]);
    assert_eq!(keys(b"\x1ba"), vec![Esc, Char('a')], "ESC then a (or Alt-a): two keys");
    assert_eq!(keys(b"\x1b\r"), vec![Esc, Enter]);
    assert_eq!(keys(b"\x1b\x1b[A"), vec![Esc, Up]);
    assert_eq!(keys(b"x\x1b"), vec![Char('x'), Esc]);
}

#[test]
fn a_stream_of_mixed_input() {
    assert_eq!(
        keys(b"=1+2\r\x1b[B\x1b[B:w\r\x1b\x12q"),
        vec![Char('='), Char('1'), Char('+'), Char('2'), Enter, Down, Down, Char(':'), Char('w'), Enter, Esc, Ctrl('r'), Char('q')]
    );
}

#[test]
fn without_flush_incomplete_tails_are_left_for_later() {
    // lone ESC: could be the start of a sequence
    assert_eq!(decode(b"\x1b", false), (vec![], 0));
    assert_eq!(decode(b"ab\x1b", false), (vec![Char('a'), Char('b')], 2));
    assert_eq!(decode(b"\x1b[", false), (vec![], 0));
    assert_eq!(decode(b"\x1bO", false), (vec![], 0));
    assert_eq!(decode(b"\x1b[1;5", false), (vec![], 0));
    assert_eq!(decode(b"x\x1b[1", false), (vec![Char('x')], 1));
    // incomplete UTF-8
    let crab = "🦀".as_bytes();
    assert_eq!(decode(&crab[..1], false), (vec![], 0));
    assert_eq!(decode(&crab[..3], false), (vec![], 0));
    assert_eq!(decode(&[b'a', crab[0], crab[1]], false), (vec![Char('a')], 1));
    // complete input is fully consumed either way
    assert_eq!(decode(b"\x1b[A", false), (vec![Up], 3));
    assert_eq!(decode(b"ab", false), (vec![Char('a'), Char('b')], 2));
    assert_eq!(decode("é".as_bytes(), false), (vec![Char('é')], 2));
    assert_eq!(decode(b"\x1b\x1b", false), (vec![Esc], 1), "the second ESC might still start a sequence");
    assert_eq!(decode(b"", false), (vec![], 0));
}

#[test]
fn split_input_decodes_the_same_as_whole_input() {
    let whole: &[u8] = "a\x1b[A\x1b[1;5C日\x1b[3~é\x1bOBz\x1b[15~!".as_bytes();
    let want = keys(whole);
    assert_eq!(want, vec![Char('a'), Up, Right, Char('日'), Delete, Char('é'), Down, Char('z'), Char('!')]);
    for split in 0..=whole.len() {
        let mut pending: Vec<u8> = vec![];
        let mut got = vec![];
        for (i, chunk) in [&whole[..split], &whole[split..]].iter().enumerate() {
            pending.extend_from_slice(chunk);
            let (k, used) = decode(&pending, i == 1);
            got.extend(k);
            pending.drain(..used);
        }
        assert_eq!(got, want, "split at byte {split}");
        assert!(pending.is_empty());
    }
}

#[test]
fn with_flush_incomplete_tails_are_resolved() {
    assert_eq!(keys(b"\x1b["), vec![], "an unfinished sequence is dropped");
    assert_eq!(keys(b"\x1b[1;5"), vec![]);
    assert_eq!(keys(b"\x1bO"), vec![]);
    assert_eq!(keys(b"a\x1b[1"), vec![Char('a')]);
    assert_eq!(keys(&"🦀".as_bytes()[..2]), vec![], "unfinished UTF-8 is dropped");
}

#[test]
fn invalid_utf8_is_skipped_not_fatal() {
    assert_eq!(keys(b"a\xffb"), vec![Char('a'), Char('b')]);
    assert_eq!(keys(b"\x80\x80x"), vec![Char('x')], "stray continuation bytes");
    assert_eq!(keys(b"\xc3\x28"), vec![Char('(')], "bad continuation: drop the lead byte, keep going");
    assert_eq!(keys(b"\xe6\x97a"), vec![Char('a')]);
    assert_eq!(decode(b"\xffa", false), (vec![Char('a')], 2));
}

#[test]
fn never_panics() {
    let mut seed: u64 = 7;
    let interesting = [0x1b, b'[', b'O', b'~', b';', b'1', b'A', 0xc3, 0xa9, 0xf0, 0x9f, 0x00, 0x7f, b'x', 0xff, 0x80];
    for _ in 0..5000 {
        let mut bytes = vec![];
        for _ in 0..(seed % 12) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            bytes.push(interesting[(seed >> 33) as usize % interesting.len()]);
        }
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let (_, used) = decode(&bytes, false);
        assert!(used <= bytes.len());
        let (_, used) = decode(&bytes, true);
        assert_eq!(used, bytes.len());
    }
}

// ----------------------------------------------------------------------------
// raw mode
// ----------------------------------------------------------------------------

type Log = Rc<RefCell<Vec<Vec<String>>>>;

/// A scripted `stty`: records every invocation, answers `-g` and `size`,
/// and fails on any invocation whose first argument is in `fail_on`.
struct FakeStty {
    log: Log,
    fail_on: Vec<&'static str>,
    size: &'static str,
}

impl FakeStty {
    fn new(log: &Log) -> FakeStty {
        FakeStty { log: log.clone(), fail_on: vec![], size: "40 120\n" }
    }
}

impl Stty for FakeStty {
    fn run(&mut self, args: &[&str]) -> io::Result<String> {
        self.log.borrow_mut().push(args.iter().map(|s| s.to_string()).collect());
        if args.first().is_some_and(|a| self.fail_on.contains(a)) {
            return Err(io::Error::other("stty: stdin isn't a terminal"));
        }
        Ok(match args {
            ["-g"] => "gfmt1:cflag=4b00:iflag=6b02:lflag=200005cb\n".to_string(),
            ["size"] => self.size.to_string(),
            _ => String::new(),
        })
    }
}

fn calls(log: &Log) -> Vec<String> {
    log.borrow().iter().map(|args| args.join(" ")).collect()
}

const SAVED: &str = "gfmt1:cflag=4b00:iflag=6b02:lflag=200005cb";

#[test]
fn enter_saves_then_switches_and_drop_restores() {
    let log: Log = Default::default();
    {
        let _raw = RawMode::enter(FakeStty::new(&log)).expect("enter");
        assert_eq!(calls(&log), ["-g", "raw -echo"]);
    }
    assert_eq!(calls(&log), ["-g", "raw -echo", SAVED], "dropped: saved settings restored, trimmed of the newline");
}

#[test]
fn leave_is_idempotent_and_drop_does_not_restore_twice() {
    let log: Log = Default::default();
    let mut raw = RawMode::enter(FakeStty::new(&log)).expect("enter");
    raw.leave().expect("leave");
    raw.leave().expect("second leave is a no-op");
    drop(raw);
    assert_eq!(calls(&log), ["-g", "raw -echo", SAVED]);
}

#[test]
fn if_saving_fails_nothing_else_is_run() {
    let log: Log = Default::default();
    let mut stty = FakeStty::new(&log);
    stty.fail_on = vec!["-g"];
    assert!(RawMode::enter(stty).is_err());
    assert_eq!(calls(&log), ["-g"]);
}

#[test]
fn if_switching_fails_the_saved_settings_are_put_back_once() {
    let log: Log = Default::default();
    let mut stty = FakeStty::new(&log);
    stty.fail_on = vec!["raw"];
    assert!(RawMode::enter(stty).is_err());
    assert_eq!(calls(&log), ["-g", "raw -echo", SAVED]);
}

#[test]
fn a_failing_restore_is_reported_by_leave_and_swallowed_by_drop() {
    let log: Log = Default::default();
    let mut stty = FakeStty::new(&log);
    stty.fail_on = vec![SAVED];
    let mut raw = RawMode::enter(stty).expect("enter");
    assert!(raw.leave().is_err());
    drop(raw); // must not panic
}

#[test]
fn size_parsing() {
    assert_eq!(parse_size("24 80\n"), Some((24, 80)));
    assert_eq!(parse_size("  50   132  "), Some((50, 132)));
    assert_eq!(parse_size("1 1"), Some((1, 1)));
    for bad in ["", "\n", "24", "24 80 3", "a b", "24x80", "0 80", "24 0", "-1 80", "24 80.5", "99999999999999999999 80"] {
        assert_eq!(parse_size(bad), None, "{bad:?}");
    }
}

#[test]
fn terminal_size_asks_stty_and_falls_back() {
    let log: Log = Default::default();
    let mut stty = FakeStty::new(&log);
    assert_eq!(terminal_size(&mut stty), (40, 120));
    assert_eq!(calls(&log), ["size"]);
    stty.size = "garbage";
    assert_eq!(terminal_size(&mut stty), (24, 80));
    stty.fail_on = vec!["size"];
    assert_eq!(terminal_size(&mut stty), (24, 80));
}

#[test]
fn the_real_stty_fails_cleanly_without_a_terminal() {
    // Whatever stdin is here, this must return (Ok or Err) rather than panic or hang.
    let mut stty = SystemStty;
    let _ = stty.run(&["size"]);
    let (rows, cols) = terminal_size(&mut stty);
    assert!(rows >= 1 && cols >= 1);
    // An invalid argument is an error, not a silent success.
    assert!(stty.run(&["--definitely-not-an-stty-option"]).is_err());
}

#[test]
fn screen_sequences() {
    assert_eq!(ENTER_SCREEN, "\x1b[?1049h\x1b[?25l");
    assert_eq!(LEAVE_SCREEN, "\x1b[?25h\x1b[?1049l");
}
