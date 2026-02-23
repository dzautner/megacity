//! Tram transit ECS systems, Saveable implementations, and plugin registration.
//!
//! Contains the Bevy systems that drive tram line activation, vehicle movement,
//! cost/revenue application, and citizen arrival simulation. Also includes
//! the `Saveable` implementations and the `TramTransitPlugin`.

use bevy::prelude::*;

use crate::economy::CityBudget;
use crate::grid::WorldGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

use super::state::{
    manhattan_distance, TramLineId, TramStopId, TramTransitState, TramTransitStats, TramVehicle,
    DEPOT_WEEKLY_COST, DWELL_TICKS, FARE_PER_RIDE, LINE_WEEKLY_COST, TRAMS_PER_LINE, TRAM_CAPACITY,
    TRAM_SPEED_CELLS_PER_TICK,
};

// =============================================================================
// Systems
// =============================================================================

/// System: check depot coverage and activate/deactivate tram lines.
///
/// A tram line is active if at least one TramDepot service building exists within
/// coverage radius of any stop on the line.
pub fn update_tram_lines(mut transit: ResMut<TramTransitState>, services: Query<&ServiceBuilding>) {
    let depots: Vec<(usize, usize, f32)> = services
        .iter()
        .filter(|s| s.service_type == ServiceType::TramDepot)
        .map(|s| (s.grid_x, s.grid_y, s.radius))
        .collect();

    // Pre-collect stop coordinates to avoid borrow conflict
    let stop_coords: Vec<(TramStopId, usize, usize)> = transit
        .stops
        .iter()
        .map(|s| (s.id, s.grid_x, s.grid_y))
        .collect();

    let mut deactivated_lines: Vec<TramLineId> = Vec::new();

    for line in &mut transit.lines {
        let was_active = line.active;
        line.active = false;

        // A line is active if any of its stops is within depot coverage
        'stop_loop: for stop_id in &line.stop_ids {
            if let Some(&(_, sx, sy)) = stop_coords.iter().find(|(id, _, _)| id == stop_id) {
                for &(dx, dy, radius) in &depots {
                    let dist = manhattan_distance(sx, sy, dx, dy) as f32;
                    if dist * crate::config::CELL_SIZE <= radius {
                        line.active = true;
                        break 'stop_loop;
                    }
                }
            }
        }

        if was_active && !line.active {
            deactivated_lines.push(line.id);
        }
    }

    // Remove tram vehicles for deactivated lines
    for line_id in deactivated_lines {
        transit.trams.retain(|t| t.line_id != line_id);
    }

    // Spawn tram vehicles on active lines that don't have enough
    let stop_positions: Vec<(TramStopId, f32, f32)> = transit
        .stops
        .iter()
        .map(|s| (s.id, s.grid_x as f32, s.grid_y as f32))
        .collect();

    struct SpawnInfo {
        line_id: TramLineId,
        current_count: usize,
        target_count: usize,
        positions: Vec<(f32, f32)>,
    }

    let spawn_infos: Vec<SpawnInfo> = transit
        .lines
        .iter()
        .filter(|l| l.active && !l.stop_ids.is_empty())
        .map(|l| {
            let current_count = transit.trams.iter().filter(|t| t.line_id == l.id).count();
            let positions: Vec<(f32, f32)> = l
                .stop_ids
                .iter()
                .filter_map(|sid| {
                    stop_positions
                        .iter()
                        .find(|(id, _, _)| id == sid)
                        .map(|(_, x, y)| (*x, *y))
                })
                .collect();
            SpawnInfo {
                line_id: l.id,
                current_count,
                target_count: TRAMS_PER_LINE as usize,
                positions,
            }
        })
        .filter(|info| info.current_count < info.target_count)
        .collect();

    for info in spawn_infos {
        if info.positions.is_empty() {
            continue;
        }
        let num_stops = info.positions.len();
        for i in info.current_count..info.target_count {
            let stop_idx = i % num_stops;
            let (sx, sy) = info.positions[stop_idx];
            let next_idx = (stop_idx + 1) % num_stops;

            transit.trams.push(TramVehicle {
                line_id: info.line_id,
                next_stop_index: next_idx,
                grid_x: sx,
                grid_y: sy,
                passengers: 0,
                dwell_ticks: 0,
                at_stop: false,
            });
        }
    }

    // Move trams along their lines and handle passenger pickup/dropoff
    struct LineStopData {
        line_id: TramLineId,
        stops: Vec<(TramStopId, f32, f32)>,
    }

    let line_data: Vec<LineStopData> = transit
        .lines
        .iter()
        .filter(|l| l.active)
        .map(|l| {
            let stops: Vec<(TramStopId, f32, f32)> = l
                .stop_ids
                .iter()
                .filter_map(|sid| {
                    transit
                        .stops
                        .iter()
                        .find(|s| s.id == *sid)
                        .map(|s| (s.id, s.grid_x as f32, s.grid_y as f32))
                })
                .collect();
            LineStopData {
                line_id: l.id,
                stops,
            }
        })
        .collect();

    let waiting_counts: Vec<(TramStopId, u32)> =
        transit.stops.iter().map(|s| (s.id, s.waiting)).collect();

    let mut ridership_increments: Vec<(TramLineId, u32)> = Vec::new();
    let mut fare_revenue = 0.0_f64;
    let mut stop_waiting_decrements: Vec<(TramStopId, u32)> = Vec::new();

    for tram in &mut transit.trams {
        let Some(ld) = line_data.iter().find(|d| d.line_id == tram.line_id) else {
            continue;
        };
        if ld.stops.is_empty() {
            continue;
        }

        // Handle dwelling at stop
        if tram.at_stop {
            if tram.dwell_ticks > 0 {
                tram.dwell_ticks -= 1;
                continue;
            }
            tram.at_stop = false;
            tram.next_stop_index = (tram.next_stop_index + 1) % ld.stops.len();
        }

        // Move toward next stop
        let next = &ld.stops[tram.next_stop_index % ld.stops.len()];
        let dx = next.1 - tram.grid_x;
        let dy = next.2 - tram.grid_y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < TRAM_SPEED_CELLS_PER_TICK {
            // Arrived at stop
            tram.grid_x = next.1;
            tram.grid_y = next.2;
            tram.at_stop = true;
            tram.dwell_ticks = DWELL_TICKS;

            // Drop off passengers (some fraction disembark at each stop)
            let dropoff = (tram.passengers / 3).max(1).min(tram.passengers);
            tram.passengers = tram.passengers.saturating_sub(dropoff);

            // Pick up waiting passengers
            let stop_id = next.0;
            let waiting = waiting_counts
                .iter()
                .find(|(id, _)| *id == stop_id)
                .map(|(_, w)| *w)
                .unwrap_or(0);
            let space = TRAM_CAPACITY.saturating_sub(tram.passengers);
            let pickup = waiting.min(space);
            if pickup > 0 {
                tram.passengers += pickup;
                fare_revenue += pickup as f64 * FARE_PER_RIDE;
                stop_waiting_decrements.push((stop_id, pickup));
                ridership_increments.push((tram.line_id, pickup));
            }
        } else {
            // Move toward stop
            let norm = 1.0 / dist;
            tram.grid_x += dx * norm * TRAM_SPEED_CELLS_PER_TICK;
            tram.grid_y += dy * norm * TRAM_SPEED_CELLS_PER_TICK;
        }
    }

    // Apply waiting decrements
    for (stop_id, decrement) in stop_waiting_decrements {
        if let Some(stop) = transit.stops.iter_mut().find(|s| s.id == stop_id) {
            stop.waiting = stop.waiting.saturating_sub(decrement);
        }
    }

    // Apply ridership increments
    for (line_id, count) in ridership_increments {
        if let Some(line) = transit.lines.iter_mut().find(|l| l.id == line_id) {
            line.total_ridership += count as u64;
            line.period_ridership += count;
        }
    }

    transit.period_fare_revenue += fare_revenue;
    transit.cumulative_ridership = transit.total_ridership();
}

