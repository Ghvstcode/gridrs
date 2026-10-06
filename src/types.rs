//! Shared plain-data types and traits: **the contract**.
//!
//! Every task compiles against this file and every acceptance test imports it.
//! It is deliberately outside every task's write scope: nothing in here is
//! `todo!()`, and nothing in here should ever need to change for a task to be
//! completed. If you think it does, the manifest is wrong — open an issue.

use std::fmt;

// ---------------------------------------------------------------------------
// Limits
// ---------------------------------------------------------------------------

/// Number of columns in a sheet (`A` ..= `XFD`), same as Excel.
pub const MAX_COLS: u32 = 16_384;
/// Number of rows in a sheet (`1` ..= `1048576`), same as Excel.
pub const MAX_ROWS: u32 = 1_048_576;
/// Width, in terminal cells, of a column that has no explicit width.
pub const DEFAULT_COL_WIDTH: usize = 10;
/// Smallest column width a user can set.
pub const MIN_COL_WIDTH: usize = 1;
/// Largest column width a user can set.
pub const MAX_COL_WIDTH: usize = 60;
/// A range with more cells than this is not materialised by the evaluator
/// (it evaluates to `#NUM!` instead of eating all memory).
pub const MAX_RANGE_CELLS: u64 = 1_000_000;

// ---------------------------------------------------------------------------
// Addresses
// ---------------------------------------------------------------------------

/// A cell position. Both coordinates are **zero-based**: `A1` is
/// `CellRef { row: 0, col: 0 }`, `B12` is `CellRef { row: 11, col: 1 }`.
///
/// The derived ordering is row-major (all of row 1, then row 2, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellRef {
    pub row: u32,
    pub col: u32,
}

impl CellRef {
    /// Column first, then row — the order you read `B12` in.
    pub const fn new(col: u32, row: u32) -> CellRef {
        CellRef { row, col }
    }
}

/// A cell reference as written in a formula: a cell plus `$` flags.
/// `$B12` is `Reference { cell: B12, abs_col: true, abs_row: false }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reference {
    pub cell: CellRef,
    pub abs_col: bool,
    pub abs_row: bool,
}

impl Reference {
    /// A reference with no `$` flags.
    pub const fn relative(cell: CellRef) -> Reference {
        Reference { cell, abs_col: false, abs_row: false }
    }
}

/// An inclusive rectangle of cells. Invariant (upheld by [`Range::new`]):
/// `start.col <= end.col && start.row <= end.row`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: CellRef,
    pub end: CellRef,
}

impl Range {
    /// Build a normalised range from any two corners.
    pub fn new(a: CellRef, b: CellRef) -> Range {
        Range {
            start: CellRef::new(a.col.min(b.col), a.row.min(b.row)),
            end: CellRef::new(a.col.max(b.col), a.row.max(b.row)),
        }
    }

    /// The one-cell range.
    pub fn single(cell: CellRef) -> Range {
        Range { start: cell, end: cell }
    }

    pub fn contains(&self, cell: CellRef) -> bool {
        cell.col >= self.start.col
            && cell.col <= self.end.col
            && cell.row >= self.start.row
            && cell.row <= self.end.row
    }

    /// Number of rows covered (at least 1).
    pub fn rows(&self) -> u32 {
        self.end.row - self.start.row + 1
    }

    /// Number of columns covered (at least 1).
    pub fn cols(&self) -> u32 {
        self.end.col - self.start.col + 1
    }

    /// Number of cells covered.
    pub fn len(&self) -> u64 {
        self.rows() as u64 * self.cols() as u64
    }

    /// Always false; a range covers at least one cell.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Every cell in the range, row-major (left to right, then top to bottom).
    pub fn cells(&self) -> impl Iterator<Item = CellRef> {
        let (c0, c1) = (self.start.col, self.end.col);
        (self.start.row..=self.end.row).flat_map(move |r| (c0..=c1).map(move |c| CellRef::new(c, r)))
    }
}

// ---------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------

