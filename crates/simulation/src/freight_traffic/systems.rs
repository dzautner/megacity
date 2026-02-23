//! System functions for freight traffic simulation.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{WorldGrid, ZoneType};
use crate::pathfinding_sys::nearest_road_grid;
use crate::road_graph_csr::{csr_find_path_with_traffic, CsrGraph};
use crate::road_maintenance::RoadConditionGrid;
use crate::traffic::TrafficGrid;
use crate::TickCounter;

use super::constants::*;
use super::types::{FreightTrafficState, FreightTruck};

/// Compute freight demand from industrial and commercial buildings.
/// Runs on the slow tick timer interval.
pub fn compute_freight_demand(
    tick: Res<TickCounter>,
    buildings: Query<&Building>,
    mut freight: ResMut<FreightTrafficState>,
) {
    if !tick.0.is_multiple_of(FREIGHT_GENERATION_INTERVAL) {
        return;
    }

    let mut ind_demand = 0.0f32;
    let mut com_demand = 0.0f32;

    for building in &buildings {
        if building.occupants == 0 {
            continue;
        }
        match building.zone_type {
            ZoneType::Industrial => {
                ind_demand += building.occupants as f32 * INDUSTRIAL_FREIGHT_RATE;
            }
            ZoneType::CommercialLow | ZoneType::CommercialHigh => {
                com_demand += building.occupants as f32 * COMMERCIAL_FREIGHT_RATE;
            }
            _ => {}
        }
    }

    freight.industrial_demand = ind_demand;
    freight.commercial_demand = com_demand;
}

/// Generate freight trips: match industrial origins to commercial destinations.
/// Spawns trucks with pre-computed A* routes on the road network.
#[allow(clippy::too_many_arguments)]
pub fn generate_freight_trips(
    tick: Res<TickCounter>,
    grid: Res<WorldGrid>,
    csr: Res<CsrGraph>,
    traffic: Res<TrafficGrid>,
    buildings: Query<&Building>,
    mut freight: ResMut<FreightTrafficState>,
) {
    if !tick.0.is_multiple_of(FREIGHT_GENERATION_INTERVAL) {
        return;
    }

    if freight.trucks.len() >= MAX_FREIGHT_TRUCKS {
        return;
    }

    // Collect industrial origins and commercial destinations
    let mut origins: Vec<(usize, usize)> = Vec::new();
    let mut destinations: Vec<(usize, usize)> = Vec::new();

    for building in &buildings {
        if building.occupants == 0 {
            continue;
        }
        match building.zone_type {
            ZoneType::Industrial => {
                origins.push((building.grid_x, building.grid_y));
            }
            ZoneType::CommercialLow | ZoneType::CommercialHigh => {
                destinations.push((building.grid_x, building.grid_y));
            }
            _ => {}
        }
    }

    if origins.is_empty() || destinations.is_empty() {
        return;
    }

    // Determine how many trips to generate this cycle based on demand
    let demand = freight.industrial_demand.min(freight.commercial_demand);
    let trips_to_generate = (demand as usize)
        .min(MAX_TRIPS_PER_CYCLE)
        .min(MAX_FREIGHT_TRUCKS - freight.trucks.len());

    if trips_to_generate == 0 {
        return;
    }

    // Use a simple hash-based matching: pair origins with nearest destinations
    let mut generated = 0usize;
    for &(ox, oy) in origins.iter() {
        if generated >= trips_to_generate {
            break;
        }

        // Find nearest commercial destination within range
        let dest = find_nearest_destination(&destinations, ox, oy, MAX_FREIGHT_DISTANCE);
        let Some((dx, dy)) = dest else {
            continue;
        };

        // Resolve to road nodes
        let start = nearest_road_grid(&grid, ox, oy);
        let goal = nearest_road_grid(&grid, dx, dy);

        if let (Some(start_node), Some(goal_node)) = (start, goal) {
            // Compute route using A* pathfinding
            if let Some(route) =
                csr_find_path_with_traffic(&csr, start_node, goal_node, &grid, &traffic)
            {
                freight.trucks.push(FreightTruck {
                    route,
                    current_index: 0,
                    origin: (ox, oy),
                    destination: (dx, dy),
                });
                freight.trips_generated += 1;
                generated += 1;
            }
        }
    }
}

/// Move freight trucks along their routes and apply traffic/wear effects.
pub fn move_freight_trucks(
    tick: Res<TickCounter>,
    mut freight: ResMut<FreightTrafficState>,
    mut traffic: ResMut<TrafficGrid>,
    mut condition_grid: ResMut<RoadConditionGrid>,
) {
    if !tick.0.is_multiple_of(FREIGHT_MOVE_INTERVAL) {
        return;
    }

    // Move each truck and apply effects
    for truck in &mut freight.trucks {
        // Apply traffic density and road wear at current position before moving
        if let Some(pos) = truck.current_position() {
            let x = pos.0.min(GRID_WIDTH - 1);
            let y = pos.1.min(GRID_HEIGHT - 1);

            // Add truck equivalence to traffic density
            let equiv = TRUCK_EQUIVALENCE_FACTOR as u16;
            let current_density = traffic.get(x, y);
            traffic.set(x, y, current_density.saturating_add(equiv));

            // Apply extra road wear from heavy truck
            let current_cond = condition_grid.get(x, y);
            if current_cond > 0 {
                condition_grid.set(x, y, current_cond.saturating_sub(TRUCK_WEAR_PER_VISIT));
            }
        }

        // Advance truck along route
        truck.advance(TRUCK_SPEED);
    }

    // Count completed trips before removing
    let completed_before = freight.trucks.len();
    freight.trucks.retain(|t| !t.is_arrived());
    let completed_now = completed_before - freight.trucks.len();
    freight.trips_completed += completed_now as u64;
}

/// Update freight satisfaction ratio based on demand vs. active deliveries.
pub fn update_freight_satisfaction(
    tick: Res<TickCounter>,
    mut freight: ResMut<FreightTrafficState>,
) {
    if !tick.0.is_multiple_of(FREIGHT_GENERATION_INTERVAL) {
        return;
    }

    let total_demand = freight.industrial_demand + freight.commercial_demand;
    if total_demand < 0.01 {
        freight.satisfaction = 1.0;
        return;
    }

    // Satisfaction based on active trucks vs. demand
    let supply = freight.trucks.len() as f32;
    let ratio = (supply / total_demand.max(1.0)).min(1.0);

    // Smooth the satisfaction value to avoid oscillation
    freight.satisfaction = freight.satisfaction * 0.8 + ratio * 0.2;
}

/// Find the nearest destination within `max_dist` Manhattan distance.
pub(crate) fn find_nearest_destination(
    destinations: &[(usize, usize)],
    from_x: usize,
    from_y: usize,
    max_dist: i32,
) -> Option<(usize, usize)> {
    destinations
        .iter()
        .filter_map(|&(x, y)| {
            let dist = (x as i32 - from_x as i32).abs() + (y as i32 - from_y as i32).abs();
            if dist <= max_dist {
                Some(((x, y), dist))
            } else {
                None
            }
        })
        .min_by_key(|&(_, dist)| dist)
        .map(|(pos, _)| pos)
}
