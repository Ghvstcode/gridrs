//! Acceptance test for task 0104-evaluator. Outside every task's write scope.
//!
//! Trees are built by hand and functions are provided by the test's own
//! `Context`, so this task depends on nothing but task 0002-values.

use gridrs::formula::eval::eval;
use gridrs::types::{Arg, BinaryOp, BinaryOp::*, CellRef, Context, ErrorKind, Expr, Grid, Reference, UnaryOp, Value};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Default)]
struct Ctx {
    cells: HashMap<CellRef, Value>,
    calls: RefCell<Vec<(String, Vec<Arg>)>>,
}

impl Ctx {
    fn with(cells: &[(u32, u32, Value)]) -> Ctx {
        Ctx { cells: cells.iter().map(|(c, r, v)| (CellRef::new(*c, *r), v.clone())).collect(), calls: RefCell::new(vec![]) }
    }
}

impl Context for Ctx {
    fn cell(&self, at: CellRef) -> Value {
        self.cells.get(&at).cloned().unwrap_or(Value::Empty)
    }

    fn call(&self, name: &str, args: &[Arg]) -> Value {
        self.calls.borrow_mut().push((name.to_string(), args.to_vec()));
        match name {
            "SEVEN" => Value::Number(7.0),
            "BLANK" => Value::Empty,
            "BOOM" => Value::Error(ErrorKind::NA),
            "ARGC" => Value::Number(args.len() as f64),
            _ => Value::Error(ErrorKind::Name),
        }
    }
}

fn n(x: f64) -> Expr {
    Expr::Number(x)
}
fn t(s: &str) -> Expr {
    Expr::Text(s.to_string())
}
fn num(x: f64) -> Value {
    Value::Number(x)
}
fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}
fn err(e: ErrorKind) -> Value {
    Value::Error(e)
}
fn bin(op: BinaryOp, a: Expr, b: Expr) -> Expr {
    Expr::Binary(op, Box::new(a), Box::new(b))
}
fn un(op: UnaryOp, a: Expr) -> Expr {
    Expr::Unary(op, Box::new(a))
}
fn cell(col: u32, row: u32) -> Expr {
    Expr::Ref(Reference::relative(CellRef::new(col, row)))
}
fn range(c0: u32, r0: u32, c1: u32, r1: u32) -> Expr {
    Expr::Range(Reference::relative(CellRef::new(c0, r0)), Reference::relative(CellRef::new(c1, r1)))
}
fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call(name.to_string(), args)
}
fn ev(e: &Expr) -> Value {
    eval(e, &Ctx::default())
}

#[test]
fn literals() {
    assert_eq!(ev(&n(1.5)), num(1.5));
    assert_eq!(ev(&t("hi")), text("hi"));
    assert_eq!(ev(&Expr::Bool(true)), Value::Bool(true));
    assert_eq!(ev(&Expr::Error(ErrorKind::Ref)), err(ErrorKind::Ref));
}

#[test]
fn arithmetic() {
    assert_eq!(ev(&bin(Add, n(1.0), n(2.0))), num(3.0));
    assert_eq!(ev(&bin(Sub, n(1.0), n(2.5))), num(-1.5));
    assert_eq!(ev(&bin(Mul, n(4.0), n(2.5))), num(10.0));
    assert_eq!(ev(&bin(Div, n(1.0), n(4.0))), num(0.25));
    assert_eq!(ev(&bin(Pow, n(2.0), n(10.0))), num(1024.0));
    assert_eq!(ev(&bin(Pow, n(9.0), n(0.5))), num(3.0));
    assert_eq!(ev(&bin(Pow, un(UnaryOp::Neg, n(2.0)), n(2.0))), num(4.0));
    assert_eq!(ev(&un(UnaryOp::Neg, n(3.0))), num(-3.0));
    assert_eq!(ev(&un(UnaryOp::Percent, n(50.0))), num(0.5));
    assert_eq!(ev(&un(UnaryOp::Pos, n(3.0))), num(3.0));
}

