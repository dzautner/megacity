//! Integration tests for the autosave system (SAVE-002).
//!
//! Tests verify that the autosave timer, slot rotation, and pending flag
//! behave correctly using the `TestCity` headless harness.

use crate::autosave::{AutosaveConfig, AutosavePending, AutosaveTimer, AUTOSAVE_SLOT_COUNT};
use crate::test_harness::TestCity;

// =============================================================================
// Timer and pending flag tests
// =============================================================================

#[test]
fn test_autosave_pending_set_after_interval() {
    let mut city = TestCity::new();

    // Default config: interval = 30 slow tick cycles.
    let interval = {
        let config = city.resource::<AutosaveConfig>();
        config.interval_slow_ticks
    };

    // Run enough slow cycles to trigger autosave.
    city.tick_slow_cycles(interval);

    let pending = city.resource::<AutosavePending>();
    assert!(
        pending.pending,
        "AutosavePending should be true after {} slow cycles",
        interval
    );
}

#[test]
fn test_autosave_not_pending_before_interval() {
    let mut city = TestCity::new();

    // Run fewer slow cycles than the interval.
    city.tick_slow_cycles(5);

    let pending = city.resource::<AutosavePending>();
    assert!(
        !pending.pending,
        "AutosavePending should be false before interval elapses"
    );
}

#[test]
fn test_autosave_disabled_does_not_trigger() {
    let mut city = TestCity::new();

    // Disable autosave.
    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .enabled = false;

    let interval = city.resource::<AutosaveConfig>().interval_slow_ticks;
    city.tick_slow_cycles(interval + 5);

    let pending = city.resource::<AutosavePending>();
    assert!(
        !pending.pending,
        "AutosavePending should be false when autosave is disabled"
    );
}

#[test]
fn test_autosave_timer_resets_after_trigger() {
    let mut city = TestCity::new();

    let interval = city.resource::<AutosaveConfig>().interval_slow_ticks;

    // Trigger once.
    city.tick_slow_cycles(interval);
    assert!(city.resource::<AutosavePending>().pending);

    // Clear the pending flag manually (normally the save bridge does this).
    city.world_mut().resource_mut::<AutosavePending>().pending = false;

    // Timer counter should have been reset. Run another interval to trigger again.
    city.tick_slow_cycles(interval);

    let pending = city.resource::<AutosavePending>();
    assert!(
        pending.pending,
        "AutosavePending should trigger again after another full interval"
    );
}

// =============================================================================
// Configuration tests
// =============================================================================

#[test]
fn test_autosave_config_persists_as_resource() {
    let city = TestCity::new();

    let config = city.resource::<AutosaveConfig>();
    assert!(config.enabled);
    assert_eq!(config.current_slot, 0);
}

#[test]
fn test_autosave_custom_interval() {
    let mut city = TestCity::new();

    // Set a short interval for testing.
    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .interval_slow_ticks = 3;

    city.tick_slow_cycles(3);

    let pending = city.resource::<AutosavePending>();
    assert!(
        pending.pending,
        "Custom interval should trigger autosave after 3 slow cycles"
    );
}

// =============================================================================
// Slot rotation tests
// =============================================================================

#[test]
fn test_slot_rotation_sequence() {
    let mut config = AutosaveConfig::default();
    assert_eq!(config.current_slot, 0);

    for expected_slot in [1, 2, 0, 1, 2, 0] {
        config.advance_slot();
        assert_eq!(
            config.current_slot, expected_slot,
            "Slot should cycle through 0..{}",
            AUTOSAVE_SLOT_COUNT
        );
    }
}

#[test]
fn test_slot_filenames_unique() {
    use std::collections::HashSet;
    let filenames: HashSet<String> = (0..AUTOSAVE_SLOT_COUNT)
        .map(crate::autosave::slot_filename)
        .collect();
    assert_eq!(
        filenames.len(),
        AUTOSAVE_SLOT_COUNT as usize,
        "All slot filenames should be unique"
    );
}

#[test]
fn test_autosave_timer_counter_increments() {
    let mut city = TestCity::new();

    city.tick_slow_cycles(5);

    let timer = city.resource::<AutosaveTimer>();
    assert_eq!(
        timer.counter, 5,
        "Timer counter should be 5 after 5 slow cycles"
    );
}

#[test]
fn test_disabled_autosave_resets_counter() {
    let mut city = TestCity::new();

    // Run a few cycles to build up the counter.
    city.tick_slow_cycles(5);
    assert_eq!(city.resource::<AutosaveTimer>().counter, 5);

    // Disable and run more cycles — counter should stay at 0.
    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .enabled = false;
    city.tick_slow_cycles(3);

    let timer = city.resource::<AutosaveTimer>();
    assert_eq!(
        timer.counter, 0,
        "Timer counter should reset to 0 when autosave is disabled"
    );
}
