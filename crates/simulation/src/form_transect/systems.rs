//! ECS systems and plugin for the form-based transect overlay.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::SlowTickTimer;

use super::{max_level_for_transect, TransectGrid, TransectZone};

// =============================================================================
// System: enforce transect constraints on existing buildings
// =============================================================================

/// Periodically checks all buildings and caps their level to respect the
/// transect overlay's FAR and stories constraints.
///
/// Runs on the slow tick. Buildings above the transect limit are downgraded
/// (level capped, capacity adjusted, excess occupants evicted).
pub fn enforce_transect_constraints(
    timer: Res<SlowTickTimer>,
    transect_grid: Res<TransectGrid>,
    mut buildings: Query<&mut Building>,
) {
    if !timer.should_run() {
        return;
    }

    for mut building in &mut buildings {
        let transect = transect_grid.get(building.grid_x, building.grid_y);

        // T1Natural: buildings shouldn't exist here, but we don't despawn --
        // that's handled by the spawner refusing to place new buildings.
        // Just prevent growth.
        if transect == TransectZone::T1Natural {
            continue;
        }

        // None: unconstrained
        if transect == TransectZone::None {
            continue;
        }

        let max_level = max_level_for_transect(transect, building.zone_type);
        if building.level > max_level {
            building.level = max_level;
            building.capacity = Building::capacity_for_level(building.zone_type, building.level);
            if building.occupants > building.capacity {
                building.occupants = building.capacity;
            }
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct FormTransectPlugin;

impl Plugin for FormTransectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransectGrid>().add_systems(
            FixedUpdate,
            // Order-independent: only caps Building levels via Query<&mut Building>;
            // no shared grid resource writes.
            enforce_transect_constraints.in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<TransectGrid>();
    }
}
