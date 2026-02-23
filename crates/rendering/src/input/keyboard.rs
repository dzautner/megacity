use bevy::prelude::*;

use simulation::bulldoze_refund;
use simulation::config::CELL_SIZE;
use simulation::curve_road_drawing::CurveDrawMode;
use simulation::economy::CityBudget;
use simulation::grid::{WorldGrid, ZoneType};
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;

use crate::terrain_render::{mark_chunk_dirty_at, ChunkDirty, TerrainChunk};

use super::types::{
    ActiveTool, CursorGridPos, DrawPhase, GridSnap, RoadDrawState, SelectedBuilding, StatusMessage,
};

// ---------------------------------------------------------------------------
// Keyboard shortcuts (core tools only; extended tools via UI toolbar)
// ---------------------------------------------------------------------------

/// Toggle grid snap mode with the F key.
pub fn toggle_grid_snap(
    keys: Res<ButtonInput<KeyCode>>,
    mut grid_snap: ResMut<GridSnap>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if bindings.toggle_grid_snap.just_pressed(&keys) {
        grid_snap.enabled = !grid_snap.enabled;
    }
}

/// Toggle curve drawing mode with the G key (configurable via keybindings).
pub fn toggle_curve_draw_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut curve_mode: ResMut<CurveDrawMode>,
    mut status: ResMut<StatusMessage>,
    mut draw_state: ResMut<RoadDrawState>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if bindings.toggle_curve_draw.just_pressed(&keys) {
        curve_mode.enabled = !curve_mode.enabled;
        // Reset drawing state when toggling to avoid confusing partial state
        if draw_state.phase != DrawPhase::Idle {
            draw_state.phase = DrawPhase::Idle;
        }
        if curve_mode.enabled {
            status.set("Curve drawing mode ON (G to toggle off)", false);
        } else {
            status.set("Curve drawing mode OFF", false);
        }
    }
}

/// Quick-access tool shortcuts (R/Z/B/I/V).
/// Digit keys 1-3 are reserved for simulation speed; overlays use Tab cycling.
pub fn keyboard_tool_switch(
    keys: Res<ButtonInput<KeyCode>>,
    mut tool: ResMut<ActiveTool>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if bindings.tool_road.just_pressed(&keys) {
        *tool = ActiveTool::Road;
    }
    if bindings.tool_zone_res.just_pressed(&keys) {
        *tool = ActiveTool::ZoneResidentialLow;
    }
    if bindings.tool_bulldoze.just_pressed(&keys) {
        *tool = ActiveTool::Bulldoze;
    }
    if bindings.tool_inspect.just_pressed(&keys) {
        *tool = ActiveTool::Inspect;
    }
    if bindings.tool_zone_com.just_pressed(&keys) {
        *tool = ActiveTool::ZoneCommercialLow;
    }
}

// ---------------------------------------------------------------------------
// Delete key bulldozes the currently selected building
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn delete_selected_building(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<simulation::keybindings::KeyBindings>,
    mut selected: ResMut<SelectedBuilding>,
    mut grid: ResMut<WorldGrid>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
    mut commands: Commands,
    service_q: Query<&ServiceBuilding>,
    utility_q: Query<&UtilitySource>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
) {
    if !bindings.delete_building.just_pressed(&keys)
        && !bindings.delete_building_alt.just_pressed(&keys)
    {
        return;
    }

    let Some(entity) = selected.0 else {
        return;
    };

    // Compute refund based on entity type
    let refund = if let Ok(service) = service_q.get(entity) {
        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
        let sx = service.grid_x;
        let sy = service.grid_y;
        for fy in sy..sy + fh {
            for fx in sx..sx + fw {
                if grid.in_bounds(fx, fy) {
                    grid.get_mut(fx, fy).building_id = None;
                    grid.get_mut(fx, fy).zone = ZoneType::None;
                    mark_chunk_dirty_at(fx, fy, &chunks, &mut commands);
                }
            }
        }
        bulldoze_refund::refund_for_service(service.service_type)
    } else if let Ok(utility) = utility_q.get(entity) {
        let ux = utility.grid_x;
        let uy = utility.grid_y;
        if grid.in_bounds(ux, uy) {
            grid.get_mut(ux, uy).building_id = None;
            grid.get_mut(ux, uy).zone = ZoneType::None;
            mark_chunk_dirty_at(ux, uy, &chunks, &mut commands);
        }
        bulldoze_refund::refund_for_utility(utility.utility_type)
    } else {
        // Regular building: scan grid for matching entity
        for y in 0..grid.height {
            for x in 0..grid.width {
                if grid.get(x, y).building_id == Some(entity) {
                    grid.get_mut(x, y).building_id = None;
                    grid.get_mut(x, y).zone = ZoneType::None;
                    mark_chunk_dirty_at(x, y, &chunks, &mut commands);
                }
            }
        }
        0.0
    };

    budget.treasury += refund;
    commands.entity(entity).despawn();
    selected.0 = None;
    let msg = if refund > 0.0 {
        format!("Building demolished (refund: ${:.0})", refund)
    } else {
        "Building demolished".to_string()
    };
    status.set(msg, false);
}

