//! Acceptance test for task 0203-funcs-text. Outside every task's write scope.

include!("common_funcs.rs");

fn f(name: &str, args: &[Arg]) -> Value {
    gridrs::funcs::text::call(name, args).unwrap_or_else(|| panic!("text::call does not implement {name}"))
}

#[test]
fn implements_exactly_its_family() {
    for name in ["LEN", "LEFT", "RIGHT", "MID", "UPPER", "LOWER", "TRIM", "CONCAT"] {
        assert!(gridrs::funcs::text::call(name, &[s("x")]).is_some(), "{name} should be implemented");
    }
    for name in ["SUM", "IF", "VLOOKUP", "NOPE", "", "len", "Len", "LENB", "LE"] {
        assert_eq!(gridrs::funcs::text::call(name, &[s("x")]), None, "{name:?} is not a text function");
    }
}

#[test]
fn reachable_through_the_dispatcher() {
    assert_eq!(gridrs::funcs::call("LEN", &[s("abc")]), num(3.0));
    assert_eq!(gridrs::funcs::call("UPPER", &[s("abc")]), text("ABC"));
}

#[test]
fn len_counts_characters_not_bytes() {
    assert_eq!(f("LEN", &[s("")]), num(0.0));
    assert_eq!(f("LEN", &[s("hello")]), num(5.0));
    assert_eq!(f("LEN", &[s("  a  ")]), num(5.0));
    assert_eq!(f("LEN", &[s("héllo")]), num(5.0));
    assert_eq!(f("LEN", &[s("日本語")]), num(3.0));
    // other types are converted to text first
    assert_eq!(f("LEN", &[n(1234.5)]), num(6.0));
    assert_eq!(f("LEN", &[b(true)]), num(4.0));
    assert_eq!(f("LEN", &[empty()]), num(0.0));
}

#[test]
fn left_and_right() {
    assert_eq!(f("LEFT", &[s("hello"), n(2.0)]), text("he"));
    assert_eq!(f("RIGHT", &[s("hello"), n(2.0)]), text("lo"));
    assert_eq!(f("LEFT", &[s("hello")]), text("h"), "count defaults to 1");
    assert_eq!(f("RIGHT", &[s("hello")]), text("o"));
    assert_eq!(f("LEFT", &[s("hello"), n(0.0)]), text(""));
    assert_eq!(f("RIGHT", &[s("hello"), n(0.0)]), text(""));
    assert_eq!(f("LEFT", &[s("hello"), n(99.0)]), text("hello"));
    assert_eq!(f("RIGHT", &[s("hello"), n(99.0)]), text("hello"));
    assert_eq!(f("LEFT", &[s("hello"), n(2.9)]), text("he"), "count is truncated");
    assert_eq!(f("LEFT", &[s("hello"), s("3")]), text("hel"));
    assert_eq!(f("LEFT", &[s("日本語テキスト"), n(3.0)]), text("日本語"));
    assert_eq!(f("RIGHT", &[s("日本語テキスト"), n(4.0)]), text("テキスト"));
    assert_eq!(f("LEFT", &[n(12345.0), n(2.0)]), text("12"));
    assert_eq!(f("RIGHT", &[b(false), n(3.0)]), text("LSE"));
    assert_eq!(f("LEFT", &[s(""), n(3.0)]), text(""));
    assert_eq!(f("LEFT", &[s("hello"), n(-1.0)]), err(ErrorKind::Value));
    assert_eq!(f("RIGHT", &[s("hello"), n(-1.0)]), err(ErrorKind::Value));
    assert_eq!(f("LEFT", &[s("hello"), n(1e300)]), text("hello"));
}

#[test]
fn mid() {
    assert_eq!(f("MID", &[s("hello world"), n(1.0), n(5.0)]), text("hello"));
    assert_eq!(f("MID", &[s("hello world"), n(7.0), n(5.0)]), text("world"));
    assert_eq!(f("MID", &[s("hello world"), n(7.0), n(99.0)]), text("world"));
    assert_eq!(f("MID", &[s("hello"), n(3.0), n(0.0)]), text(""));
    assert_eq!(f("MID", &[s("hello"), n(6.0), n(2.0)]), text(""), "start just past the end");
    assert_eq!(f("MID", &[s("hello"), n(60.0), n(2.0)]), text(""));
    assert_eq!(f("MID", &[s("日本語テキスト"), n(4.0), n(2.0)]), text("テキ"));
    assert_eq!(f("MID", &[s("hello"), n(2.7), n(2.7)]), text("el"), "both numbers are truncated");
    assert_eq!(f("MID", &[s("hello"), n(0.0), n(2.0)]), err(ErrorKind::Value), "positions start at 1");
    assert_eq!(f("MID", &[s("hello"), n(-1.0), n(2.0)]), err(ErrorKind::Value));
    assert_eq!(f("MID", &[s("hello"), n(1.0), n(-2.0)]), err(ErrorKind::Value));
    assert_eq!(f("MID", &[s("hello"), n(1.0)]), err(ErrorKind::Value), "MID has no optional arguments");
    assert_eq!(f("MID", &[s("hello"), n(1e300), n(1e300)]), text(""));
}