#[test]
fn arithmetic_coerces_operands() {
    assert_eq!(ev(&bin(Add, t("2"), n(3.0))), num(5.0));
    assert_eq!(ev(&bin(Mul, t(" 1e2 "), t("50%"))), num(50.0));
    assert_eq!(ev(&bin(Add, Expr::Bool(true), Expr::Bool(true))), num(2.0));
    assert_eq!(ev(&un(UnaryOp::Neg, t("4"))), num(-4.0));
    assert_eq!(ev(&un(UnaryOp::Percent, Expr::Bool(true))), num(0.01));
    assert_eq!(ev(&bin(Add, t("abc"), n(1.0))), err(ErrorKind::Value));
    assert_eq!(ev(&bin(Sub, n(1.0), t(""))), err(ErrorKind::Value));
    assert_eq!(ev(&un(UnaryOp::Neg, t("x"))), err(ErrorKind::Value));
}

#[test]
fn unary_plus_is_the_identity() {
    assert_eq!(ev(&un(UnaryOp::Pos, t("abc"))), text("abc"));
    assert_eq!(ev(&un(UnaryOp::Pos, Expr::Bool(true))), Value::Bool(true));
    assert_eq!(ev(&un(UnaryOp::Pos, Expr::Error(ErrorKind::NA))), err(ErrorKind::NA));
}

#[test]
fn numeric_errors() {
    assert_eq!(ev(&bin(Div, n(1.0), n(0.0))), err(ErrorKind::Div0));
    assert_eq!(ev(&bin(Div, n(0.0), n(0.0))), err(ErrorKind::Div0));
    assert_eq!(ev(&bin(Div, n(1.0), t("0"))), err(ErrorKind::Div0));
    assert_eq!(ev(&bin(Pow, un(UnaryOp::Neg, n(8.0)), n(0.5))), err(ErrorKind::Num), "NaN is #NUM!");
    assert_eq!(ev(&bin(Pow, n(10.0), n(400.0))), err(ErrorKind::Num), "infinity is #NUM!");
    assert_eq!(ev(&bin(Mul, n(1e200), n(1e200))), err(ErrorKind::Num));
    assert_eq!(ev(&bin(Add, n(f64::MAX), n(f64::MAX))), err(ErrorKind::Num));
    assert_eq!(ev(&bin(Pow, n(0.0), un(UnaryOp::Neg, n(1.0)))), err(ErrorKind::Num));
}

#[test]
fn concatenation() {
    assert_eq!(ev(&bin(Concat, t("a"), t("b"))), text("ab"));
    assert_eq!(ev(&bin(Concat, n(1.0), n(2.0))), text("12"));
    assert_eq!(ev(&bin(Concat, n(0.5), t("x"))), text("0.5x"));
    assert_eq!(ev(&bin(Concat, bin(Div, n(1.0), n(3.0)), t(""))), text("0.333333333333333"));
    assert_eq!(ev(&bin(Concat, Expr::Bool(true), t("!"))), text("TRUE!"));
    assert_eq!(ev(&bin(Concat, t("x"), cell(9, 9))), text("x"), "an empty cell concatenates as nothing");
}

#[test]
fn comparisons() {
    assert_eq!(ev(&bin(Lt, n(1.0), n(2.0))), Value::Bool(true));
    assert_eq!(ev(&bin(Ge, n(1.0), n(2.0))), Value::Bool(false));
    assert_eq!(ev(&bin(Le, n(2.0), n(2.0))), Value::Bool(true));
    assert_eq!(ev(&bin(Gt, n(2.0), n(2.0))), Value::Bool(false));
    assert_eq!(ev(&bin(Eq, t("ABC"), t("abc"))), Value::Bool(true));
    assert_eq!(ev(&bin(Ne, t("a"), t("b"))), Value::Bool(true));
    assert_eq!(ev(&bin(Lt, t("apple"), t("Banana"))), Value::Bool(true));
    assert_eq!(ev(&bin(Eq, t("1"), n(1.0))), Value::Bool(false), "no coercion in comparisons");
    assert_eq!(ev(&bin(Lt, n(999.0), t("a"))), Value::Bool(true), "numbers sort before text");
    assert_eq!(ev(&bin(Gt, Expr::Bool(false), t("zzz"))), Value::Bool(true), "booleans sort after text");
    assert_eq!(ev(&bin(Eq, cell(5, 5), n(0.0))), Value::Bool(true), "an empty cell equals 0");
    assert_eq!(ev(&bin(Eq, cell(5, 5), t(""))), Value::Bool(true), "an empty cell equals the empty string");
    assert_eq!(ev(&bin(Eq, cell(5, 5), cell(6, 6))), Value::Bool(true));
}

