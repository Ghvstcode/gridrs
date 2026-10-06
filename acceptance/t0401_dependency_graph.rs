//! Acceptance test for task 0401-dependency-graph. Outside every task's write scope.

use gridrs::depgraph::DepGraph;
use gridrs::types::{CellRef, Range, RecalcPlan};
use std::collections::HashSet;

fn c(col: u32, row: u32) -> CellRef {
    CellRef::new(col, row)
}
fn one(col: u32, row: u32) -> Range {
    Range::single(c(col, row))
}
fn rng(c0: u32, r0: u32, c1: u32, r1: u32) -> Range {
    Range::new(c(c0, r0), c(c1, r1))
}
fn sorted(mut v: Vec<CellRef>) -> Vec<CellRef> {
    v.sort();
    v
}

/// Check the invariants of a plan against the graph and return (order, cyclic).
fn check(g: &DepGraph, changed: &[CellRef], want_order_set: &[CellRef], want_cyclic: &[CellRef]) -> RecalcPlan {
    let plan = g.recalc_plan(changed);
    assert_eq!(sorted(plan.order.clone()), sorted(want_order_set.to_vec()), "cells in `order` (as a set) for changed={changed:?}");
    assert_eq!(plan.cyclic, sorted(want_cyclic.to_vec()), "`cyclic` (sorted ascending) for changed={changed:?}");
    let unique: HashSet<_> = plan.order.iter().collect();
    assert_eq!(unique.len(), plan.order.len(), "no cell appears twice in `order`");
    // every cell comes after all the cells in the plan that it reads
    for (i, cell) in plan.order.iter().enumerate() {
        for range in g.precedents(*cell) {
            for (j, earlier) in plan.order.iter().enumerate() {
                if range.contains(*earlier) {
                    assert!(j < i, "{cell:?} reads {earlier:?} but is ordered before it: {:?}", plan.order);
                }
            }
        }
    }
    plan
}

#[test]
fn empty_graph() {
    let g = DepGraph::new();
    assert_eq!(g.recalc_plan(&[]), RecalcPlan::default());
    assert_eq!(g.recalc_plan(&[c(0, 0), c(5, 5)]), RecalcPlan::default());
    assert_eq!(g.precedents(c(0, 0)), vec![]);
    assert_eq!(g.dependents(c(0, 0)), vec![]);
}

#[test]
fn precedents_are_stored_and_replaced() {
    let mut g = DepGraph::new();
    g.set_precedents(c(2, 0), vec![one(0, 0), rng(0, 0, 1, 5), one(0, 0)]);
    assert_eq!(g.precedents(c(2, 0)), vec![one(0, 0), rng(0, 0, 1, 5), one(0, 0)], "kept exactly as given");
    g.set_precedents(c(2, 0), vec![one(9, 9)]);
    assert_eq!(g.precedents(c(2, 0)), vec![one(9, 9)]);
    assert_eq!(g.dependents(c(0, 0)), vec![]);
    assert_eq!(g.dependents(c(9, 9)), vec![c(2, 0)]);
    g.remove(c(2, 0));
    assert_eq!(g.precedents(c(2, 0)), vec![]);
    assert_eq!(g.dependents(c(9, 9)), vec![]);
    g.remove(c(2, 0)); // removing twice is fine
    g.set_precedents(c(2, 0), vec![one(9, 9)]);
    g.set_precedents(c(2, 0), vec![]); // same as remove
    assert_eq!(g.dependents(c(9, 9)), vec![]);
}

#[test]
fn dependents_are_direct_sorted_and_unique() {
    let mut g = DepGraph::new();
    // C1 = A1 + A1,  B2 = SUM(A1:A3),  A2 = A1,  D1 = C1
    g.set_precedents(c(2, 0), vec![one(0, 0), one(0, 0)]);
    g.set_precedents(c(1, 1), vec![rng(0, 0, 0, 2)]);
    g.set_precedents(c(0, 1), vec![one(0, 0)]);
    g.set_precedents(c(3, 0), vec![one(2, 0)]);
    assert_eq!(g.dependents(c(0, 0)), vec![c(2, 0), c(0, 1), c(1, 1)], "row-major order, each once");
    assert_eq!(g.dependents(c(0, 2)), vec![c(1, 1)]);
    assert_eq!(g.dependents(c(2, 0)), vec![c(3, 0)]);
    assert_eq!(g.dependents(c(3, 0)), vec![]);
}

