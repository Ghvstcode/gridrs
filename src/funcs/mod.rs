//! The built-in function library.
//!
//! Each family lives in its own file and exposes
//! `pub fn call(name: &str, args: &[Arg]) -> Option<Value>`, returning `None`
//! for names it does not implement. This dispatcher is outside every task's
//! write scope, so four donors can write four families at the same time
//! without ever touching the same file.

use crate::types::{Arg, ErrorKind, Value};

pub mod logic;
pub mod lookup;
pub mod math;
pub mod text;

/// Call a built-in function. `name` must already be ASCII upper case.
/// Unknown names evaluate to `#NAME?`.
pub fn call(name: &str, args: &[Arg]) -> Value {
    math::call(name, args)
        .or_else(|| logic::call(name, args))
        .or_else(|| text::call(name, args))
        .or_else(|| lookup::call(name, args))
        .unwrap_or(Value::Error(ErrorKind::Name))
}
