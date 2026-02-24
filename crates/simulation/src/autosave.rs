//! SAVE-002: Autosave with Configurable Interval.
//!
//! Provides periodic autosave that saves game state without player action.
//! Configurable interval (default 5 minutes), with 3 rotating save slots
//! (autosave_1, autosave_2, autosave_3). Settings persist across saves via
//! the `Saveable` trait.
//!
//! The simulation crate owns the timer and configuration. When the timer
//! fires, `AutosavePending` is set, and the save crate's bridge system
//! converts it into a `SaveGameEvent` + file rotation.

use bevy::prelude::*;

use crate::notifications::{NotificationEvent, NotificationPriority};
use crate::Saveable;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Number of rotating autosave slots.
pub const AUTOSAVE_SLOT_COUNT: u8 = 3;

/// Default autosave interval in slow-tick cycles.
/// Each slow tick is ~100 fixed-update ticks (~10 seconds at 10 Hz).
/// 30 slow cycles = ~300 seconds = 5 minutes of game time.
pub const DEFAULT_INTERVAL_SLOW_TICKS: u32 = 30;

// =============================================================================
// Resources
// =============================================================================

/// Player-configurable autosave settings.
#[derive(Resource, Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct AutosaveConfig {
    /// Whether autosave is enabled.
    pub enabled: bool,
    /// Interval in slow-tick cycles between autosaves.
    /// Each slow-tick cycle is `SlowTickTimer::INTERVAL` fixed-update ticks.
    pub interval_slow_ticks: u32,
    /// The next slot index to write to (0, 1, or 2).
    pub current_slot: u8,
}

impl Default for AutosaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_slow_ticks: DEFAULT_INTERVAL_SLOW_TICKS,
            current_slot: 0,
        }
    }
}

impl AutosaveConfig {
    /// Returns the filename for the current autosave slot.
    pub fn current_slot_filename(&self) -> String {
        slot_filename(self.current_slot)
    }

    /// Advance to the next rotating slot.
    pub fn advance_slot(&mut self) {
        self.current_slot = (self.current_slot + 1) % AUTOSAVE_SLOT_COUNT;
    }

    /// Returns the approximate interval in seconds (assuming 10 Hz fixed update).
    pub fn interval_seconds(&self) -> f32 {
        self.interval_slow_ticks as f32 * SlowTickTimer::INTERVAL as f32 * 0.1
    }
}

/// Returns the save filename for a given slot index.
pub fn slot_filename(slot: u8) -> String {
    format!("megacity_autosave_{}.bin", slot + 1)
}

/// Internal timer that counts slow-tick cycles between autosaves.
#[derive(Resource, Default)]
pub struct AutosaveTimer {
    /// Number of slow-tick cycles since the last autosave.
    pub counter: u32,
}

/// Flag resource indicating that an autosave should be performed.
///
/// Set by the simulation-side `autosave_tick_system` when the timer fires.
/// Consumed by the save crate's bridge system which performs the actual save
/// and file rotation.
#[derive(Resource, Default)]
pub struct AutosavePending {
    /// When `true`, the save bridge should trigger a save and copy to the
    /// slot indicated by `AutosaveConfig::current_slot`.
    pub pending: bool,
}

// =============================================================================
// Saveable
// =============================================================================

impl Saveable for AutosaveConfig {
    const SAVE_KEY: &'static str = "autosave_config";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Ticks the autosave timer on each slow-tick cycle and sets `AutosavePending`
/// when it's time to save.
///
/// Runs in `SimulationSet::PostSim` so it only fires after all simulation
/// work for the current tick is complete.
pub fn autosave_tick_system(
    slow_timer: Res<SlowTickTimer>,
    config: Res<AutosaveConfig>,
    mut timer: ResMut<AutosaveTimer>,
    mut pending: ResMut<AutosavePending>,
) {
    // Only process on slow-tick boundaries.
    if !slow_timer.should_run() {
        return;
    }

    if !config.enabled {
        timer.counter = 0;
        return;
    }

    timer.counter += 1;

    if timer.counter >= config.interval_slow_ticks {
        timer.counter = 0;
        pending.pending = true;
    }
}

/// Sends a notification when an autosave is triggered.
///
/// Runs in `Update` so that the notification appears promptly. Checks
/// `AutosavePending` and emits a notification (the save bridge will
/// clear the pending flag after performing the save).
pub fn autosave_notification_system(
    pending: Res<AutosavePending>,
    config: Res<AutosaveConfig>,
    mut notifications: EventWriter<NotificationEvent>,
) {
    if pending.pending {
        notifications.send(NotificationEvent {
            text: format!("Autosaving to slot {}...", config.current_slot + 1),
            priority: NotificationPriority::Info,
            location: None,
        });
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct AutosavePlugin;

impl Plugin for AutosavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutosaveConfig>()
            .init_resource::<AutosaveTimer>()
            .init_resource::<AutosavePending>()
            .add_systems(
                FixedUpdate,
                autosave_tick_system.in_set(crate::SimulationSet::PostSim),
            )
            .add_systems(Update, autosave_notification_system);

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<AutosaveConfig>();
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AutosaveConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_slow_ticks, DEFAULT_INTERVAL_SLOW_TICKS);
        assert_eq!(config.current_slot, 0);
    }

    #[test]
    fn test_slot_filename() {
        assert_eq!(slot_filename(0), "megacity_autosave_1.bin");
        assert_eq!(slot_filename(1), "megacity_autosave_2.bin");
        assert_eq!(slot_filename(2), "megacity_autosave_3.bin");
    }

    #[test]
    fn test_advance_slot_wraps() {
        let mut config = AutosaveConfig::default();
        assert_eq!(config.current_slot, 0);
        config.advance_slot();
        assert_eq!(config.current_slot, 1);
        config.advance_slot();
        assert_eq!(config.current_slot, 2);
        config.advance_slot();
        assert_eq!(config.current_slot, 0); // wraps around
    }

    #[test]
    fn test_interval_seconds() {
        let config = AutosaveConfig::default();
        // 30 slow ticks * 100 fixed ticks * 0.1 sec/tick = 300 seconds
        let expected = 300.0_f32;
        assert!((config.interval_seconds() - expected).abs() < 0.1);
    }

    #[test]
    fn test_current_slot_filename() {
        let config = AutosaveConfig {
            current_slot: 1,
            ..Default::default()
        };
        assert_eq!(config.current_slot_filename(), "megacity_autosave_2.bin");
    }

    #[test]
    fn test_saveable_roundtrip() {
        let config = AutosaveConfig {
            enabled: false,
            interval_slow_ticks: 60,
            current_slot: 2,
        };
        let bytes = config.save_to_bytes().unwrap();
        let loaded = AutosaveConfig::load_from_bytes(&bytes);
        assert!(!loaded.enabled);
        assert_eq!(loaded.interval_slow_ticks, 60);
        assert_eq!(loaded.current_slot, 2);
    }
}
