//! Action executor system — drains the [`ActionQueue`] each fixed-update tick
//! and applies every queued [`GameAction`] to the world, recording results in
//! the [`ActionResultLog`].
//!
//! Each action variant has a dedicated, minimal execution function that
//! validates inputs, mutates the grid/resources, and returns an
//! [`ActionResult`].

use std::collections::HashMap;

use bevy::prelude::*;

use crate::budget::ExtendedBudget;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::roads::RoadNetwork;
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;
use crate::utilities::{UtilitySource, UtilityType};
use crate::zones::is_adjacent_to_road;

use super::result_log::ActionResultLog;
use super::{ActionError, ActionQueue, ActionResult, GameAction};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Drains all pending actions from the queue and executes them in order.
#[allow(clippy::too_many_arguments)]
pub fn execute_queued_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    mut log: ResMut<ActionResultLog>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut budget: ResMut<CityBudget>,
    mut extended: ResMut<ExtendedBudget>,
    mut clock: ResMut<GameClock>,
) {
    let actions = queue.drain();
    for queued in actions {
        let result = execute_single(
            &queued.action,
            &mut commands,
            &mut grid,
            &mut roads,
            &mut budget,
            &mut extended,
            &mut clock,
        );
        log.push(queued.action, result);
    }
}

// ---------------------------------------------------------------------------
// Dispatcher
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn execute_single(
    action: &GameAction,
    commands: &mut Commands,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    budget: &mut CityBudget,
    extended: &mut ExtendedBudget,
    clock: &mut GameClock,
) -> ActionResult {
    match action {
        GameAction::PlaceRoadLine {
            start,
            end,
            road_type,
        } => execute_place_road_line(*start, *end, *road_type, grid, roads, budget),
        GameAction::ZoneRect {
            min,
            max,
            zone_type,
        } => execute_zone_rect(*min, *max, *zone_type, grid),
        GameAction::PlaceUtility { pos, utility_type } => {
            execute_place_utility(*pos, *utility_type, commands, grid, budget)
        }
        GameAction::PlaceService { pos, service_type } => {
            execute_place_service(*pos, *service_type, commands, grid, budget)
        }
        GameAction::BulldozeRect { min, max } => execute_bulldoze_rect(*min, *max, grid, budget),
        GameAction::SetTaxRates {
            residential,
            commercial,
            industrial,
            office,
        } => execute_set_tax_rates(*residential, *commercial, *industrial, *office, extended),
        GameAction::SetSpeed { speed } => execute_set_speed(*speed, clock),
        GameAction::SetPaused { paused } => execute_set_paused(*paused, clock),
        GameAction::Save { .. } | GameAction::Load { .. } => {
            // Stub — save/load are handled by a different pipeline.
            ActionResult::Success
        }
        GameAction::NewGame { .. } => {
            // NewGame is handled by the app state machine, not here.
            ActionResult::Success
        }
    }
}

// ---------------------------------------------------------------------------
// Execution functions
// ---------------------------------------------------------------------------

fn bounds_check(x: u32, y: u32) -> Result<(usize, usize), ActionResult> {
    let xu = x as usize;
    let yu = y as usize;
    if xu >= GRID_WIDTH || yu >= GRID_HEIGHT {
        Err(ActionResult::Error(ActionError::OutOfBounds))
    } else {
        Ok((xu, yu))
    }
}

/// Place a straight line of road cells from `start` to `end`.
fn execute_place_road_line(
    start: (u32, u32),
    end: (u32, u32),
    road_type: RoadType,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    budget: &mut CityBudget,
) -> ActionResult {
    let (x0, y0) = match bounds_check(start.0, start.1) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let (x1, y1) = match bounds_check(end.0, end.1) {
        Ok(v) => v,
        Err(e) => return e,
    };

    // Collect cells along the line (Bresenham-style simple walk for axis-
    // aligned or diagonal lines).
    let cells = bresenham_cells(x0, y0, x1, y1);

    // Pre-validate costs
    let per_cell_cost = road_type.cost();
    let total_cost = per_cell_cost * cells.len() as f64;
    if budget.treasury < total_cost {
        return ActionResult::Error(ActionError::InsufficientFunds);
    }

    // Check all cells are placeable
    for &(cx, cy) in &cells {
        let cell = grid.get(cx, cy);
        if cell.cell_type == CellType::Water {
            return ActionResult::Error(ActionError::BlockedByWater);
        }
    }

    // Place roads
    let mut placed = 0u64;
    for &(cx, cy) in &cells {
        if roads.place_road_typed(grid, cx, cy, road_type) {
            placed += 1;
        }
    }

    budget.treasury -= per_cell_cost * placed as f64;
    ActionResult::Success
}

