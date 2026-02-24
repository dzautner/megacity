//! Integration tests for SAVE-013: Rotating Autosave Slots.
//!
//! Tests the `interval_minutes` config, slot rotation across multiple
//! autosave cycles, `AutosaveLastSaveTime` tracking, and clamping behavior.

use crate::autosave::{
    AutosaveConfig, AutosaveLastSaveTime, AutosavePending, AutosaveTimer,
    DEFAULT_INTERVAL_MINUTES, MAX_INTERVAL_MINUTES, MIN_INTERVAL_MINUTES,
};
use crate::test_harness::TestCity;

// =============================================================================
// interval_minutes integration tests
// =============================================================================

#[test]
fn test_autosave_default_interval_minutes() {
    let city = TestCity::new();
    let config = city.resource::<AutosaveConfig>();
    assert_eq!(
        config.interval_minutes, DEFAULT_INTERVAL_MINUTES,
        "Default interval_minutes should be {}",
        DEFAULT_INTERVAL_MINUTES
    );
}

#[test]
fn test_autosave_set_interval_minutes_updates_slow_ticks() {
    let mut city = TestCity::new();

    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .set_interval_minutes(10.0);

    let config = city.resource::<AutosaveConfig>();
    assert_eq!(config.interval_minutes, 10.0);
    // 10 min = 600 sec, each slow tick = 10 sec => 60 ticks
    assert_eq!(config.interval_slow_ticks, 60);
}

#[test]
fn test_autosave_set_interval_clamps_below_minimum() {
    let mut city = TestCity::new();

    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .set_interval_minutes(0.1);

    let config = city.resource::<AutosaveConfig>();
    assert_eq!(
        config.interval_minutes, MIN_INTERVAL_MINUTES,
        "Interval should be clamped to minimum"
    );
}

#[test]
fn test_autosave_set_interval_clamps_above_maximum() {
    let mut city = TestCity::new();

    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .set_interval_minutes(99.0);

    let config = city.resource::<AutosaveConfig>();
    assert_eq!(
        config.interval_minutes, MAX_INTERVAL_MINUTES,
        "Interval should be clamped to maximum"
    );
}

#[test]
fn test_autosave_short_interval_triggers_quickly() {
    let mut city = TestCity::new();

    // Set 1-minute interval (6 slow ticks).
    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .set_interval_minutes(1.0);

    let interval = city.resource::<AutosaveConfig>().interval_slow_ticks;
    assert_eq!(interval, 6, "1 minute should be 6 slow ticks");

    city.tick_slow_cycles(interval);

    let pending = city.resource::<AutosavePending>();
    assert!(
        pending.pending,
        "Autosave should trigger after 1-minute interval"
    );
}

// =============================================================================
// Rotating slot cycling over multiple autosave rounds
// =============================================================================

#[test]
fn test_autosave_full_rotation_cycle() {
    let mut city = TestCity::new();

    // Use a short interval for fast testing.
    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .interval_slow_ticks = 2;

    // Cycle through all 3 slots and verify filenames.
    let expected_files = [
        "megacity_autosave_1.bin",
        "megacity_autosave_2.bin",
        "megacity_autosave_3.bin",
    ];

    for (i, expected_file) in expected_files.iter().enumerate() {
        let filename = city.resource::<AutosaveConfig>().current_slot_filename();
        assert_eq!(
            &filename, expected_file,
            "Slot {} should produce filename {}",
            i, expected_file
        );

        // Trigger autosave.
        city.tick_slow_cycles(2);
        assert!(
            city.resource::<AutosavePending>().pending,
            "Autosave round {} should trigger pending",
            i
        );

        // Simulate the save bridge: clear pending and advance slot.
        city.world_mut().resource_mut::<AutosavePending>().pending = false;
        city.world_mut()
            .resource_mut::<AutosaveConfig>()
            .advance_slot();
    }

    // After 3 advances, slot should wrap back to 0.
    let config = city.resource::<AutosaveConfig>();
    assert_eq!(
        config.current_slot, 0,
        "Slot should wrap back to 0 after full rotation"
    );
    assert_eq!(
        config.current_slot_filename(),
        "megacity_autosave_1.bin",
        "Wrapped slot filename should match slot 1"
    );
}

