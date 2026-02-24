//! Saveable implementations for EventJournal, ActiveCityEffects, and
//! MilestoneTracker so that event history and active modifiers persist
//! across save/load cycles.

use bevy::prelude::*;

use crate::events::{ActiveCityEffects, EventJournal, MilestoneTracker};
use crate::Saveable;

// ---------------------------------------------------------------------------
// EventJournal
// ---------------------------------------------------------------------------

impl Saveable for EventJournal {
    const SAVE_KEY: &'static str = "event_journal";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.events.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// ActiveCityEffects
// ---------------------------------------------------------------------------

impl Saveable for ActiveCityEffects {
    const SAVE_KEY: &'static str = "active_city_effects";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.festival_ticks == 0
            && self.economic_boom_ticks == 0
            && self.epidemic_ticks == 0
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// MilestoneTracker
// ---------------------------------------------------------------------------

impl Saveable for MilestoneTracker {
    const SAVE_KEY: &'static str = "milestone_tracker";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.reached_milestones.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EventJournalSavePlugin;

impl Plugin for EventJournalSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<EventJournal>();
        registry.register::<ActiveCityEffects>();
        registry.register::<MilestoneTracker>();
    }
}
