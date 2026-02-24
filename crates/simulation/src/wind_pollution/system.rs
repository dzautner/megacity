//! Main Gaussian plume pollution system and source collection.

use bevy::prelude::*;

use crate::building_emissions::{
    building_emission_profile, category_multiplier, road_emission_q, service_emission_profile,
};
use crate::buildings::Building;
use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::pollution::PollutionGrid;
use crate::services::ServiceBuilding;
use crate::traffic::TrafficGrid;
use crate::wind::WindState;
use crate::SlowTickTimer;

use super::config::WindPollutionConfig;
use super::dispersion::{apply_isotropic_source, apply_plume_source, PollutionSource};

// =============================================================================
// Constants
// =============================================================================

/// Minimum wind speed below which we fall back to isotropic dispersion.
const CALM_WIND_THRESHOLD: f32 = 0.1;

/// Emission rate for coal power plants.
const COAL_Q: f32 = 100.0;

/// Emission rate for gas power plants.
const GAS_Q: f32 = 35.0;

/// Scrubber emission reduction factor (50% reduction).
const SCRUBBER_REDUCTION: f32 = 0.5;

/// Park pollution reduction intensity.
const PARK_REDUCTION: u8 = 8;

/// Park pollution reduction radius in grid cells.
const PARK_RADIUS: i32 = 6;

// =============================================================================
// Source collection
// =============================================================================

/// Collects all pollution sources from the world, using per-building-type
/// emission profiles from `building_emissions`.
#[allow(clippy::too_many_arguments)]
fn collect_sources(
    grid: &WorldGrid,
    buildings: &Query<&Building>,
    power_plants: &Query<&PowerPlant>,
    services: &Query<&ServiceBuilding>,
    traffic: &TrafficGrid,
    policies: &crate::policies::Policies,
    scrubber_mult: f32,
) -> Vec<PollutionSource> {
    let mut sources = Vec::new();

    // Roads: traffic-scaled emissions (POLL-002)
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type == CellType::Road {
                let congestion = traffic.congestion_level(x, y);
                let q = road_emission_q(congestion) * scrubber_mult;
                sources.push(PollutionSource {
                    x,
                    y,
                    emission_q: q,
                });
            }
        }
    }

    // Zoned buildings: per-type emission profiles (POLL-002)
    for building in buildings {
        if let Some(profile) = building_emission_profile(building.zone_type, building.level) {
            let cat_mult = category_multiplier(profile.category, policies);
            let q = profile.base_q * cat_mult * scrubber_mult;
            sources.push(PollutionSource {
                x: building.grid_x,
                y: building.grid_y,
                emission_q: q,
            });
        }
    }

    // Service buildings: combustion-related services (POLL-002)
    for service in services {
        if let Some(profile) = service_emission_profile(service.service_type) {
            let cat_mult = category_multiplier(profile.category, policies);
            let q = profile.base_q * cat_mult * scrubber_mult;
            sources.push(PollutionSource {
                x: service.grid_x,
                y: service.grid_y,
                emission_q: q,
            });
        }
    }

    // Power plants (coal, gas — solar/wind have Q=0)
    for plant in power_plants {
        let base_q = match plant.plant_type {
            PowerPlantType::Coal => COAL_Q,
            PowerPlantType::NaturalGas => GAS_Q,
            _ => 0.0,
        };
        if base_q > 0.0 {
            let policy_mult = policies.pollution_multiplier();
            sources.push(PollutionSource {
                x: plant.grid_x,
                y: plant.grid_y,
                emission_q: base_q * policy_mult * scrubber_mult,
            });
        }
    }

    sources
}

// =============================================================================
// Main system
// =============================================================================

/// Wind-aware Gaussian plume pollution dispersion system.
///
/// Replaces the old isotropic diffusion. Each tick:
/// 1. Clear the pollution grid
/// 2. Collect all pollution sources (buildings, services, power plants, roads)
/// 3. For each source, apply Gaussian plume dispersion in the wind direction
/// 4. Apply park reduction
/// 5. Clamp values to u8 range
#[allow(clippy::too_many_arguments)]
pub fn update_pollution_gaussian_plume(
    slow_timer: Res<SlowTickTimer>,
    mut pollution: ResMut<PollutionGrid>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    power_plants: Query<&PowerPlant>,
    services: Query<&ServiceBuilding>,
    policies: Res<crate::policies::Policies>,
    wind: Res<WindState>,
    config: Res<WindPollutionConfig>,
    traffic: Res<TrafficGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Clear grid
    pollution.levels.fill(0);

    // Compute multipliers
    let scrubber_mult = if config.scrubbers_enabled {
        SCRUBBER_REDUCTION
    } else {
        1.0
    };

    // Collect sources (POLL-002: per-building-type profiles)
    let sources = collect_sources(
        &grid,
        &buildings,
        &power_plants,
        &services,
        &traffic,
        &policies,
        scrubber_mult,
    );

    // Floating-point accumulator for precision
    let total_cells = GRID_WIDTH * GRID_HEIGHT;
    let mut float_levels = vec![0.0f32; total_cells];

    let (wind_dx, wind_dy) = wind.direction_vector();
    let is_calm = wind.speed < CALM_WIND_THRESHOLD;

    // Apply dispersion for each source
    for src in &sources {
        if is_calm {
            apply_isotropic_source(&mut float_levels, src);
        } else {
            apply_plume_source(&mut float_levels, src, wind_dx, wind_dy, wind.speed);
        }
    }

    // Write to pollution grid, clamping to u8
    for (level, &val) in pollution.levels.iter_mut().zip(float_levels.iter()) {
        *level = val.clamp(0.0, 255.0) as u8;
    }

    // Parks reduce pollution
    apply_park_reduction(&mut pollution, &services);
}

/// Applies park pollution reduction around park service buildings.
fn apply_park_reduction(pollution: &mut PollutionGrid, services: &Query<&ServiceBuilding>) {
    for service in services {
        if ServiceBuilding::is_park(service.service_type) {
            let radius = PARK_RADIUS;
            let reduction = PARK_REDUCTION;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = service.grid_x as i32 + dx;
                    let ny = service.grid_y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        let dist = dx.abs() + dy.abs();
                        let effect = reduction.saturating_sub(dist as u8);
                        let cur = pollution.get(nx as usize, ny as usize);
                        pollution.set(nx as usize, ny as usize, cur.saturating_sub(effect));
                    }
                }
            }
        }
    }
}