/// Zone a rectangular area. Only grass cells adjacent to a road are zoned.
/// Returns a warning if any cells with a different zone type were overwritten.
fn execute_zone_rect(
    min: (u32, u32),
    max: (u32, u32),
    zone_type: ZoneType,
    grid: &mut WorldGrid,
) -> ActionResult {
    let (x0, y0) = match bounds_check(min.0, min.1) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let (x1, y1) = match bounds_check(max.0, max.1) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let lx = x0.min(x1);
    let ly = y0.min(y1);
    let hx = x0.max(x1);
    let hy = y0.max(y1);

    // First pass: count existing zones that will be overwritten
    let mut overwritten: HashMap<ZoneType, u32> = HashMap::new();
    for y in ly..=hy {
        for x in lx..=hx {
            if !grid.in_bounds(x, y) {
                continue;
            }
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Grass || cell.building_id.is_some() {
                continue;
            }
            if !is_adjacent_to_road(grid, x, y) {
                continue;
            }
            // This cell will be zoned — check if it already has a different zone
            let existing = cell.zone;
            if existing != ZoneType::None && existing != zone_type {
                *overwritten.entry(existing).or_insert(0) += 1;
            }
        }
    }

    // Second pass: apply the zone
    for y in ly..=hy {
        for x in lx..=hx {
            if !grid.in_bounds(x, y) {
                continue;
            }
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Grass || cell.building_id.is_some() {
                continue;
            }
            if !is_adjacent_to_road(grid, x, y) {
                continue;
            }
            grid.get_mut(x, y).zone = zone_type;
        }
    }

    if overwritten.is_empty() {
        ActionResult::Success
    } else {
        let mut parts: Vec<String> = overwritten
            .iter()
            .map(|(zt, count)| format!("{} {:?}", count, zt))
            .collect();
        parts.sort();
        let warning = format!("Overwrote existing zones: {}", parts.join(", "));
        ActionResult::SuccessWithWarning(warning)
    }
}

/// Place a utility source: validate, deduct cost, spawn entity, set grid cell.
fn execute_place_utility(
    pos: (u32, u32),
    utility_type: UtilityType,
    commands: &mut Commands,
    grid: &mut WorldGrid,
    budget: &mut CityBudget,
) -> ActionResult {
    let cost = crate::services::utility_cost(utility_type);
    let (x, y) = match validate_building_placement(pos, cost, grid, budget) {
        Ok(coords) => coords,
        Err(result) => return result,
    };

    budget.treasury -= cost;

    let range = crate::services::utility_range(utility_type);
    let entity = commands
        .spawn(UtilitySource {
            utility_type,
            grid_x: x,
            grid_y: y,
            range,
        })
        .id();

    grid.get_mut(x, y).building_id = Some(entity);
    ActionResult::Success
}

/// Place a service building: validate, deduct cost, spawn entity, set grid
/// cells for the full footprint.
fn execute_place_service(
    pos: (u32, u32),
    service_type: ServiceType,
    commands: &mut Commands,
    grid: &mut WorldGrid,
    budget: &mut CityBudget,
) -> ActionResult {
    let cost = ServiceBuilding::cost(service_type);
    let (x, y) = match validate_building_placement(pos, cost, grid, budget) {
        Ok(coords) => coords,
        Err(result) => return result,
    };

    // Check the full footprint is clear
    let (fw, fh) = ServiceBuilding::footprint(service_type);
    for dy in 0..fh {
        for dx in 0..fw {
            let cx = x + dx;
            let cy = y + dy;
            if !grid.in_bounds(cx, cy) {
                return ActionResult::Error(ActionError::OutOfBounds);
            }
            // Skip (x, y) itself — already validated above
            if dx == 0 && dy == 0 {
                continue;
            }
            let cell = grid.get(cx, cy);
            if cell.cell_type == CellType::Water {
                return ActionResult::Error(ActionError::BlockedByWater);
            }
            if cell.building_id.is_some() {
                return ActionResult::Error(ActionError::AlreadyExists);
            }
        }
    }

    budget.treasury -= cost;

    let entity = commands
        .spawn(ServiceBuilding {
            service_type,
            grid_x: x,
            grid_y: y,
            radius: ServiceBuilding::coverage_radius(service_type),
        })
        .id();

    // Mark all footprint cells
    for dy in 0..fh {
        for dx in 0..fw {
            grid.get_mut(x + dx, y + dy).building_id = Some(entity);
        }
    }
    ActionResult::Success
}

