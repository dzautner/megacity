//! Superblock ECS systems and plugin registration.

use bevy::prelude::*;

use crate::SlowTickTimer;

use super::state::SuperblockState;

/// System that periodically recomputes superblock coverage statistics.
/// The grid itself is rebuilt on add/remove, but statistics are recalculated
/// on the slow tick in case the grid dimensions or superblock definitions
/// change through save/load.
pub fn update_superblock_stats(timer: Res<SlowTickTimer>, mut state: ResMut<SuperblockState>) {
    if !timer.should_run() {
        return;
    }

    // Rebuild grid (idempotent — ensures consistency after save/load)
    state.rebuild_grid();
}

// =============================================================================
// Plugin
// =============================================================================

pub struct SuperblockPlugin;

impl Plugin for SuperblockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuperblockState>().add_systems(
            FixedUpdate,
            update_superblock_stats
                .after(crate::districts::district_stats)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<SuperblockState>();
    }
}
