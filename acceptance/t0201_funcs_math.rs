//! Acceptance test for task 0201-funcs-math. Outside every task's write scope.

include!("common_funcs.rs");

fn f(name: &str, args: &[Arg]) -> Value {
    gridrs::funcs::math::call(name, args).unwrap_or_else(|| panic!("math::call does not implement {name}"))
}

fn mixed() -> Arg {
    // numbers 1, 2.5, -4 plus things a range aggregate must skip
    grid(3, vec![num(1.0), text("7"), Value::Empty, boolean(true), num(2.5), text("x"), num(-4.0), Value::Empty, text("")])
}

#[test]
fn implements_exactly_its_family() {
    for name in ["SUM", "AVERAGE", "MIN", "MAX", "COUNT", "COUNTA", "PRODUCT", "ABS", "ROUND", "INT", "MOD", "SQRT", "POWER"] {
        assert!(gridrs::funcs::math::call(name, &[n(1.0), n(1.0)]).is_some(), "{name} should be implemented");
    }
    for name in ["IF", "LEN", "VLOOKUP", "NOPE", "", "sum", "Sum", "SUMX", "SU"] {
        assert_eq!(gridrs::funcs::math::call(name, &[n(1.0)]), None, "{name:?} is not a math function");
    }
}

#[test]
fn reachable_through_the_dispatcher() {
    assert_eq!(gridrs::funcs::call("SUM", &[n(1.0), n(2.0)]), num(3.0));
    assert_eq!(gridrs::funcs::call("ABS", &[n(-2.0)]), num(2.0));
}

#[test]
fn sum() {
    assert_eq!(f("SUM", &[n(1.0), n(2.0), n(3.5)]), num(6.5));
    assert_eq!(f("SUM", &[mixed()]), num(-0.5));
    assert_eq!(f("SUM", &[mixed(), n(10.0), mixed()]), num(9.0));
    assert_eq!(f("SUM", &[column(vec![Value::Empty, text("a")])]), num(0.0));
    assert_eq!(f("SUM", &[n(0.1), n(0.2)]), num(0.1 + 0.2));
}

#[test]
fn scalar_arguments_are_coerced_range_cells_are_not() {
    // typed directly into the formula: text that looks like a number counts, booleans count
    assert_eq!(f("SUM", &[s("7"), b(true), n(1.0)]), num(9.0));
    assert_eq!(f("SUM", &[s("abc")]), err(ErrorKind::Value));
    // the same things inside a range are skipped
    assert_eq!(f("SUM", &[row(vec![text("7"), boolean(true), num(1.0)])]), num(1.0));
    // a reference to an empty cell is skipped
    assert_eq!(f("SUM", &[empty(), n(2.0)]), num(2.0));
    assert_eq!(f("COUNT", &[empty(), n(2.0)]), num(1.0));
    assert_eq!(f("AVERAGE", &[empty(), n(2.0), n(4.0)]), num(3.0));
    assert_eq!(f("PRODUCT", &[empty(), n(2.0), n(4.0)]), num(8.0));
    assert_eq!(f("MIN", &[empty(), n(2.0), n(4.0)]), num(2.0));
}

#[test]
fn aggregates_propagate_the_first_error() {
    let bad = grid(2, vec![num(1.0), err(NA), err(Div0), num(2.0)]);
    for name in ["SUM", "AVERAGE", "MIN", "MAX", "PRODUCT"] {
        assert_eq!(f(name, std::slice::from_ref(&bad)), err(NA), "{name}: first error in row-major order");
        assert_eq!(f(name, &[n(1.0), e(Ref), bad.clone()]), err(Ref), "{name}: arguments left to right");
        assert_eq!(f(name, &[bad.clone(), e(Ref)]), err(NA), "{name}");
        assert_eq!(f(name, &[s("abc"), e(Ref)]), err(ErrorKind::Value), "{name}: a failed coercion is an error too");
    }
}

