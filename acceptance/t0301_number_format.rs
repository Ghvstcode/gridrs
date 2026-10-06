//! Acceptance test for task 0301-number-format. Outside every task's write scope.

use gridrs::numfmt::{align_of, format_value};
use gridrs::types::{Align, ErrorKind, NumFormat, NumFormat::*, Value};

fn fmt(n: f64, f: NumFormat) -> String {
    format_value(&Value::Number(n), f)
}

#[test]
fn non_numbers_ignore_the_format() {
    for f in [General, Fixed(2), Thousands(0), Percent(1)] {
        assert_eq!(format_value(&Value::Empty, f), "");
        assert_eq!(format_value(&Value::Text("hello".into()), f), "hello");
        assert_eq!(format_value(&Value::Text("1234.5".into()), f), "1234.5", "text that looks like a number is still text");
        assert_eq!(format_value(&Value::Text("  spaced ".into()), f), "  spaced ");
        assert_eq!(format_value(&Value::Bool(true), f), "TRUE");
        assert_eq!(format_value(&Value::Bool(false), f), "FALSE");
        for e in ErrorKind::ALL {
            assert_eq!(format_value(&Value::Error(e), f), e.as_str());
        }
    }
}

#[test]
fn general() {
    assert_eq!(fmt(0.0, General), "0");
    assert_eq!(fmt(42.0, General), "42");
    assert_eq!(fmt(-1.5, General), "-1.5");
    assert_eq!(fmt(0.1 + 0.2, General), "0.3");
    assert_eq!(fmt(1.0 / 3.0, General), "0.333333333333333");
    assert_eq!(fmt(1e15, General), "1E+15");
    assert_eq!(fmt(1234567.0, General), "1234567");
}

#[test]
fn fixed() {
    assert_eq!(fmt(1234.5, Fixed(2)), "1234.50");
    assert_eq!(fmt(1234.5, Fixed(0)), "1235");
    assert_eq!(fmt(0.0, Fixed(2)), "0.00");
    assert_eq!(fmt(0.0, Fixed(0)), "0");
    assert_eq!(fmt(-1234.567, Fixed(1)), "-1234.6");
    assert_eq!(fmt(3.0, Fixed(3)), "3.000");
    assert_eq!(fmt(0.5, Fixed(0)), "1");
    assert_eq!(fmt(1.0 / 3.0, Fixed(4)), "0.3333");
    assert_eq!(fmt(2.0 / 3.0, Fixed(4)), "0.6667");
    assert_eq!(fmt(999.999, Fixed(2)), "1000.00");
    assert_eq!(fmt(0.999, Fixed(0)), "1");
}

#[test]
fn fixed_rounds_in_decimal_half_away_from_zero() {
    assert_eq!(fmt(2.675, Fixed(2)), "2.68");
    assert_eq!(fmt(1.005, Fixed(2)), "1.01");
    assert_eq!(fmt(0.125, Fixed(2)), "0.13");
    assert_eq!(fmt(2.5, Fixed(0)), "3");
    assert_eq!(fmt(-2.5, Fixed(0)), "-3");
    assert_eq!(fmt(-2.675, Fixed(2)), "-2.68");
    assert_eq!(fmt(0.1 + 0.2, Fixed(15)), "0.300000000000000");
}

#[test]
fn nothing_is_ever_negative_zero() {
    assert_eq!(fmt(-0.001, Fixed(2)), "0.00");
    assert_eq!(fmt(-0.4, Fixed(0)), "0");
    assert_eq!(fmt(-0.0, Fixed(1)), "0.0");
    assert_eq!(fmt(-0.004, Thousands(2)), "0.00");
    assert_eq!(fmt(-0.00004, Percent(2)), "0.00%");
    assert_eq!(fmt(-0.005, Fixed(2)), "-0.01", "but this one really rounds to -0.01");
}

#[test]
fn thousands() {
    assert_eq!(fmt(1234567.891, Thousands(2)), "1,234,567.89");
    assert_eq!(fmt(1234567.891, Thousands(0)), "1,234,568");
    assert_eq!(fmt(999.0, Thousands(0)), "999");
    assert_eq!(fmt(1000.0, Thousands(0)), "1,000");
    assert_eq!(fmt(999.5, Thousands(0)), "1,000");
    assert_eq!(fmt(999999.999, Thousands(2)), "1,000,000.00");
    assert_eq!(fmt(100000.0, Thousands(1)), "100,000.0");
    assert_eq!(fmt(-1234.5, Thousands(2)), "-1,234.50");
    assert_eq!(fmt(-123.0, Thousands(0)), "-123");
    assert_eq!(fmt(0.0, Thousands(2)), "0.00");
    assert_eq!(fmt(0.5, Thousands(3)), "0.500");
    assert_eq!(fmt(1e12, Thousands(0)), "1,000,000,000,000");
    assert_eq!(fmt(12.0, Thousands(4)), "12.0000", "decimals are never grouped");
}

#[test]
fn percent() {
    assert_eq!(fmt(0.125, Percent(1)), "12.5%");
    assert_eq!(fmt(0.125, Percent(0)), "13%");
    assert_eq!(fmt(1.0, Percent(0)), "100%");
    assert_eq!(fmt(0.0, Percent(2)), "0.00%");
    assert_eq!(fmt(-0.5, Percent(0)), "-50%");
    assert_eq!(fmt(12.0, Percent(0)), "1200%", "no thousands separator in percentages");
    assert_eq!(fmt(0.07, Percent(0)), "7%", "0.07 * 100 must not show up as 7.000000000000001");
    assert_eq!(fmt(0.07, Percent(2)), "7.00%");
    assert_eq!(fmt(0.285, Percent(0)), "29%", "28.5 rounds half away from zero");
    assert_eq!(fmt(0.1 + 0.2, Percent(1)), "30.0%");
    assert_eq!(fmt(1.0 / 3.0, Percent(2)), "33.33%");
}

#[test]
fn large_and_small_magnitudes_do_not_panic() {
    assert_eq!(fmt(1e20, Fixed(2)), "100000000000000000000.00");
    assert_eq!(fmt(1e20, Thousands(0)), "100,000,000,000,000,000,000");
    assert_eq!(fmt(1e-20, Fixed(2)), "0.00");
    assert_eq!(fmt(123.456, Fixed(15)), "123.456000000000000");
    assert_eq!(fmt(1e300, Percent(0)).len(), 304, "1 followed by 302 zeros and a percent sign");
    assert!(fmt(f64::MAX, Fixed(15)).starts_with("17976931348623"));
    assert!(fmt(f64::MIN_POSITIVE, Fixed(15)).starts_with("0.000"));
}

#[test]
fn alignment() {
    assert_eq!(align_of(&Value::Number(1.0)), Align::Right);
    assert_eq!(align_of(&Value::Text("x".into())), Align::Left);
    assert_eq!(align_of(&Value::Text("12".into())), Align::Left);
    assert_eq!(align_of(&Value::Empty), Align::Left);
    assert_eq!(align_of(&Value::Bool(true)), Align::Center);
    assert_eq!(align_of(&Value::Error(ErrorKind::Div0)), Align::Center);
}
