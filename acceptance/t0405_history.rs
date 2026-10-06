//! Acceptance test for task 0405-history. Outside every task's write scope.

use gridrs::history::History;

/// Simulates an editor whose whole state is one integer.
struct Editor {
    state: i32,
    history: History<i32>,
}

impl Editor {
    fn new(limit: usize) -> Editor {
        Editor { state: 0, history: History::new(limit) }
    }
    fn edit(&mut self, new_state: i32) {
        self.history.record(self.state);
        self.state = new_state;
    }
    fn undo(&mut self) -> bool {
        match self.history.undo(self.state) {
            Some(previous) => {
                self.state = previous;
                true
            }
            None => false,
        }
    }
    fn redo(&mut self) -> bool {
        match self.history.redo(self.state) {
            Some(next) => {
                self.state = next;
                true
            }
            None => false,
        }
    }
}

#[test]
fn a_new_history_has_nothing_to_undo_or_redo() {
    let mut h: History<String> = History::new(10);
    assert!(!h.can_undo());
    assert!(!h.can_redo());
    assert_eq!(h.undo("current".to_string()), None);
    assert_eq!(h.redo("current".to_string()), None);
    assert!(!h.can_undo() && !h.can_redo(), "a failed undo/redo must not push anything");
}

#[test]
fn undo_walks_back_and_redo_walks_forward() {
    let mut e = Editor::new(100);
    e.edit(1);
    e.edit(2);
    e.edit(3);
    assert!(e.history.can_undo() && !e.history.can_redo());
    assert!(e.undo());
    assert_eq!(e.state, 2);
    assert!(e.undo());
    assert_eq!(e.state, 1);
    assert!(e.history.can_undo() && e.history.can_redo());
    assert!(e.redo());
    assert_eq!(e.state, 2);
    assert!(e.redo());
    assert_eq!(e.state, 3);
    assert!(!e.redo(), "nothing left to redo");
    assert_eq!(e.state, 3);
    assert!(e.undo() && e.undo() && e.undo());
    assert_eq!(e.state, 0);
    assert!(!e.undo(), "nothing left to undo");
    assert_eq!(e.state, 0);
    assert!(e.redo() && e.redo() && e.redo());
    assert_eq!(e.state, 3);
}

#[test]
fn a_new_edit_discards_the_redo_stack() {
    let mut e = Editor::new(100);
    e.edit(1);
    e.edit(2);
    e.edit(3);
    e.undo();
    e.undo();
    assert_eq!(e.state, 1);
    e.edit(10);
    assert!(!e.history.can_redo());
    assert!(!e.redo());
    assert_eq!(e.state, 10);
    assert!(e.undo());
    assert_eq!(e.state, 1);
    assert!(e.undo());
    assert_eq!(e.state, 0);
    assert!(!e.undo());
}

#[test]
fn the_limit_drops_the_oldest_steps() {
    let mut e = Editor::new(3);
    for i in 1..=10 {
        e.edit(i);
    }
    assert!(e.undo());
    assert_eq!(e.state, 9);
    assert!(e.undo());
    assert_eq!(e.state, 8);
    assert!(e.undo());
    assert_eq!(e.state, 7);
    assert!(!e.undo(), "only 3 steps are remembered");
    assert_eq!(e.state, 7);
    // everything undone can still be redone
    assert!(e.redo() && e.redo() && e.redo());
    assert_eq!(e.state, 10);
    assert!(!e.redo());
    // and undone again
    assert!(e.undo() && e.undo() && e.undo());
    assert_eq!(e.state, 7);
    assert!(!e.undo());
}

#[test]
fn a_limit_of_zero_remembers_nothing() {
    let mut e = Editor::new(0);
    e.edit(1);
    e.edit(2);
    assert!(!e.history.can_undo());
    assert!(!e.undo());
    assert!(!e.redo());
    assert_eq!(e.state, 2);
}

#[test]
fn a_limit_of_one() {
    let mut e = Editor::new(1);
    e.edit(1);
    e.edit(2);
    assert!(e.undo());
    assert_eq!(e.state, 1);
    assert!(!e.undo());
    assert!(e.redo());
    assert_eq!(e.state, 2);
    assert!(e.undo());
    assert_eq!(e.state, 1);
}

#[test]
fn clear_forgets_both_directions() {
    let mut e = Editor::new(100);
    e.edit(1);
    e.edit(2);
    e.undo();
    assert!(e.history.can_undo() && e.history.can_redo());
    e.history.clear();
    assert!(!e.history.can_undo() && !e.history.can_redo());
    assert!(!e.undo() && !e.redo());
    assert_eq!(e.state, 1);
    // still usable afterwards, with the same limit
    e.edit(5);
    assert!(e.undo());
    assert_eq!(e.state, 1);
}

#[test]
fn works_with_types_that_are_not_copy_or_clone() {
    struct Snapshot(Vec<String>);
    let mut h: History<Snapshot> = History::new(5);
    h.record(Snapshot(vec!["a".into()]));
    let previous = h.undo(Snapshot(vec!["a".into(), "b".into()])).expect("one step to undo");
    assert_eq!(previous.0, ["a"]);
    let next = h.redo(previous).expect("one step to redo");
    assert_eq!(next.0, ["a", "b"]);
}

#[test]
fn clone_is_independent() {
    let mut h: History<i32> = History::new(5);
    h.record(1);
    let mut copy = h.clone();
    assert_eq!(h.undo(2), Some(1));
    assert!(!h.can_undo());
    assert!(copy.can_undo());
    assert_eq!(copy.undo(99), Some(1));
}

#[test]
fn many_steps() {
    let mut e = Editor::new(1000);
    for i in 1..=5000 {
        e.edit(i);
    }
    let mut undone = 0;
    while e.undo() {
        undone += 1;
    }
    assert_eq!(undone, 1000);
    assert_eq!(e.state, 4000);
    let mut redone = 0;
    while e.redo() {
        redone += 1;
    }
    assert_eq!(redone, 1000);
    assert_eq!(e.state, 5000);
}
