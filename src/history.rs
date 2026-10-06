//! Undo/redo as a bounded stack of snapshots.
//!
//! Owned by task `0405-history`. Stub: the signatures are the contract, the bodies are yours.
#![allow(unused_variables)] // skeleton stub: delete this line when you implement the file

/// Undo/redo history over snapshots of type `T`. Fields are private and yours to design.
#[derive(Debug, Clone)]
pub struct History<T> {
    _todo: std::marker::PhantomData<T>,
}

impl<T> History<T> {
    /// A history that remembers at most `limit` undo steps.
    pub fn new(limit: usize) -> History<T> {
        todo!("task 0405-history")
    }

    /// Record the state as it was *before* a change. Clears the redo stack.
    pub fn record(&mut self, before: T) {
        todo!("task 0405-history")
    }

    /// Step back: give the current state, get the previous one (or `None`, and nothing changes).
    pub fn undo(&mut self, current: T) -> Option<T> {
        todo!("task 0405-history")
    }

    /// Step forward again after an undo.
    pub fn redo(&mut self, current: T) -> Option<T> {
        todo!("task 0405-history")
    }

    pub fn can_undo(&self) -> bool {
        todo!("task 0405-history")
    }

    pub fn can_redo(&self) -> bool {
        todo!("task 0405-history")
    }

    /// Forget everything.
    pub fn clear(&mut self) {
        todo!("task 0405-history")
    }
}