/// System: apply tram transit costs and revenue to the city budget.
/// Runs on slow tick (every 100 ticks).
pub fn update_tram_costs(
    timer: Res<SlowTickTimer>,
    clock: Res<GameClock>,
    mut transit: ResMut<TramTransitState>,
    mut budget: ResMut<CityBudget>,
    services: Query<&ServiceBuilding>,
) {
    if !timer.should_run() {
        return;
    }

    // Apply costs weekly (every 7 days)
    if clock.day <= transit.last_cost_day + 7 {
        return;
    }
    transit.last_cost_day = clock.day;

    // Line operating costs
    let line_cost = transit.lines.iter().filter(|l| l.active).count() as f64 * LINE_WEEKLY_COST;

    // Depot maintenance costs
    let depot_count = services
        .iter()
        .filter(|s| s.service_type == ServiceType::TramDepot)
        .count();
    let depot_cost = depot_count as f64 * DEPOT_WEEKLY_COST;

    let total_cost = line_cost + depot_cost;
    transit.period_operating_cost = total_cost;

    // Apply to budget: deduct costs, add fare revenue
    budget.treasury -= total_cost;
    budget.treasury += transit.period_fare_revenue;

    // Reset period counters
    transit.period_fare_revenue = 0.0;
    for line in &mut transit.lines {
        line.period_ridership = 0;
    }
}