#[test]
fn upper_lower() {
    assert_eq!(f("UPPER", &[s("Hello, World 1")]), text("HELLO, WORLD 1"));
    assert_eq!(f("LOWER", &[s("Hello, World 1")]), text("hello, world 1"));
    assert_eq!(f("UPPER", &[s("héllo")]), text("HÉLLO"));
    assert_eq!(f("LOWER", &[s("ÉCOLE")]), text("école"));
    assert_eq!(f("UPPER", &[b(true)]), text("TRUE"));
    assert_eq!(f("LOWER", &[b(true)]), text("true"));
    assert_eq!(f("UPPER", &[n(1.5)]), text("1.5"));
}

#[test]
fn trim_like_excel() {
    assert_eq!(f("TRIM", &[s("  hello  ")]), text("hello"));
    assert_eq!(f("TRIM", &[s("  hello    big   world ")]), text("hello big world"), "inner runs collapse to one space");
    assert_eq!(f("TRIM", &[s("nothing")]), text("nothing"));
    assert_eq!(f("TRIM", &[s("     ")]), text(""));
    assert_eq!(f("TRIM", &[s("")]), text(""));
    assert_eq!(f("TRIM", &[s(" a\tb \n c ")]), text("a\tb \n c"), "only the space character U+0020 is trimmed");
    assert_eq!(f("TRIM", &[n(5.0)]), text("5"));
}

#[test]
fn concat() {
    assert_eq!(f("CONCAT", &[s("a"), s("b"), s("c")]), text("abc"));
    assert_eq!(f("CONCAT", &[s("a")]), text("a"));
    assert_eq!(f("CONCAT", &[s("n="), n(1.5), s(" ok="), b(true), empty(), s("!")]), text("n=1.5 ok=TRUE!"));
    // ranges are flattened row by row; empty cells contribute nothing
    assert_eq!(f("CONCAT", &[grid(2, vec![text("a"), num(1.0), Value::Empty, boolean(false)]), s("-"), row(vec![text("z")])]), text("a1FALSE-z"));
    assert_eq!(f("CONCAT", &[n(1.0 / 3.0)]), text("0.333333333333333"));
    assert_eq!(f("CONCAT", &[]), err(ErrorKind::Value));
    assert_eq!(f("CONCAT", &[s("a"), e(NA), e(Div0)]), err(NA), "first error wins");
    assert_eq!(f("CONCAT", &[row(vec![text("a"), err(Ref)]), e(NA)]), err(Ref));
}

#[test]
fn errors_and_arity() {
    for name in ["LEN", "UPPER", "LOWER", "TRIM"] {
        assert_eq!(f(name, &[e(NA)]), err(NA), "{name}");
        assert_eq!(f(name, &[]), err(ErrorKind::Value), "{name}()");
        assert_eq!(f(name, &[s("a"), s("b")]), err(ErrorKind::Value), "{name}: too many arguments");
        assert_eq!(f(name, &[column(vec![text("a")])]), err(ErrorKind::Value), "{name}: a range is not a scalar");
    }
    for name in ["LEFT", "RIGHT"] {
        assert_eq!(f(name, &[e(NA), n(1.0)]), err(NA), "{name}");
        assert_eq!(f(name, &[s("abc"), e(Div0)]), err(Div0), "{name}");
        assert_eq!(f(name, &[e(NA), e(Div0)]), err(NA), "{name}: first argument first");
        assert_eq!(f(name, &[s("abc"), s("x")]), err(ErrorKind::Value), "{name}");
        assert_eq!(f(name, &[]), err(ErrorKind::Value), "{name}()");
        assert_eq!(f(name, &[s("abc"), n(1.0), n(1.0)]), err(ErrorKind::Value), "{name}: too many arguments");
        assert_eq!(f(name, &[column(vec![text("a")]), n(1.0)]), err(ErrorKind::Value), "{name}");
    }
    assert_eq!(f("MID", &[e(Ref), n(1.0), n(1.0)]), err(Ref));
    assert_eq!(f("MID", &[s("abc"), e(Ref), e(NA)]), err(Ref));
    assert_eq!(f("MID", &[s("abc"), n(1.0), e(NA)]), err(NA));
    assert_eq!(f("MID", &[s("abc"), n(1.0), n(1.0), n(1.0)]), err(ErrorKind::Value));
}
