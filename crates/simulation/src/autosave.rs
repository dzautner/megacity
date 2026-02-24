//! SAVE-013: Rotating Autosave Slots with Configurable Interval.
//!
//! Provides periodic autosave that saves game state without player action.
//! Configurable interval (default 5 minutes, range 1-30 min), with 3 rotating
//! save slots (`autosave_1.bin`, `autosave_2.bin`, `autosave_3.bin`). Settings
//! persist across saves via the `Saveable` trait.
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

/// Default autosave interval in minutes.
pub const DEFAULT_INTERVAL_MINUTES: f32 = 5.0;

/// Minimum configurable interval in minutes.
pub const MIN_INTERVAL_MINUTES: f32 = 1.0;

/// Maximum configurable interval in minutes.
pub const MAX_INTERVAL_MINUTES: f32 = 30.0;

/// Default autosave interval in slow-tick cycles (kept for backward compat).
/// Each slow tick is ~100 fixed-update ticks (~10 seconds at 10 Hz).
/// 30 slow cycles = ~300 seconds = 5 minutes of game time.
pub const DEFAULT_INTERVAL_SLOW_TICKS: u32 = 30;

// =============================================================================
// Helpers
// =============================================================================

/// Converts a minutes value to the equivalent number of slow-tick cycles.
///
/// Each slow tick cycle is `SlowTickTimer::INTERVAL` fixed-update ticks
/// at ~10 Hz, so each slow tick is approximately 10 seconds.
pub fn minutes_to_slow_ticks(minutes: f32) -> u32 {
    let seconds = minutes * 60.0;
    let seconds_per_slow_tick = SlowTickTimer::INTERVAL as f32 * 0.1;
    (seconds / seconds_per_slow_tick).round().max(1.0) as u32
}

// =============================================================================
// Resources
// =============================================================================

/// Player-configurable autosave settings.
///
/// Provides `interval_minutes` (1-30 min, clamped) as the primary user-facing
/// setting. The internal `interval_slow_ticks` is derived from it. The config
/// tracks which of the 3 rotating slots will be written next, ensuring the
/// player always has at least 2 intact autosaves if the latest is corrupted.
#[derive(Resource, Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct AutosaveConfig {
    /// Whether autosave is enabled.
    pub enabled: bool,
    /// User-facing interval in minutes (clamped to 1.0-30.0).
    pub interval_minutes: f32,
    /// Interval in slow-tick cycles between autosaves (derived from
    /// `interval_minutes`). Kept in sync by `set_interval_minutes()`.
    pub interval_slow_ticks: u32,
    /// The next slot index to write to (0, 1, or 2).
    pub current_slot: u8,
}

impl Default for AutosaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: DEFAULT_INTERVAL_MINUTES,
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

    /// Sets the autosave interval in minutes, clamped to the valid range
    /// (`MIN_INTERVAL_MINUTES`..=`MAX_INTERVAL_MINUTES`), and updates the
    /// derived `interval_slow_ticks` field.
    pub fn set_interval_minutes(&mut self, minutes: f32) {
        self.interval_minutes = minutes.clamp(MIN_INTERVAL_MINUTES, MAX_INTERVAL_MINUTES);
        self.interval_slow_ticks = minutes_to_slow_ticks(self.interval_minutes);
    }

    /// Returns the total number of rotating autosave slots.
    pub fn slot_count(&self) -> u8 {
        AUTOSAVE_SLOT_COUNT
    }

    /// Returns filenames for all autosave slots.
    pub fn all_slot_filenames(&self) -> Vec<String> {
        (0..AUTOSAVE_SLOT_COUNT).map(slot_filename).collect()
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

/// Tracks the elapsed game time of the last autosave.
///
/// This is a runtime-only resource (not serialized) that records the
/// `Time::elapsed_secs_f64()` at the moment an autosave is triggered.
/// Useful for UI display (e.g., "last autosave: 2 minutes ago").
#[derive(Resource, Default)]
pub struct AutosaveLastSaveTime {
    /// Elapsed game time (seconds) when the last autosave was triggered,
    /// or `None` if no autosave has occurred this session.
    pub elapsed_secs: Option<f64>,
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
    mut last_save_time: ResMut<AutosaveLastSaveTime>,
    time: Res<Time>,
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
        last_save_time.elapsed_secs = Some(time.elapsed_secs_f64());
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
            .init_resource::<AutosaveLastSaveTime>()
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
        assert_eq!(config.interval_minutes, DEFAULT_INTERVAL_MINUTES);
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
            interval_minutes: 10.0,
            interval_slow_ticks: 60,
            current_slot: 2,
        };
        let bytes = config.save_to_bytes().unwrap();
        let loaded = AutosaveConfig::load_from_bytes(&bytes);
        assert!(!loaded.enabled);
        assert_eq!(loaded.interval_minutes, 10.0);
        assert_eq!(loaded.interval_slow_ticks, 60);
        assert_eq!(loaded.current_slot, 2);
    }

    #[test]
    fn test_set_interval_minutes_clamps_low() {
        let mut config = AutosaveConfig::default();
        config.set_interval_minutes(0.5);
        assert_eq!(config.interval_minutes, MIN_INTERVAL_MINUTES);
        assert_eq!(
            config.interval_slow_ticks,
            minutes_to_slow_ticks(MIN_INTERVAL_MINUTES)
        );
    }

    #[test]
    fn test_set_interval_minutes_clamps_high() {
        let mut config = AutosaveConfig::default();
        config.set_interval_minutes(60.0);
        assert_eq!(config.interval_minutes, MAX_INTERVAL_MINUTES);
        assert_eq!(
            config.interval_slow_ticks,
            minutes_to_slow_ticks(MAX_INTERVAL_MINUTES)
        );
    }

    #[test]
    fn test_set_interval_minutes_valid() {
        let mut config = AutosaveConfig::default();
        config.set_interval_minutes(10.0);
        assert_eq!(config.interval_minutes, 10.0);
        assert_eq!(config.interval_slow_ticks, minutes_to_slow_ticks(10.0));
    }

    #[test]
    fn test_minutes_to_slow_ticks() {
        // 5 minutes = 300 seconds, each slow tick = 10 seconds => 30 ticks
        assert_eq!(minutes_to_slow_ticks(5.0), 30);
        // 1 minute = 60 seconds => 6 ticks
        assert_eq!(minutes_to_slow_ticks(1.0), 6);
        // 30 minutes = 1800 seconds => 180 ticks
        assert_eq!(minutes_to_slow_ticks(30.0), 180);
    }

    #[test]
    fn test_slot_count() {
        let config = AutosaveConfig::default();
        assert_eq!(config.slot_count(), 3);
    }

    #[test]
    fn test_all_slot_filenames() {
        let config = AutosaveConfig::default();
        let filenames = config.all_slot_filenames();
        assert_eq!(filenames.len(), 3);
        assert_eq!(filenames[0], "megacity_autosave_1.bin");
        assert_eq!(filenames[1], "megacity_autosave_2.bin");
        assert_eq!(filenames[2], "megacity_autosave_3.bin");
    }

    #[test]
    fn test_last_save_time_default() {
        let last = AutosaveLastSaveTime::default();
        assert!(last.elapsed_secs.is_none());
    }
}
