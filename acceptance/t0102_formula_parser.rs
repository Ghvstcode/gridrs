//! Acceptance test for task 0102-formula-parser (parser + printer).
//! Outside every task's write scope.

use gridrs::formula::parser::parse;
use gridrs::formula::print::to_string;
use gridrs::types::{BinaryOp, BinaryOp::*, CellRef, ErrorKind, Expr, Reference, UnaryOp};

fn n(x: f64) -> Expr {
    Expr::Number(x)
}
fn t(s: &str) -> Expr {
    Expr::Text(s.to_string())
}
fn bin(op: BinaryOp, a: Expr, b: Expr) -> Expr {
    Expr::Binary(op, Box::new(a), Box::new(b))
}
fn neg(a: Expr) -> Expr {
    Expr::Unary(UnaryOp::Neg, Box::new(a))
}
fn pos(a: Expr) -> Expr {
    Expr::Unary(UnaryOp::Pos, Box::new(a))
}
fn pct(a: Expr) -> Expr {
    Expr::Unary(UnaryOp::Percent, Box::new(a))
}
fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call(name.to_string(), args)
}
fn rf(col: u32, row: u32) -> Reference {
    Reference::relative(CellRef::new(col, row))
}
fn cell(col: u32, row: u32) -> Expr {
    Expr::Ref(rf(col, row))
}
fn p(s: &str) -> Expr {
    parse(s).unwrap_or_else(|e| panic!("parse({s:?}) failed: {e}"))
}

#[test]
fn literals() {
    assert_eq!(p("42"), n(42.0));
    assert_eq!(p("\"hi\""), t("hi"));
    assert_eq!(p("TRUE"), Expr::Bool(true));
    assert_eq!(p("false"), Expr::Bool(false));
    assert_eq!(p("#REF!"), Expr::Error(ErrorKind::Ref));
    assert_eq!(p("#N/A"), Expr::Error(ErrorKind::NA));
}

#[test]
fn leading_equals_is_optional() {
    assert_eq!(p("=1+2"), p("1+2"));
    assert_eq!(p(" = 1 + 2 "), bin(Add, n(1.0), n(2.0)));
    assert!(parse("==1").is_err());
}

#[test]
fn references_and_ranges() {
    assert_eq!(p("A1"), cell(0, 0));
    assert_eq!(p("b12"), cell(1, 11));
    assert_eq!(p("$B12"), Expr::Ref(Reference { cell: CellRef::new(1, 11), abs_col: true, abs_row: false }));
    assert_eq!(p("B$12"), Expr::Ref(Reference { cell: CellRef::new(1, 11), abs_col: false, abs_row: true }));
    assert_eq!(p("A1:B7"), Expr::Range(rf(0, 0), rf(1, 6)));
    assert_eq!(p("A1 : B7"), Expr::Range(rf(0, 0), rf(1, 6)));
    // endpoints are kept as written, not normalised
    assert_eq!(p("B7:A1"), Expr::Range(rf(1, 6), rf(0, 0)));
    assert_eq!(
        p("$A$1:B$7"),
        Expr::Range(
            Reference { cell: CellRef::new(0, 0), abs_col: true, abs_row: true },
            Reference { cell: CellRef::new(1, 6), abs_col: false, abs_row: true }
        )
    );
}

#[test]
fn precedence() {
    assert_eq!(p("1+2*3"), bin(Add, n(1.0), bin(Mul, n(2.0), n(3.0))));
    assert_eq!(p("1*2+3"), bin(Add, bin(Mul, n(1.0), n(2.0)), n(3.0)));
    assert_eq!(p("(1+2)*3"), bin(Mul, bin(Add, n(1.0), n(2.0)), n(3.0)));
    assert_eq!(p("2*3^2"), bin(Mul, n(2.0), bin(Pow, n(3.0), n(2.0))));
    assert_eq!(p("1&2+3"), bin(Concat, n(1.0), bin(Add, n(2.0), n(3.0))));
    assert_eq!(p("1+2&3"), bin(Concat, bin(Add, n(1.0), n(2.0)), n(3.0)));
    assert_eq!(p("1&2=3"), bin(Eq, bin(Concat, n(1.0), n(2.0)), n(3.0)));
    assert_eq!(p("1<2&3"), bin(Lt, n(1.0), bin(Concat, n(2.0), n(3.0))));
    assert_eq!(p("1+2>=3*4"), bin(Ge, bin(Add, n(1.0), n(2.0)), bin(Mul, n(3.0), n(4.0))));
}

