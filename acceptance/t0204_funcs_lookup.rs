//! Acceptance test for task 0204-funcs-lookup. Outside every task's write scope.

include!("common_funcs.rs");

fn f(name: &str, args: &[Arg]) -> Value {
    gridrs::funcs::lookup::call(name, args).unwrap_or_else(|| panic!("lookup::call does not implement {name}"))
}

/// | id | fruit    | price | stock |
/// |  1 | apple    |  0.5  | TRUE  |
/// |  2 | Banana   |  0.25 | FALSE |
/// |  4 | cherry   |  3    |       |
/// |  7 | date     |  #N/A | TRUE  |
fn fruit() -> Arg {
    grid(
        4,
        vec![
            num(1.0), text("apple"), num(0.5), boolean(true),
            num(2.0), text("Banana"), num(0.25), boolean(false),
            num(4.0), text("cherry"), num(3.0), Value::Empty,
            num(7.0), text("date"), err(NA), boolean(true),
        ],
    )
}

/// Same data keyed by name (sorted ascending, case-insensitively).
fn by_name() -> Arg {
    grid(2, vec![text("apple"), num(1.0), text("Banana"), num(2.0), text("cherry"), num(4.0), text("date"), num(7.0)])
}

#[test]
fn implements_exactly_its_family() {
    for name in ["VLOOKUP", "INDEX", "MATCH"] {
        assert!(gridrs::funcs::lookup::call(name, &[n(1.0)]).is_some(), "{name} should be implemented");
    }
    for name in ["SUM", "IF", "LEN", "NOPE", "", "vlookup", "HLOOKUP", "XMATCH", "INDE"] {
        assert_eq!(gridrs::funcs::lookup::call(name, &[n(1.0)]), None, "{name:?} is not a lookup function");
    }
}

#[test]
fn reachable_through_the_dispatcher() {
    assert_eq!(gridrs::funcs::call("INDEX", &[fruit(), n(1.0), n(2.0)]), text("apple"));
}

#[test]
fn vlookup_exact() {
    assert_eq!(f("VLOOKUP", &[n(2.0), fruit(), n(2.0), b(false)]), text("Banana"));
    assert_eq!(f("VLOOKUP", &[n(4.0), fruit(), n(3.0), b(false)]), num(3.0));
    assert_eq!(f("VLOOKUP", &[n(1.0), fruit(), n(1.0), b(false)]), num(1.0));
    assert_eq!(f("VLOOKUP", &[n(1.0), fruit(), n(4.0), b(false)]), boolean(true));
    assert_eq!(f("VLOOKUP", &[n(3.0), fruit(), n(2.0), b(false)]), err(NA), "no exact match");
    assert_eq!(f("VLOOKUP", &[n(4.0), fruit(), n(4.0), b(false)]), Value::Empty, "an empty result cell is returned as empty");
    assert_eq!(f("VLOOKUP", &[n(7.0), fruit(), n(3.0), b(false)]), err(NA), "an error in the result cell is returned");
    // range_lookup accepts anything boolean-like
    assert_eq!(f("VLOOKUP", &[n(2.0), fruit(), n(2.0), n(0.0)]), text("Banana"));
    assert_eq!(f("VLOOKUP", &[n(3.0), fruit(), n(2.0), n(0.0)]), err(NA));
}

#[test]
fn vlookup_text_keys_ignore_case() {
    assert_eq!(f("VLOOKUP", &[s("banana"), by_name(), n(2.0), b(false)]), num(2.0));
    assert_eq!(f("VLOOKUP", &[s("CHERRY"), by_name(), n(2.0), b(false)]), num(4.0));
    assert_eq!(f("VLOOKUP", &[s("cherr"), by_name(), n(2.0), b(false)]), err(NA));
}

#[test]
fn vlookup_never_matches_across_types() {
    let t = grid(2, vec![text("1"), text("text one"), num(1.0), text("number one"), Value::Empty, text("empty"), boolean(true), text("bool")]);
    assert_eq!(f("VLOOKUP", &[n(1.0), t.clone(), n(2.0), b(false)]), text("number one"));
    assert_eq!(f("VLOOKUP", &[s("1"), t.clone(), n(2.0), b(false)]), text("text one"));
    assert_eq!(f("VLOOKUP", &[b(true), t.clone(), n(2.0), b(false)]), text("bool"));
    assert_eq!(f("VLOOKUP", &[n(0.0), t.clone(), n(2.0), b(false)]), err(NA), "0 does not match an empty cell");
    assert_eq!(f("VLOOKUP", &[s(""), t.clone(), n(2.0), b(false)]), err(NA), "\"\" does not match an empty cell");
    assert_eq!(f("VLOOKUP", &[empty(), t, n(2.0), b(false)]), err(NA), "an empty key matches nothing");
}

