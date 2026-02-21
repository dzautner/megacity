//! Freehand road drawing input system (UX-020).
//!
//! Handles mouse input for freehand drawing mode: toggling with H key,
//! collecting sample points while the mouse is held, and committing
//! simplified road segments on release.

use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::economy::CityBudget;
use simulation::freehand_road::{
    filter_short_segments, simplify_rdp, FreehandDrawState, FREEHAND_MIN_SEGMENT_LEN,
    FREEHAND_SIMPLIFY_TOLERANCE,
};
use simulation::grid::RoadType;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;

use crate::camera::LeftClickDrag;
use crate::input::{ActiveTool, CursorGridPos, StatusMessage};
use crate::terrain_render::{mark_chunk_dirty_at, ChunkDirty, TerrainChunk};

pub struct FreehandDrawPlugin;

impl Plugin for FreehandDrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (toggle_freehand_mode, handle_freehand_draw)
                .chain()
                .before(crate::input::handle_tool_input),
        );
    }
}

/// Toggle freehand drawing mode with H key.
pub fn toggle_freehand_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut freehand: ResMut<FreehandDrawState>,
    mut status: ResMut<StatusMessage>,
    tool: Res<ActiveTool>,
) {
    if !keys.just_pressed(KeyCode::KeyH) {
        return;
    }

    freehand.enabled = !freehand.enabled;
    freehand.reset_stroke();

    if freehand.enabled {
        let tool_name = road_type_for_tool(&tool)
            .map(|_| "")
            .unwrap_or(" (select a road tool first)");
        status.set(
            format!(
                "Freehand drawing ON{} — hold mouse and drag to draw",
                tool_name
            ),
            false,
        );
    } else {
        status.set("Freehand drawing OFF", false);
    }
}

/// Main freehand drawing system: collect samples while mouse is held,
/// commit segments on release.
#[allow(clippy::too_many_arguments)]
pub fn handle_freehand_draw(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    mut freehand: ResMut<FreehandDrawState>,
    mut segments: ResMut<RoadSegmentStore>,
    mut grid: ResMut<simulation::grid::WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
    left_drag: Res<LeftClickDrag>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    mut commands: Commands,
) {
    if !freehand.enabled {
        return;
    }

    // Only activate for road tools
    let Some(road_type) = road_type_for_tool(&tool) else {
        return;
    };

    // Don't interfere with camera panning
    if left_drag.is_dragging {
        freehand.reset_stroke();
        return;
    }

    if !cursor.valid {
        return;
    }

    // Mouse button just pressed — start a new stroke
    if buttons.just_pressed(MouseButton::Left) {
        freehand.raw_points.clear();
        freehand.drawing = true;
        freehand.add_sample(cursor.world_pos);
        return;
    }

    // Mouse held — collect samples
    if buttons.pressed(MouseButton::Left) && freehand.drawing {
        freehand.add_sample(cursor.world_pos);
        return;
    }

    // Mouse released — commit the stroke
    if buttons.just_released(MouseButton::Left) && freehand.drawing {
        // Add the final cursor position
        if let Some(&last) = freehand.raw_points.last() {
            if (cursor.world_pos - last).length() > 1.0 {
                freehand.raw_points.push(cursor.world_pos);
            }
        }

        let raw_count = freehand.raw_points.len();
        if raw_count < 2 {
            freehand.reset_stroke();
            return;
        }

        // Simplify the path
        let simplified = simplify_rdp(&freehand.raw_points, FREEHAND_SIMPLIFY_TOLERANCE);
        let simplified = filter_short_segments(&simplified, FREEHAND_MIN_SEGMENT_LEN);

        if simplified.len() < 2 {
            freehand.reset_stroke();
            return;
        }

        // Estimate total cost
        let total_world_dist: f32 = simplified.windows(2).map(|w| (w[1] - w[0]).length()).sum();
        let approx_cells = (total_world_dist / CELL_SIZE).ceil() as usize;
        let total_cost = road_type.cost() * approx_cells as f64;

        if budget.treasury < total_cost {
            status.set("Not enough money for freehand road", true);
            freehand.reset_stroke();
            return;
        }

        // Create segments between consecutive simplified points
        let mut total_actual_cost = 0.0;
        let segment_count = simplified.len() - 1;

        for pair in simplified.windows(2) {
            let from = pair[0];
            let to = pair[1];

            if (to - from).length() < CELL_SIZE * 0.5 {
                continue;
            }

            let (_seg_id, cells) =
                segments.add_straight_segment(from, to, road_type, 24.0, &mut grid, &mut roads);

            total_actual_cost += road_type.cost() * cells.len() as f64;

            for &(cx, cy) in &cells {
                mark_chunk_dirty_at(cx, cy, &chunks, &mut commands);
            }
        }

        budget.treasury -= total_actual_cost;

        status.set(
            format!(
                "Freehand: {} segments placed (${:.0})",
                segment_count, total_actual_cost
            ),
            false,
        );

        freehand.reset_stroke();
    }

    // Right-click cancels current stroke
    if buttons.just_pressed(MouseButton::Right) && freehand.drawing {
        freehand.reset_stroke();
        status.set("Freehand stroke cancelled", false);
    }
}

/// Map the active tool to a road type, if applicable.
fn road_type_for_tool(tool: &ActiveTool) -> Option<RoadType> {
    match tool {
        ActiveTool::Road => Some(RoadType::Local),
        ActiveTool::RoadAvenue => Some(RoadType::Avenue),
        ActiveTool::RoadBoulevard => Some(RoadType::Boulevard),
        ActiveTool::RoadHighway => Some(RoadType::Highway),
        ActiveTool::RoadOneWay => Some(RoadType::OneWay),
        ActiveTool::RoadPath => Some(RoadType::Path),
        _ => None,
    }
}