#[test]
fn binary_operators_are_left_associative() {
    assert_eq!(p("1-2-3"), bin(Sub, bin(Sub, n(1.0), n(2.0)), n(3.0)));
    assert_eq!(p("8/4/2"), bin(Div, bin(Div, n(8.0), n(4.0)), n(2.0)));
    assert_eq!(p("1-2+3"), bin(Add, bin(Sub, n(1.0), n(2.0)), n(3.0)));
    // Excel: 2^3^2 is (2^3)^2 = 64, not 2^9
    assert_eq!(p("2^3^2"), bin(Pow, bin(Pow, n(2.0), n(3.0)), n(2.0)));
    assert_eq!(p("1=2=3"), bin(Eq, bin(Eq, n(1.0), n(2.0)), n(3.0)));
    assert_eq!(p("1<>2<3"), bin(Lt, bin(Ne, n(1.0), n(2.0)), n(3.0)));
    assert_eq!(p("\"a\"&\"b\"&\"c\""), bin(Concat, bin(Concat, t("a"), t("b")), t("c")));
}

#[test]
fn unary_minus_binds_tighter_than_power_like_excel() {
    // Excel: =-2^2 is 4
    assert_eq!(p("-2^2"), bin(Pow, neg(n(2.0)), n(2.0)));
    assert_eq!(p("2^-2"), bin(Pow, n(2.0), neg(n(2.0))));
    assert_eq!(p("-(2^2)"), neg(bin(Pow, n(2.0), n(2.0))));
    assert_eq!(p("--1"), neg(neg(n(1.0))));
    assert_eq!(p("-+1"), neg(pos(n(1.0))));
    assert_eq!(p("1--1"), bin(Sub, n(1.0), neg(n(1.0))));
    assert_eq!(p("2*-3"), bin(Mul, n(2.0), neg(n(3.0))));
    assert_eq!(p("-A1"), neg(cell(0, 0)));
    assert_eq!(p("-1+2"), bin(Add, neg(n(1.0)), n(2.0)));
}

#[test]
fn percent_is_postfix_and_binds_tightest() {
    assert_eq!(p("50%"), pct(n(50.0)));
    assert_eq!(p("-50%"), neg(pct(n(50.0))));
    assert_eq!(p("50%%"), pct(pct(n(50.0))));
    assert_eq!(p("2^50%"), bin(Pow, n(2.0), pct(n(50.0))));
    assert_eq!(p("200*10%"), bin(Mul, n(200.0), pct(n(10.0))));
    assert_eq!(p("(1+2)%"), pct(bin(Add, n(1.0), n(2.0))));
    assert_eq!(p("A1%"), pct(cell(0, 0)));
}

#[test]
fn all_comparison_operators() {
    for (src, op) in [("=", Eq), ("<>", Ne), ("<", Lt), ("<=", Le), (">", Gt), (">=", Ge)] {
        assert_eq!(p(&format!("1{src}2")), bin(op, n(1.0), n(2.0)));
    }
}

#[test]
fn function_calls() {
    assert_eq!(p("PI()"), call("PI", vec![]));
    assert_eq!(p("sum(1,2)"), call("SUM", vec![n(1.0), n(2.0)]));
    assert_eq!(p("Sum ( 1 , 2 )"), call("SUM", vec![n(1.0), n(2.0)]));
    assert_eq!(p("SUM(A1:B7)"), call("SUM", vec![Expr::Range(rf(0, 0), rf(1, 6))]));
    assert_eq!(
        p("IF(A1>0,\"pos\",-A1)"),
        call("IF", vec![bin(Gt, cell(0, 0), n(0.0)), t("pos"), neg(cell(0, 0))])
    );
    assert_eq!(p("ROUND(SUM(A1:A3)/3,2)"), call("ROUND", vec![bin(Div, call("SUM", vec![Expr::Range(rf(0, 0), rf(0, 2))]), n(3.0)), n(2.0)]));
    // a name that looks like a cell is still a function when followed by `(`
    assert_eq!(p("LOG10(100)"), call("LOG10", vec![n(100.0)]));
    assert_eq!(p("A1(2)"), call("A1", vec![n(2.0)]));
    assert_eq!(p("my.func_2(1)"), call("MY.FUNC_2", vec![n(1.0)]));
    assert_eq!(p("true()"), call("TRUE", vec![]));
}

