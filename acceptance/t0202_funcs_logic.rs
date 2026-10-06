//! Acceptance test for task 0202-funcs-logic. Outside every task's write scope.

include!("common_funcs.rs");

fn f(name: &str, args: &[Arg]) -> Value {
    gridrs::funcs::logic::call(name, args).unwrap_or_else(|| panic!("logic::call does not implement {name}"))
}

#[test]
fn implements_exactly_its_family() {
    for name in ["IF", "AND", "OR", "NOT", "IFERROR", "ISBLANK", "ISNUMBER", "ISTEXT", "ISERROR", "NA", "WHOPAID"] {
        assert!(gridrs::funcs::logic::call(name, &[b(true)]).is_some(), "{name} should be implemented");
    }
    for name in ["SUM", "LEN", "VLOOKUP", "NOPE", "", "if", "If", "IFS", "I"] {
        assert_eq!(gridrs::funcs::logic::call(name, &[b(true)]), None, "{name:?} is not a logic function");
    }
}

#[test]
fn reachable_through_the_dispatcher() {
    assert_eq!(gridrs::funcs::call("NOT", &[b(true)]), boolean(false));
    assert_eq!(gridrs::funcs::call("NA", &[]), err(NA));
    assert_eq!(gridrs::funcs::call("NO_SUCH_FUNCTION", &[]), err(Name));
}

#[test]
fn if_picks_a_branch() {
    assert_eq!(f("IF", &[b(true), n(1.0), n(2.0)]), num(1.0));
    assert_eq!(f("IF", &[b(false), n(1.0), n(2.0)]), num(2.0));
    assert_eq!(f("IF", &[b(true), s("yes"), s("no")]), text("yes"));
    // the condition is coerced like any boolean
    assert_eq!(f("IF", &[n(5.0), s("yes"), s("no")]), text("yes"));
    assert_eq!(f("IF", &[n(0.0), s("yes"), s("no")]), text("no"));
    assert_eq!(f("IF", &[empty(), s("yes"), s("no")]), text("no"));
    assert_eq!(f("IF", &[s("TRUE"), s("yes"), s("no")]), text("yes"));
    assert_eq!(f("IF", &[s("maybe"), s("yes"), s("no")]), err(ErrorKind::Value));
}

#[test]
fn if_without_else_is_false() {
    assert_eq!(f("IF", &[b(true), n(1.0)]), num(1.0));
    assert_eq!(f("IF", &[b(false), n(1.0)]), boolean(false));
}

#[test]
fn if_only_looks_at_the_branch_it_returns() {
    assert_eq!(f("IF", &[e(NA), n(1.0), n(2.0)]), err(NA), "an error condition propagates");
    assert_eq!(f("IF", &[b(true), n(1.0), e(Div0)]), num(1.0), "the untaken branch may be an error");
    assert_eq!(f("IF", &[b(false), e(Div0), n(2.0)]), num(2.0));
    assert_eq!(f("IF", &[b(true), e(Div0), n(2.0)]), err(Div0), "the taken branch is returned as is");
    assert_eq!(f("IF", &[b(true), empty(), n(2.0)]), Value::Empty);
}

#[test]
fn if_argument_errors() {
    assert_eq!(f("IF", &[]), err(ErrorKind::Value));
    assert_eq!(f("IF", &[b(true)]), err(ErrorKind::Value));
    assert_eq!(f("IF", &[b(true), n(1.0), n(2.0), n(3.0)]), err(ErrorKind::Value));
    assert_eq!(f("IF", &[column(vec![boolean(true)]), n(1.0), n(2.0)]), err(ErrorKind::Value), "range as condition");
    assert_eq!(f("IF", &[b(true), column(vec![num(1.0)]), n(2.0)]), err(ErrorKind::Value), "range as taken branch");
    assert_eq!(f("IF", &[b(false), column(vec![num(1.0)]), n(2.0)]), num(2.0), "range as untaken branch is never looked at");
}

#[test]
fn and_or() {
    assert_eq!(f("AND", &[b(true), b(true)]), boolean(true));
    assert_eq!(f("AND", &[b(true), b(false), b(true)]), boolean(false));
    assert_eq!(f("OR", &[b(false), b(false)]), boolean(false));
    assert_eq!(f("OR", &[b(false), b(true)]), boolean(true));
    assert_eq!(f("AND", &[n(1.0), n(-2.0)]), boolean(true));
    assert_eq!(f("AND", &[n(1.0), n(0.0)]), boolean(false));
    assert_eq!(f("OR", &[n(0.0), s("true")]), boolean(true));
    assert_eq!(f("AND", &[b(true)]), boolean(true));
    assert_eq!(f("OR", &[b(false)]), boolean(false));
}