/// Spreadsheet error values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// `#DIV/0!` — division by zero.
    Div0,
    /// `#REF!` — a reference that no longer points anywhere.
    Ref,
    /// `#NAME?` — unknown function or unparseable formula.
    Name,
    /// `#VALUE!` — wrong type or wrong number of arguments.
    Value,
    /// `#NUM!` — numeric result out of range.
    Num,
    /// `#N/A` — lookup found nothing.
    NA,
    /// `#CYCLE!` — the cell is part of (or depends on) a circular reference.
    Cycle,
}

impl ErrorKind {
    pub const ALL: [ErrorKind; 7] = [
        ErrorKind::Div0,
        ErrorKind::Ref,
        ErrorKind::Name,
        ErrorKind::Value,
        ErrorKind::Num,
        ErrorKind::NA,
        ErrorKind::Cycle,
    ];

    /// The text shown in a cell, e.g. `#DIV/0!`.
    pub const fn as_str(self) -> &'static str {
        match self {
            ErrorKind::Div0 => "#DIV/0!",
            ErrorKind::Ref => "#REF!",
            ErrorKind::Name => "#NAME?",
            ErrorKind::Value => "#VALUE!",
            ErrorKind::Num => "#NUM!",
            ErrorKind::NA => "#N/A",
            ErrorKind::Cycle => "#CYCLE!",
        }
    }

    /// Inverse of [`ErrorKind::as_str`], ASCII case-insensitive, exact match only.
    pub fn parse(s: &str) -> Option<ErrorKind> {
        ErrorKind::ALL.iter().copied().find(|e| e.as_str().eq_ignore_ascii_case(s))
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The value of a cell (or of any formula sub-expression).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Empty,
    Number(f64),
    Text(String),
    Bool(bool),
    Error(ErrorKind),
}

/// How a number is displayed. Does not affect the stored value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NumFormat {
    /// Excel-adjacent "General": as many digits as needed, up to 15 significant.
    #[default]
    General,
    /// Fixed number of decimals: `1234.5` with `Fixed(2)` is `1234.50`.
    Fixed(u8),
    /// Fixed decimals with thousands separators: `1,234.50`.
    Thousands(u8),
    /// Multiplied by 100 with a `%` sign: `0.125` with `Percent(1)` is `12.5%`.
    Percent(u8),
}

impl NumFormat {
    /// Largest number of decimals a format may ask for.
    pub const MAX_DECIMALS: u8 = 15;

    /// Parse the spelling used by the native file format and the `:fmt`
    /// command: `general`, `fixed N`, `thousands N`, `percent N`
    /// (lower case, single spaces, `0 <= N <= 15`, no leading zeros or signs).
    pub fn parse(spec: &str) -> Option<NumFormat> {
        if spec == "general" {
            return Some(NumFormat::General);
        }
        let (kind, n) = spec.split_once(' ')?;
        if n.is_empty() || n.len() > 2 || !n.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        if n.len() == 2 && n.starts_with('0') {
            return None;
        }
        let n: u8 = n.parse().ok()?;
        if n > NumFormat::MAX_DECIMALS {
            return None;
        }
        match kind {
            "fixed" => Some(NumFormat::Fixed(n)),
            "thousands" => Some(NumFormat::Thousands(n)),
            "percent" => Some(NumFormat::Percent(n)),
            _ => None,
        }
    }
}

/// Prints the spelling accepted by [`NumFormat::parse`].
impl fmt::Display for NumFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumFormat::General => write!(f, "general"),
            NumFormat::Fixed(n) => write!(f, "fixed {n}"),
            NumFormat::Thousands(n) => write!(f, "thousands {n}"),
            NumFormat::Percent(n) => write!(f, "percent {n}"),
        }
    }
}

/// Horizontal alignment of a cell's text inside its column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    Left,
    Right,
    Center,
}

// ---------------------------------------------------------------------------
// Formulas
// ---------------------------------------------------------------------------

