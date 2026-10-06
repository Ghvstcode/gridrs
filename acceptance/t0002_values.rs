//! Acceptance test for task 0002-values. Outside every task's write scope.

use gridrs::types::{ErrorKind, Value};
use gridrs::value::*;
use std::cmp::Ordering::{Equal, Greater, Less};

fn num(n: f64) -> Value {
    Value::Number(n)
}
fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}

#[test]
fn parse_number_accepts() {
    for (s, want) in [
        ("0", 0.0),
        ("12", 12.0),
        ("-1.5", -1.5),
        ("+3", 3.0),
        (".5", 0.5),
        ("5.", 5.0),
        ("-.25", -0.25),
        ("1e3", 1000.0),
        ("1E3", 1000.0),
        ("1.5e-2", 0.015),
        ("2E+2", 200.0),
        ("12%", 0.12),
        ("-50%", -0.5),
        ("  42  ", 42.0),
        ("\t7\n", 7.0),
        ("007", 7.0),
    ] {
        assert_eq!(parse_number(s), Some(want), "parse_number({s:?})");
    }
}

#[test]
fn parse_number_rejects() {
    for s in ["", " ", ".", "-", "+", "e3", "1e", "1e+", "1,234", "1 2", "0x10", "inf", "-inf", "infinity", "NaN", "nan", "1.2.3", "--1", "+-1", "1%%", "% ", "%", "12 %", "$5", "1e999", "1_000", "١٢"] {
        assert_eq!(parse_number(s), None, "parse_number({s:?})");
    }
}

#[test]
fn parse_literal_classifies_input() {
    assert_eq!(parse_literal(""), Value::Empty);
    assert_eq!(parse_literal("42"), num(42.0));
    assert_eq!(parse_literal(" 42 "), num(42.0));
    assert_eq!(parse_literal("-3.5e1"), num(-35.0));
    assert_eq!(parse_literal("15%"), num(0.15));
    assert_eq!(parse_literal("true"), Value::Bool(true));
    assert_eq!(parse_literal("FALSE"), Value::Bool(false));
    assert_eq!(parse_literal(" True "), Value::Bool(true));
    assert_eq!(parse_literal("#N/A"), Value::Error(ErrorKind::NA));
    assert_eq!(parse_literal("#div/0!"), Value::Error(ErrorKind::Div0));
    assert_eq!(parse_literal("hello"), text("hello"));
    assert_eq!(parse_literal("  padded  "), text("  padded  "), "text keeps its whitespace");
    assert_eq!(parse_literal("   "), text("   "));
    assert_eq!(parse_literal("1,234"), text("1,234"));
    assert_eq!(parse_literal("#nope"), text("#nope"));
    assert_eq!(parse_literal("truely"), text("truely"));
}

#[test]
fn apostrophe_forces_text() {
    assert_eq!(parse_literal("'42"), text("42"));
    assert_eq!(parse_literal("'=A1"), text("=A1"));
    assert_eq!(parse_literal("'TRUE"), text("TRUE"));
    assert_eq!(parse_literal("'"), text(""));
    assert_eq!(parse_literal("''quoted'"), text("'quoted'"));
}

#[test]
fn round_to_is_decimal_half_away_from_zero() {
    for (x, digits, want) in [
        (2.675, 2, 2.68),
        (1.005, 2, 1.01),
        (0.125, 2, 0.13),
        (-2.675, 2, -2.68),
        (2.5, 0, 3.0),
        (-2.5, 0, -3.0),
        (0.5, 0, 1.0),
        (1.4999, 0, 1.0),
        (2.345, 2, 2.35),
        (8.325, 2, 8.33),
        (1234.5678, 2, 1234.57),
        (1234.5678, 0, 1235.0),
        (1234.5678, -2, 1200.0),
        (1250.0, -2, 1300.0),
        (49.0, -2, 0.0),
        (50.0, -2, 100.0),
        (0.1 + 0.2, 15, 0.3),
        (1e20, 2, 1e20),
        (123.456, 20, 123.456),
        (0.0, 3, 0.0),
        (0.000123456, 6, 0.000123),
        (5e-7, 6, 0.000001),
    ] {
        assert_eq!(round_to(x, digits), want, "round_to({x}, {digits})");
    }
    assert_eq!(round_to(123.456, -3), 0.0);
    assert_eq!(round_to(1e300, -400), 0.0);
    assert_eq!(round_to(1.5, i32::MAX), 1.5, "absurd digit counts must not panic");
    assert_eq!(round_to(1.5, i32::MIN), 0.0);
}

