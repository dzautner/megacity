use bevy::prelude::*;

use super::systems::update_wind;
use super::types::WindState;

pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindState>().add_systems(
            FixedUpdate,
            update_wind
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