#[test]
fn vlookup_first_match_wins() {
    let t = grid(2, vec![num(5.0), text("first"), num(5.0), text("second")]);
    assert_eq!(f("VLOOKUP", &[n(5.0), t, n(2.0), b(false)]), text("first"));
}

#[test]
fn vlookup_approximate_is_the_default() {
    // largest key <= the lookup value
    assert_eq!(f("VLOOKUP", &[n(3.0), fruit(), n(2.0)]), text("Banana"));
    assert_eq!(f("VLOOKUP", &[n(3.0), fruit(), n(2.0), b(true)]), text("Banana"));
    assert_eq!(f("VLOOKUP", &[n(4.0), fruit(), n(2.0)]), text("cherry"));
    assert_eq!(f("VLOOKUP", &[n(6.9), fruit(), n(2.0)]), text("cherry"));
    assert_eq!(f("VLOOKUP", &[n(1000.0), fruit(), n(2.0)]), text("date"));
    assert_eq!(f("VLOOKUP", &[n(0.5), fruit(), n(2.0)]), err(NA), "smaller than every key");
    assert_eq!(f("VLOOKUP", &[s("blueberry"), by_name(), n(2.0)]), num(2.0), "Banana <= blueberry < cherry");
    assert_eq!(f("VLOOKUP", &[s("zebra"), by_name(), n(2.0)]), num(7.0));
    assert_eq!(f("VLOOKUP", &[s("aardvark"), by_name(), n(2.0)]), err(NA));
    // only keys of the same type as the lookup value are considered
    assert_eq!(f("VLOOKUP", &[n(100.0), by_name(), n(2.0)]), err(NA));
    // tax-bracket style: last row wins among equal keys
    let t = grid(2, vec![num(0.0), text("a"), num(10.0), text("b"), num(10.0), text("c"), num(20.0), text("d")]);
    assert_eq!(f("VLOOKUP", &[n(15.0), t, n(2.0)]), text("c"));
}

#[test]
fn vlookup_column_index() {
    assert_eq!(f("VLOOKUP", &[n(2.0), fruit(), n(5.0), b(false)]), err(Ref), "column past the table");
    assert_eq!(f("VLOOKUP", &[n(2.0), fruit(), n(0.0), b(false)]), err(ErrorKind::Value));
    assert_eq!(f("VLOOKUP", &[n(2.0), fruit(), n(-1.0), b(false)]), err(ErrorKind::Value));
    assert_eq!(f("VLOOKUP", &[n(2.0), fruit(), n(2.9), b(false)]), text("Banana"), "index is truncated");
    assert_eq!(f("VLOOKUP", &[n(2.0), fruit(), s("2"), b(false)]), text("Banana"));
    // the column index is validated even when the key is not found
    assert_eq!(f("VLOOKUP", &[n(99.0), fruit(), n(5.0), b(false)]), err(Ref));
}

#[test]
fn vlookup_argument_errors() {
    assert_eq!(f("VLOOKUP", &[n(1.0), fruit()]), err(ErrorKind::Value), "too few");
    assert_eq!(f("VLOOKUP", &[n(1.0), fruit(), n(1.0), b(false), n(1.0)]), err(ErrorKind::Value), "too many");
    assert_eq!(f("VLOOKUP", &[n(1.0), n(1.0), n(1.0)]), err(ErrorKind::Value), "table must be a range");
    assert_eq!(f("VLOOKUP", &[fruit(), fruit(), n(1.0)]), err(ErrorKind::Value), "key must be a scalar");
    assert_eq!(f("VLOOKUP", &[e(Div0), fruit(), n(1.0)]), err(Div0), "an error key propagates");
    assert_eq!(f("VLOOKUP", &[n(1.0), fruit(), e(Num)]), err(Num));
    assert_eq!(f("VLOOKUP", &[n(1.0), fruit(), n(1.0), e(Ref)]), err(Ref));
    assert_eq!(f("VLOOKUP", &[n(1.0), fruit(), s("x")]), err(ErrorKind::Value));
    assert_eq!(f("VLOOKUP", &[n(1.0), fruit(), n(1.0), s("x")]), err(ErrorKind::Value));
    // errors in the key column are skipped, not propagated
    let t = grid(2, vec![err(Div0), text("bad"), num(2.0), text("good")]);
    assert_eq!(f("VLOOKUP", &[n(2.0), t.clone(), n(2.0), b(false)]), text("good"));
    assert_eq!(f("VLOOKUP", &[n(2.0), t, n(2.0)]), text("good"));
}