#[test]
fn format_fixed_works_on_decimal_digits() {
    for (x, decimals, want) in [
        (1234.5, 2, "1234.50"),
        (1234.5, 0, "1235"),
        (0.0, 2, "0.00"),
        (0.0, 0, "0"),
        (3.0, 3, "3.000"),
        (2.675, 2, "2.68"),
        (1.005, 2, "1.01"),
        (0.125, 2, "0.13"),
        (-2.5, 0, "-3"),
        (-1234.567, 1, "-1234.6"),
        (999.999, 2, "1000.00"),
        (0.999, 0, "1"),
        (5e-7, 6, "0.000001"),
        (0.1 + 0.2, 15, "0.300000000000000"),
        // a double carries 15 significant decimal digits; the rest are zeros, not binary noise
        (123.456, 15, "123.456000000000000"),
        (1.0 / 3.0, 20, "0.33333333333333300000"),
        (1e20, 2, "100000000000000000000.00"),
        (1e-20, 2, "0.00"),
        // never a negative zero
        (-0.001, 2, "0.00"),
        (-0.4, 0, "0"),
        (-0.0, 1, "0.0"),
        (-0.005, 2, "-0.01"),
    ] {
        assert_eq!(format_fixed(x, decimals), want, "format_fixed({x}, {decimals})");
    }
    let huge = format_fixed(f64::MAX, 3);
    assert_eq!(huge.len(), 313, "309 integer digits, a point, 3 decimals");
    assert!(huge.starts_with("179769313486232000") && huge.ends_with("000.000"), "{huge}");
    assert_eq!(format_fixed(f64::MIN_POSITIVE, 4), "0.0000");
}

#[test]
fn number_to_text_general() {
    for (n, want) in [
        (0.0, "0"),
        (-0.0, "0"),
        (1.0, "1"),
        (-1.0, "-1"),
        (1.5, "1.5"),
        (100.0, "100"),
        (1234567.0, "1234567"),
        (0.1, "0.1"),
        (0.1 + 0.2, "0.3"),
        (1.0 / 3.0, "0.333333333333333"),
        (2.0 / 3.0, "0.666666666666667"),
        (-2.0 / 3.0, "-0.666666666666667"),
        (123456789012345.0, "123456789012345"),
        (999999999999999.0, "999999999999999"),
        (1e15, "1E+15"),
        (1.5e15, "1.5E+15"),
        (1e21, "1E+21"),
        (1.2345e100, "1.2345E+100"),
        (-1e15, "-1E+15"),
        (0.001, "0.001"),
        (0.000000001, "0.000000001"),
        (0.000000000123, "1.23E-10"),
        (1e-9, "0.000000001"),
        (9.99e-10, "9.99E-10"),
        (1e-300, "1E-300"),
        (1234.5678, "1234.5678"),
        (0.1 + 0.7, "0.8"),
        (100.0 * 1.1, "110"),
    ] {
        assert_eq!(number_to_text(n), want, "number_to_text({n:e})");
    }
}

#[test]
fn to_number_coercions() {
    assert_eq!(to_number(&num(2.5)), Ok(2.5));
    assert_eq!(to_number(&Value::Empty), Ok(0.0));
    assert_eq!(to_number(&Value::Bool(true)), Ok(1.0));
    assert_eq!(to_number(&Value::Bool(false)), Ok(0.0));
    assert_eq!(to_number(&text("12")), Ok(12.0));
    assert_eq!(to_number(&text(" 1e2 ")), Ok(100.0));
    assert_eq!(to_number(&text("50%")), Ok(0.5));
    assert_eq!(to_number(&text("abc")), Err(ErrorKind::Value));
    assert_eq!(to_number(&text("")), Err(ErrorKind::Value));
    assert_eq!(to_number(&text("TRUE")), Err(ErrorKind::Value));
    assert_eq!(to_number(&Value::Error(ErrorKind::Div0)), Err(ErrorKind::Div0));
}

