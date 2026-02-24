//! SAVE-008: Reset commuting citizens to AtHome on load.
//!
//! When a save file is loaded, citizens in commuting states may have empty or
//! stale `PathCache` data and zero `Velocity`, causing them to freeze in place.
//! This module provides a one-shot post-load system that detects commuting
//! citizens and resets them to `AtHome` with their home position, allowing the
//! movement state machine to naturally re-dispatch them.

use bevy::prelude::*;

use crate::citizen::{
    Citizen, CitizenState, CitizenStateComp, HomeLocation, PathCache, Position, Velocity,
};
use crate::grid::WorldGrid;
use crate::movement::ActivityTimer;

/// Marker resource inserted when a load completes, signalling the reset system
/// to scan and fix commuting citizens on the next `FixedUpdate` tick.
#[derive(Resource, Default)]
pub struct PostLoadResetPending;

pub struct ResetCommutingOnLoadPlugin;

impl Plugin for ResetCommutingOnLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            reset_commuting_citizens_after_load
                .run_if(resource_exists::<PostLoadResetPending>)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

/// One-shot system that resets all commuting citizens to `AtHome`.
///
/// For each citizen whose state is a commuting variant (`CommutingToWork`,
/// `CommutingHome`, `CommutingToShop`, `CommutingToLeisure`,
/// `CommutingToSchool`):
/// - State is set to `AtHome`
/// - Position is moved to the citizen's home coordinates
/// - `PathCache` is cleared
/// - `Velocity` is zeroed
/// - `ActivityTimer` is reset
///
/// After processing, the `PostLoadResetPending` resource is removed so this
/// system does not run again until the next load.
#[allow(clippy::type_complexity)]
fn reset_commuting_citizens_after_load(
    mut commands: Commands,
    mut query: Query<
        (
            &mut CitizenStateComp,
            &mut Position,
            &mut Velocity,
            &mut PathCache,
            &HomeLocation,
            &mut ActivityTimer,
        ),
        With<Citizen>,
    >,
) {
    let mut reset_count: u32 = 0;

    for (mut state, mut pos, mut vel, mut path, home, mut timer) in &mut query {
        if !state.0.is_commuting() {
            continue;
        }

        // Only reset if the path is empty or already completed (stale).
        // Citizens with a valid in-progress path are left alone — the
        // movement system can handle them normally.
        if !path.is_complete() && !path.waypoints.is_empty() {
            continue;
        }

        // Reset to AtHome
        state.0 = CitizenState::AtHome;

        // Move position to home coordinates
        let (hx, hy) = WorldGrid::grid_to_world(home.grid_x, home.grid_y);
        pos.x = hx;
        pos.y = hy;

        // Clear path and velocity
        *path = PathCache::new(Vec::new());
        vel.x = 0.0;
        vel.y = 0.0;
        timer.0 = 0;

        reset_count += 1;
    }

    if reset_count > 0 {
        info!(
            "Post-load reset: moved {} commuting citizen(s) back to AtHome",
            reset_count
        );
    }

    // Remove the flag so this system doesn't run again.
    commands.remove_resource::<PostLoadResetPending>();
}
