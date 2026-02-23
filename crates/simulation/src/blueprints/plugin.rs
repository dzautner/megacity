//! Blueprint events, systems, and Bevy plugin registration.

use bevy::prelude::*;

use crate::grid::WorldGrid;
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;

use super::library::BlueprintLibrary;

// =============================================================================
// Events
// =============================================================================

/// Event to capture a blueprint from a rectangular region.
#[derive(Event)]
pub struct CaptureBlueprint {
    pub origin_x: usize,
    pub origin_y: usize,
    pub width: usize,
    pub height: usize,
    pub name: String,
}

/// Event to place a blueprint at a target location.
#[derive(Event)]
pub struct PlaceBlueprint {
    /// Index of the blueprint in the library.
    pub blueprint_index: usize,
    /// Target grid origin for placement.
    pub target_x: usize,
    pub target_y: usize,
}

/// Event fired after a blueprint is successfully captured.
#[derive(Event)]
pub struct BlueprintCaptured {
    pub name: String,
    pub index: usize,
}

/// Event fired after a blueprint is successfully placed.
#[derive(Event)]
pub struct BlueprintPlaced {
    pub name: String,
    pub segments_placed: u32,
    pub zones_placed: u32,
}

// =============================================================================
// Systems
// =============================================================================

/// System that processes `CaptureBlueprint` events.
fn handle_capture_blueprint(
    mut events: EventReader<CaptureBlueprint>,
    mut library: ResMut<BlueprintLibrary>,
    grid: Res<WorldGrid>,
    segments: Res<RoadSegmentStore>,
    mut captured_events: EventWriter<BlueprintCaptured>,
) {
    for ev in events.read() {
        let blueprint = super::blueprint::Blueprint::capture(
            &grid,
            &segments,
            ev.origin_x,
            ev.origin_y,
            ev.width,
            ev.height,
            ev.name.clone(),
        );
        let index = library.add(blueprint);
        info!(
            "Blueprint '{}' captured (index {}) from ({},{}) size {}x{}",
            ev.name, index, ev.origin_x, ev.origin_y, ev.width, ev.height
        );
        captured_events.send(BlueprintCaptured {
            name: ev.name.clone(),
            index,
        });
    }
}

/// System that processes `PlaceBlueprint` events.
fn handle_place_blueprint(
    mut events: EventReader<PlaceBlueprint>,
    library: Res<BlueprintLibrary>,
    mut grid: ResMut<WorldGrid>,
    mut segments: ResMut<RoadSegmentStore>,
    mut roads: ResMut<RoadNetwork>,
    mut placed_events: EventWriter<BlueprintPlaced>,
) {
    for ev in events.read() {
        let Some(blueprint) = library.get(ev.blueprint_index) else {
            warn!(
                "PlaceBlueprint: invalid index {} (library has {} blueprints)",
                ev.blueprint_index,
                library.count()
            );
            continue;
        };
        let name = blueprint.name.clone();
        let result = blueprint.place(
            &mut grid,
            &mut segments,
            &mut roads,
            ev.target_x,
            ev.target_y,
        );
        info!(
            "Blueprint '{}' placed at ({},{}) — {} segments, {} zones",
            name, ev.target_x, ev.target_y, result.segments_placed, result.zones_placed
        );
        placed_events.send(BlueprintPlaced {
            name,
            segments_placed: result.segments_placed,
            zones_placed: result.zones_placed,
        });
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct BlueprintPlugin;

impl Plugin for BlueprintPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueprintLibrary>()
            .add_event::<CaptureBlueprint>()
            .add_event::<PlaceBlueprint>()
            .add_event::<BlueprintCaptured>()
            .add_event::<BlueprintPlaced>()
            .add_systems(
                FixedUpdate,
                (handle_capture_blueprint, handle_place_blueprint)
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register with save system
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<BlueprintLibrary>();
    }
}
