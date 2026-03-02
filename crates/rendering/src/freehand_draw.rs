//! Freehand road drawing input system (UX-020).
//!
//! Handles mouse input for freehand drawing mode: toggling with H key,
//! collecting sample points while the mouse is held, and committing
//! simplified road segments on release.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use simulation::app_state::AppState;
use simulation::config::CELL_SIZE;
use simulation::economy::CityBudget;
use simulation::freehand_road::{
    bezier_arc_length, bezier_point, filter_short_segments, fit_catmull_rom_beziers, simplify_rdp,
    FreehandDrawState, FREEHAND_MIN_SEGMENT_LEN, FREEHAND_SIMPLIFY_TOLERANCE,
};
use simulation::grid::RoadType;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;

use crate::camera::LeftClickDrag;
use crate::egui_input_guard::egui_wants_pointer;
use crate::input::{ActiveTool, CursorGridPos, StatusMessage};
use crate::terrain_render::{mark_chunk_dirty_at, ChunkDirty, TerrainChunk};

pub struct FreehandDrawPlugin;

impl Plugin for FreehandDrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (toggle_freehand_mode, handle_freehand_draw)
                .chain()
                .before(crate::input::handle_tool_input)
                .run_if(in_state(AppState::Playing)),
        );
        app.add_systems(
            Update,
            draw_freehand_preview.run_if(in_state(AppState::Playing)),
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
    mut contexts: EguiContexts,
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

    // Prevent click-through: skip world actions when egui is handling pointer input.
    if egui_wants_pointer(&mut contexts) {
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

        // Estimate total cost using Bezier arc lengths
        let bezier_preview = fit_catmull_rom_beziers(&simplified);
        let total_world_dist: f32 = bezier_preview
            .iter()
            .map(|(p0, c1, c2, p3)| bezier_arc_length(*p0, *c1, *c2, *p3, 32))
            .sum();
        let approx_cells = (total_world_dist / CELL_SIZE).ceil() as usize;
        let total_cost = road_type.cost() * approx_cells as f64;

        if budget.treasury < total_cost {
            status.set(format!("Not enough funds (need ${:.0}, have ${:.0})", total_cost, budget.treasury), true);
            freehand.reset_stroke();
            return;
        }

        // Fit smooth Catmull-Rom Bezier curves through simplified points
        let beziers = fit_catmull_rom_beziers(&simplified);
        let mut total_actual_cost = 0.0;
        let segment_count = beziers.len();

        for (p0, c1, c2, p3) in &beziers {
            if (*p3 - *p0).length() < CELL_SIZE * 0.5 {
                continue;
            }

            let start_node = segments.find_or_create_node(*p0, 24.0);
            let end_node = segments.find_or_create_node(*p3, 24.0);
            let seg_id = segments.add_segment(
                start_node, end_node, *p0, *c1, *c2, *p3, road_type, &mut grid, &mut roads,
            );

            if let Some(seg) = segments.get_segment(seg_id) {
                total_actual_cost += road_type.cost() * seg.rasterized_cells.len() as f64;
                for &(cx, cy) in &seg.rasterized_cells {
                    mark_chunk_dirty_at(cx, cy, &chunks, &mut commands);
                }
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


/// Draw a real-time gizmo preview of the freehand curve while the user is dragging.
#[allow(clippy::too_many_arguments)]
pub fn draw_freehand_preview(
    freehand: Res<FreehandDrawState>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    mut gizmos: Gizmos,
) {
    if !freehand.drawing || freehand.raw_points.len() < 2 || !cursor.valid {
        return;
    }

    // Only show for road tools
    if road_type_for_tool(&tool).is_none() {
        return;
    }

    // Build preview points: raw samples + current cursor position
    let mut preview_pts = freehand.raw_points.clone();
    if let Some(&last) = preview_pts.last() {
        if (cursor.world_pos - last).length() > 1.0 {
            preview_pts.push(cursor.world_pos);
        }
    }

    if preview_pts.len() < 2 {
        return;
    }

    // Simplify and fit curves
    let simplified = simplify_rdp(&preview_pts, FREEHAND_SIMPLIFY_TOLERANCE);
    let simplified = filter_short_segments(&simplified, FREEHAND_MIN_SEGMENT_LEN);

    if simplified.len() < 2 {
        return;
    }

    let beziers = fit_catmull_rom_beziers(&simplified);
    let preview_color = Color::srgba(1.0, 1.0, 1.0, 0.6);
    let y = 0.6; // slightly above ground
    let samples_per_curve = 20;

    for (p0, c1, c2, p3) in &beziers {
        let mut prev = Vec3::new(p0.x, y, p0.y);
        for j in 1..=samples_per_curve {
            let t = j as f32 / samples_per_curve as f32;
            let pt = bezier_point(*p0, *c1, *c2, *p3, t);
            let curr = Vec3::new(pt.x, y, pt.y);
            gizmos.line(prev, curr, preview_color);
            prev = curr;
        }
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
