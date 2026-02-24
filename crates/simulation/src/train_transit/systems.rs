//! ECS systems and plugin for the train transit feature.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::land_value::LandValueGrid;
use crate::stats::CityStats;
use crate::SlowTickTimer;

use super::types::*;

/// Update train transit statistics and ridership every slow tick.
pub fn update_train_lines(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<TrainTransitState>,
    city_stats: Res<CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let total_stations = state.stations.len() as u32;
    let total_active_lines = state.lines.iter().filter(|l| l.active).count() as u32;
    let daily_ridership = state.estimate_daily_ridership(city_stats.population);
    let monthly_cost = state.total_monthly_cost();

    // Distribute ridership across stations proportionally
    if total_stations > 0 && daily_ridership > 0 {
        let per_station = daily_ridership / total_stations;
        let remainder = daily_ridership % total_stations;
        for (i, station) in state.stations.iter_mut().enumerate() {
            let riders = per_station + if (i as u32) < remainder { 1 } else { 0 };
            station.period_ridership = riders;
            station.total_ridership += riders as u64;
            // Add some passengers to the queue (capped by capacity)
            station.passenger_queue = (station.passenger_queue + riders / 4).min(station.capacity);
        }
    } else {
        for station in &mut state.stations {
            station.period_ridership = 0;
        }
    }

    // Simulate trains moving between stations
    simulate_train_movement(&mut state);

    // Estimate cargo moved (simplified: proportional to ridership)
    let cargo_increment = (daily_ridership as u64) / 10;

    // Calculate fare revenue from ridership
    let fare_revenue = daily_ridership as f64 * FARE_PER_RIDE;

    state.stats = TrainTransitStats {
        total_stations,
        total_active_lines,
        daily_ridership,
        cargo_moved: state.stats.cargo_moved + cargo_increment,
        monthly_maintenance_cost: monthly_cost,
        monthly_fare_revenue: state.stats.monthly_fare_revenue + fare_revenue,
        cumulative_ridership: state.stations.iter().map(|s| s.total_ridership).sum(),
    };
}

/// Simulate train movement along lines, picking up and dropping off passengers.
fn simulate_train_movement(state: &mut TrainTransitState) {
    // Pre-collect station data to avoid borrow conflicts
    struct StationData {
        id: StationId,
        grid_x: f32,
        grid_y: f32,
        passenger_queue: u32,
    }

    let station_data: Vec<StationData> = state
        .stations
        .iter()
        .map(|s| StationData {
            id: s.id,
            grid_x: s.grid_x as f32,
            grid_y: s.grid_y as f32,
            passenger_queue: s.passenger_queue,
        })
        .collect();

    // Pre-collect line data
    struct LineData {
        id: LineId,
        station_ids: Vec<StationId>,
        active: bool,
    }

    let line_data: Vec<LineData> = state
        .lines
        .iter()
        .map(|l| LineData {
            id: l.id,
            station_ids: l.station_ids.clone(),
            active: l.active,
        })
        .collect();

    let mut queue_decrements: Vec<(StationId, u32)> = Vec::new();
    let mut ridership_increments: Vec<(StationId, u32)> = Vec::new();

    for train in &mut state.trains {
        let Some(ld) = line_data.iter().find(|l| l.id == train.line_id) else {
            continue;
        };
        if !ld.active || ld.station_ids.is_empty() {
            continue;
        }

        // Handle dwelling at station
        if train.at_station {
            if train.dwell_ticks > 0 {
                train.dwell_ticks -= 1;
                continue;
            }
            train.at_station = false;
            train.next_station_index = (train.next_station_index + 1) % ld.station_ids.len();
        }

        // Move toward next station
        let next_station_id = ld.station_ids[train.next_station_index % ld.station_ids.len()];
        let Some(next_sd) = station_data.iter().find(|s| s.id == next_station_id) else {
            continue;
        };

        let dx = next_sd.grid_x - train.grid_x;
        let dy = next_sd.grid_y - train.grid_y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Train moves ~1 cell per tick at this simplified rate
        let speed = 1.0_f32;

        if dist < speed {
            // Arrived at station
            train.grid_x = next_sd.grid_x;
            train.grid_y = next_sd.grid_y;
            train.at_station = true;
            train.dwell_ticks = 3;

            // Drop off passengers
            let dropoff = (train.passengers / 3).max(1).min(train.passengers);
            train.passengers = train.passengers.saturating_sub(dropoff);

            // Pick up waiting passengers
            let waiting = next_sd.passenger_queue;
            let space = TRAIN_CAPACITY.saturating_sub(train.passengers);
            let pickup = waiting.min(space);
            if pickup > 0 {
                train.passengers += pickup;
                queue_decrements.push((next_station_id, pickup));
                ridership_increments.push((next_station_id, pickup));
            }
        } else {
            let norm = 1.0 / dist;
            train.grid_x += dx * norm * speed;
            train.grid_y += dy * norm * speed;
        }
    }

    // Apply queue decrements
    for (station_id, decrement) in queue_decrements {
        if let Some(station) = state.stations.iter_mut().find(|s| s.id == station_id) {
            station.passenger_queue = station.passenger_queue.saturating_sub(decrement);
        }
    }

    // Apply ridership increments from train pickups
    for (station_id, count) in ridership_increments {
        if let Some(station) = state.stations.iter_mut().find(|s| s.id == station_id) {
            station.total_ridership += count as u64;
        }
    }
}

