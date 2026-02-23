use bevy::prelude::*;

use crate::buildings::Building;
use crate::drought::DroughtState;
use crate::grid::ZoneType;
use crate::natural_resources::{ResourceBalance, ResourceGrid, ResourceType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::weather::{Season, Weather};
use crate::SlowTickTimer;

use super::helpers::{
    calculate_crop_yield, calculate_frost_risk, is_growing_season, rainfall_adequacy,
    temperature_suitability,
};
use super::types::{
    AgricultureState, FrostEvent, BASE_SOIL_QUALITY, FROST_DAMAGE_FRACTION,
    IRRIGATION_FERTILIZER_BONUS, IRRIGATION_RADIUS,
};

// =============================================================================
// System
// =============================================================================

/// System: Update agricultural growing season and crop yield modifiers.
///
/// Runs on the slow tick timer. Reads weather, drought state, resource grid,
/// and building data to determine growing season status and crop yield.
/// Modifies `ResourceBalance.food_production` based on the crop yield modifier.
#[allow(clippy::too_many_arguments)]
pub fn update_agriculture(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    drought: Res<DroughtState>,
    resource_grid: Res<ResourceGrid>,
    mut agriculture: ResMut<AgricultureState>,
    mut balance: ResMut<ResourceBalance>,
    buildings: Query<&Building>,
    service_buildings: Query<&ServiceBuilding>,
    mut frost_events: EventWriter<FrostEvent>,
) {
    if !timer.should_run() {
        return;
    }

    // 1. Count farms (industrial buildings on fertile land) and check irrigation
    let mut farm_count = 0u32;
    let mut total_soil_quality = 0.0f32;
    let mut irrigated_farms = 0u32;

    // Collect irrigation building positions
    let irrigation_positions: Vec<(usize, usize)> = service_buildings
        .iter()
        .filter(|sb| sb.service_type == ServiceType::WellPump)
        .map(|sb| (sb.grid_x, sb.grid_y))
        .collect();

    let has_irrigation = !irrigation_positions.is_empty();

    for building in &buildings {
        if building.zone_type != ZoneType::Industrial {
            continue;
        }
        if let Some(deposit) = resource_grid.get(building.grid_x, building.grid_y) {
            if deposit.resource_type == ResourceType::FertileLand && deposit.amount > 0 {
                farm_count += 1;
                // Soil quality scales with remaining deposit amount
                let quality =
                    BASE_SOIL_QUALITY * (deposit.amount as f32 / deposit.max_amount as f32);
                total_soil_quality += quality;

                // Check if this farm is within irrigation radius
                if has_irrigation {
                    for &(ix, iy) in &irrigation_positions {
                        let dx = building.grid_x as i32 - ix as i32;
                        let dy = building.grid_y as i32 - iy as i32;
                        let dist_sq = (dx * dx + dy * dy) as u32;
                        if dist_sq <= IRRIGATION_RADIUS * IRRIGATION_RADIUS {
                            irrigated_farms += 1;
                            break;
                        }
                    }
                }
            }
        }
    }

    agriculture.farm_count = farm_count;
    agriculture.has_irrigation = has_irrigation;

    // 2. Calculate average soil quality
    agriculture.soil_quality = if farm_count > 0 {
        total_soil_quality / farm_count as f32
    } else {
        BASE_SOIL_QUALITY
    };

    // 3. Update rainfall estimate
    // Precipitation intensity is in inches/hour; accumulate daily
    let current_day = weather.last_update_day;
    if current_day > agriculture.last_rainfall_day {
        // Approximate daily rainfall from current intensity (hours of rain per day)
        // Assume average 8 hours of possible rain per day
        let daily_rainfall = weather.precipitation_intensity * 8.0;
        // Rolling estimate: exponential moving average scaled to annual
        // annual_rainfall = daily_avg * 365
        let alpha = 1.0 / 30.0; // ~30-day smoothing
        let daily_avg = agriculture.annual_rainfall_estimate / 365.0;
        let new_daily_avg = daily_avg * (1.0 - alpha) + daily_rainfall * alpha;
        agriculture.annual_rainfall_estimate = new_daily_avg * 365.0;
        agriculture.last_rainfall_day = current_day;
    }

    // Reset frost counters on year boundary (every 360 days)
    if current_day > 0 && current_day % 360 == 1 && agriculture.frost_events_this_year > 0 {
        agriculture.frost_events_this_year = 0;
        agriculture.frost_damage_total = 0.0;
    }

    // 4. Calculate frost risk
    agriculture.frost_risk = calculate_frost_risk(weather.temperature, weather.season);

    // 5. Determine growing season
    agriculture.growing_season_active =
        is_growing_season(weather.temperature, weather.season, agriculture.frost_risk);

    // 6. Calculate temperature suitability
    agriculture.temperature_suitability = temperature_suitability(weather.temperature);

    // 7. Calculate rainfall adequacy
    let is_mostly_irrigated = has_irrigation && farm_count > 0 && irrigated_farms * 2 >= farm_count;
    agriculture.rainfall_adequacy =
        rainfall_adequacy(agriculture.annual_rainfall_estimate, is_mostly_irrigated);

    // 8. Calculate fertilizer bonus
    agriculture.fertilizer_bonus = if is_mostly_irrigated {
        IRRIGATION_FERTILIZER_BONUS
    } else {
        1.0
    };

    // 9. Calculate composite crop yield modifier
    agriculture.crop_yield_modifier = calculate_crop_yield(
        agriculture.rainfall_adequacy,
        agriculture.temperature_suitability,
        agriculture.soil_quality,
        agriculture.fertilizer_bonus,
    );

    // Apply drought modifier
    agriculture.crop_yield_modifier *= drought.agriculture_modifier;

    // 10. Check for frost events (Spring/Autumn only)
    if matches!(weather.season, Season::Spring | Season::Autumn)
        && current_day > agriculture.last_frost_check_day
        && agriculture.frost_risk > 0.0
        && farm_count > 0
    {
        // Use a deterministic frost check based on day and temperature
        let frost_hash = (current_day.wrapping_mul(7919) ^ (weather.temperature.to_bits())) % 100;
        let frost_threshold = (agriculture.frost_risk * 100.0) as u32;

        if frost_hash < frost_threshold {
            let damage = FROST_DAMAGE_FRACTION;
            agriculture.frost_events_this_year += 1;
            agriculture.frost_damage_total += damage;

            frost_events.send(FrostEvent {
                damage_fraction: damage,
                season: weather.season,
            });
        }
        agriculture.last_frost_check_day = current_day;
    }

    // 11. Apply crop yield modifier to food production
    // Only modify food production when growing season is active
    if agriculture.growing_season_active && farm_count > 0 {
        // The base food production is already calculated in update_resource_production.
        // We multiply it by the crop yield modifier here.
        let frost_penalty = 1.0 - agriculture.frost_damage_total.min(0.9);
        let effective_modifier = agriculture.crop_yield_modifier * frost_penalty;
        balance.food_production *= effective_modifier;
    } else if !agriculture.growing_season_active && farm_count > 0 {
        // Outside growing season, food production drops to the seasonal base
        // (winter = 0.3, etc. from Weather::agriculture_multiplier)
        balance.food_production *= weather.agriculture_multiplier();
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct AgriculturePlugin;

impl Plugin for AgriculturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AgricultureState>()
            .add_event::<FrostEvent>()
            .add_systems(
                FixedUpdate,
                update_agriculture
                    .after(crate::natural_resources::update_resource_production)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
