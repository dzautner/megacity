//! Integration tests for AchievementTracker save/load (issue #724).
//!
//! Verifies that unlocked achievements, progress counters, and state flags
//! persist correctly across save/load cycles.

use crate::achievements::{Achievement, AchievementTracker};
use crate::SaveableRegistry;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helper: round-trip an AchievementTracker through the SaveableRegistry
// ---------------------------------------------------------------------------

fn round_trip_tracker(tracker: &AchievementTracker) -> AchievementTracker {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();
    app.insert_resource(tracker.clone());

    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<AchievementTracker>();
    }

    // Save
    let extensions: BTreeMap<String, Vec<u8>> = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };

    // Reset to default
    {
        let registry_entries: Vec<_> = app
            .world()
            .resource::<SaveableRegistry>()
            .entries
            .iter()
            .map(|e| e.key.clone())
            .collect();
        let _ = registry_entries; // just to prove registry exists
        app.insert_resource(AchievementTracker::default());
    }

    // Load
    {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.load_all(app.world_mut(), &extensions);
    }

    app.world().resource::<AchievementTracker>().clone()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Unlocked achievements persist across save/load.
#[test]
fn test_achievement_save_unlocked_achievements_persist() {
    let mut tracker = AchievementTracker::default();
    tracker.unlock(Achievement::Population1K, 100);
    tracker.unlock(Achievement::Millionaire, 250);
    tracker.unlock(Achievement::RoadDiversity, 500);

    let restored = round_trip_tracker(&tracker);

    assert!(restored.is_unlocked(Achievement::Population1K));
    assert!(restored.is_unlocked(Achievement::Millionaire));
    assert!(restored.is_unlocked(Achievement::RoadDiversity));
    assert_eq!(restored.unlocked_count(), 3);
}

/// Unlock tick is preserved across save/load.
#[test]
fn test_achievement_save_unlock_tick_preserved() {
    let mut tracker = AchievementTracker::default();
    tracker.unlock(Achievement::HappyCity, 42);
    tracker.unlock(Achievement::FullEmployment, 999);

    let restored = round_trip_tracker(&tracker);

    assert_eq!(restored.unlocked.get(&Achievement::HappyCity), Some(&42));
    assert_eq!(
        restored.unlocked.get(&Achievement::FullEmployment),
        Some(&999)
    );
}

/// Progress counters (positive_trade_ticks) persist across save/load.
#[test]
fn test_achievement_save_progress_counters_preserved() {
    let mut tracker = AchievementTracker::default();
    tracker.positive_trade_ticks = 73;

    let restored = round_trip_tracker(&tracker);

    assert_eq!(restored.positive_trade_ticks, 73);
}

/// State flags (had_active_disaster) persist across save/load.
#[test]
fn test_achievement_save_state_flags_preserved() {
    let mut tracker = AchievementTracker::default();
    tracker.had_active_disaster = true;

    let restored = round_trip_tracker(&tracker);

    assert!(restored.had_active_disaster);
}

/// Default tracker (no achievements) produces None from save_to_bytes.
#[test]
fn test_achievement_save_default_returns_none() {
    use crate::Saveable;
    let tracker = AchievementTracker::default();
    assert!(tracker.save_to_bytes().is_none());
}

/// A tracker with only progress (no unlocks) still saves and restores.
#[test]
fn test_achievement_save_progress_only_round_trips() {
    let mut tracker = AchievementTracker::default();
    tracker.positive_trade_ticks = 50;
    tracker.had_active_disaster = false;

    let restored = round_trip_tracker(&tracker);

    assert_eq!(restored.positive_trade_ticks, 50);
    assert!(!restored.had_active_disaster);
    assert_eq!(restored.unlocked_count(), 0);
}

/// All achievements can be unlocked and round-tripped.
#[test]
fn test_achievement_save_all_achievements_round_trip() {
    let mut tracker = AchievementTracker::default();
    for (i, &achievement) in Achievement::ALL.iter().enumerate() {
        tracker.unlock(achievement, i as u64 * 10);
    }
    tracker.positive_trade_ticks = 100;
    tracker.had_active_disaster = true;

    let restored = round_trip_tracker(&tracker);

    assert_eq!(restored.unlocked_count(), Achievement::total_count());
    for &achievement in Achievement::ALL {
        assert!(
            restored.is_unlocked(achievement),
            "Achievement {:?} should be unlocked after round-trip",
            achievement
        );
    }
    assert_eq!(restored.positive_trade_ticks, 100);
    assert!(restored.had_active_disaster);
}

/// Loading save A, then save B (without achievements), then save A again
/// restores achievements correctly.
#[test]
fn test_achievement_save_sequential_load_a_b_a() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();
    app.init_resource::<AchievementTracker>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<AchievementTracker>();
    }

    // Build save A: has unlocked achievements
    {
        let mut tracker = app.world_mut().resource_mut::<AchievementTracker>();
        tracker.unlock(Achievement::Population10K, 300);
        tracker.unlock(Achievement::EuphoricCity, 600);
        tracker.positive_trade_ticks = 88;
    }
    let save_a = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };

    // Build save B: empty (default)
    {
        app.insert_resource(AchievementTracker::default());
    }
    let save_b = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };

    // Load save B (empty)
    {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.load_all(app.world_mut(), &save_b);
    }
    {
        let tracker = app.world().resource::<AchievementTracker>();
        assert_eq!(tracker.unlocked_count(), 0, "Save B should have no achievements");
        assert_eq!(tracker.positive_trade_ticks, 0);
    }

    // Load save A again
    {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.load_all(app.world_mut(), &save_a);
    }
    {
        let tracker = app.world().resource::<AchievementTracker>();
        assert!(
            tracker.is_unlocked(Achievement::Population10K),
            "Population10K should be restored from save A"
        );
        assert!(
            tracker.is_unlocked(Achievement::EuphoricCity),
            "EuphoricCity should be restored from save A"
        );
        assert_eq!(tracker.positive_trade_ticks, 88);
        assert_eq!(tracker.unlocked_count(), 2);
    }
}