/// A lexical token of a formula.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A numeric literal (never negative: the minus sign is its own token).
    Number(f64),
    /// A string literal, already unescaped (no surrounding quotes).
    Text(String),
    /// A bare word exactly as written: a function name, a cell reference
    /// (possibly with `$`), `TRUE`/`FALSE`, or something unknown. The parser decides.
    Ident(String),
    /// An error literal such as `#REF!`.
    Error(ErrorKind),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Amp,
    Percent,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    LParen,
    RParen,
    Comma,
    Colon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// Prefix `-`.
    Neg,
    /// Prefix `+`.
    Pos,
    /// Postfix `%` (divide by 100).
    Percent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    /// `&` — text concatenation.
    Concat,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// A parsed formula.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Text(String),
    Bool(bool),
    Error(ErrorKind),
    /// A single cell reference, e.g. `$A1`.
    Ref(Reference),
    /// A range `first:second`, endpoints **as written** (not normalised).
    Range(Reference, Reference),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    /// A function call. The name is stored in ASCII upper case.
    Call(String, Vec<Expr>),
}

/// Why a formula could not be tokenised or parsed. The message is for humans;
/// nothing matches on its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaError {
    pub message: String,
}

impl FormulaError {
    pub fn new(message: impl Into<String>) -> FormulaError {
        FormulaError { message: message.into() }
    }
}

impl fmt::Display for FormulaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FormulaError {}

/// A rectangular block of values handed to a function for a range argument.
/// `cells` is row-major and `cells.len() == rows * cols`.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Value>,
}

impl Grid {
    /// The value at zero-based (`row`, `col`). Panics when out of bounds.
    pub fn get(&self, row: usize, col: usize) -> &Value {
        assert!(row < self.rows && col < self.cols, "Grid::get out of bounds");
        &self.cells[row * self.cols + col]
    }
}

/// One evaluated function argument.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    /// Anything that is not a range: a literal, a single cell reference, a
    /// nested expression. A reference to an empty cell arrives as `Value::Empty`;
    /// an expression that failed arrives as `Value::Error(_)` — functions decide
    /// whether to propagate it.
    Value(Value),
    /// A range argument such as `A1:B7`, materialised.
    Range(Grid),
}

/// What the evaluator needs from the outside world.
pub trait Context {
    /// The current value of a cell (`Value::Empty` if nothing is there).
    fn cell(&self, at: CellRef) -> Value;

    /// Call a function by (upper-case) name. The default goes to the built-in
    /// library; tests override it to observe calls.
    fn call(&self, name: &str, args: &[Arg]) -> Value {
        crate::funcs::call(name, args)
    }
}

/// A row/column insertion or deletion. Indices are zero-based.
/// Insertions happen *before* index `at`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructuralEdit {
    InsertRows { at: u32, count: u32 },
    DeleteRows { at: u32, count: u32 },
    InsertCols { at: u32, count: u32 },
    DeleteCols { at: u32, count: u32 },
}

/// Result of asking the dependency graph what to recalculate.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecalcPlan {
    /// Cells to recalculate, each one after everything it depends on.
    pub order: Vec<CellRef>,
    /// Cells that are on a cycle or depend on one; they become `#CYCLE!`.
    /// Sorted ascending.
    pub cyclic: Vec<CellRef>,
}

// ---------------------------------------------------------------------------
// Files
// ---------------------------------------------------------------------------

/// A CSV syntax error. `line` is the 1-based physical line where it was noticed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for CsvError {}

/// What a CSV export writes for each cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CsvMode {
    /// What the user typed (`=SUM(A1:A3)`): lossless for gridrs, and other
    /// spreadsheets re-evaluate it.
    Inputs,
    /// What the user sees (`6.00`): values rendered with each cell's number format.
    Values,
}

/// Which on-disk format a path uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    Csv,
    /// The native `.grid` format, which also keeps number formats and column widths.
    Native,
}

/// Everything the native `.grid` format stores, as plain data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    /// Non-empty cell inputs, exactly as typed.
    pub cells: Vec<(CellRef, String)>,
    /// Cells whose number format is not `General`.
    pub formats: Vec<(CellRef, NumFormat)>,
    /// Columns whose width is not [`DEFAULT_COL_WIDTH`]: (column, width).
    pub col_widths: Vec<(u32, usize)>,
}

// ---------------------------------------------------------------------------
// Clipboard
// ---------------------------------------------------------------------------