#[test]
fn syntax_errors() {
    for bad in [
        "", "=", "1+", "+", "*2", "1 2", "(1", "1)", "()", "SUM(", "SUM(1", "SUM(1,", "SUM(1,)", "SUM(,1)", "SUM(1,,2)", "1,2", "A1:", ":A1", "A1:B", "A1:2", "A1:B2:C3", "A1:SUM(1)", "(A1):B2", "\"a\" \"b\"", "1%2", "%", "1 = ", "<1", "foo", "foo+1", "A0", "XFE1", "A1048577", "$", "A$", "\"open", "1 ? 2", "SUM 1", "2(3)", "(1)(2)",
    ] {
        assert!(parse(bad).is_err(), "parse({bad:?}) should fail");
    }
}

#[test]
fn deep_nesting_is_fine() {
    let depth = 200;
    let src = format!("{}1{}", "(".repeat(depth), ")".repeat(depth));
    assert_eq!(p(&src), n(1.0));
    let src = format!("{}1", "-".repeat(depth));
    let mut want = n(1.0);
    for _ in 0..depth {
        want = neg(want);
    }
    assert_eq!(p(&src), want);
}

// ----------------------------------------------------------------------------
// printer
// ----------------------------------------------------------------------------

#[test]
fn printing_is_canonical() {
    for (src, want) in [
        ("1", "1"),
        ("1.50", "1.5"),
        ("1e3", "1000"),
        (".5", "0.5"),
        ("1e21", "1000000000000000000000"),
        ("1e-7", "0.0000001"),
        ("\"hi\"", "\"hi\""),
        ("\"say \"\"hi\"\"\"", "\"say \"\"hi\"\"\""),
        ("true", "TRUE"),
        ("False", "FALSE"),
        ("#n/a", "#N/A"),
        ("a1", "A1"),
        ("$a$1", "$A$1"),
        ("b7:a1", "B7:A1"),
        ("$A1:b$7", "$A1:B$7"),
        (" 1 +  2 ", "1+2"),
        ("sum( a1:b2 , 3 )", "SUM(A1:B2,3)"),
        ("pi( )", "PI()"),
        ("1<>2", "1<>2"),
        ("1>=2", "1>=2"),
        ("\"a\"&b1", "\"a\"&B1"),
        ("50%", "50%"),
        ("=1+2", "1+2"),
    ] {
        assert_eq!(to_string(&p(src)), want, "to_string(parse({src:?}))");
    }
}

#[test]
fn printing_uses_the_fewest_parentheses() {
    for (src, want) in [
        ("(1)", "1"),
        ("((1+2))", "1+2"),
        ("(1+2)*3", "(1+2)*3"),
        ("1+(2*3)", "1+2*3"),
        ("(1*2)+3", "1*2+3"),
        ("(1-2)-3", "1-2-3"),
        ("1-(2-3)", "1-(2-3)"),
        ("1+(2+3)", "1+(2+3)"),
        ("(8/4)/2", "8/4/2"),
        ("8/(4/2)", "8/(4/2)"),
        ("(2^3)^2", "2^3^2"),
        ("2^(3^2)", "2^(3^2)"),
        ("-2^2", "-2^2"),
        ("(-2)^2", "-2^2"),
        ("-(2^2)", "-(2^2)"),
        ("2^-2", "2^-2"),
        ("2^(-2)", "2^-2"),
        ("-(1+2)", "-(1+2)"),
        ("-(-1)", "--1"),
        ("-(1)", "-1"),
        ("(-1)+2", "-1+2"),
        ("2*(-3)", "2*-3"),
        ("(1+2)%", "(1+2)%"),
        ("(-2)%", "(-2)%"),
        ("-(2%)", "-2%"),
        ("(2%)%", "2%%"),
        ("(2^2)%", "(2^2)%"),
        ("2^(2%)", "2^2%"),
        ("(1=2)=3", "1=2=3"),
        ("1=(2=3)", "1=(2=3)"),
        ("(1&2)&3", "1&2&3"),
        ("1&(2&3)", "1&(2&3)"),
        ("(1<2)&3", "(1<2)&3"),
        ("1<(2&3)", "1<2&3"),
        ("(1+2)&3", "1+2&3"),
        ("IF((A1>0),(1+2),(3))", "IF(A1>0,1+2,3)"),
        ("SUM((A1:B2))", "SUM(A1:B2)"),
    ] {
        let tree = p(src);
        let out = to_string(&tree);
        assert_eq!(out, want, "to_string(parse({src:?}))");
        assert_eq!(p(&out), tree, "{out:?} must parse back to the same tree as {src:?}");
    }
}