#[test]
fn a_chain_is_ordered() {
    // B1 = A1, C1 = B1, D1 = C1, E1 = D1  (declared in a scrambled order)
    let mut g = DepGraph::new();
    g.set_precedents(c(3, 0), vec![one(2, 0)]);
    g.set_precedents(c(1, 0), vec![one(0, 0)]);
    g.set_precedents(c(4, 0), vec![one(3, 0)]);
    g.set_precedents(c(2, 0), vec![one(1, 0)]);
    let plan = check(&g, &[c(0, 0)], &[c(1, 0), c(2, 0), c(3, 0), c(4, 0)], &[]);
    assert_eq!(plan.order, vec![c(1, 0), c(2, 0), c(3, 0), c(4, 0)]);
    // changing the middle only touches what is downstream
    let plan = check(&g, &[c(2, 0)], &[c(2, 0), c(3, 0), c(4, 0)], &[]);
    assert_eq!(plan.order, vec![c(2, 0), c(3, 0), c(4, 0)]);
    check(&g, &[c(4, 0)], &[c(4, 0)], &[]);
    check(&g, &[c(9, 9)], &[], &[]);
}

#[test]
fn a_changed_formula_cell_is_itself_recalculated() {
    let mut g = DepGraph::new();
    g.set_precedents(c(1, 0), vec![one(0, 0)]);
    check(&g, &[c(1, 0)], &[c(1, 0)], &[]);
    // a changed cell that has no formula is not in the plan
    check(&g, &[c(0, 0)], &[c(1, 0)], &[]);
}

#[test]
fn diamond() {
    //      A1
    //     /  \
    //   B1    B2      B1 = A1, B2 = A1, C1 = B1 + B2, D1 = C1 + A1
    //     \  /
    //      C1 -> D1
    let mut g = DepGraph::new();
    g.set_precedents(c(3, 0), vec![one(2, 0), one(0, 0)]);
    g.set_precedents(c(2, 0), vec![one(1, 0), one(1, 1)]);
    g.set_precedents(c(1, 0), vec![one(0, 0)]);
    g.set_precedents(c(1, 1), vec![one(0, 0)]);
    let plan = check(&g, &[c(0, 0)], &[c(1, 0), c(1, 1), c(2, 0), c(3, 0)], &[]);
    assert_eq!(&plan.order[2..], &[c(2, 0), c(3, 0)]);
    check(&g, &[c(1, 1)], &[c(1, 1), c(2, 0), c(3, 0)], &[]);
}

#[test]
fn ranges_create_dependencies() {
    // D1 = SUM(A1:B3), D2 = SUM(A1:A1048576), D3 = D1 + D2
    let mut g = DepGraph::new();
    g.set_precedents(c(3, 0), vec![rng(0, 0, 1, 2)]);
    g.set_precedents(c(3, 1), vec![rng(0, 0, 0, 1048575)]);
    g.set_precedents(c(3, 2), vec![one(3, 0), one(3, 1)]);
    check(&g, &[c(1, 2)], &[c(3, 0), c(3, 2)], &[]);
    check(&g, &[c(0, 500_000)], &[c(3, 1), c(3, 2)], &[]);
    check(&g, &[c(0, 0)], &[c(3, 0), c(3, 1), c(3, 2)], &[]);
    check(&g, &[c(2, 0)], &[], &[]);
    check(&g, &[c(1, 3)], &[], &[]);
}

#[test]
fn several_changed_cells_at_once() {
    let mut g = DepGraph::new();
    g.set_precedents(c(1, 0), vec![one(0, 0)]); // B1 = A1
    g.set_precedents(c(1, 1), vec![one(0, 1)]); // B2 = A2
    g.set_precedents(c(2, 0), vec![one(1, 0), one(1, 1)]); // C1 = B1 + B2
    g.set_precedents(c(5, 5), vec![one(4, 4)]); // unrelated
    check(&g, &[c(0, 0), c(0, 1)], &[c(1, 0), c(1, 1), c(2, 0)], &[]);
    check(&g, &[c(0, 1), c(0, 1), c(1, 1)], &[c(1, 1), c(2, 0)], &[]);
    check(&g, &[c(0, 0), c(4, 4)], &[c(1, 0), c(2, 0), c(5, 5)], &[]);
}

#[test]
fn self_reference_is_a_cycle() {
    let mut g = DepGraph::new();
    g.set_precedents(c(0, 0), vec![one(0, 0)]);
    check(&g, &[c(0, 0)], &[], &[c(0, 0)]);
    // through a range that contains the cell itself: A2 = SUM(A1:A3)
    let mut g = DepGraph::new();
    g.set_precedents(c(0, 1), vec![rng(0, 0, 0, 2)]);
    check(&g, &[c(0, 1)], &[], &[c(0, 1)]);
    check(&g, &[c(0, 0)], &[], &[c(0, 1)]);
}

