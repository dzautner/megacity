//! Stormwater management aggregate state and plugin.

use bevy::prelude::*;

use crate::Saveable;

/// Aggregate stormwater management statistics.
///
/// Tracks green infrastructure effects, flood road damage, citizen displacement,
/// and flood risk overlay statistics.
#[derive(Resource, Debug, Clone, Default, bitcode::Encode, bitcode::Decode)]
pub struct StormwaterMgmtState {
    /// Total stormwater runoff absorbed by green infrastructure this tick.
    pub green_infra_absorbed: f32,
    /// Number of road cells damaged by flooding this tick.
    pub flood_damaged_roads: u32,
    /// Number of citizens affected by flooding (in flooded buildings).
    pub displaced_citizens: u32,
    /// Average flood risk score across the grid (0.0 to 255.0).
    pub avg_flood_risk: f32,
    /// Number of cells with high flood risk (risk > 180).
    pub high_risk_cells: u32,
}

impl Saveable for StormwaterMgmtState {
    const SAVE_KEY: &'static str = "stormwater_mgmt";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.green_infra_absorbed == 0.0
            && self.flood_damaged_roads == 0
            && self.displaced_citizens == 0
            && self.avg_flood_risk == 0.0
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

/// Plugin that registers the stormwater management system.
pub struct StormwaterMgmtPlugin;

impl Plugin for StormwaterMgmtPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StormwaterMgmtState>();
        app.init_resource::<super::flood_risk::FloodRiskGrid>();

        app.add_systems(
            FixedUpdate,
            super::systems::update_stormwater_mgmt
                .after(crate::flood_simulation::update_flood_simulation)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<StormwaterMgmtState>();
    }
}
