//! Ring-buffer log of recently executed game actions and their results.
//!
//! The [`ActionResultLog`] resource stores the last 64 `(GameAction, ActionResult)`
//! pairs, giving callers (LLM agents, UI, replay verification) a way to inspect
//! what happened without polling the ECS every tick.

use bevy::prelude::*;

use super::{ActionResult, GameAction};

/// Maximum number of entries retained in the ring buffer.
const MAX_ENTRIES: usize = 64;

/// A ring-buffer log of the last [`MAX_ENTRIES`] action/result pairs.
#[derive(Resource, Debug, Clone, Default)]
pub struct ActionResultLog {
    entries: Vec<(GameAction, ActionResult)>,
}

impl ActionResultLog {
    /// Record a new action/result pair. If the buffer is full the oldest entry
    /// is evicted.
    pub fn push(&mut self, action: GameAction, result: ActionResult) {
        if self.entries.len() >= MAX_ENTRIES {
            self.entries.remove(0);
        }
        self.entries.push((action, result));
    }

    /// Return the last `n` entries (or fewer if the log is shorter).
    pub fn last_n(&self, n: usize) -> &[(GameAction, ActionResult)] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_actions::{ActionError, ActionResult, GameAction};

    #[test]
    fn push_and_last_n() {
        let mut log = ActionResultLog::default();
        log.push(
            GameAction::SetPaused { paused: true },
            ActionResult::Success,
        );
        log.push(
            GameAction::SetSpeed { speed: 3 },
            ActionResult::Error(ActionError::NotSupported),
        );

        let last = log.last_n(1);
        assert_eq!(last.len(), 1);
        assert_eq!(last[0].0, GameAction::SetSpeed { speed: 3 });

        let all = log.last_n(10);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn evicts_oldest_when_full() {
        let mut log = ActionResultLog::default();
        for i in 0..70 {
            log.push(
                GameAction::SetSpeed { speed: i },
                ActionResult::Success,
            );
        }
        assert_eq!(log.len(), MAX_ENTRIES);
        // The oldest retained should be speed=6 (70 - 64)
        let first = &log.last_n(MAX_ENTRIES)[0];
        assert_eq!(first.0, GameAction::SetSpeed { speed: 6 });
    }

    #[test]
    fn clear_empties_log() {
        let mut log = ActionResultLog::default();
        log.push(
            GameAction::SetPaused { paused: false },
            ActionResult::Success,
        );
        assert!(!log.is_empty());
        log.clear();
        assert!(log.is_empty());
    }
}