#[test]
fn cycles_and_everything_downstream_of_them_are_cyclic() {
    // A1 = B1, B1 = C1, C1 = A1   (a 3-cycle)
    // D1 = A1                      (reads the cycle)
    // E1 = D1                      (downstream of that)
    // F1 = G1                      (G1 is a plain value: unrelated)
    // H1 = Z9                      (Z9 feeds... the cycle too: A1 = B1 + Z9)
    let mut g = DepGraph::new();
    g.set_precedents(c(0, 0), vec![one(1, 0), one(25, 8)]);
    g.set_precedents(c(1, 0), vec![one(2, 0)]);
    g.set_precedents(c(2, 0), vec![one(0, 0)]);
    g.set_precedents(c(3, 0), vec![one(0, 0)]);
    g.set_precedents(c(4, 0), vec![one(3, 0)]);
    g.set_precedents(c(5, 0), vec![one(6, 0)]);
    g.set_precedents(c(7, 0), vec![one(25, 8)]);
    let all_cyclic = [c(0, 0), c(1, 0), c(2, 0), c(3, 0), c(4, 0)];
    check(&g, &[c(1, 0)], &[], &all_cyclic);
    check(&g, &[c(0, 0)], &[], &all_cyclic);
    // a change upstream of the cycle: H1 is healthy, the rest is cyclic
    check(&g, &[c(25, 8)], &[c(7, 0)], &all_cyclic);
    // a change downstream of the cycle only: D1 still reads a cyclic cell, so it stays cyclic
    check(&g, &[c(3, 0)], &[], &[c(3, 0), c(4, 0)]);
    check(&g, &[c(4, 0)], &[], &[c(4, 0)]);
    // unrelated cells are unaffected
    check(&g, &[c(6, 0)], &[c(5, 0)], &[]);
}

#[test]
fn breaking_a_cycle_heals_it() {
    let mut g = DepGraph::new();
    g.set_precedents(c(0, 0), vec![one(1, 0)]); // A1 = B1
    g.set_precedents(c(1, 0), vec![one(0, 0)]); // B1 = A1
    g.set_precedents(c(2, 0), vec![one(1, 0)]); // C1 = B1
    check(&g, &[c(0, 0)], &[], &[c(0, 0), c(1, 0), c(2, 0)]);
    g.remove(c(0, 0)); // A1 becomes a plain value
    let plan = check(&g, &[c(0, 0)], &[c(1, 0), c(2, 0)], &[]);
    assert_eq!(plan.order, vec![c(1, 0), c(2, 0)]);
}

#[test]
fn two_separate_cycles() {
    let mut g = DepGraph::new();
    g.set_precedents(c(0, 0), vec![one(0, 1)]);
    g.set_precedents(c(0, 1), vec![one(0, 0)]);
    g.set_precedents(c(5, 0), vec![one(5, 1)]);
    g.set_precedents(c(5, 1), vec![one(5, 0)]);
    check(&g, &[c(0, 0)], &[], &[c(0, 0), c(0, 1)]);
    check(&g, &[c(5, 1)], &[], &[c(5, 0), c(5, 1)]);
    check(&g, &[c(0, 0), c(5, 0)], &[], &[c(0, 0), c(5, 0), c(0, 1), c(5, 1)]);
}

#[test]
fn a_long_chain_does_not_overflow_the_stack() {
    // A1 is a value; A2 = A1, A3 = A2, ... A20001 = A20000
    let mut g = DepGraph::new();
    let n = 20_000;
    for row in 1..=n {
        g.set_precedents(c(0, row), vec![one(0, row - 1)]);
    }
    let plan = g.recalc_plan(&[c(0, 0)]);
    assert!(plan.cyclic.is_empty());
    assert_eq!(plan.order, (1..=n).map(|row| c(0, row)).collect::<Vec<_>>());
}

#[test]
fn clone_is_independent() {
    let mut g = DepGraph::new();
    g.set_precedents(c(1, 0), vec![one(0, 0)]);
    let snapshot = g.clone();
    g.remove(c(1, 0));
    assert_eq!(snapshot.dependents(c(0, 0)), vec![c(1, 0)]);
    assert_eq!(g.dependents(c(0, 0)), vec![]);
}