/// One copied cell, positioned relative to the top-left corner of the copied range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipCell {
    pub drow: u32,
    pub dcol: u32,
    /// The input exactly as it was in the source cell (may be empty if only a format was set).
    pub input: String,
    pub format: NumFormat,
}

/// A copied rectangle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clip {
    /// Top-left corner of the range that was copied.
    pub origin: CellRef,
    pub rows: u32,
    pub cols: u32,
    /// The cells of the range that had an input or a non-`General` format, row-major.
    pub cells: Vec<ClipCell>,
}

// ---------------------------------------------------------------------------
// Terminal
// ---------------------------------------------------------------------------

/// A decoded key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A printable character (anything from U+0020 up, except DEL).
    Char(char),
    /// Ctrl plus a letter, stored as the lower-case letter: Ctrl-R is `Ctrl('r')`.
    Ctrl(char),
    Enter,
    Tab,
    /// Shift-Tab.
    BackTab,
    Backspace,
    Delete,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
}

/// Runs the `stty` program. Abstracted so raw-mode handling is testable without a terminal.
pub trait Stty {
    /// Run `stty` with these arguments and return its standard output.
    /// A non-zero exit status is an `Err`.
    fn run(&mut self, args: &[&str]) -> std::io::Result<String>;
}

/// What the renderer needs to know about the grid contents.
pub trait GridSource {
    /// The text to display in a cell, already formatted. Empty string for an empty cell.
    fn text(&self, at: CellRef) -> String;
    /// How that text is aligned within the column.
    fn align(&self, at: CellRef) -> Align;
    /// Width of a column in terminal cells (at least 1).
    fn col_width(&self, col: u32) -> usize;
}

/// Everything about a frame that is not cell content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ui {
    /// Terminal width in cells.
    pub width: usize,
    /// Terminal height in lines.
    pub height: usize,
    /// First visible row (zero-based).
    pub top: u32,
    /// First visible column (zero-based).
    pub left: u32,
    /// The selected cell.
    pub cursor: CellRef,
    /// Text of the formula bar (the selected cell's input, or what is being typed).
    pub bar: String,
    /// Text on the left of the status line.
    pub status: String,
    /// Mode label on the right of the status line, e.g. `NORMAL`.
    pub mode: String,
}

/// One visible column of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColSlot {
    /// Zero-based sheet column.
    pub col: u32,
    /// Zero-based x position of the column's first character on the line.
    pub x: usize,
    /// Width in characters.
    pub width: usize,
}

/// Where things land in a frame; computed by `tui::render::layout`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// Width of the row-number gutter.
    pub gutter: usize,
    /// Number of sheet rows shown.
    pub rows: usize,
    /// The fully visible columns, left to right.
    pub cols: Vec<ColSlot>,
}

// ---------------------------------------------------------------------------
// Application
// ---------------------------------------------------------------------------

/// A parsed `:` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `:w [file]`
    Write(Option<String>),
    /// `:q` / `:q!`
    Quit { force: bool },
    /// `:wq [file]` / `:x [file]`
    WriteQuit(Option<String>),
    /// `:e file`
    Edit(String),
    /// `:export file` — write displayed values as CSV.
    Export(String),
    /// `:goto B12` / `:B12`
    Goto(CellRef),
    InsertRow,
    DeleteRow,
    InsertCol,
    DeleteCol,
    /// `:fmt fixed 2`
    Format(NumFormat),
    /// `:width 14`
    Width(usize),
    Undo,
    Redo,
    Help,
}

/// The editor's input mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Moving around; keys are commands.
    Normal,
    /// Typing a cell's content.
    Edit,
    /// Typing a `:` command.
    Command,
}

/// What the event loop should do after a key was handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    Continue,
    Quit,
}

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    Help,
    Version,
    /// Evaluate formulas non-interactively (optionally against a file) and print the results.
    Eval { file: Option<String>, formulas: Vec<String> },
    /// Print one plain-text frame of the file and exit.
    Dump { file: String, width: usize, height: usize },
    /// Start the interactive editor.
    Open { file: Option<String> },
}
