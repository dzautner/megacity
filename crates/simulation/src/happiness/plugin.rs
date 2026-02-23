use bevy::prelude::*;

use super::coverage::{update_service_coverage, ServiceCoverageGrid};
use super::systems::update_happiness;

pub struct HappinessPlugin;

impl Plugin for HappinessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServiceCoverageGrid>().add_systems(
            FixedUpdate,
            (update_service_coverage, update_happiness)
                .chain()
                .after(crate::postal::update_postal_coverage)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
