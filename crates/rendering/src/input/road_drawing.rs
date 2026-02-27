//! Freeform Bezier road drawing logic (straight and curved segments).

use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::curve_road_drawing::CurveDrawMode;
use simulation::economy::CityBudget;
use simulation::grid::{RoadType, WorldGrid};
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::undo_redo::CityAction;

use crate::angle_snap::AngleSnapState;
use crate::terrain_render::{mark_chunk_dirty_at, ChunkDirty, TerrainChunk};

use super::cursor::estimate_arc_length;
use super::types::{CursorGridPos, DrawPhase, IntersectionSnap, RoadDrawState, StatusMessage};

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_freeform_road(
    road_type: RoadType,
    buttons: &ButtonInput<MouseButton>,
    cursor: &CursorGridPos,
    snap: &IntersectionSnap,
    angle_snap: &AngleSnapState,
    curve_mode: &CurveDrawMode,
    draw_state: &mut RoadDrawState,
    segments: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    budget: &mut CityBudget,
    status: &mut StatusMessage,
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
    action_writer: &mut EventWriter<CityAction>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let click_pos = if let Some(snapped) = snap.snapped_pos {
        snapped
    } else if angle_snap.active {
        angle_snap.snapped_pos
    } else {
        cursor.world_pos
    };

    match draw_state.phase {
        DrawPhase::Idle => {
            draw_state.start_pos = click_pos;
            draw_state.phase = DrawPhase::PlacedStart;
            if curve_mode.enabled {
                status.set(
                    "Click to place control point (C=toggle curve, Esc=cancel)",
                    false,
                );
            } else {
                status.set(
                    "Click to place end point (C=curve mode, Shift=snap angle, Esc=cancel)",
                    false,
                );
            }
        }
        DrawPhase::PlacedStart if curve_mode.enabled => {
            draw_state.control_pos = click_pos;
            draw_state.phase = DrawPhase::PlacedControl;
            status.set("Click to place end point (Esc=cancel)", false);
        }
        DrawPhase::PlacedStart => {
            commit_straight_segment(
                draw_state,
                click_pos,
                road_type,
                segments,
                grid,
                roads,
                budget,
                status,
                chunks,
                commands,
                action_writer,
            );
        }
        DrawPhase::PlacedControl => {
            commit_curved_segment(
                draw_state,
                click_pos,
                road_type,
                segments,
                grid,
                roads,
                budget,
                status,
                chunks,
                commands,
                action_writer,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn commit_straight_segment(
    draw_state: &mut RoadDrawState,
    end_pos: Vec2,
    road_type: RoadType,
    segments: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    budget: &mut CityBudget,
    status: &mut StatusMessage,
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
    action_writer: &mut EventWriter<CityAction>,
) {
    let start_pos = draw_state.start_pos;

    if (end_pos - start_pos).length() < CELL_SIZE {
        status.set("Road too short", true);
        return;
    }

    let approx_cells = ((end_pos - start_pos).length() / CELL_SIZE).ceil() as usize;
    let total_cost = road_type.cost() * approx_cells as f64;
    if budget.treasury < total_cost {
        status.set(
            format!(
                "Not enough funds (need ${:.0}, have ${:.0})",
                total_cost, budget.treasury
            ),
            true,
        );
        return;
    }

    let (seg_id, cells) =
        segments.add_straight_segment(start_pos, end_pos, road_type, 24.0, grid, roads);

    let actual_cost = road_type.cost() * cells.len() as f64;
    budget.treasury -= actual_cost;

    if let Some(seg) = segments.get_segment(seg_id) {
        action_writer.send(CityAction::PlaceRoadSegment {
            segment_id: seg_id,
            start_node: seg.start_node,
            end_node: seg.end_node,
            p0: seg.p0,
            p1: seg.p1,
            p2: seg.p2,
            p3: seg.p3,
            road_type,
            rasterized_cells: cells.clone(),
            cost: actual_cost,
        });
    }
    for &(cx, cy) in &cells {
        mark_chunk_dirty_at(cx, cy, chunks, commands);
    }

    draw_state.start_pos = end_pos;
}

#[allow(clippy::too_many_arguments)]
fn commit_curved_segment(
    draw_state: &mut RoadDrawState,
    end_pos: Vec2,
    road_type: RoadType,
    segments: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    budget: &mut CityBudget,
    status: &mut StatusMessage,
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
    action_writer: &mut EventWriter<CityAction>,
) {
    let start_pos = draw_state.start_pos;
    let control_pos = draw_state.control_pos;

    if (end_pos - start_pos).length() < CELL_SIZE {
        status.set("Road too short", true);
        return;
    }

    let (p1, p2) =
        simulation::curve_road_drawing::quadratic_to_cubic(start_pos, control_pos, end_pos);
    let arc_len = estimate_arc_length(start_pos, p1, p2, end_pos);
    let approx_cells = (arc_len / CELL_SIZE).ceil() as usize;
    let total_cost = road_type.cost() * approx_cells as f64;
    if budget.treasury < total_cost {
        status.set(
            format!(
                "Not enough funds (need ${:.0}, have ${:.0})",
                total_cost, budget.treasury
            ),
            true,
        );
        return;
    }

    let (seg_id, cells) = segments.add_curved_segment(
        start_pos,
        control_pos,
        end_pos,
        road_type,
        24.0,
        grid,
        roads,
    );

    let actual_cost = road_type.cost() * cells.len() as f64;

    if let Some(seg) = segments.get_segment(seg_id) {
        action_writer.send(CityAction::PlaceRoadSegment {
            segment_id: seg_id,
            start_node: seg.start_node,
            end_node: seg.end_node,
            p0: seg.p0,
            p1: seg.p1,
            p2: seg.p2,
            p3: seg.p3,
            road_type,
            rasterized_cells: cells.clone(),
            cost: actual_cost,
        });
    }
    budget.treasury -= actual_cost;

    for &(cx, cy) in &cells {
        mark_chunk_dirty_at(cx, cy, chunks, commands);
    }

    draw_state.start_pos = end_pos;
    draw_state.phase = DrawPhase::PlacedStart;
    status.set("Click to place control point (Esc=cancel)", false);
}