#[test]
fn index_two_dimensional() {
    assert_eq!(f("INDEX", &[fruit(), n(1.0), n(1.0)]), num(1.0));
    assert_eq!(f("INDEX", &[fruit(), n(2.0), n(2.0)]), text("Banana"));
    assert_eq!(f("INDEX", &[fruit(), n(4.0), n(4.0)]), boolean(true));
    assert_eq!(f("INDEX", &[fruit(), n(3.0), n(4.0)]), Value::Empty);
    assert_eq!(f("INDEX", &[fruit(), n(4.0), n(3.0)]), err(NA), "an error cell is returned as is");
    assert_eq!(f("INDEX", &[fruit(), n(2.9), n(2.9)]), text("Banana"), "indices are truncated");
    assert_eq!(f("INDEX", &[fruit(), n(5.0), n(1.0)]), err(Ref));
    assert_eq!(f("INDEX", &[fruit(), n(1.0), n(5.0)]), err(Ref));
    assert_eq!(f("INDEX", &[fruit(), n(0.0), n(1.0)]), err(ErrorKind::Value));
    assert_eq!(f("INDEX", &[fruit(), n(1.0), n(0.0)]), err(ErrorKind::Value));
    assert_eq!(f("INDEX", &[fruit(), n(-1.0), n(1.0)]), err(ErrorKind::Value));
    assert_eq!(f("INDEX", &[fruit(), n(1e300), n(1.0)]), err(Ref));
    assert_eq!(f("INDEX", &[fruit(), n(99.0), n(0.0)]), err(ErrorKind::Value), "lower bounds are checked before upper bounds");
    assert_eq!(f("INDEX", &[fruit(), n(0.0), n(99.0)]), err(ErrorKind::Value));
}

#[test]
fn index_one_dimensional() {
    let col = || column(vec![text("a"), text("b"), text("c")]);
    let rw = || row(vec![text("x"), text("y"), text("z")]);
    assert_eq!(f("INDEX", &[col(), n(2.0)]), text("b"));
    assert_eq!(f("INDEX", &[col(), n(4.0)]), err(Ref));
    // with one index and a single-row range, the index picks the column
    assert_eq!(f("INDEX", &[rw(), n(3.0)]), text("z"));
    assert_eq!(f("INDEX", &[rw(), n(4.0)]), err(Ref));
    // a 1x1 range
    assert_eq!(f("INDEX", &[row(vec![num(9.0)]), n(1.0)]), num(9.0));
    // one index on a 2-D range: the column defaults to 1
    assert_eq!(f("INDEX", &[fruit(), n(3.0)]), num(4.0));
}

#[test]
fn index_argument_errors() {
    assert_eq!(f("INDEX", &[fruit()]), err(ErrorKind::Value));
    assert_eq!(f("INDEX", &[fruit(), n(1.0), n(1.0), n(1.0)]), err(ErrorKind::Value));
    assert_eq!(f("INDEX", &[n(5.0), n(1.0)]), err(ErrorKind::Value), "first argument must be a range");
    assert_eq!(f("INDEX", &[fruit(), e(Div0), n(1.0)]), err(Div0));
    assert_eq!(f("INDEX", &[fruit(), n(1.0), e(Num)]), err(Num));
    assert_eq!(f("INDEX", &[fruit(), s("x"), n(1.0)]), err(ErrorKind::Value));
    assert_eq!(f("INDEX", &[fruit(), fruit(), n(1.0)]), err(ErrorKind::Value));
}

#[test]
fn match_exact() {
    let names = || column(vec![text("apple"), text("Banana"), text("cherry"), text("banana")]);
    assert_eq!(f("MATCH", &[s("BANANA"), names(), n(0.0)]), num(2.0), "first match, case-insensitive");
    assert_eq!(f("MATCH", &[s("cherry"), names(), n(0.0)]), num(3.0));
    assert_eq!(f("MATCH", &[s("durian"), names(), n(0.0)]), err(NA));
    assert_eq!(f("MATCH", &[n(1.0), names(), n(0.0)]), err(NA));
    let nums = || row(vec![num(10.0), num(20.0), num(30.0)]);
    assert_eq!(f("MATCH", &[n(30.0), nums(), n(0.0)]), num(3.0), "works along a row too");
    assert_eq!(f("MATCH", &[n(25.0), nums(), n(0.0)]), err(NA));
    assert_eq!(f("MATCH", &[n(20.0), nums(), b(false)]), num(2.0), "FALSE is 0");
}

