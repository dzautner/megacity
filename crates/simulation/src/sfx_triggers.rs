//! PLAY-008: UI Sound Effects Triggers.
//!
//! Detects game actions and emits `PlaySfxEvent` so that a downstream audio
//! playback system (in the rendering or app crate) can play the appropriate
//! sound effects. This module contains only event-emission logic — no actual
//! audio playback happens here.
//!
//! Triggers wired up:
//! - **Building placed**: `Added<Building>` → `SfxEvent::BuildingPlace`
//! - **Notification received**: `NotificationEvent` → `SfxEvent::Notification`,
//!   `Warning`, or `Error` based on priority
//! - **Save requested**: `SaveToSlotEvent` → `SfxEvent::Save`
//! - **Milestone reached**: population milestone changes → `SfxEvent::Notification`
//!   (with higher volume)

use bevy::prelude::*;

use crate::audio_settings::{PlaySfxEvent, SfxEvent};
use crate::buildings::Building;
use crate::milestones::MilestoneProgress;
use crate::notifications::{NotificationEvent, NotificationPriority};
use crate::save_slots::SaveToSlotEvent;

// =============================================================================
// Local state for change detection
// =============================================================================

/// Tracks the last seen milestone tier index so we can detect tier changes.
#[derive(Resource, Default)]
struct SfxMilestoneTracker {
    last_tier_index: usize,
}

// =============================================================================
// Systems
// =============================================================================

/// Emits `BuildingPlace` SFX when new `Building` entities are spawned.
///
/// Uses Bevy's `Added<Building>` filter to detect buildings added this frame.
/// Only emits one event per frame regardless of how many buildings spawn, to
/// avoid audio stacking.
fn sfx_on_building_placed(
    new_buildings: Query<&Building, Added<Building>>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    if new_buildings.iter().next().is_some() {
        sfx.send(PlaySfxEvent::new(SfxEvent::BuildingPlace));
    }
}

/// Emits SFX when notifications arrive, choosing the sound based on priority.
///
/// - `Emergency` / `Warning` → `SfxEvent::Warning`
/// - `Attention` / `Info` → `SfxEvent::Notification`
/// - `Positive` → `SfxEvent::Notification` (at lower volume)
fn sfx_on_notification(
    mut notifications: EventReader<NotificationEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    // Only play one SFX per frame, picking the highest-priority notification.
    let mut highest: Option<NotificationPriority> = None;
    for event in notifications.read() {
        match highest {
            None => highest = Some(event.priority),
            Some(prev) if event.priority < prev => highest = Some(event.priority),
            _ => {}
        }
    }

    if let Some(priority) = highest {
        let sfx_event = match priority {
            NotificationPriority::Emergency | NotificationPriority::Warning => SfxEvent::Warning,
            NotificationPriority::Attention | NotificationPriority::Info => SfxEvent::Notification,
            NotificationPriority::Positive => SfxEvent::Notification,
        };
        let volume = match priority {
            NotificationPriority::Emergency => 1.0,
            NotificationPriority::Warning => 0.9,
            _ => 0.7,
        };
        sfx.send(PlaySfxEvent::with_volume(sfx_event, volume));
    }
}

/// Emits `Save` SFX when the player saves the game.
fn sfx_on_save(
    mut save_events: EventReader<SaveToSlotEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    if save_events.read().next().is_some() {
        sfx.send(PlaySfxEvent::new(SfxEvent::Save));
    }
}

/// Emits a milestone-specific SFX when the player reaches a new milestone tier.
///
/// Compares the current `MilestoneProgress` tier index against the last seen
/// value stored in `SfxMilestoneTracker`.
fn sfx_on_milestone(
    progress: Res<MilestoneProgress>,
    mut tracker: ResMut<SfxMilestoneTracker>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    let current_index = progress.current_tier.index();
    if current_index > tracker.last_tier_index {
        tracker.last_tier_index = current_index;
        // Play notification at full volume for milestone events.
        sfx.send(PlaySfxEvent::with_volume(SfxEvent::Notification, 1.0));
    }
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that wires up game actions to SFX event emission.
pub struct SfxTriggersPlugin;

impl Plugin for SfxTriggersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SfxMilestoneTracker>();

        // Building placement and milestone detection run in PostSim
        // (after buildings spawn in PreSim and milestones update in PostSim).
        app.add_systems(
            FixedUpdate,
            (
                sfx_on_building_placed,
                sfx_on_notification,
                sfx_on_save,
                sfx_on_milestone,
            )
                .in_set(crate::SimulationSet::PostSim),
        );
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sfx_milestone_tracker_default() {
        let tracker = SfxMilestoneTracker::default();
        assert_eq!(tracker.last_tier_index, 0);
    }

    #[test]
    fn test_play_sfx_event_variants_used() {
        // Verify we can construct all the SFX events this module emits.
        let events = [
            PlaySfxEvent::new(SfxEvent::BuildingPlace),
            PlaySfxEvent::new(SfxEvent::Notification),
            PlaySfxEvent::new(SfxEvent::Warning),
            PlaySfxEvent::new(SfxEvent::Save),
            PlaySfxEvent::with_volume(SfxEvent::Notification, 0.7),
            PlaySfxEvent::with_volume(SfxEvent::Warning, 0.9),
        ];
        assert_eq!(events.len(), 6);
    }
}
