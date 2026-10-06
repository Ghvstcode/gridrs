//! Acceptance test for task 0101-formula-lexer. Outside every task's write scope.

use gridrs::formula::lexer::tokenize;
use gridrs::types::{ErrorKind, Token, Token::*};

fn id(s: &str) -> Token {
    Ident(s.to_string())
}
fn txt(s: &str) -> Token {
    Text(s.to_string())
}
fn lex(s: &str) -> Vec<Token> {
    tokenize(s).unwrap_or_else(|e| panic!("tokenize({s:?}) failed: {e}"))
}

#[test]
fn empty_and_whitespace() {
    assert_eq!(lex(""), vec![]);
    assert_eq!(lex("  \t\r\n "), vec![]);
}

#[test]
fn numbers() {
    assert_eq!(lex("0"), vec![Number(0.0)]);
    assert_eq!(lex("42"), vec![Number(42.0)]);
    assert_eq!(lex("3.25"), vec![Number(3.25)]);
    assert_eq!(lex(".5"), vec![Number(0.5)]);
    assert_eq!(lex("5."), vec![Number(5.0)]);
    assert_eq!(lex("1e3"), vec![Number(1000.0)]);
    assert_eq!(lex("1E3"), vec![Number(1000.0)]);
    assert_eq!(lex("2.5e-1"), vec![Number(0.25)]);
    assert_eq!(lex("2E+2"), vec![Number(200.0)]);
    assert_eq!(lex("007"), vec![Number(7.0)]);
}

#[test]
fn a_sign_is_never_part_of_a_number() {
    assert_eq!(lex("-1"), vec![Minus, Number(1.0)]);
    assert_eq!(lex("+1"), vec![Plus, Number(1.0)]);
    assert_eq!(lex("1-1"), vec![Number(1.0), Minus, Number(1.0)]);
    assert_eq!(lex("1e3-2"), vec![Number(1000.0), Minus, Number(2.0)]);
}

#[test]
fn exponent_needs_digits() {
    // `e` is only an exponent when digits follow (after an optional sign).
    assert_eq!(lex("2e"), vec![Number(2.0), id("e")]);
    assert_eq!(lex("2e+x"), vec![Number(2.0), id("e"), Plus, id("x")]);
    assert_eq!(lex("1.5E2E"), vec![Number(150.0), id("E")]);
}

#[test]
fn number_followed_by_dot_or_letters() {
    assert_eq!(lex("1.2.3"), vec![Number(1.2), Number(0.3)]);
    assert_eq!(lex("3A"), vec![Number(3.0), id("A")]);
}

#[test]
fn strings() {
    assert_eq!(lex(r#""hello""#), vec![txt("hello")]);
    assert_eq!(lex(r#""""#), vec![txt("")]);
    assert_eq!(lex(r#""say ""hi""""#), vec![txt(r#"say "hi""#)]);
    assert_eq!(lex(r#""a,b) + 1""#), vec![txt("a,b) + 1")]);
    assert_eq!(lex("\"line\nbreak\""), vec![txt("line\nbreak")]);
    assert_eq!(lex(r#""héllo wörld ✓""#), vec![txt("héllo wörld ✓")]);
    assert_eq!(lex(r#""a"&"b""#), vec![txt("a"), Amp, txt("b")]);
    assert_eq!(lex(r#""a" "b""#), vec![txt("a"), txt("b")]);
}

#[test]
fn identifiers_are_kept_exactly_as_written() {
    assert_eq!(lex("SUM"), vec![id("SUM")]);
    assert_eq!(lex("sum"), vec![id("sum")]);
    assert_eq!(lex("A1"), vec![id("A1")]);
    assert_eq!(lex("$A$1"), vec![id("$A$1")]);
    assert_eq!(lex("a$1"), vec![id("a$1")]);
    assert_eq!(lex("_x.y_2"), vec![id("_x.y_2")]);
    assert_eq!(lex("TRUE false"), vec![id("TRUE"), id("false")]);
    assert_eq!(lex("A1:B2"), vec![id("A1"), Colon, id("B2")]);
}

#[test]
fn operators_and_punctuation() {
    assert_eq!(lex("+-*/^&%"), vec![Plus, Minus, Star, Slash, Caret, Amp, Percent]);
    assert_eq!(lex("= <> < <= > >="), vec![Eq, Ne, Lt, Le, Gt, Ge]);
    assert_eq!(lex("<=>"), vec![Le, Gt]);
    assert_eq!(lex("<>="), vec![Ne, Eq]);
    assert_eq!(lex("><"), vec![Gt, Lt]);
    assert_eq!(lex("=="), vec![Eq, Eq]);
    assert_eq!(lex("(),:"), vec![LParen, RParen, Comma, Colon]);
}

#[test]
fn error_literals() {
    assert_eq!(lex("#DIV/0!"), vec![Error(ErrorKind::Div0)]);
    assert_eq!(lex("#REF!"), vec![Error(ErrorKind::Ref)]);
    assert_eq!(lex("#NAME?"), vec![Error(ErrorKind::Name)]);
    assert_eq!(lex("#VALUE!"), vec![Error(ErrorKind::Value)]);
    assert_eq!(lex("#NUM!"), vec![Error(ErrorKind::Num)]);
    assert_eq!(lex("#N/A"), vec![Error(ErrorKind::NA)]);
    assert_eq!(lex("#CYCLE!"), vec![Error(ErrorKind::Cycle)]);
    assert_eq!(lex("#ref!"), vec![Error(ErrorKind::Ref)]);
    assert_eq!(lex("#n/a"), vec![Error(ErrorKind::NA)]);
    assert_eq!(lex("#REF!+1"), vec![Error(ErrorKind::Ref), Plus, Number(1.0)]);
    assert_eq!(lex("A1+#N/A"), vec![id("A1"), Plus, Error(ErrorKind::NA)]);
}

#[test]
fn a_whole_formula() {
    assert_eq!(
        lex(r#"=IF( $A$1 >= 10, SUM(B1:B7) * 2.5e-1, "n/a" & C3 )"#),
        vec![
            Eq,
            id("IF"),
            LParen,
            id("$A$1"),
            Ge,
            Number(10.0),
            Comma,
            id("SUM"),
            LParen,
            id("B1"),
            Colon,
            id("B7"),
            RParen,
            Star,
            Number(0.25),
            Comma,
            txt("n/a"),
            Amp,
            id("C3"),
            RParen,
        ]
    );
}

#[test]
fn errors() {
    for bad in [r#""unterminated"#, r#""a""#.trim_end_matches('"'), r#""ends with escaped quote"""#, "#", "#NOPE!", "#REF", "#DIV/0", "1 ? 2", "a @ b", "{1,2}", "[A1]", "x;y", "a!b", "1e999", "~", "\\", "'text'", "é", "1 \u{00a0} 2"] {
        assert!(tokenize(bad).is_err(), "tokenize({bad:?}) should fail");
    }
}

#[test]
fn never_panics_on_arbitrary_input() {
    let alphabet = ["1", ".", "e", "E", "+", "-", "\"", "#", "A", "$", "(", ")", ",", ":", "<", ">", "=", " ", "%", "é", "N/A", "!", "?", "_", "0", "/"];
    let mut seed: u64 = 0x9E3779B97F4A7C15;
    for _ in 0..4000 {
        let mut s = String::new();
        for _ in 0..(seed % 9) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            s.push_str(alphabet[(seed >> 33) as usize % alphabet.len()]);
        }
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let _ = tokenize(&s);
    }
}