// =============================================================================
// Two-good-autosaves invariant
// =============================================================================

#[test]
fn test_two_good_autosaves_invariant() {
    // With 3 rotating slots, at any point after the first 2 autosaves,
    // the player has at least 2 older (completed) autosaves while the
    // newest one is being written. This test verifies the slot indices
    // cycle correctly to ensure this property.
    let mut config = AutosaveConfig::default();

    // Simulate 6 autosave rounds and track which slots were written.
    let mut written_slots: Vec<u8> = Vec::new();

    for _ in 0..6 {
        written_slots.push(config.current_slot);
        config.advance_slot();
    }

    // Verify the pattern: 0, 1, 2, 0, 1, 2
    assert_eq!(written_slots, vec![0, 1, 2, 0, 1, 2]);

    // At round 3 (writing slot 0), slots 1 and 2 are intact.
    // At round 4 (writing slot 1), slots 2 and 0 are intact.
    // At round 5 (writing slot 2), slots 0 and 1 are intact.
    // This confirms the 2-good-autosaves invariant.
}

// =============================================================================
// AutosaveLastSaveTime tracking
// =============================================================================

#[test]
fn test_last_save_time_initially_none() {
    let city = TestCity::new();
    let last = city.resource::<AutosaveLastSaveTime>();
    assert!(
        last.elapsed_secs.is_none(),
        "Last save time should be None before any autosave"
    );
}

#[test]
fn test_last_save_time_set_after_autosave() {
    let mut city = TestCity::new();

    // Set short interval.
    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .interval_slow_ticks = 2;

    city.tick_slow_cycles(2);

    let last = city.resource::<AutosaveLastSaveTime>();
    assert!(
        last.elapsed_secs.is_some(),
        "Last save time should be recorded after autosave triggers"
    );
}

// =============================================================================
// Slot count and filenames
// =============================================================================

#[test]
fn test_slot_count_returns_three() {
    let city = TestCity::new();
    let config = city.resource::<AutosaveConfig>();
    assert_eq!(config.slot_count(), 3, "Should have 3 rotating autosave slots");
}

#[test]
fn test_all_slot_filenames_returns_three_unique() {
    let config = AutosaveConfig::default();
    let filenames = config.all_slot_filenames();
    assert_eq!(filenames.len(), 3);

    // Verify uniqueness.
    let mut unique = filenames.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        3,
        "All 3 autosave slot filenames must be unique"
    );
}

// =============================================================================
// Enable/disable with interval_minutes
// =============================================================================

#[test]
fn test_autosave_re_enable_with_custom_interval() {
    let mut city = TestCity::new();

    // Disable, change interval, re-enable.
    {
        let mut config = city.world_mut().resource_mut::<AutosaveConfig>();
        config.enabled = false;
        config.set_interval_minutes(2.0);
    }

    // Run some cycles while disabled — should not trigger.
    city.tick_slow_cycles(20);
    assert!(
        !city.resource::<AutosavePending>().pending,
        "Should not trigger while disabled"
    );

    // Re-enable.
    city.world_mut()
        .resource_mut::<AutosaveConfig>()
        .enabled = true;

    // 2 minutes = 12 slow ticks.
    let interval = city.resource::<AutosaveConfig>().interval_slow_ticks;
    assert_eq!(interval, 12);

    city.tick_slow_cycles(interval);
    assert!(
        city.resource::<AutosavePending>().pending,
        "Should trigger after re-enable and interval elapses"
    );
}

#[test]
fn test_autosave_timer_resource_exists() {
    let city = TestCity::new();
    let timer = city.resource::<AutosaveTimer>();
    assert_eq!(timer.counter, 0, "Timer should start at 0");
}
