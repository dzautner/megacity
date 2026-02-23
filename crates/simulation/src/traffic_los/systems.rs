//! ECS systems for computing LOS grades from traffic data.
//!
//! - `update_traffic_los`: per-cell LOS from traffic density grid.
//! - `update_segment_los`: per-segment LOS from averaged cell densities.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::road_segments::RoadSegmentStore;
use crate::traffic::TrafficGrid;

use super::grades::LosGrade;
use super::grid::TrafficLosGrid;
use super::segment_los::{LosDistribution, TrafficLosState};

/// System that computes LOS grades for all road cells.
/// Runs every 10 ticks, after traffic density is updated.
pub fn update_traffic_los(
    tick: Res<crate::TickCounter>,
    grid: Res<WorldGrid>,
    traffic: Res<TrafficGrid>,
    mut los_grid: ResMut<TrafficLosGrid>,
) {
    // Run every 10 ticks (aligned with traffic updates which run every 5)
    if !tick.0.is_multiple_of(10) {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Road {
                // Non-road cells default to A
                los_grid.set(x, y, LosGrade::A);
                continue;
            }

            let density = traffic.get(x, y) as f32;
            let capacity = cell.road_type.capacity() as f32;
            let vc_ratio = density / capacity;
            let grade = LosGrade::from_vc_ratio(vc_ratio);
            los_grid.set(x, y, grade);
        }
    }
}

/// System that computes per-segment LOS from averaged V/C ratios and
/// updates city-wide distribution statistics.
/// Runs every 10 ticks, after the per-cell LOS is computed.
pub fn update_segment_los(
    tick: Res<crate::TickCounter>,
    segments: Res<RoadSegmentStore>,
    traffic: Res<TrafficGrid>,
    grid: Res<WorldGrid>,
    mut los_state: ResMut<TrafficLosState>,
    mut distribution: ResMut<LosDistribution>,
) {
    if !tick.0.is_multiple_of(10) {
        return;
    }

    // Clear stale entries for segments that no longer exist
    let valid_ids: std::collections::HashSet<u32> =
        segments.segments.iter().map(|s| s.id.0).collect();
    los_state
        .segment_grades
        .retain(|id, _| valid_ids.contains(id));

    // Compute per-segment LOS from averaged V/C ratio across rasterized cells
    for segment in &segments.segments {
        if segment.rasterized_cells.is_empty() {
            los_state.set(segment.id, LosGrade::A);
            continue;
        }

        let capacity = segment.road_type.capacity() as f32;
        let mut total_vc = 0.0_f32;
        let mut cell_count = 0u32;

        for &(cx, cy) in &segment.rasterized_cells {
            if cx < GRID_WIDTH && cy < GRID_HEIGHT {
                let cell = grid.get(cx, cy);
                if cell.cell_type == CellType::Road {
                    let density = traffic.get(cx, cy) as f32;
                    total_vc += density / capacity;
                    cell_count += 1;
                }
            }
        }

        let avg_vc = if cell_count > 0 {
            total_vc / cell_count as f32
        } else {
            0.0
        };

        let grade = LosGrade::from_vc_ratio(avg_vc);
        los_state.set(segment.id, grade);
    }

    // Recompute city-wide distribution
    distribution.recompute(&los_state);
}
