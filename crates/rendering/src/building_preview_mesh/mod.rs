//! Building Preview Meshes (UX-016)
//!
//! Replaces the generic cuboid cursor preview with zone-type-specific
//! procedural meshes that give the player a visual hint of what will be
//! built. Each zone type gets a distinct silhouette:
//!
//! - **Residential Low**: compact house with pitched roof
//! - **Residential Medium**: taller townhouse/duplex
//! - **Residential High**: tall apartment tower
//! - **Commercial Low**: medium shop building
//! - **Commercial High**: tall commercial skyscraper
//! - **Industrial**: wide, low warehouse/factory
//! - **Office**: tall glass tower
//! - **MixedUse**: medium multi-story building
//!
//! Preview meshes are cached in a resource so they are only generated once.

mod generators;
mod mesh_data;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use std::collections::HashMap;

use simulation::config::CELL_SIZE;
use simulation::grid::ZoneType;

use generators::*;

// ---------------------------------------------------------------------------
// Resource: cached preview mesh handles per zone type
// ---------------------------------------------------------------------------

/// Holds pre-built procedural mesh handles for each zone type preview.
#[derive(Resource)]
pub struct BuildingPreviewMeshes {
    meshes: HashMap<ZoneType, Handle<Mesh>>,
    /// Fallback flat cuboid for non-zone tools (road, bulldoze, etc.)
    pub flat_cuboid: Handle<Mesh>,
}

impl BuildingPreviewMeshes {
    /// Get the preview mesh for a given zone type. Falls back to the flat
    /// cuboid if no specific mesh is registered (e.g. `ZoneType::None`).
    pub fn get(&self, zone: ZoneType) -> Handle<Mesh> {
        self.meshes
            .get(&zone)
            .cloned()
            .unwrap_or_else(|| self.flat_cuboid.clone())
    }
}

// ---------------------------------------------------------------------------
// Startup system
// ---------------------------------------------------------------------------

/// Generates and caches all zone-type preview meshes at startup.
pub fn setup_building_preview_meshes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut map = HashMap::new();

    map.insert(
        ZoneType::ResidentialLow,
        meshes.add(generate_residential_low()),
    );
    map.insert(
        ZoneType::ResidentialMedium,
        meshes.add(generate_residential_medium()),
    );
    map.insert(
        ZoneType::ResidentialHigh,
        meshes.add(generate_residential_high()),
    );
    map.insert(
        ZoneType::CommercialLow,
        meshes.add(generate_commercial_low()),
    );
    map.insert(
        ZoneType::CommercialHigh,
        meshes.add(generate_commercial_high()),
    );
    map.insert(ZoneType::Industrial, meshes.add(generate_industrial()));
    map.insert(ZoneType::Office, meshes.add(generate_office()));
    map.insert(ZoneType::MixedUse, meshes.add(generate_mixed_use()));

    let flat_cuboid = meshes.add(Cuboid::new(CELL_SIZE, 1.0, CELL_SIZE));

    commands.insert_resource(BuildingPreviewMeshes {
        meshes: map,
        flat_cuboid,
    });
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct BuildingPreviewMeshPlugin;

impl Plugin for BuildingPreviewMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_building_preview_meshes.before(crate::cursor_preview::spawn_cursor_preview),
        );
    }
}
