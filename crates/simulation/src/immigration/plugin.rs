use bevy::prelude::*;

use super::attractiveness::compute_attractiveness;
use super::types::{CityAttractiveness, ImmigrationStats};
use super::waves::immigration_wave;

pub struct ImmigrationPlugin;

impl Plugin for ImmigrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityAttractiveness>()
            .init_resource::<ImmigrationStats>()
            .add_systems(
                FixedUpdate,
                (compute_attractiveness, immigration_wave)
                    .chain()
                    .after(crate::stats::update_stats)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