/// Shared placement validator for buildings. Returns grid coordinates on
/// success, or an `ActionResult` error on failure.
fn validate_building_placement(
    pos: (u32, u32),
    cost: f64,
    grid: &WorldGrid,
    budget: &CityBudget,
) -> Result<(usize, usize), ActionResult> {
    let (x, y) = bounds_check(pos.0, pos.1)?;

    if budget.treasury < cost {
        return Err(ActionResult::Error(ActionError::InsufficientFunds));
    }

    let cell = grid.get(x, y);
    if cell.cell_type == CellType::Water {
        return Err(ActionResult::Error(ActionError::BlockedByWater));
    }
    if cell.building_id.is_some() {
        return Err(ActionResult::Error(ActionError::AlreadyExists));
    }

    Ok((x, y))
}

/// Clear all cells in a rectangle, refunding road costs.
fn execute_bulldoze_rect(
    min: (u32, u32),
    max: (u32, u32),
    grid: &mut WorldGrid,
    budget: &mut CityBudget,
) -> ActionResult {
    let (x0, y0) = match bounds_check(min.0, min.1) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let (x1, y1) = match bounds_check(max.0, max.1) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let lx = x0.min(x1);
    let ly = y0.min(y1);
    let hx = x0.max(x1);
    let hy = y0.max(y1);

    let mut refund = 0.0_f64;
    for y in ly..=hy {
        for x in lx..=hx {
            if !grid.in_bounds(x, y) {
                continue;
            }
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road {
                refund += crate::bulldoze_refund::refund_for_road(cell.road_type);
                let c = grid.get_mut(x, y);
                c.cell_type = CellType::Grass;
                c.road_type = RoadType::Local;
                c.zone = ZoneType::None;
            } else if cell.cell_type == CellType::Grass {
                let c = grid.get_mut(x, y);
                c.zone = ZoneType::None;
                c.building_id = None;
            }
        }
    }
    budget.treasury += refund;
    ActionResult::Success
}

/// Update per-zone tax rates.
fn execute_set_tax_rates(
    residential: f32,
    commercial: f32,
    industrial: f32,
    office: f32,
    extended: &mut ExtendedBudget,
) -> ActionResult {
    extended.zone_taxes.residential = residential.clamp(0.0, 1.0);
    extended.zone_taxes.commercial = commercial.clamp(0.0, 1.0);
    extended.zone_taxes.industrial = industrial.clamp(0.0, 1.0);
    extended.zone_taxes.office = office.clamp(0.0, 1.0);
    ActionResult::Success
}

/// Set game speed (clamped 1..=3).
fn execute_set_speed(speed: u32, clock: &mut GameClock) -> ActionResult {
    clock.speed = (speed.clamp(1, 3)) as f32;
    ActionResult::Success
}

/// Pause or unpause the game clock.
fn execute_set_paused(paused: bool, clock: &mut GameClock) -> ActionResult {
    clock.paused = paused;
    ActionResult::Success
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Bresenham line rasterization returning all cells along the line.
fn bresenham_cells(x0: usize, y0: usize, x1: usize, y1: usize) -> Vec<(usize, usize)> {
    let mut cells = Vec::new();
    let dx = (x1 as isize - x0 as isize).abs();
    let dy = -(y1 as isize - y0 as isize).abs();
    let sx: isize = if x0 < x1 { 1 } else { -1 };
    let sy: isize = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cx = x0 as isize;
    let mut cy = y0 as isize;

    loop {
        cells.push((cx as usize, cy as usize));
        if cx == x1 as isize && cy == y1 as isize {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            err += dx;
            cy += sy;
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bresenham_horizontal() {
        let cells = bresenham_cells(5, 10, 10, 10);
        assert_eq!(cells.len(), 6);
        assert_eq!(cells[0], (5, 10));
        assert_eq!(cells[5], (10, 10));
    }

    #[test]
    fn bresenham_vertical() {
        let cells = bresenham_cells(3, 0, 3, 4);
        assert_eq!(cells.len(), 5);
        for (i, &(x, y)) in cells.iter().enumerate() {
            assert_eq!(x, 3);
            assert_eq!(y, i);
        }
    }

    #[test]
    fn bresenham_single_cell() {
        let cells = bresenham_cells(7, 7, 7, 7);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0], (7, 7));
    }
}
