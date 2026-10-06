//! Who depends on whom: recalculation order and cycle detection.
//!
//! Owned by task `0401-dependency-graph`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

use crate::types::{CellRef, Range, RecalcPlan};

/// The dependency graph of a sheet's formula cells. Fields are private and yours to design.
#[derive(Debug, Clone, Default)]
pub struct DepGraph {
    _todo: (),
}

impl DepGraph {
    pub fn new() -> DepGraph {
        todo!("task 0401-dependency-graph")
    }

    /// Declare that the formula in `cell` reads `ranges` (replacing whatever was declared before).
    /// An empty list is the same as [`DepGraph::remove`].
    pub fn set_precedents(&mut self, cell: CellRef, ranges: Vec<Range>) {
        todo!("task 0401-dependency-graph")
    }

    /// Forget `cell`'s formula (it no longer reads anything).
    pub fn remove(&mut self, cell: CellRef) {
        todo!("task 0401-dependency-graph")
    }

    /// The ranges last declared for `cell` (empty if none).
    pub fn precedents(&self, cell: CellRef) -> Vec<Range> {
        todo!("task 0401-dependency-graph")
    }

    /// Formula cells that read `cell` directly, sorted ascending.
    pub fn dependents(&self, cell: CellRef) -> Vec<CellRef> {
        todo!("task 0401-dependency-graph")
    }

    /// Given the cells whose content just changed, which formula cells must be
    /// recalculated, in which order, and which are caught in a cycle.
    pub fn recalc_plan(&self, changed: &[CellRef]) -> RecalcPlan {
        todo!("task 0401-dependency-graph")
    }
}
