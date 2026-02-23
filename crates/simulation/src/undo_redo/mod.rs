//! Undo/Redo System for Player Actions (UX-001).
//!
//! Implements the command pattern for all player actions. An `ActionHistory`
//! resource maintains undo and redo stacks (capped at 100 entries). Actions
//! are recorded via the `RecordAction` event, and Ctrl+Z / Ctrl+Y (or
//! Ctrl+Shift+Z) trigger undo/redo respectively.
//!
//! Composite actions group drag operations (e.g., road drag = 1 undo step).
//! Treasury is restored on undo.

pub mod history;
pub mod systems;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export everything so external callers don't break.
pub use history::{ActionHistory, RedoRequested, UndoRequested};
pub use systems::{
    collect_actions, keyboard_undo_redo, process_redo, process_undo, UndoRedoPlugin,
};
pub use types::{CityAction, MAX_HISTORY};