#[test]
fn and_or_over_ranges_skip_text_and_empty_cells() {
    let r = || grid(2, vec![boolean(true), text("nope"), Value::Empty, num(1.0)]);
    assert_eq!(f("AND", &[r()]), boolean(true));
    assert_eq!(f("OR", &[r()]), boolean(true));
    assert_eq!(f("AND", &[r(), b(false)]), boolean(false));
    let zeros = || row(vec![num(0.0), boolean(false), text("TRUE")]);
    assert_eq!(f("OR", &[zeros()]), boolean(false), "text in a range is skipped even if it spells TRUE");
    assert_eq!(f("AND", &[zeros()]), boolean(false));
    // a reference to an empty cell is skipped as well
    assert_eq!(f("AND", &[empty(), b(true)]), boolean(true));
    assert_eq!(f("OR", &[empty(), b(false)]), boolean(false));
}

#[test]
fn and_or_errors() {
    for name in ["AND", "OR"] {
        assert_eq!(f(name, &[]), err(ErrorKind::Value), "{name}()");
        assert_eq!(f(name, &[s("maybe")]), err(ErrorKind::Value), "{name}: scalar text that is not a boolean");
        assert_eq!(f(name, &[column(vec![text("a"), Value::Empty])]), err(ErrorKind::Value), "{name}: nothing to evaluate");
        assert_eq!(f(name, &[empty()]), err(ErrorKind::Value), "{name}: nothing to evaluate");
        // no short-circuit: every argument is inspected, first error wins
        assert_eq!(f(name, &[b(true), e(NA), e(Div0)]), err(NA), "{name}");
        assert_eq!(f(name, &[b(false), e(NA)]), err(NA), "{name}");
        assert_eq!(f(name, &[row(vec![boolean(true), err(Ref)]), e(NA)]), err(Ref), "{name}");
    }
}

#[test]
fn not() {
    assert_eq!(f("NOT", &[b(true)]), boolean(false));
    assert_eq!(f("NOT", &[b(false)]), boolean(true));
    assert_eq!(f("NOT", &[n(0.0)]), boolean(true));
    assert_eq!(f("NOT", &[n(3.0)]), boolean(false));
    assert_eq!(f("NOT", &[empty()]), boolean(true));
    assert_eq!(f("NOT", &[s("false")]), boolean(true));
    assert_eq!(f("NOT", &[s("x")]), err(ErrorKind::Value));
    assert_eq!(f("NOT", &[e(Num)]), err(Num));
    assert_eq!(f("NOT", &[]), err(ErrorKind::Value));
    assert_eq!(f("NOT", &[b(true), b(true)]), err(ErrorKind::Value));
    assert_eq!(f("NOT", &[column(vec![boolean(true)])]), err(ErrorKind::Value));
}

#[test]
fn iferror() {
    assert_eq!(f("IFERROR", &[n(1.0), n(2.0)]), num(1.0));
    assert_eq!(f("IFERROR", &[s("ok"), n(2.0)]), text("ok"));
    for kind in ErrorKind::ALL {
        assert_eq!(f("IFERROR", &[e(kind), s("fallback")]), text("fallback"), "{kind}");
    }
    assert_eq!(f("IFERROR", &[e(NA), e(Div0)]), err(Div0), "the fallback is returned as is");
    assert_eq!(f("IFERROR", &[n(1.0), e(Div0)]), num(1.0), "an unused fallback may be an error");
    assert_eq!(f("IFERROR", &[empty(), n(2.0)]), Value::Empty);
    assert_eq!(f("IFERROR", &[n(1.0)]), err(ErrorKind::Value));
    assert_eq!(f("IFERROR", &[n(1.0), n(2.0), n(3.0)]), err(ErrorKind::Value));
    assert_eq!(f("IFERROR", &[column(vec![num(1.0)]), n(2.0)]), err(ErrorKind::Value));
}

#[test]
fn is_functions() {
    let cases: [(Arg, [bool; 4]); 6] = [
        // (argument, [ISBLANK, ISNUMBER, ISTEXT, ISERROR])
        (empty(), [true, false, false, false]),
        (n(0.0), [false, true, false, false]),
        (s(""), [false, false, true, false]),
        (s("12"), [false, false, true, false]),
        (b(false), [false, false, false, false]),
        (e(NA), [false, false, false, true]),
    ];
    for (arg, want) in cases {
        for (name, want) in ["ISBLANK", "ISNUMBER", "ISTEXT", "ISERROR"].iter().zip(want) {
            assert_eq!(f(name, std::slice::from_ref(&arg)), boolean(want), "{name}({arg:?})");
        }
    }
    for name in ["ISBLANK", "ISNUMBER", "ISTEXT", "ISERROR"] {
        assert_eq!(f(name, &[]), err(ErrorKind::Value), "{name}()");
        assert_eq!(f(name, &[n(1.0), n(2.0)]), err(ErrorKind::Value), "{name}: too many arguments");
        assert_eq!(f(name, &[column(vec![num(1.0)])]), err(ErrorKind::Value), "{name}: a range is not a scalar");
    }
}

#[test]
fn na() {
    assert_eq!(f("NA", &[]), err(NA));
    assert_eq!(f("NA", &[n(1.0)]), err(ErrorKind::Value));
}

#[test]
fn whopaid() {
    assert_eq!(f("WHOPAID", &[]), text("Nobody. Strangers' spare tokens wrote this, one signed task at a time."));
    assert_eq!(f("WHOPAID", &[n(1.0)]), err(ErrorKind::Value));
}