// ---------------------------------------------------------------------------
// Tree tool system (separate from handle_tool_input to stay within param limit)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn handle_tree_tool(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    grid: Res<WorldGrid>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
    mut tree_grid: ResMut<simulation::trees::TreeGrid>,
    planted_trees: Query<(Entity, &simulation::trees::PlantedTree)>,
    mut commands: Commands,
    left_drag: Res<crate::camera::LeftClickDrag>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
) {
    if left_drag.is_dragging {
        return;
    }

    let is_tree_tool = matches!(*tool, ActiveTool::TreePlant | ActiveTool::TreeRemove);
    if !is_tree_tool {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    let changed = match *tool {
        ActiveTool::TreePlant => {
            if tree_grid.has_tree(gx, gy) {
                status.set("Tree already here", true);
                false
            } else if grid.get(gx, gy).cell_type != simulation::grid::CellType::Grass {
                status.set("Can only plant trees on grass", true);
                false
            } else if grid.get(gx, gy).building_id.is_some() {
                status.set("Cell occupied by a building", true);
                false
            } else if budget.treasury < simulation::trees::TREE_PLANT_COST {
                status.set("Not enough money", true);
                false
            } else {
                budget.treasury -= simulation::trees::TREE_PLANT_COST;
                tree_grid.set(gx, gy, true);
                commands.spawn(simulation::trees::PlantedTree {
                    grid_x: gx,
                    grid_y: gy,
                });
                true
            }
        }
        ActiveTool::TreeRemove => {
            if !tree_grid.has_tree(gx, gy) {
                status.set("No tree here", true);
                false
            } else {
                tree_grid.set(gx, gy, false);
                for (entity, planted) in &planted_trees {
                    if planted.grid_x == gx && planted.grid_y == gy {
                        commands.entity(entity).despawn();
                        break;
                    }
                }
                true
            }
        }
        _ => false,
    };

    if changed {
        mark_chunk_dirty_at(gx, gy, &chunks, &mut commands);
    }
}

// ---------------------------------------------------------------------------
// Road upgrade tool system (separate from handle_tool_input for param limit)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn handle_road_upgrade_tool(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut segments: ResMut<RoadSegmentStore>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
    left_drag: Res<crate::camera::LeftClickDrag>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    mut commands: Commands,
) {
    if left_drag.is_dragging {
        return;
    }

    if *tool != ActiveTool::RoadUpgrade {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    // Find the closest segment to the cursor position
    let seg_id = match simulation::road_upgrade::find_segment_near(
        cursor.world_pos,
        &segments,
        CELL_SIZE * 2.0,
    ) {
        Some(id) => id,
        None => {
            status.set("No road segment here", true);
            return;
        }
    };

    // Get current type for status message
    let current_type = match segments.get_segment(seg_id) {
        Some(seg) => seg.road_type,
        None => {
            status.set("Segment not found", true);
            return;
        }
    };

    match simulation::road_upgrade::upgrade_segment(
        seg_id,
        &mut segments,
        &mut grid,
        &mut roads,
        &mut budget,
    ) {
        Ok(new_type) => {
            // Mark all affected chunks dirty for re-rendering
            if let Some(seg) = segments.get_segment(seg_id) {
                for &(gx, gy) in &seg.rasterized_cells {
                    mark_chunk_dirty_at(gx, gy, &chunks, &mut commands);
                }
            }
            status.set(
                format!("Upgraded {:?} to {:?}", current_type, new_type),
                false,
            );
        }
        Err(reason) => {
            status.set(reason, true);
        }
    }
}

// ---------------------------------------------------------------------------
// Escape key cascade: cancel draw -> deselect building -> reset tool
// ---------------------------------------------------------------------------

/// Handles the Escape key with cascading behavior:
/// 1. Cancel active road drawing (if `RoadDrawState` is not Idle)
/// 2. Deselect the selected building (if `SelectedBuilding` has a value)
/// 3. Reset the active tool back to `Inspect`
///
/// Each press handles exactly one level.
#[allow(clippy::too_many_arguments)]
pub fn handle_escape_key(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<simulation::keybindings::KeyBindings>,
    mut draw_state: ResMut<RoadDrawState>,
    mut selected: ResMut<SelectedBuilding>,
    mut tool: ResMut<ActiveTool>,
    mut selection_kind: ResMut<crate::enhanced_select::SelectionKind>,
    mut freehand: ResMut<simulation::freehand_road::FreehandDrawState>,
    mut auto_grid_state: ResMut<simulation::auto_grid_road::AutoGridState>,
) {
    if !bindings.escape.just_pressed(&keys) {
        return;
    }

    // Level 0: Cancel active freehand stroke
    if freehand.drawing {
        freehand.reset_stroke();
        return;
    }

    // Level 0b: Cancel active auto-grid placement
    if auto_grid_state.phase != simulation::auto_grid_road::AutoGridPhase::Idle {
        auto_grid_state.phase = simulation::auto_grid_road::AutoGridPhase::Idle;
        return;
    }
    // Level 1: Cancel active road drawing
    if draw_state.phase != DrawPhase::Idle {
        draw_state.phase = DrawPhase::Idle;
        return;
    }

    // Level 2: Deselect any selection (building, citizen, road, cell)
    if selected.0.is_some() || *selection_kind != crate::enhanced_select::SelectionKind::None {
        selected.0 = None;
        *selection_kind = crate::enhanced_select::SelectionKind::None;
        return;
    }

    // Level 3: Reset to Inspect tool
    if *tool != ActiveTool::Inspect {
        *tool = ActiveTool::Inspect;
    }
}
