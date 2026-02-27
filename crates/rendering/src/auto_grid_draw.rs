//! Auto-Grid Road Drawing System (TRAF-010)
//!
//! Handles the two-click interaction for the auto-grid tool:
//! 1. First click places corner 1
//! 2. Second click places corner 2 and commits the grid
//!
//! Also draws the preview overlay (rectangle outline + grid lines) between clicks.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use simulation::app_state::AppState;
use simulation::auto_grid_road::{self, AutoGridConfig, AutoGridPhase, AutoGridState};
use simulation::config::CELL_SIZE;
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;

use crate::egui_input_guard::egui_wants_pointer;
use crate::input::{ActiveTool, CursorGridPos, StatusMessage};
use crate::terrain_render::{mark_chunk_dirty_at, ChunkDirty, TerrainChunk};

/// System that handles clicks for the auto-grid tool.
#[allow(clippy::too_many_arguments)]
pub fn handle_auto_grid_tool(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    mut auto_grid_state: ResMut<AutoGridState>,
    auto_grid_config: Res<AutoGridConfig>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut segments: ResMut<RoadSegmentStore>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    mut commands: Commands,
    drag: Res<crate::camera::LeftClickDrag>,
) {
    if *tool != ActiveTool::AutoGrid {
        // Reset state when switching away from auto-grid
        if auto_grid_state.phase != AutoGridPhase::Idle {
            auto_grid_state.phase = AutoGridPhase::Idle;
        }
        return;
    }

    // Prevent click-through: skip world actions when egui is handling pointer input.
    if egui_wants_pointer(&mut contexts) {
        return;
    }

    if drag.is_dragging || !cursor.valid {
        return;
    }

    // Right click cancels
    if buttons.just_pressed(MouseButton::Right) {
        auto_grid_state.phase = AutoGridPhase::Idle;
        status.set("Auto-grid cancelled", false);
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    match auto_grid_state.phase {
        AutoGridPhase::Idle => {
            auto_grid_state.corner1 = (gx, gy);
            auto_grid_state.phase = AutoGridPhase::PlacedFirstCorner;
            status.set(
                "Click second corner to define grid area (Esc=cancel, Right-click=cancel)",
                false,
            );
        }
        AutoGridPhase::PlacedFirstCorner => {
            let corner1 = auto_grid_state.corner1;
            let corner2 = (gx, gy);

            // Minimum area check
            let dx = (corner2.0 as i32 - corner1.0 as i32).unsigned_abs() as usize;
            let dy = (corner2.1 as i32 - corner1.1 as i32).unsigned_abs() as usize;
            let min_dim = auto_grid_config.block_size as usize + 2;
            if dx < min_dim || dy < min_dim {
                status.set(
                    format!(
                        "Area too small (min {}x{} for block size {})",
                        min_dim, min_dim, auto_grid_config.block_size
                    ),
                    true,
                );
                return;
            }

            // Compute plan and check budget
            let plan =
                auto_grid_road::compute_grid_plan(corner1, corner2, &auto_grid_config, &grid);

            if plan.total_cells == 0 {
                status.set("No roads can be placed in this area", true);
                auto_grid_state.phase = AutoGridPhase::Idle;
                return;
            }

            if budget.treasury < plan.total_cost {
                status.set(
                    format!(
                        "Not enough money (need ${:.0}, have ${:.0})",
                        plan.total_cost, budget.treasury
                    ),
                    true,
                );
                return;
            }

            // Execute the plan
            let cells = auto_grid_road::execute_grid_plan(
                &plan,
                &auto_grid_config,
                &mut segments,
                &mut grid,
                &mut roads,
            );

            budget.treasury -= plan.total_cost;

            // Mark dirty chunks for all affected cells
            for &(cx, cy) in &cells {
                mark_chunk_dirty_at(cx, cy, &chunks, &mut commands);
            }

            status.set(
                format!(
                    "Placed {} road grid ({} cells, ${:.0})",
                    road_type_label(auto_grid_config.road_type),
                    cells.len(),
                    plan.total_cost,
                ),
                false,
            );

            // Reset to idle for next placement
            auto_grid_state.phase = AutoGridPhase::Idle;
        }
    }
}

/// Draw preview gizmos showing the grid rectangle and planned road lines.
pub fn draw_auto_grid_preview(
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    auto_grid_state: Res<AutoGridState>,
    auto_grid_config: Res<AutoGridConfig>,
    grid: Res<WorldGrid>,
    budget: Res<CityBudget>,
    mut gizmos: Gizmos,
) {
    if *tool != ActiveTool::AutoGrid || auto_grid_state.phase != AutoGridPhase::PlacedFirstCorner {
        return;
    }

    if !cursor.valid {
        return;
    }

    let corner1 = auto_grid_state.corner1;
    let corner2 = (cursor.grid_x as usize, cursor.grid_y as usize);

    // Draw the rectangle outline
    let min_x = corner1.0.min(corner2.0);
    let max_x = corner1.0.max(corner2.0);
    let min_y = corner1.1.min(corner2.1);
    let max_y = corner1.1.max(corner2.1);

    let y_height = 0.5; // slightly above ground

    // Rectangle corners in world coords
    let (wx_min, wy_min) = (min_x as f32 * CELL_SIZE, min_y as f32 * CELL_SIZE);
    let (wx_max, wy_max) = (
        (max_x + 1) as f32 * CELL_SIZE,
        (max_y + 1) as f32 * CELL_SIZE,
    );

    let c1 = Vec3::new(wx_min, y_height, wy_min);
    let c2 = Vec3::new(wx_max, y_height, wy_min);
    let c3 = Vec3::new(wx_max, y_height, wy_max);
    let c4 = Vec3::new(wx_min, y_height, wy_max);

    // Compute plan to determine affordability
    let plan = auto_grid_road::compute_grid_plan(corner1, corner2, &auto_grid_config, &grid);
    let affordable = budget.treasury >= plan.total_cost;
    let outline_color = if affordable {
        Color::srgba(0.2, 0.8, 0.2, 0.8)
    } else {
        Color::srgba(0.8, 0.2, 0.2, 0.8)
    };

    // Draw rectangle outline
    gizmos.line(c1, c2, outline_color);
    gizmos.line(c2, c3, outline_color);
    gizmos.line(c3, c4, outline_color);
    gizmos.line(c4, c1, outline_color);

    // Draw planned road lines
    let line_color = if affordable {
        Color::srgba(0.3, 0.7, 1.0, 0.5)
    } else {
        Color::srgba(0.7, 0.3, 0.3, 0.5)
    };

    for &((x0, y0), (x1, y1)) in &plan.segments {
        let (sx, sy) = WorldGrid::grid_to_world(x0, y0);
        let (ex, ey) = WorldGrid::grid_to_world(x1, y1);
        gizmos.line(
            Vec3::new(sx, y_height, sy),
            Vec3::new(ex, y_height, ey),
            line_color,
        );
    }

    // Draw first corner marker
    let (fx, fy) = WorldGrid::grid_to_world(corner1.0, corner1.1);
    gizmos.circle(
        Isometry3d::new(
            Vec3::new(fx, y_height + 0.1, fy),
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        ),
        CELL_SIZE * 0.5,
        Color::srgba(1.0, 1.0, 0.0, 0.9),
    );
}

fn road_type_label(rt: simulation::grid::RoadType) -> &'static str {
    match rt {
        simulation::grid::RoadType::Local => "Local",
        simulation::grid::RoadType::Avenue => "Avenue",
        simulation::grid::RoadType::Boulevard => "Boulevard",
        simulation::grid::RoadType::Highway => "Highway",
        simulation::grid::RoadType::OneWay => "One-Way",
        simulation::grid::RoadType::Path => "Path",
    }
}

pub struct AutoGridDrawPlugin;

impl Plugin for AutoGridDrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_auto_grid_tool, draw_auto_grid_preview)
                .run_if(in_state(AppState::Playing)),
        );
    }
}