#[test]
fn errors_propagate_left_operand_first() {
    let div0 = || bin(Div, n(1.0), n(0.0));
    let na = || Expr::Error(ErrorKind::NA);
    for op in [Add, Sub, Mul, Div, Pow, Concat, Eq, Ne, Lt, Le, Gt, Ge] {
        assert_eq!(ev(&bin(op, div0(), n(1.0))), err(ErrorKind::Div0), "{op:?} left");
        assert_eq!(ev(&bin(op, n(1.0), na())), err(ErrorKind::NA), "{op:?} right");
        assert_eq!(ev(&bin(op, div0(), na())), err(ErrorKind::Div0), "{op:?} both: the left one wins");
        assert_eq!(ev(&bin(op, na(), div0())), err(ErrorKind::NA), "{op:?} both: the left one wins");
    }
    assert_eq!(ev(&bin(Add, na(), t("not a number"))), err(ErrorKind::NA), "left error beats right #VALUE!");
    assert_eq!(ev(&bin(Add, t("not a number"), na())), err(ErrorKind::Value), "left #VALUE! beats right error");
    assert_eq!(ev(&un(UnaryOp::Neg, na())), err(ErrorKind::NA));
    assert_eq!(ev(&un(UnaryOp::Percent, div0())), err(ErrorKind::Div0));
}

#[test]
fn cell_references() {
    let ctx = Ctx::with(&[(0, 0, num(10.0)), (1, 0, text("5")), (2, 0, Value::Bool(true)), (3, 0, err(ErrorKind::Ref))]);
    assert_eq!(eval(&cell(0, 0), &ctx), num(10.0));
    assert_eq!(eval(&cell(1, 0), &ctx), text("5"));
    assert_eq!(eval(&cell(3, 0), &ctx), err(ErrorKind::Ref));
    assert_eq!(eval(&bin(Add, cell(0, 0), cell(1, 0)), &ctx), num(15.0));
    assert_eq!(eval(&bin(Add, cell(0, 0), cell(2, 0)), &ctx), num(11.0));
    assert_eq!(eval(&bin(Add, cell(0, 0), cell(3, 0)), &ctx), err(ErrorKind::Ref));
    assert_eq!(eval(&bin(Add, cell(0, 0), cell(9, 9)), &ctx), num(10.0), "empty cells count as 0 in arithmetic");
    // `$` flags make no difference to evaluation
    let abs = Expr::Ref(Reference { cell: CellRef::new(0, 0), abs_col: true, abs_row: true });
    assert_eq!(eval(&abs, &ctx), num(10.0));
}

#[test]
fn the_result_is_never_empty() {
    // =Z99 on an empty sheet shows 0, like Excel
    assert_eq!(ev(&cell(25, 98)), num(0.0));
    assert_eq!(ev(&un(UnaryOp::Pos, cell(25, 98))), num(0.0));
    assert_eq!(ev(&call("BLANK", vec![])), num(0.0));
}

#[test]
fn a_bare_range_is_a_value_error() {
    assert_eq!(ev(&range(0, 0, 1, 1)), err(ErrorKind::Value));
    assert_eq!(ev(&bin(Add, range(0, 0, 1, 1), n(1.0))), err(ErrorKind::Value));
    assert_eq!(ev(&bin(Concat, t("x"), range(0, 0, 0, 0))), err(ErrorKind::Value));
    assert_eq!(ev(&un(UnaryOp::Neg, range(0, 0, 1, 1))), err(ErrorKind::Value));
    assert_eq!(ev(&bin(Add, Expr::Error(ErrorKind::NA), range(0, 0, 1, 1))), err(ErrorKind::NA));
}

#[test]
fn calls_go_through_the_context() {
    let ctx = Ctx::default();
    assert_eq!(eval(&call("SEVEN", vec![]), &ctx), num(7.0));
    assert_eq!(eval(&bin(Mul, call("SEVEN", vec![]), n(6.0)), &ctx), num(42.0));
    assert_eq!(eval(&call("BOOM", vec![]), &ctx), err(ErrorKind::NA));
    assert_eq!(eval(&call("NO_SUCH", vec![n(1.0)]), &ctx), err(ErrorKind::Name));
    let names: Vec<String> = ctx.calls.borrow().iter().map(|(name, _)| name.clone()).collect();
    assert_eq!(names, ["SEVEN", "SEVEN", "BOOM", "NO_SUCH"]);
}