/// Boost land value around train stations.
///
/// Each station provides a +10-20 land value bonus in a radius around it,
/// with the boost diminishing linearly with distance.
pub fn train_station_land_value(
    slow_timer: Res<SlowTickTimer>,
    state: Res<TrainTransitState>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for station in &state.stations {
        let cx = station.grid_x as i32;
        let cy = station.grid_y as i32;
        let radius = STATION_LAND_VALUE_BOOST_RADIUS;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }

                let dist = dx.abs() + dy.abs();
                if dist > radius {
                    continue;
                }

                // Linear interpolation: center gets BOOST_CENTER, edge gets BOOST_MIN
                let t = dist as f32 / radius as f32;
                let boost = STATION_LAND_VALUE_BOOST_CENTER as f32 * (1.0 - t)
                    + STATION_LAND_VALUE_BOOST_MIN as f32 * t;
                let boost = boost as i32;
                if boost <= 0 {
                    continue;
                }

                let ux = nx as usize;
                let uy = ny as usize;
                let cur = land_value.get(ux, uy) as i32;
                land_value.set(ux, uy, (cur + boost).min(255) as u8);
            }
        }
    }
}

/// Deduct train operating costs from the city budget.
///
/// Runs on slow tick, deducting costs every 7 days (weekly).
pub fn update_train_costs(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<TrainTransitState>,
    mut budget: ResMut<CityBudget>,
    clock: Res<crate::time_of_day::GameClock>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Only deduct every ~7 days (weekly costs)
    if clock.day <= state.last_cost_day + 7 {
        return;
    }
    state.last_cost_day = clock.day;

    let weekly_cost = state.total_weekly_cost();
    if weekly_cost > 0.0 {
        budget.treasury -= weekly_cost;
    }

    // Add fare revenue to budget
    let fare_revenue = state.stats.monthly_fare_revenue;
    if fare_revenue > 0.0 {
        budget.treasury += fare_revenue;
        state.stats.monthly_fare_revenue = 0.0;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct TrainTransitPlugin;

impl Plugin for TrainTransitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrainTransitState>().add_systems(
            FixedUpdate,
            (
                // Order-independent: only writes TrainTransitState (private resource).
                update_train_lines,
                // Writes LandValueGrid; must run after base land value is computed.
                train_station_land_value.after(crate::land_value::update_land_value),
                // Order-independent: only writes TrainTransitState (private resource).
                update_train_costs,
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the extension map
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<TrainTransitState>();
    }
}
