use bevy::prelude::*;

use simulation::economy::CityBudget;
use simulation::grid::{CellType, RoadType, WorldGrid, ZoneType};
use simulation::roads::RoadNetwork;
use simulation::services::{self, ServiceType};
use simulation::undo_redo::CityAction;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::utilities::UtilityType;

use super::types::StatusMessage;

// ---------------------------------------------------------------------------
// Shared helpers for formatting money messages
// ---------------------------------------------------------------------------

fn not_enough_funds_msg(need: f64, have: f64) -> String {
    format!("Not enough funds (need ${:.0}, have ${:.0})", need, have)
}

// ---------------------------------------------------------------------------
// Helper: road placement with cost
// ---------------------------------------------------------------------------

/// Diagnose why a road cannot be placed at (gx, gy) and set a status message.
fn diagnose_road_failure(
    grid: &WorldGrid,
    gx: usize,
    gy: usize,
    status: &mut StatusMessage,
) {
    if !grid.in_bounds(gx, gy) {
        status.set("Cannot place road outside map bounds", true);
        return;
    }
    let cell = grid.get(gx, gy);
    if cell.cell_type == CellType::Water {
        status.set("Cannot place road on water", true);
    } else if cell.cell_type == CellType::Road {
        status.set("Road already exists here", true);
    } else {
        status.set("Cannot place road here", true);
    }
}

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
            if buttons.just_pressed(MouseButton::Left) {
                diagnose_road_failure(grid, gx, gy, status);
            }
            false
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            status.set(not_enough_funds_msg(cost, budget.treasury), true);
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
    AlreadyZoned,
    OccupiedByBuilding,
}

pub(crate) fn try_zone(
    grid: &WorldGrid,
    x: usize,
    y: usize,
    zone: ZoneType,
    ugb: &UrbanGrowthBoundary,
) -> ZoneResult {
    let cell = grid.get(x, y);
    if cell.building_id.is_some() {
        return ZoneResult::OccupiedByBuilding;
    }
    if cell.cell_type != CellType::Grass {
        return ZoneResult::InvalidCell;
    }
    if cell.zone == zone {
        return ZoneResult::AlreadyZoned;
    }
    // Urban Growth Boundary: block zoning outside the boundary (ZONE-009).
    if !ugb.allows_zoning(x, y) {
        return ZoneResult::OutsideUgb;
    }
    let (n4, n4c) = grid.neighbors4(x, y);
    let has_road = n4[..n4c]
        .iter()
        .any(|(nx, ny)| grid.get(*nx, *ny).cell_type == CellType::Road);
    if !has_road {
        return ZoneResult::NotAdjacentToRoad;
    }
    ZoneResult::Success
}

fn zone_failure_message(result: &ZoneResult) -> Option<&'static str> {
    match result {
        ZoneResult::NotAdjacentToRoad => {
            Some("Zone must be adjacent to a road")
        }
        ZoneResult::OutsideUgb => {
            Some("Cannot zone outside urban growth boundary")
        }
        ZoneResult::InvalidCell => {
            Some("Cannot zone here — invalid terrain")
        }
        ZoneResult::AlreadyZoned => {
            Some("Already zoned as this type")
        }
        ZoneResult::OccupiedByBuilding => {
            Some("Cannot zone — cell occupied by a building")
        }
        ZoneResult::Success => None,
    }
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
                let result = try_zone(grid, ux, uy, zone, ugb);
                if let Some(msg) = zone_failure_message(&result) {
                    status.set(msg, true);
                }
            }
        }
        return Vec::new();
    }

    let total_cost = valid_cells.len() as f64 * cost_per_cell;
    if budget.treasury < total_cost {
        if buttons.just_pressed(MouseButton::Left) {
            status.set(
                not_enough_funds_msg(total_cost, budget.treasury),
                true,
            );
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
// Helper: diagnose building/utility placement failure
// ---------------------------------------------------------------------------

/// Check a single cell and return a specific error message if it cannot
/// accept a building placement.
fn diagnose_cell_failure(grid: &WorldGrid, gx: usize, gy: usize) -> &'static str {
    if !grid.in_bounds(gx, gy) {
        return "Cannot place outside map bounds";
    }
    let cell = grid.get(gx, gy);
    if cell.building_id.is_some() {
        return "Overlaps existing building";
    }
    match cell.cell_type {
        CellType::Water => "Cannot place on water",
        CellType::Road => "Cannot place on a road",
        _ => "Cannot place here — invalid terrain",
    }
}

/// Diagnose why a multi-cell service footprint cannot be placed and return
/// the most relevant error message.
fn diagnose_service_failure(
    grid: &WorldGrid,
    gx: usize,
    gy: usize,
    fw: usize,
    fh: usize,
) -> &'static str {
    for dy in 0..fh {
        for dx in 0..fw {
            let cx = gx + dx;
            let cy = gy + dy;
            if !grid.in_bounds(cx, cy) {
                return "Building footprint extends outside map bounds";
            }
            let cell = grid.get(cx, cy);
            if cell.building_id.is_some() {
                return "Overlaps existing building";
            }
            if cell.cell_type != CellType::Grass {
                return match cell.cell_type {
                    CellType::Water => "Cannot place on water",
                    CellType::Road => "Building footprint overlaps a road",
                    _ => "Cannot place here — invalid terrain",
                };
            }
        }
    }
    "Cannot place here"
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
            if buttons.just_pressed(MouseButton::Left) {
                let msg = diagnose_cell_failure(grid, gx, gy);
                status.set(msg, true);
            }
            false
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            status.set(not_enough_funds_msg(cost, budget.treasury), true);
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
            if buttons.just_pressed(MouseButton::Left) {
                let (fw, fh) = ServiceBuilding::footprint(service_type);
                let msg = diagnose_service_failure(grid, gx, gy, fw, fh);
                status.set(msg, true);
            }
            false
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            status.set(not_enough_funds_msg(cost, budget.treasury), true);
        }
        false
    }
}