#[test]
fn call_arguments() {
    let ctx = Ctx::with(&[(0, 0, num(1.0)), (1, 0, text("x")), (0, 1, Value::Bool(true)), (1, 1, err(ErrorKind::Div0))]);
    let e = call(
        "ARGC",
        vec![
            n(5.0),                                // literal
            bin(Add, n(1.0), n(2.0)),              // nested expression
            cell(0, 0),                            // single cell
            cell(7, 7),                            // single empty cell: stays Empty
            bin(Div, n(1.0), n(0.0)),              // an error is passed, not propagated
            range(0, 0, 1, 2),                     // A1:B3
            range(1, 1, 0, 0),                     // B2:A1, written backwards
            range(4, 4, 4, 4),                     // E5:E5, one empty cell: still a range
            call("SEVEN", vec![]),                 // nested call
            bin(Add, call("BLANK", vec![]), n(0.0)),
        ],
    );
    assert_eq!(eval(&e, &ctx), num(10.0));
    let calls = ctx.calls.borrow();
    let (name, args) = calls.iter().find(|(name, _)| name == "ARGC").expect("ARGC was called");
    assert_eq!(name, "ARGC");
    assert_eq!(
        args,
        &vec![
            Arg::Value(num(5.0)),
            Arg::Value(num(3.0)),
            Arg::Value(num(1.0)),
            Arg::Value(Value::Empty),
            Arg::Value(err(ErrorKind::Div0)),
            Arg::Range(Grid { rows: 3, cols: 2, cells: vec![num(1.0), text("x"), Value::Bool(true), err(ErrorKind::Div0), Value::Empty, Value::Empty] }),
            Arg::Range(Grid { rows: 2, cols: 2, cells: vec![num(1.0), text("x"), Value::Bool(true), err(ErrorKind::Div0)] }),
            Arg::Range(Grid { rows: 1, cols: 1, cells: vec![Value::Empty] }),
            Arg::Value(num(7.0)),
            Arg::Value(num(0.0)),
        ]
    );
}

#[test]
fn a_function_returning_empty_is_empty_as_an_argument_but_zero_at_the_top() {
    let ctx = Ctx::default();
    assert_eq!(eval(&call("ARGC", vec![call("BLANK", vec![])]), &ctx), num(1.0));
    let calls = ctx.calls.borrow();
    let (_, args) = calls.iter().find(|(name, _)| name == "ARGC").unwrap();
    assert_eq!(args, &vec![Arg::Value(Value::Empty)]);
}

#[test]
fn oversized_ranges_are_not_materialised() {
    let ctx = Ctx::default();
    // A1:XFD1048576 is 17 billion cells; this must return quickly, not allocate.
    let everything = range(0, 0, 16383, 1048575);
    assert_eq!(eval(&call("ARGC", vec![everything]), &ctx), err(ErrorKind::Num));
    assert!(ctx.calls.borrow().is_empty(), "the function must not be called");
    // arguments are processed left to right: what comes before the oversized range has run, what comes after has not
    let ctx2 = Ctx::default();
    let e = call("ARGC", vec![call("SEVEN", vec![]), range(0, 0, 16383, 1048575), call("BOOM", vec![])]);
    assert_eq!(eval(&e, &ctx2), err(ErrorKind::Num));
    let names: Vec<String> = ctx2.calls.borrow().iter().map(|(name, _)| name.clone()).collect();
    assert_eq!(names, ["SEVEN"]);
    // exactly at the limit is fine: 1000 columns x 1000 rows
    assert_eq!(eval(&call("ARGC", vec![range(0, 0, 999, 999)]), &ctx), num(1.0));
    // one more row is not
    assert_eq!(eval(&call("ARGC", vec![range(0, 0, 999, 1000)]), &ctx), err(ErrorKind::Num));
}

#[test]
fn deep_trees_do_not_overflow_the_stack() {
    let mut e = n(1.0);
    for _ in 0..500 {
        e = bin(Add, e, n(1.0));
    }
    assert_eq!(ev(&e), num(501.0));
}
