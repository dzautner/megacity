//! System for detecting right-click release and opening the context menu.

use bevy::prelude::*;

use rendering::camera::RightClickDrag;
use rendering::input::CursorGridPos;
use simulation::buildings::Building;
use simulation::citizen::{Citizen, Position};
use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, WorldGrid};
use simulation::road_segments::RoadSegmentStore;
use simulation::services::ServiceBuilding;

use super::types::{ContextMenuState, ContextTarget};

/// Detect right-click release (without drag) and open the context menu.
#[allow(clippy::too_many_arguments)]
pub(crate) fn detect_right_click_context_menu(
    right_click: Res<RightClickDrag>,
    cursor: Res<CursorGridPos>,
    windows: Query<&Window>,
    grid: Res<WorldGrid>,
    segments: Res<RoadSegmentStore>,
    buildings: Query<(Entity, &Building)>,
    services: Query<(Entity, &ServiceBuilding)>,
    citizens: Query<(Entity, &Position), With<Citizen>>,
    mut state: ResMut<ContextMenuState>,
) {
    if !right_click.just_released_click {
        return;
    }

    if !cursor.valid {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    // Get screen position for the menu
    let screen_pos = if let Some(pos) = window.cursor_position() {
        bevy_egui::egui::pos2(pos.x, pos.y)
    } else {
        return;
    };

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    // 1. Check for citizen under cursor (small radius)
    let world_x = gx as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    let world_y = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    let radius_sq = (CELL_SIZE * 2.0) * (CELL_SIZE * 2.0);

    let mut best_citizen: Option<(Entity, f32)> = None;
    for (entity, pos) in &citizens {
        let dx = pos.x - world_x;
        let dy = pos.y - world_y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < radius_sq && (best_citizen.is_none() || dist_sq < best_citizen.unwrap().1) {
            best_citizen = Some((entity, dist_sq));
        }
    }

    if let Some((entity, _)) = best_citizen {
        state.open = true;
        state.screen_pos = screen_pos;
        state.target = Some(ContextTarget::Citizen { entity });
        return;
    }

    let cell = grid.get(gx, gy);

    // 2. Check for building
    if let Some(building_entity) = cell.building_id {
        // Check if it's a service building
        if let Ok((ent, service)) = services.get(building_entity) {
            state.open = true;
            state.screen_pos = screen_pos;
            state.target = Some(ContextTarget::Service {
                entity: ent,
                name: service.service_type.name().to_string(),
                grid_x: service.grid_x,
                grid_y: service.grid_y,
            });
            return;
        }

        // Check if it's a zoned building
        if let Ok((ent, building)) = buildings.get(building_entity) {
            state.open = true;
            state.screen_pos = screen_pos;
            state.target = Some(ContextTarget::Building {
                entity: ent,
                zone_type: building.zone_type,
                level: building.level,
                grid_x: building.grid_x,
                grid_y: building.grid_y,
            });
            return;
        }
    }

    // 3. Check for road
    if cell.cell_type == CellType::Road {
        // Try to find the road segment this cell belongs to
        let mut found_segment = None;
        for segment in &segments.segments {
            if segment.rasterized_cells.contains(&(gx, gy)) {
                found_segment = Some(segment.id);
                break;
            }
        }

        state.open = true;
        state.screen_pos = screen_pos;
        state.target = Some(ContextTarget::Road {
            grid_x: gx,
            grid_y: gy,
            segment_id: found_segment,
        });
        return;
    }

    // 4. Empty cell
    if cell.cell_type == CellType::Grass {
        state.open = true;
        state.screen_pos = screen_pos;
        state.target = Some(ContextTarget::Empty {
            grid_x: gx,
            grid_y: gy,
            zone_type: cell.zone,
        });
    }
}