/// System: simulate citizens arriving at tram stops (simplified model).
///
/// Each slow tick, a fraction of citizens near active tram stops are added
/// as waiting passengers. Trams have higher demand than buses due to
/// higher capacity and reliability.
pub fn tram_depot_coverage(
    timer: Res<SlowTickTimer>,
    mut transit: ResMut<TramTransitState>,
    grid: Res<WorldGrid>,
    mut stats: ResMut<TramTransitStats>,
) {
    if !timer.should_run() {
        return;
    }

    // Pre-collect active line stop IDs
    let active_stop_ids: Vec<TramStopId> = transit
        .lines
        .iter()
        .filter(|l| l.active)
        .flat_map(|l| l.stop_ids.iter().copied())
        .collect();

    for stop in &mut transit.stops {
        let on_active_line = active_stop_ids.contains(&stop.id);

        if !on_active_line {
            stop.waiting = 0;
            continue;
        }

        // Count nearby zoned cells as demand proxy
        let mut demand = 0u32;
        let range = 6i32; // Slightly wider catchment than bus stops
        for dy in -range..=range {
            for dx in -range..=range {
                let nx = stop.grid_x as i32 + dx;
                let ny = stop.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && grid.in_bounds(nx as usize, ny as usize) {
                    let cell = grid.get(nx as usize, ny as usize);
                    if cell.zone != crate::grid::ZoneType::None {
                        demand += 1;
                    }
                }
            }
        }

        // Higher demand multiplier than buses (trams attract more riders)
        let new_waiting = (demand / 8).min(8);
        stop.waiting = (stop.waiting + new_waiting).min(TRAM_CAPACITY * 2);
    }

    // Update aggregate stats
    stats.active_lines = transit.active_line_count() as u32;
    stats.total_stops = transit.stops.len() as u32;
    stats.daily_ridership = transit
        .lines
        .iter()
        .map(|l| l.period_ridership)
        .sum::<u32>();
    stats.monthly_operating_cost = transit.period_operating_cost * 4.0; // ~4 weeks
    stats.monthly_fare_revenue = transit.period_fare_revenue * 4.0;
    stats.cumulative_ridership = transit.cumulative_ridership;
}

// =============================================================================
// Saveable implementations
// =============================================================================

impl crate::Saveable for TramTransitState {
    const SAVE_KEY: &'static str = "tram_transit";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.stops.is_empty() && self.lines.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

impl crate::Saveable for TramTransitStats {
    const SAVE_KEY: &'static str = "tram_transit_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.active_lines == 0 && self.total_stops == 0 && self.cumulative_ridership == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct TramTransitPlugin;

impl Plugin for TramTransitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TramTransitState>()
            .init_resource::<TramTransitStats>()
            .add_systems(
                FixedUpdate,
                (update_tram_lines, update_tram_costs, tram_depot_coverage)
                    .chain()
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<TramTransitState>();
        registry.register::<TramTransitStats>();
    }
}
