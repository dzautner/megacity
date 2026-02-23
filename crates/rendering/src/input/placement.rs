use bevy::prelude::*;

use simulation::economy::CityBudget;
use simulation::grid::{RoadType, WorldGrid, ZoneType};
use simulation::roads::RoadNetwork;
use simulation::services::{self, ServiceType};
use simulation::undo_redo::CityAction;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::utilities::UtilityType;

use super::types::StatusMessage;

// ---------------------------------------------------------------------------
// Helper: road placement with cost
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn place_road_if_affordable(
    roads: &mut RoadNetwork,
    grid: &mut WorldGrid,
    budget: &mut CityBudget,
    status: &mut StatusMessage,
    buttons: &ButtonInput<MouseButton>,
    road_type: RoadType,
    gx: usize,
    gy: usize,
    action_writer: &mut EventWriter<CityAction>,
) -> bool {
    let cost = road_type.cost();
    if budget.treasury >= cost {
        if roads.place_road_typed(grid, gx, gy, road_type) {
            budget.treasury -= cost;
            action_writer.send(CityAction::PlaceGridRoad {
                x: gx,
                y: gy,
                road_type,
                cost,
            });
            true
        } else {
            false
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            status.set("Not enough money", true);
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Helper: zone placement
// ---------------------------------------------------------------------------

pub(crate) enum ZoneResult {
    Success,
    NotAdjacentToRoad,
    OutsideUgb,
    InvalidCell,
}

pub(crate) fn try_zone(
    grid: &WorldGrid,
    x: usize,
    y: usize,
    zone: ZoneType,
    ugb: &UrbanGrowthBoundary,
) -> ZoneResult {
    let cell = grid.get(x, y);
    if cell.cell_type != simulation::grid::CellType::Grass {
        return ZoneResult::InvalidCell;
    }
    if cell.zone == zone {
        return ZoneResult::InvalidCell;
    }
    // Urban Growth Boundary: block zoning outside the boundary (ZONE-009).
    if !ugb.allows_zoning(x, y) {
        return ZoneResult::OutsideUgb;
    }
    let (n4, n4c) = grid.neighbors4(x, y);
    let has_road = n4[..n4c]
        .iter()
        .any(|(nx, ny)| grid.get(*nx, *ny).cell_type == simulation::grid::CellType::Road);
    if !has_road {
        return ZoneResult::NotAdjacentToRoad;
    }
    ZoneResult::Success
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_zone_brush(
    grid: &mut WorldGrid,
    status: &mut StatusMessage,
    budget: &mut simulation::economy::CityBudget,
    buttons: &ButtonInput<MouseButton>,
    cx: i32,
    cy: i32,
    zone: ZoneType,
    ugb: &UrbanGrowthBoundary,
    brush: &crate::zone_brush_preview::ZoneBrushSize,
) -> Vec<(usize, usize)> {
    let half = brush.half_extent;
    let cost_per_cell = crate::zone_brush_preview::ZONE_COST_PER_CELL;

    // Collect valid cells in the brush area
    let mut valid_cells = Vec::new();
    for dy in -half..=half {
        for dx in -half..=half {
            let gx = cx + dx;
            let gy = cy + dy;
            if gx >= 0 && gy >= 0 {
                let ux = gx as usize;
                let uy = gy as usize;
                if grid.in_bounds(ux, uy) {
                    let result = try_zone(grid, ux, uy, zone, ugb);
                    if matches!(result, ZoneResult::Success) {
                        valid_cells.push((ux, uy));
                    }
                }
            }
        }
    }

    if valid_cells.is_empty() {
        if buttons.just_pressed(MouseButton::Left) {
            // Show reason for center cell
            let ux = cx as usize;
            let uy = cy as usize;
            if grid.in_bounds(ux, uy) {
                match try_zone(grid, ux, uy, zone, ugb) {
                    ZoneResult::NotAdjacentToRoad => {
                        status.set("Zone must be adjacent to road", true);
                    }
                    ZoneResult::OutsideUgb => {
                        status.set("Cannot zone outside urban growth boundary", true);
                    }
                    _ => {}
                }
            }
        }
        return Vec::new();
    }

    let total_cost = valid_cells.len() as f64 * cost_per_cell;
    if budget.treasury < total_cost {
        if buttons.just_pressed(MouseButton::Left) {
            status.set("Not enough money", true);
        }
        return Vec::new();
    }

    // Apply zones to all valid cells
    budget.treasury -= total_cost;
    for (gx, gy) in &valid_cells {
        grid.get_mut(*gx, *gy).zone = zone;
    }
    valid_cells
}

// ---------------------------------------------------------------------------
// Helper: utility placement
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn place_utility_if_affordable(
    commands: &mut Commands,
    grid: &mut WorldGrid,
    budget: &mut CityBudget,
    status: &mut StatusMessage,
    buttons: &ButtonInput<MouseButton>,
    utility_type: UtilityType,
    gx: usize,
    gy: usize,
    action_writer: &mut EventWriter<CityAction>,
) -> bool {
    let cost = services::utility_cost(utility_type);
    if budget.treasury >= cost {
        if services::place_utility_source(commands, grid, utility_type, gx, gy) {
            budget.treasury -= cost;
            action_writer.send(CityAction::PlaceUtility {
                utility_type,
                grid_x: gx,
                grid_y: gy,
                cost,
            });
            true
        } else {
            false
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            status.set("Not enough money", true);
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Helper: service placement
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn place_service_if_affordable(
    commands: &mut Commands,
    grid: &mut WorldGrid,
    budget: &mut CityBudget,
    status: &mut StatusMessage,
    buttons: &ButtonInput<MouseButton>,
    service_type: ServiceType,
    gx: usize,
    gy: usize,
    action_writer: &mut EventWriter<CityAction>,
) -> bool {
    use simulation::services::ServiceBuilding;
    let cost = ServiceBuilding::cost(service_type);
    if budget.treasury >= cost {
        if services::place_service(commands, grid, service_type, gx, gy) {
            budget.treasury -= cost;
            action_writer.send(CityAction::PlaceService {
                service_type,
                grid_x: gx,
                grid_y: gy,
                cost,
            });
            true
        } else {
            false
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            status.set("Not enough money", true);
        }
        false
    }
}