#[test]
fn average() {
    assert_eq!(f("AVERAGE", &[n(1.0), n(2.0), n(6.0)]), num(3.0));
    assert_eq!(f("AVERAGE", &[mixed()]), num(-0.5 / 3.0));
    assert_eq!(f("AVERAGE", &[column(vec![text("a"), Value::Empty])]), err(Div0));
    assert_eq!(f("AVERAGE", &[empty()]), err(Div0));
}

#[test]
fn min_max() {
    assert_eq!(f("MIN", &[n(3.0), n(-1.0), n(2.0)]), num(-1.0));
    assert_eq!(f("MAX", &[n(3.0), n(-1.0), n(2.0)]), num(3.0));
    assert_eq!(f("MIN", &[mixed()]), num(-4.0));
    assert_eq!(f("MAX", &[mixed()]), num(2.5));
    assert_eq!(f("MAX", &[mixed(), n(9.0)]), num(9.0));
    assert_eq!(f("MIN", &[column(vec![text("a")])]), num(0.0), "no numbers at all: 0, like Excel");
    assert_eq!(f("MAX", &[column(vec![Value::Empty])]), num(0.0));
    assert_eq!(f("MAX", &[n(-5.0)]), num(-5.0));
    assert_eq!(f("MIN", &[b(true), n(5.0)]), num(1.0));
}

#[test]
fn count_and_counta() {
    assert_eq!(f("COUNT", &[mixed()]), num(3.0));
    assert_eq!(f("COUNTA", &[mixed()]), num(7.0), "everything except the two empty cells");
    assert_eq!(f("COUNT", &[n(1.0), s("2"), s("x"), b(true), empty()]), num(3.0));
    assert_eq!(f("COUNTA", &[n(1.0), s("2"), s("x"), b(true), empty(), s("")]), num(5.0));
    // COUNT ignores errors, COUNTA counts them
    assert_eq!(f("COUNT", &[row(vec![num(1.0), err(NA)]), e(Div0)]), num(1.0));
    assert_eq!(f("COUNTA", &[row(vec![num(1.0), err(NA)]), e(Div0)]), num(3.0));
    assert_eq!(f("COUNT", &[column(vec![Value::Empty])]), num(0.0));
}

#[test]
fn product() {
    assert_eq!(f("PRODUCT", &[n(2.0), n(3.0), n(4.0)]), num(24.0));
    assert_eq!(f("PRODUCT", &[mixed()]), num(-10.0));
    assert_eq!(f("PRODUCT", &[column(vec![text("a")])]), num(0.0), "no numbers at all: 0, like Excel");
    assert_eq!(f("PRODUCT", &[n(1e200), n(1e200)]), err(Num));
}

#[test]
fn overflow_is_num_error() {
    assert_eq!(f("SUM", &[n(f64::MAX), n(f64::MAX)]), err(Num));
    assert_eq!(f("POWER", &[n(10.0), n(999.0)]), err(Num));
}

#[test]
fn abs_int_sqrt() {
    assert_eq!(f("ABS", &[n(-2.5)]), num(2.5));
    assert_eq!(f("ABS", &[s("-3")]), num(3.0));
    assert_eq!(f("ABS", &[empty()]), num(0.0), "a scalar function reads an empty cell as 0");
    assert_eq!(f("INT", &[n(2.7)]), num(2.0));
    assert_eq!(f("INT", &[n(-2.1)]), num(-3.0), "INT rounds down, not towards zero");
    assert_eq!(f("INT", &[b(true)]), num(1.0));
    assert_eq!(f("SQRT", &[n(16.0)]), num(4.0));
    assert_eq!(f("SQRT", &[n(0.0)]), num(0.0));
    assert_eq!(f("SQRT", &[n(-1.0)]), err(Num));
}

