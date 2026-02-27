use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::GameAction;
use crate::Saveable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum ActionSource {
    Player,
    Agent,
    Replay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct QueuedAction {
    pub tick: u64,
    pub source: ActionSource,
    pub action: GameAction,
}

#[derive(Resource, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionQueue {
    pending: Vec<QueuedAction>,
}

impl ActionQueue {
    pub fn push(&mut self, tick: u64, source: ActionSource, action: GameAction) {
        self.pending.push(QueuedAction {
            tick,
            source,
            action,
        });
    }

    pub fn push_queued(&mut self, queued: QueuedAction) {
        self.pending.push(queued);
    }

    pub fn drain(&mut self) -> Vec<QueuedAction> {
        self.pending.drain(..).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }
}

#[derive(Encode, Decode, Default)]
struct ActionQueueSave {
    pending: Vec<QueuedAction>,
}

impl Saveable for ActionQueue {
    const SAVE_KEY: &'static str = "action_queue";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.pending.is_empty() {
            return None;
        }
        let save = ActionQueueSave {
            pending: self.pending.clone(),
        };
        Some(bitcode::encode(&save))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let save: ActionQueueSave = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        Self {
            pending: save.pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Saveable;
    use crate::grid::{RoadType, ZoneType};

    #[test]
    fn push_and_drain_preserves_fifo() {
        let mut queue = ActionQueue::default();
        queue.push(10, ActionSource::Player, GameAction::SetPaused { paused: true });
        queue.push(
            10,
            ActionSource::Agent,
            GameAction::SetSpeed { speed: 2 },
        );
        queue.push(
            11,
            ActionSource::Replay,
            GameAction::PlaceRoadLine {
                start: (5, 5),
                end: (10, 5),
                road_type: RoadType::Local,
            },
        );

        assert_eq!(queue.len(), 3);
        assert!(!queue.is_empty());

        let drained = queue.drain();
        assert_eq!(drained.len(), 3);
        assert!(queue.is_empty());

        assert_eq!(drained[0].tick, 10);
        assert_eq!(drained[0].source, ActionSource::Player);
        assert_eq!(drained[0].action, GameAction::SetPaused { paused: true });

        assert_eq!(drained[1].tick, 10);
        assert_eq!(drained[1].source, ActionSource::Agent);
        assert_eq!(drained[1].action, GameAction::SetSpeed { speed: 2 });

        assert_eq!(drained[2].tick, 11);
        assert_eq!(drained[2].source, ActionSource::Replay);
        assert_eq!(
            drained[2].action,
            GameAction::PlaceRoadLine {
                start: (5, 5),
                end: (10, 5),
                road_type: RoadType::Local
            }
        );
    }

    #[test]
    fn saveable_roundtrip_restores_pending_actions() {
        let mut queue = ActionQueue::default();
        queue.push(
            42,
            ActionSource::Player,
            GameAction::ZoneRect {
                min: (1, 2),
                max: (3, 4),
                zone_type: ZoneType::ResidentialLow,
            },
        );

        let bytes = queue
            .save_to_bytes()
            .expect("non-empty queue should produce save bytes");
        let restored = ActionQueue::load_from_bytes(&bytes);

        assert_eq!(restored.len(), 1);
        assert_eq!(
            restored,
            ActionQueue {
                pending: vec![QueuedAction {
                    tick: 42,
                    source: ActionSource::Player,
                    action: GameAction::ZoneRect {
                        min: (1, 2),
                        max: (3, 4),
                        zone_type: ZoneType::ResidentialLow,
                    },
                }],
            }
        );
    }
}
