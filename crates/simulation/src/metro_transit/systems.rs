//! ECS systems for the metro transit system.
//!
//! These systems run every slow tick to update metro statistics, deduct
//! maintenance costs, and boost land value around stations.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::land_value::LandValueGrid;
use crate::stats::CityStats;
use crate::SlowTickTimer;

use super::constants::*;
use super::state::MetroTransitState;
use super::types::MetroStats;

/// Update metro statistics and ridership every slow tick.
///
/// - Counts stations and operational lines
/// - Estimates daily ridership based on population and network size
/// - Calculates maintenance costs
/// - Updates per-station ridership counters
pub fn update_metro_stats(
    slow_timer: Res<SlowTickTimer>,
    mut metro: ResMut<MetroTransitState>,
    city_stats: Res<CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let total_stations = metro.stations.len() as u32;
    let total_lines = metro.lines.iter().filter(|l| l.operational).count() as u32;
    let daily_ridership = metro.estimate_daily_ridership(city_stats.population);
    let monthly_maintenance = metro.total_monthly_maintenance();

    // Distribute ridership across stations proportionally
    if total_stations > 0 && daily_ridership > 0 {
        let per_station = daily_ridership / total_stations;
        let remainder = daily_ridership % total_stations;
        for (i, station) in metro.stations.iter_mut().enumerate() {
            let riders = per_station + if (i as u32) < remainder { 1 } else { 0 };
            station.period_ridership = riders;
            station.total_ridership += riders as u64;
        }
    } else {
        for station in &mut metro.stations {
            station.period_ridership = 0;
        }
    }

    metro.stats = MetroStats {
        total_stations,
        total_lines,
        daily_ridership,
        monthly_maintenance_cost: monthly_maintenance,
        cumulative_ridership: metro.stations.iter().map(|s| s.total_ridership).sum(),
    };
}

/// Deduct metro maintenance costs from the city budget every 30 days.
///
/// This runs alongside the main tax collection cycle. The cost is based
/// on the number of stations and operational lines.
pub fn deduct_metro_costs(
    slow_timer: Res<SlowTickTimer>,
    metro: Res<MetroTransitState>,
    mut budget: ResMut<CityBudget>,
    clock: Res<crate::time_of_day::GameClock>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Only deduct every ~30 days, aligned with the budget cycle.
    if !clock.day.is_multiple_of(30) {
        return;
    }

    let cost = metro.total_monthly_maintenance();
    if cost > 0.0 {
        budget.treasury -= cost;
    }
}

/// Boost land value around metro stations.
///
/// Each station provides a +15-25 land value bonus in a radius around it,
/// with the boost diminishing linearly with distance. This runs every
/// slow tick after the main land value update.
pub fn metro_land_value_boost(
    slow_timer: Res<SlowTickTimer>,
    metro: Res<MetroTransitState>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for station in &metro.stations {
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
