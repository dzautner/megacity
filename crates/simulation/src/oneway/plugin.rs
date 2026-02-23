use bevy::prelude::*;

use super::systems::{handle_toggle_oneway, rebuild_csr_with_oneway};
use super::types::{OneWayDirectionMap, ToggleOneWayEvent};

pub struct OneWayPlugin;

impl Plugin for OneWayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OneWayDirectionMap>()
            .add_event::<ToggleOneWayEvent>()
            .add_systems(
                Update,
                handle_toggle_oneway.in_set(crate::SimulationUpdateSet::Input),
            )
            .add_systems(
                Update,
                rebuild_csr_with_oneway
                    .after(handle_toggle_oneway)
                    .in_set(crate::SimulationUpdateSet::Input),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<OneWayDirectionMap>();
    }
}
