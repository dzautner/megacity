// ---------------------------------------------------------------------------
// Restore functions for infrastructure: roads, stormwater, drainage, floods, sewers
// ---------------------------------------------------------------------------

use crate::save_codec::*;
use crate::save_types::*;

use simulation::cso::SewerSystemState;
use simulation::flood_simulation::FloodState;
use simulation::road_segments::{
    RoadSegment, RoadSegmentStore, SegmentId, SegmentNode, SegmentNodeId,
};
use simulation::stormwater::StormwaterGrid;

/// Reconstruct a `RoadSegmentStore` from saved data.
/// After calling this, call `store.rasterize_all(grid, roads)` to rebuild grid cells.
pub fn restore_road_segment_store(save: &SaveRoadSegmentStore) -> RoadSegmentStore {
    use bevy::math::Vec2;

    let nodes: Vec<SegmentNode> = save
        .nodes
        .iter()
        .map(|n| SegmentNode {
            id: SegmentNodeId(n.id),
            position: Vec2::new(n.x, n.y),
            connected_segments: n.connected_segments.iter().map(|&s| SegmentId(s)).collect(),
        })
        .collect();

    let segments: Vec<RoadSegment> = save
        .segments
        .iter()
        .map(|s| RoadSegment {
            id: SegmentId(s.id),
            start_node: SegmentNodeId(s.start_node),
            end_node: SegmentNodeId(s.end_node),
            p0: Vec2::new(s.p0_x, s.p0_y),
            p1: Vec2::new(s.p1_x, s.p1_y),
            p2: Vec2::new(s.p2_x, s.p2_y),
            p3: Vec2::new(s.p3_x, s.p3_y),
            road_type: u8_to_road_type(s.road_type),
            arc_length: 0.0,
            rasterized_cells: Vec::new(),
        })
        .collect();

    RoadSegmentStore::from_parts(nodes, segments)
}

/// Restore a `StormwaterGrid` resource from saved data.
pub fn restore_stormwater_grid(save: &SaveStormwaterGrid) -> StormwaterGrid {
    StormwaterGrid {
        runoff: save.runoff.clone(),
        total_runoff: save.total_runoff,
        total_infiltration: save.total_infiltration,
        width: save.width,
        height: save.height,
    }
}

/// Restore a `StormDrainageState` resource from saved data.
pub fn restore_storm_drainage(
    save: &crate::save_types::SaveStormDrainageState,
) -> simulation::storm_drainage::StormDrainageState {
    simulation::storm_drainage::StormDrainageState {
        total_drain_capacity: save.total_drain_capacity,
        total_retention_capacity: save.total_retention_capacity,
        current_retention_stored: save.current_retention_stored,
        drain_count: save.drain_count,
        retention_pond_count: save.retention_pond_count,
        rain_garden_count: save.rain_garden_count,
        overflow_cells: save.overflow_cells,
        drainage_coverage: save.drainage_coverage,
    }
}

/// Restore a `FloodState` resource from saved data.
pub fn restore_flood_state(save: &SaveFloodState) -> FloodState {
    FloodState {
        is_flooding: save.is_flooding,
        total_flooded_cells: save.total_flooded_cells,
        total_damage: save.total_damage,
        max_depth: save.max_depth,
    }
}

/// Restore a `SewerSystemState` resource from saved data.
pub fn restore_cso(state: &SaveCsoState) -> SewerSystemState {
    SewerSystemState {
        sewer_type: u8_to_sewer_type(state.sewer_type),
        combined_capacity: state.combined_capacity,
        current_flow: state.current_flow,
        cso_active: state.cso_active,
        cso_discharge_gallons: state.cso_discharge_gallons,
        cso_events_total: state.cso_events_total,
        cso_events_this_year: state.cso_events_this_year,
        cells_with_separated_sewer: state.cells_with_separated_sewer,
        total_sewer_cells: state.total_sewer_cells,
        separation_coverage: state.separation_coverage,
        annual_cso_volume: state.annual_cso_volume,
        pollution_contribution: state.pollution_contribution,
    }
}