#[test]
fn round() {
    assert_eq!(f("ROUND", &[n(2.675), n(2.0)]), num(2.68), "decimal rounding, not binary");
    assert_eq!(f("ROUND", &[n(1.005), n(2.0)]), num(1.01));
    assert_eq!(f("ROUND", &[n(2.5), n(0.0)]), num(3.0));
    assert_eq!(f("ROUND", &[n(-2.5), n(0.0)]), num(-3.0), "half away from zero");
    assert_eq!(f("ROUND", &[n(1234.5678), n(-2.0)]), num(1200.0));
    assert_eq!(f("ROUND", &[n(1234.5678), n(1.9)]), num(1234.6), "digits are truncated towards zero");
    assert_eq!(f("ROUND", &[n(1234.5678), n(-1.9)]), num(1230.0));
    assert_eq!(f("ROUND", &[s("2.5"), s("0")]), num(3.0));
    assert_eq!(f("ROUND", &[n(1.5), n(1e9)]), num(1.5), "absurd digit counts are clamped, not a panic");
    assert_eq!(f("ROUND", &[n(1.5), n(-1e9)]), num(0.0));
}

#[test]
fn mod_takes_the_sign_of_the_divisor() {
    assert_eq!(f("MOD", &[n(7.0), n(3.0)]), num(1.0));
    assert_eq!(f("MOD", &[n(-7.0), n(3.0)]), num(2.0));
    assert_eq!(f("MOD", &[n(7.0), n(-3.0)]), num(-2.0));
    assert_eq!(f("MOD", &[n(-7.0), n(-3.0)]), num(-1.0));
    assert_eq!(f("MOD", &[n(7.5), n(2.0)]), num(1.5));
    assert_eq!(f("MOD", &[n(6.0), n(3.0)]), num(0.0));
    assert_eq!(f("MOD", &[n(7.0), n(0.0)]), err(Div0));
}

#[test]
fn power() {
    assert_eq!(f("POWER", &[n(2.0), n(10.0)]), num(1024.0));
    assert_eq!(f("POWER", &[n(4.0), n(0.5)]), num(2.0));
    assert_eq!(f("POWER", &[n(2.0), n(-1.0)]), num(0.5));
    assert_eq!(f("POWER", &[n(-8.0), n(0.5)]), err(Num));
}

#[test]
fn scalar_functions_propagate_errors_and_reject_bad_input() {
    for name in ["ABS", "INT", "SQRT"] {
        assert_eq!(f(name, &[e(NA)]), err(NA), "{name}");
        assert_eq!(f(name, &[s("abc")]), err(ErrorKind::Value), "{name}");
        assert_eq!(f(name, &[column(vec![num(1.0)])]), err(ErrorKind::Value), "{name}: a range is not a scalar");
        assert_eq!(f(name, &[]), err(ErrorKind::Value), "{name}: too few arguments");
        assert_eq!(f(name, &[n(1.0), n(2.0)]), err(ErrorKind::Value), "{name}: too many arguments");
    }
    for name in ["ROUND", "MOD", "POWER"] {
        assert_eq!(f(name, &[e(NA), n(1.0)]), err(NA), "{name}");
        assert_eq!(f(name, &[n(1.0), e(Div0)]), err(Div0), "{name}");
        assert_eq!(f(name, &[e(NA), e(Div0)]), err(NA), "{name}: first argument first");
        assert_eq!(f(name, &[s("abc"), n(1.0)]), err(ErrorKind::Value), "{name}");
        assert_eq!(f(name, &[n(1.0), column(vec![num(1.0)])]), err(ErrorKind::Value), "{name}");
        assert_eq!(f(name, &[n(1.0)]), err(ErrorKind::Value), "{name}: too few arguments");
        assert_eq!(f(name, &[n(1.0), n(1.0), n(1.0)]), err(ErrorKind::Value), "{name}: too many arguments");
    }
}

#[test]
fn aggregates_need_at_least_one_argument() {
    for name in ["SUM", "AVERAGE", "MIN", "MAX", "COUNT", "COUNTA", "PRODUCT"] {
        assert_eq!(f(name, &[]), err(ErrorKind::Value), "{name}()");
    }
}