#[test]
fn to_text_coercions() {
    assert_eq!(to_text(&text("hi")), Ok("hi".to_string()));
    assert_eq!(to_text(&Value::Empty), Ok(String::new()));
    assert_eq!(to_text(&num(3.0)), Ok("3".to_string()));
    assert_eq!(to_text(&num(1.0 / 3.0)), Ok("0.333333333333333".to_string()));
    assert_eq!(to_text(&Value::Bool(true)), Ok("TRUE".to_string()));
    assert_eq!(to_text(&Value::Bool(false)), Ok("FALSE".to_string()));
    assert_eq!(to_text(&Value::Error(ErrorKind::NA)), Err(ErrorKind::NA));
}

#[test]
fn to_bool_coercions() {
    assert_eq!(to_bool(&Value::Bool(true)), Ok(true));
    assert_eq!(to_bool(&num(0.0)), Ok(false));
    assert_eq!(to_bool(&num(-2.0)), Ok(true));
    assert_eq!(to_bool(&Value::Empty), Ok(false));
    assert_eq!(to_bool(&text("true")), Ok(true));
    assert_eq!(to_bool(&text("FALSE")), Ok(false));
    assert_eq!(to_bool(&text("yes")), Err(ErrorKind::Value));
    assert_eq!(to_bool(&text("1")), Err(ErrorKind::Value));
    assert_eq!(to_bool(&Value::Error(ErrorKind::Ref)), Err(ErrorKind::Ref));
}

#[test]
fn compare_within_a_type() {
    assert_eq!(compare(&num(1.0), &num(2.0)), Less);
    assert_eq!(compare(&num(2.0), &num(2.0)), Equal);
    assert_eq!(compare(&num(-1.0), &num(-2.0)), Greater);
    assert_eq!(compare(&text("apple"), &text("Banana")), Less);
    assert_eq!(compare(&text("APPLE"), &text("apple")), Equal, "text compares case-insensitively");
    assert_eq!(compare(&text("b"), &text("A")), Greater);
    assert_eq!(compare(&text("abc"), &text("abcd")), Less);
    assert_eq!(compare(&text(""), &text("a")), Less);
    assert_eq!(compare(&Value::Bool(false), &Value::Bool(true)), Less);
    assert_eq!(compare(&Value::Bool(true), &Value::Bool(true)), Equal);
}

#[test]
fn compare_sees_numbers_at_15_significant_digits_like_excel() {
    assert_eq!(compare(&num(0.1 + 0.2), &num(0.3)), Equal, "the famous one");
    assert_eq!(compare(&num(0.3), &num(0.1 + 0.2)), Equal);
    assert_eq!(compare(&num(1.0), &num(1.0000000000000002)), Equal, "differs only in the 16th digit");
    assert_eq!(compare(&num(1.0), &num(1.00000000000001)), Less, "differs in the 15th digit");
    assert_eq!(compare(&num(-1.00000000000001), &num(-1.0)), Less);
    assert_eq!(compare(&num(100.0 * 1.1), &num(110.0)), Equal);
    assert_eq!(compare(&num(1e300), &num(1.0000000000000002e300)), Equal);
    assert_eq!(compare(&num(0.0), &num(1e-300)), Less, "tiny is still not zero");
    assert_eq!(compare(&num(0.0), &num(-0.0)), Equal);
}

#[test]
fn compare_across_types_orders_number_text_bool() {
    assert_eq!(compare(&num(1e300), &text("")), Less);
    assert_eq!(compare(&text("zzz"), &Value::Bool(false)), Less);
    assert_eq!(compare(&num(5.0), &Value::Bool(false)), Less);
    assert_eq!(compare(&Value::Bool(true), &num(5.0)), Greater);
    assert_eq!(compare(&text("1"), &num(1.0)), Greater, "text is never equal to a number");
}

#[test]
fn compare_with_empty_adapts_to_the_other_side() {
    assert_eq!(compare(&Value::Empty, &Value::Empty), Equal);
    assert_eq!(compare(&Value::Empty, &num(0.0)), Equal);
    assert_eq!(compare(&Value::Empty, &num(1.0)), Less);
    assert_eq!(compare(&num(-1.0), &Value::Empty), Less);
    assert_eq!(compare(&Value::Empty, &text("")), Equal);
    assert_eq!(compare(&Value::Empty, &text("a")), Less);
    assert_eq!(compare(&Value::Empty, &Value::Bool(false)), Equal);
    assert_eq!(compare(&Value::Bool(true), &Value::Empty), Greater);
}
