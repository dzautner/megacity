//! Saveable implementation for AchievementTracker.
//!
//! Persists unlocked achievements, progress counters, and state flags
//! across save/load cycles using bitcode encoding.

use super::types::AchievementTracker;
use crate::Saveable;

impl Saveable for AchievementTracker {
    const SAVE_KEY: &'static str = "achievement_tracker";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.unlocked.is_empty()
            && self.positive_trade_ticks == 0
            && !self.had_active_disaster
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
