// Shared helpers for the function-library acceptance tests (included with `include!`).
// Outside every task's write scope.

#[allow(unused_imports)]
use gridrs::types::{Arg, ErrorKind, ErrorKind::{Cycle, Div0, Name, Num, Ref, NA}, Grid, Value};

#[allow(dead_code)]
fn num(x: f64) -> Value {
    Value::Number(x)
}
#[allow(dead_code)]
fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}
#[allow(dead_code)]
fn boolean(b: bool) -> Value {
    Value::Bool(b)
}
#[allow(dead_code)]
fn err(e: ErrorKind) -> Value {
    Value::Error(e)
}
/// Scalar number argument.
#[allow(dead_code)]
fn n(x: f64) -> Arg {
    Arg::Value(Value::Number(x))
}
/// Scalar text argument.
#[allow(dead_code)]
fn s(v: &str) -> Arg {
    Arg::Value(Value::Text(v.to_string()))
}
/// Scalar boolean argument.
#[allow(dead_code)]
fn b(v: bool) -> Arg {
    Arg::Value(Value::Bool(v))
}
/// Scalar error argument.
#[allow(dead_code)]
fn e(kind: ErrorKind) -> Arg {
    Arg::Value(Value::Error(kind))
}
/// A reference to an empty cell.
#[allow(dead_code)]
fn empty() -> Arg {
    Arg::Value(Value::Empty)
}
/// A range argument with `cols` columns.
#[allow(dead_code)]
fn grid(cols: usize, cells: Vec<Value>) -> Arg {
    assert!(cols > 0 && cells.len() % cols == 0, "bad test grid");
    Arg::Range(Grid { rows: cells.len() / cols, cols, cells })
}
/// A one-column range.
#[allow(dead_code)]
fn column(cells: Vec<Value>) -> Arg {
    grid(1, cells)
}
/// A one-row range.
#[allow(dead_code)]
fn row(cells: Vec<Value>) -> Arg {
    let cols = cells.len();
    grid(cols, cells)
}
