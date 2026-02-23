//! Action history resource and marker events for the undo/redo system.

use bevy::prelude::*;

use super::types::{CityAction, MAX_HISTORY};

// ---------------------------------------------------------------------------
// ActionHistory resource
// ---------------------------------------------------------------------------

/// Stores undo and redo stacks for player actions.
#[derive(Resource, Default)]
pub struct ActionHistory {
    pub undo_stack: Vec<CityAction>,
    pub redo_stack: Vec<CityAction>,
}

impl ActionHistory {
    /// Push a new action onto the undo stack, clearing the redo stack.
    /// If the stack exceeds `MAX_HISTORY`, the oldest action is dropped.
    pub fn push(&mut self, action: CityAction) {
        self.redo_stack.clear();
        self.undo_stack.push(action);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    /// Pop the most recent action from the undo stack for undoing.
    pub fn pop_undo(&mut self) -> Option<CityAction> {
        self.undo_stack.pop()
    }

    /// Pop the most recent action from the redo stack for redoing.
    pub fn pop_redo(&mut self) -> Option<CityAction> {
        self.redo_stack.pop()
    }

    /// Push an action onto the redo stack (after undo).
    pub fn push_redo(&mut self, action: CityAction) {
        self.redo_stack.push(action);
    }

    /// Push an action onto the undo stack (after redo), without clearing redo.
    pub fn push_undo_no_clear(&mut self, action: CityAction) {
        self.undo_stack.push(action);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Events for triggering undo/redo from keyboard input
// ---------------------------------------------------------------------------

/// Marker event: the player wants to undo.
#[derive(Event)]
pub struct UndoRequested;

/// Marker event: the player wants to redo.
#[derive(Event)]
pub struct RedoRequested;
