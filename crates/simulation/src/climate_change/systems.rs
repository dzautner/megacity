//! Bevy ECS systems for climate change simulation.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::time_of_day::GameClock;
use crate::utilities::UtilitySource;
use crate::SlowTickTimer;

use super::calculations::*;
use super::constants::*;
use super::state::ClimateState;

/// Yearly climate assessment system. Runs every slow tick but only performs the
/// assessment once per game year (every 360 days).
///
/// 1. Calculates CO2 emissions from power plants and industrial buildings.
/// 2. Updates cumulative CO2 total.
/// 3. Determines temperature increase and disaster frequency multiplier.
/// 4. At +3F, applies sea level rise (permanent coastal flooding).
/// 5. Updates environmental score.
#[allow(clippy::too_many_arguments)]
pub fn yearly_climate_assessment(
    timer: Res<SlowTickTimer>,
    clock: Res<GameClock>,
    mut climate: ResMut<ClimateState>,
    mut grid: ResMut<WorldGrid>,
    utility_sources: Query<&UtilitySource>,
    buildings: Query<&Building>,
) {
    if !timer.should_run() {
        return;
    }

    let current_day = clock.day;

    // Only run assessment once per year (every 360 days)
    if current_day < climate.last_assessment_day + DAYS_PER_YEAR {
        return;
    }

    // --- Calculate yearly CO2 emissions ---

    let mut yearly_co2: f64 = 0.0;

    // CO2 from power-generating utility sources
    for source in &utility_sources {
        let rate = co2_rate_for_utility(source.utility_type);
        if rate > 0.0 {
            yearly_co2 += (rate * BASE_MWH_PER_PLANT) as f64;
        }
    }

    // CO2 from industrial buildings
    let industrial_count = buildings
        .iter()
        .filter(|b| b.zone_type == ZoneType::Industrial)
        .count();
    yearly_co2 += industrial_count as f64 * INDUSTRIAL_CO2_PER_BUILDING as f64;

    // --- Update cumulative totals ---
    climate.cumulative_co2 += yearly_co2;
    climate.yearly_co2 = yearly_co2;
    climate.last_assessment_day = current_day;

    // --- Calculate climate effects ---
    climate.temperature_increase_f = temperature_increase_from_co2(climate.cumulative_co2);
    climate.disaster_frequency_multiplier =
        disaster_multiplier_from_temp_increase(climate.temperature_increase_f);
    climate.drought_duration_multiplier =
        drought_multiplier_from_temp_increase(climate.temperature_increase_f);
    climate.environmental_score =
        calculate_environmental_score(climate.cumulative_co2, climate.yearly_co2);

    // --- Sea level rise at +3F ---
    if climate.temperature_increase_f >= 3.0 && !climate.sea_level_rise_applied {
        let threshold = sea_level_rise_threshold(&grid);
        let mut flooded = 0u32;

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.cell_type != CellType::Water
                    && cell.elevation <= threshold
                    && is_water_adjacent(&grid, x, y)
                {
                    let cell_mut = grid.get_mut(x, y);
                    cell_mut.cell_type = CellType::Water;
                    cell_mut.zone = ZoneType::None;
                    cell_mut.building_id = None;
                    flooded += 1;
                }
            }
        }

        climate.sea_level_rise_applied = true;
        climate.flooded_cells_count = flooded;

        if flooded > 0 {
            info!(
                "CLIMATE CHANGE: Sea level rise has permanently flooded {} coastal cells!",
                flooded
            );
        }
    }

    // Log the assessment
    if yearly_co2 > 0.0 {
        info!(
            "CLIMATE ASSESSMENT (Day {}): +{:.0} tons CO2 this year, {:.0} total, +{:.1}F warming, env score: {:.0}",
            current_day,
            yearly_co2,
            climate.cumulative_co2,
            climate.temperature_increase_f,
            climate.environmental_score
        );
    }
}
