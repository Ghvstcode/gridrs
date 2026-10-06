//! gridrs — a terminal spreadsheet in Rust. Standard library only.
//!
//! This file, `src/types.rs`, the `mod.rs` files, `src/main.rs` and
//! `Cargo.toml` are laid out up front and are outside every task's write
//! scope. Each remaining module is owned by exactly one task under `tasks/`.

pub mod types;

pub mod addr;
pub mod value;

pub mod formula;
pub mod funcs;

pub mod depgraph;
pub mod numfmt;

pub mod io;

pub mod history;
pub mod ops;
pub mod sheet;

pub mod command;
pub mod tui;

pub mod app;
pub mod cli;
