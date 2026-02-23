use bevy::prelude::*;

use super::construction::progress_construction;
use super::spawning::{
    building_spawner, rebuild_eligible_cells, BuildingSpawnTimer, EligibleCells,
};

pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildingSpawnTimer>()
            .init_resource::<EligibleCells>()
            .add_systems(
                FixedUpdate,
                (
                    rebuild_eligible_cells,
                    building_spawner,
                    progress_construction,
                )
                    .chain()
                    .after(crate::zones::update_zone_demand)
                    .in_set(crate::SimulationSet::PreSim),
            );
    }
}
