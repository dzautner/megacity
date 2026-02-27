//! Plugin that registers the city observation system.
//!
//! Adds `CurrentObservation` resource and the `build_observation` system to
//! `FixedUpdate` in `SimulationSet::PostSim`.

use bevy::prelude::*;

use crate::observation_builder::{build_observation, CurrentObservation};
use crate::SimulationSet;

pub struct ObservationPlugin;

impl Plugin for ObservationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentObservation>();
        app.add_systems(
            FixedUpdate,
            build_observation.in_set(SimulationSet::PostSim),
        );
    }
}
