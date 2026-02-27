//! Action queue resource for batching actions from any source.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::GameAction;

/// Identifies the origin of an action — useful for logging, replay
/// filtering, and trust decisions.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionSource {
    /// Direct player input (mouse/keyboard/UI).
    Player,
    /// An autonomous AI agent operating on the city.
    Agent,
    /// A replayed action from a recorded session.
    Replay,
}

/// A single action together with metadata about when and who enqueued it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedAction {
    /// The simulation tick at which this action was enqueued.
    pub tick: u64,
    /// Who or what submitted this action.
    pub source: ActionSource,
    /// The action payload.
    pub action: GameAction,
}

/// FIFO queue of pending game actions.
///
/// Systems that accept player input, agent commands, or replay data push
/// actions here. A future executor system will drain and apply them each
/// frame.
#[derive(Resource, Default, Debug)]
pub struct ActionQueue {
    pending: Vec<QueuedAction>,
}

impl ActionQueue {
    /// Enqueue a new action.
    pub fn push(&mut self, action: QueuedAction) {
        self.pending.push(action);
    }

    /// Drain all pending actions in FIFO order, leaving the queue empty.
    pub fn drain(&mut self) -> Vec<QueuedAction> {
        std::mem::take(&mut self.pending)
    }

    /// Returns `true` when there are no pending actions.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Number of pending actions.
    pub fn len(&self) -> usize {
        self.pending.len()
    }
}