#[test]
fn match_ascending_is_the_default() {
    let nums = || column(vec![num(10.0), num(20.0), num(20.0), num(30.0)]);
    assert_eq!(f("MATCH", &[n(25.0), nums()]), num(3.0), "largest value <= 25, last of equals");
    assert_eq!(f("MATCH", &[n(25.0), nums(), n(1.0)]), num(3.0));
    assert_eq!(f("MATCH", &[n(10.0), nums(), n(1.0)]), num(1.0));
    assert_eq!(f("MATCH", &[n(99.0), nums(), n(1.0)]), num(4.0));
    assert_eq!(f("MATCH", &[n(9.0), nums(), n(1.0)]), err(NA));
    assert_eq!(f("MATCH", &[n(25.0), nums(), n(7.5)]), num(3.0), "any positive match type means ascending");
    assert_eq!(f("MATCH", &[s("blueberry"), column(vec![text("apple"), text("Banana"), text("cherry")])]), num(2.0));
}

#[test]
fn match_descending() {
    let nums = || column(vec![num(30.0), num(20.0), num(20.0), num(10.0)]);
    assert_eq!(f("MATCH", &[n(25.0), nums(), n(-1.0)]), num(1.0), "smallest value >= 25");
    assert_eq!(f("MATCH", &[n(20.0), nums(), n(-1.0)]), num(3.0), "last of equals");
    assert_eq!(f("MATCH", &[n(5.0), nums(), n(-1.0)]), num(4.0));
    assert_eq!(f("MATCH", &[n(31.0), nums(), n(-1.0)]), err(NA));
    assert_eq!(f("MATCH", &[n(25.0), nums(), n(-0.5)]), num(1.0), "any negative match type means descending");
}

#[test]
fn match_skips_other_types_empties_and_errors() {
    let messy = || column(vec![Value::Empty, text("10"), err(Div0), num(10.0), boolean(true), num(20.0)]);
    assert_eq!(f("MATCH", &[n(10.0), messy(), n(0.0)]), num(4.0));
    assert_eq!(f("MATCH", &[n(15.0), messy(), n(1.0)]), num(4.0));
    assert_eq!(f("MATCH", &[s("10"), messy(), n(0.0)]), num(2.0));
    assert_eq!(f("MATCH", &[b(true), messy(), n(0.0)]), num(5.0));
    assert_eq!(f("MATCH", &[n(0.0), messy(), n(0.0)]), err(NA));
    assert_eq!(f("MATCH", &[empty(), messy(), n(0.0)]), err(NA));
}

#[test]
fn match_argument_errors() {
    let nums = || column(vec![num(10.0), num(20.0)]);
    assert_eq!(f("MATCH", &[n(1.0)]), err(ErrorKind::Value));
    assert_eq!(f("MATCH", &[n(1.0), nums(), n(0.0), n(0.0)]), err(ErrorKind::Value));
    assert_eq!(f("MATCH", &[n(1.0), n(1.0), n(0.0)]), err(ErrorKind::Value), "second argument must be a range");
    assert_eq!(f("MATCH", &[nums(), nums(), n(0.0)]), err(ErrorKind::Value), "key must be a scalar");
    assert_eq!(f("MATCH", &[n(1.0), fruit(), n(0.0)]), err(NA), "a 2-D range cannot be searched");
    assert_eq!(f("MATCH", &[e(Div0), nums(), n(0.0)]), err(Div0));
    assert_eq!(f("MATCH", &[n(1.0), nums(), e(Num)]), err(Num));
    assert_eq!(f("MATCH", &[n(1.0), nums(), s("x")]), err(ErrorKind::Value));
}

#[test]
fn index_match_together() {
    // the classic: INDEX(prices, MATCH("cherry", names, 0))
    let names = column(vec![text("apple"), text("Banana"), text("cherry"), text("date")]);
    let prices = column(vec![num(0.5), num(0.25), num(3.0), num(9.0)]);
    let pos = f("MATCH", &[s("cherry"), names, n(0.0)]);
    assert_eq!(pos, num(3.0));
    assert_eq!(f("INDEX", &[prices, Arg::Value(pos)]), num(3.0));
}