#[test]
fn printing_hand_built_trees() {
    assert_eq!(to_string(&bin(Mul, bin(Add, cell(0, 0), n(1.0)), neg(cell(1, 1)))), "(A1+1)*-B2");
    assert_eq!(to_string(&call("NOW", vec![])), "NOW()");
    assert_eq!(to_string(&call("F", vec![t("a,b"), Expr::Bool(true), Expr::Error(ErrorKind::Div0)])), "F(\"a,b\",TRUE,#DIV/0!)");
    assert_eq!(to_string(&bin(Sub, n(1.0), bin(Sub, n(2.0), bin(Sub, n(3.0), n(4.0))))), "1-(2-(3-4))");
    assert_eq!(to_string(&pct(neg(pct(n(5.0))))), "(-5%)%");
}

/// A tiny deterministic generator of random trees: whatever the printer
/// prints must parse back to exactly the same tree.
fn gen(seed: &mut u64, depth: u32) -> Expr {
    fn next(seed: &mut u64) -> u64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *seed >> 33
    }
    let leaf = depth == 0 || next(seed) % 4 == 0;
    if leaf {
        return match next(seed) % 6 {
            0 => n((next(seed) % 1000) as f64 / 8.0),
            1 => t(["", "a", "x y", "q\"q", "1+1"][next(seed) as usize % 5]),
            2 => Expr::Bool(next(seed) % 2 == 0),
            3 => Expr::Error(ErrorKind::ALL[next(seed) as usize % 7]),
            4 => Expr::Ref(Reference {
                cell: CellRef::new(next(seed) as u32 % 30, next(seed) as u32 % 50),
                abs_col: next(seed) % 2 == 0,
                abs_row: next(seed) % 2 == 0,
            }),
            _ => call("RAND", vec![]),
        };
    }
    match next(seed) % 10 {
        0 => neg(gen(seed, depth - 1)),
        1 => pos(gen(seed, depth - 1)),
        2 => pct(gen(seed, depth - 1)),
        3 => {
            let count = next(seed) % 4;
            let mut args: Vec<Expr> = (0..count).map(|_| gen(seed, depth - 1)).collect();
            if next(seed) % 2 == 0 {
                args.push(Expr::Range(rf(next(seed) as u32 % 9, next(seed) as u32 % 9), rf(next(seed) as u32 % 9, next(seed) as u32 % 9)));
            }
            call(["SUM", "IF", "F_1", "X.Y"][next(seed) as usize % 4], args)
        }
        _ => {
            let ops = [Add, Sub, Mul, Div, Pow, Concat, Eq, Ne, Lt, Le, Gt, Ge];
            let op = ops[next(seed) as usize % ops.len()];
            bin(op, gen(seed, depth - 1), gen(seed, depth - 1))
        }
    }
}

#[test]
fn print_then_parse_is_identity_on_random_trees() {
    let mut seed = 20261006;
    for i in 0..3000 {
        let tree = gen(&mut seed, 1 + (i % 5));
        let text = to_string(&tree);
        match parse(&text) {
            Ok(back) => assert_eq!(back, tree, "printed as {text:?}"),
            Err(e) => panic!("printed {tree:?} as {text:?}, which does not parse: {e}"),
        }
        // and with the `=` the sheet stores
        assert_eq!(parse(&format!("={text}")).ok(), Some(tree));
    }
}
