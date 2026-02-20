use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::drought::DroughtState;
use crate::grid::ZoneType;
use crate::natural_resources::{ResourceBalance, ResourceGrid, ResourceType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::weather::{Season, Weather};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Temperature threshold (Fahrenheit) below which crops cannot grow.
const GROWING_TEMP_THRESHOLD_F: f32 = 50.0;

/// Frost risk threshold: if frost risk exceeds this, the growing season is inactive.
const FROST_RISK_THRESHOLD: f32 = 0.10;

/// Adequate rainfall range (inches per year).
const RAINFALL_ADEQUATE_LOW: f32 = 20.0;
const RAINFALL_ADEQUATE_HIGH: f32 = 40.0;

/// Rainfall adequacy multiplier for excess rainfall (>40 in/yr).
const RAINFALL_EXCESS_MULTIPLIER: f32 = 0.8;

/// Rainfall adequacy multiplier for deficit rainfall (<20 in/yr).
const RAINFALL_DEFICIT_MULTIPLIER: f32 = 0.6;

/// Base soil quality for fertile land deposits.
const BASE_SOIL_QUALITY: f32 = 0.8;

/// Fertilizer bonus when irrigation infrastructure is present.
const IRRIGATION_FERTILIZER_BONUS: f32 = 1.15;

/// Frost damage probability in Spring (early frost).
const SPRING_FROST_BASE_RISK: f32 = 0.15;

/// Frost damage probability in Autumn (late frost).
const AUTUMN_FROST_BASE_RISK: f32 = 0.12;

/// Fraction of crop yield destroyed by a frost event.
const FROST_DAMAGE_FRACTION: f32 = 0.3;

/// Irrigation coverage radius (in grid cells) from an irrigation building.
const IRRIGATION_RADIUS: u32 = 12;

// =============================================================================
// Frost event
// =============================================================================

/// Event fired when a frost event damages crops.
#[derive(Event, Debug, Clone)]
pub struct FrostEvent {
    /// Fraction of total crop yield destroyed (0.0 to 1.0).
    pub damage_fraction: f32,
    /// Season during which the frost occurred.
    pub season: Season,
}

// =============================================================================
// Resource
// =============================================================================

/// City-wide agricultural growing season and crop yield tracking.
///
/// Updated every slow tick based on weather, rainfall, soil quality, and
/// irrigation infrastructure. Affects food production in `ResourceBalance`.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AgricultureState {
    /// Whether the growing season is currently active.
    pub growing_season_active: bool,
    /// Current crop yield modifier (0.0 to ~1.5).
    /// `rainfall_adequacy * temperature_suitability * soil_quality * fertilizer_bonus`
    pub crop_yield_modifier: f32,
    /// Rainfall adequacy factor (0.6, 0.8, or 1.0 for deficit/excess/adequate).
    pub rainfall_adequacy: f32,
    /// Temperature suitability factor (0.0 to 1.0).
    pub temperature_suitability: f32,
    /// Average soil quality across agricultural zones (0.0 to 1.0).
    pub soil_quality: f32,
    /// Fertilizer/irrigation bonus multiplier (1.0 = none, 1.15 = irrigated).
    pub fertilizer_bonus: f32,
    /// Current frost risk (0.0 to 1.0), based on season and temperature.
    pub frost_risk: f32,
    /// Number of frost events that have occurred this year.
    pub frost_events_this_year: u32,
    /// Total crop damage from frost this year (as a fraction of production lost).
    pub frost_damage_total: f32,
    /// Whether irrigation infrastructure is present in the city.
    pub has_irrigation: bool,
    /// Number of agricultural buildings (industrial on fertile land).
    pub farm_count: u32,
    /// Accumulated annual rainfall estimate (inches), derived from daily precipitation.
    pub annual_rainfall_estimate: f32,
    /// Last game day that checked for frost events.
    pub last_frost_check_day: u32,
    /// Last game day that updated rainfall accumulation.
    pub last_rainfall_day: u32,
}

impl Default for AgricultureState {
    fn default() -> Self {
        Self {
            growing_season_active: false,
            crop_yield_modifier: 1.0,
            rainfall_adequacy: 1.0,
            temperature_suitability: 1.0,
            soil_quality: BASE_SOIL_QUALITY,
            fertilizer_bonus: 1.0,
            frost_risk: 0.0,
            frost_events_this_year: 0,
            frost_damage_total: 0.0,
            has_irrigation: false,
            farm_count: 0,
            annual_rainfall_estimate: 30.0, // Default to adequate range
            last_frost_check_day: 0,
            last_rainfall_day: 0,
        }
    }
}

// =============================================================================
// Helper functions (pure, testable)
// =============================================================================

/// Convert Celsius to Fahrenheit.
fn celsius_to_fahrenheit(c: f32) -> f32 {
    c * 9.0 / 5.0 + 32.0
}

/// Determine if the growing season is active based on temperature, season, and frost risk.
///
/// Growing season is active when:
/// - Temperature > 50F (10C)
/// - Frost risk < 10%
/// - Not winter
pub fn is_growing_season(temperature_c: f32, season: Season, frost_risk: f32) -> bool {
    let temp_f = celsius_to_fahrenheit(temperature_c);
    temp_f > GROWING_TEMP_THRESHOLD_F
        && frost_risk < FROST_RISK_THRESHOLD
        && !matches!(season, Season::Winter)
}

/// Calculate frost risk based on season and temperature.
///
/// Frost risk is highest in Spring and Autumn at low temperatures.
/// Winter always has 100% frost risk. Summer has 0%.
pub fn calculate_frost_risk(temperature_c: f32, season: Season) -> f32 {
    match season {
        Season::Winter => 1.0,
        Season::Summer => 0.0,
        Season::Spring => {
            // Higher frost risk at lower temperatures
            if temperature_c < 0.0 {
                1.0
            } else if temperature_c < 5.0 {
                SPRING_FROST_BASE_RISK + (5.0 - temperature_c) / 5.0 * 0.5
            } else if temperature_c < 10.0 {
                SPRING_FROST_BASE_RISK * (10.0 - temperature_c) / 5.0
            } else {
                0.0
            }
        }
        Season::Autumn => {
            if temperature_c < 0.0 {
                1.0
            } else if temperature_c < 5.0 {
                AUTUMN_FROST_BASE_RISK + (5.0 - temperature_c) / 5.0 * 0.5
            } else if temperature_c < 10.0 {
                AUTUMN_FROST_BASE_RISK * (10.0 - temperature_c) / 5.0
            } else {
                0.0
            }
        }
    }
}

/// Calculate temperature suitability for crop growth.
///
/// Optimal range: 15-30C (59-86F). Falls off linearly outside this range.
/// Returns 0.0 below 10C or above 40C.
pub fn temperature_suitability(temperature_c: f32) -> f32 {
    if temperature_c < 10.0 {
        0.0
    } else if temperature_c < 15.0 {
        (temperature_c - 10.0) / 5.0
    } else if temperature_c <= 30.0 {
        1.0
    } else if temperature_c < 40.0 {
        (40.0 - temperature_c) / 10.0
    } else {
        0.0
    }
}

/// Calculate rainfall adequacy from estimated annual rainfall.
///
/// - Adequate (20-40 in/yr): 1.0
/// - Excess (>40 in/yr): 0.8
/// - Deficit (<20 in/yr): 0.6
/// - If irrigated, returns min(1.0, supply/demand) where supply scales with rainfall.
pub fn rainfall_adequacy(annual_rainfall: f32, irrigated: bool) -> f32 {
    if irrigated {
        // Irrigation makes up for deficit; effectiveness scales with available water
        let base = if annual_rainfall < RAINFALL_ADEQUATE_LOW {
            RAINFALL_DEFICIT_MULTIPLIER
        } else if annual_rainfall > RAINFALL_ADEQUATE_HIGH {
            RAINFALL_EXCESS_MULTIPLIER
        } else {
            1.0
        };
        // Irrigation boosts deficit scenarios to near-adequate
        let supply_ratio = (annual_rainfall / RAINFALL_ADEQUATE_LOW).min(1.0);
        (base + (1.0 - base) * supply_ratio).min(1.0)
    } else if annual_rainfall < RAINFALL_ADEQUATE_LOW {
        RAINFALL_DEFICIT_MULTIPLIER
    } else if annual_rainfall > RAINFALL_ADEQUATE_HIGH {
        RAINFALL_EXCESS_MULTIPLIER
    } else {
        1.0
    }
}

/// Calculate the composite crop yield modifier.
pub fn calculate_crop_yield(
    rainfall_adequacy: f32,
    temp_suitability: f32,
    soil_quality: f32,
    fertilizer_bonus: f32,
) -> f32 {
    rainfall_adequacy * temp_suitability * soil_quality * fertilizer_bonus
}

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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Celsius to Fahrenheit
    // -------------------------------------------------------------------------

    #[test]
    fn test_celsius_to_fahrenheit_freezing() {
        let f = celsius_to_fahrenheit(0.0);
        assert!((f - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_celsius_to_fahrenheit_boiling() {
        let f = celsius_to_fahrenheit(100.0);
        assert!((f - 212.0).abs() < 0.01);
    }

    #[test]
    fn test_celsius_to_fahrenheit_ten() {
        // 10C = 50F
        let f = celsius_to_fahrenheit(10.0);
        assert!((f - 50.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Growing season
    // -------------------------------------------------------------------------

    #[test]
    fn test_growing_season_summer_warm() {
        // 25C, summer, no frost -> active
        assert!(is_growing_season(25.0, Season::Summer, 0.0));
    }

    #[test]
    fn test_growing_season_winter_inactive() {
        // Winter always inactive
        assert!(!is_growing_season(25.0, Season::Winter, 0.0));
    }

    #[test]
    fn test_growing_season_cold_inactive() {
        // Below 50F (10C) -> inactive
        assert!(!is_growing_season(5.0, Season::Spring, 0.0));
    }

    #[test]
    fn test_growing_season_high_frost_inactive() {
        // Frost risk >= 10% -> inactive
        assert!(!is_growing_season(12.0, Season::Spring, 0.15));
    }

    #[test]
    fn test_growing_season_spring_warm_no_frost() {
        // 15C, spring, low frost -> active
        assert!(is_growing_season(15.0, Season::Spring, 0.0));
    }

    #[test]
    fn test_growing_season_autumn_warm_no_frost() {
        // 15C, autumn, low frost -> active
        assert!(is_growing_season(15.0, Season::Autumn, 0.0));
    }

    #[test]
    fn test_growing_season_exactly_threshold() {
        // Exactly 10C = 50F, should NOT be active (must be > 50F)
        assert!(!is_growing_season(10.0, Season::Spring, 0.0));
    }

    // -------------------------------------------------------------------------
    // Frost risk
    // -------------------------------------------------------------------------

    #[test]
    fn test_frost_risk_winter() {
        assert!((calculate_frost_risk(5.0, Season::Winter) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_summer() {
        assert!((calculate_frost_risk(25.0, Season::Summer)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_spring_warm() {
        // 15C in spring -> no frost risk
        assert!((calculate_frost_risk(15.0, Season::Spring)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_spring_cold() {
        // Below 0C in spring -> high frost risk
        assert!((calculate_frost_risk(-5.0, Season::Spring) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_spring_marginal() {
        // 3C in spring -> some frost risk
        let risk = calculate_frost_risk(3.0, Season::Spring);
        assert!(risk > 0.0);
        assert!(risk < 1.0);
    }

    #[test]
    fn test_frost_risk_autumn_cold() {
        let risk = calculate_frost_risk(-1.0, Season::Autumn);
        assert!((risk - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_autumn_warm() {
        assert!((calculate_frost_risk(12.0, Season::Autumn)).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Temperature suitability
    // -------------------------------------------------------------------------

    #[test]
    fn test_temp_suitability_optimal() {
        assert!((temperature_suitability(20.0) - 1.0).abs() < f32::EPSILON);
        assert!((temperature_suitability(25.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_cold() {
        assert!((temperature_suitability(5.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_hot() {
        assert!((temperature_suitability(42.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_transition_low() {
        // 12.5C -> halfway between 10 and 15 = 0.5
        let s = temperature_suitability(12.5);
        assert!((s - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_transition_high() {
        // 35C -> halfway between 30 and 40 = 0.5
        let s = temperature_suitability(35.0);
        assert!((s - 0.5).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Rainfall adequacy
    // -------------------------------------------------------------------------

    #[test]
    fn test_rainfall_adequate() {
        let r = rainfall_adequacy(30.0, false);
        assert!((r - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainfall_deficit() {
        let r = rainfall_adequacy(15.0, false);
        assert!((r - RAINFALL_DEFICIT_MULTIPLIER).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainfall_excess() {
        let r = rainfall_adequacy(50.0, false);
        assert!((r - RAINFALL_EXCESS_MULTIPLIER).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainfall_irrigated_deficit_improved() {
        // Irrigation should improve deficit scenario
        let without = rainfall_adequacy(10.0, false);
        let with = rainfall_adequacy(10.0, true);
        assert!(with > without);
    }

    #[test]
    fn test_rainfall_irrigated_capped_at_one() {
        let r = rainfall_adequacy(30.0, true);
        assert!(r <= 1.0);
    }

    // -------------------------------------------------------------------------
    // Crop yield calculation
    // -------------------------------------------------------------------------

    #[test]
    fn test_crop_yield_all_optimal() {
        let y = calculate_crop_yield(1.0, 1.0, 1.0, 1.0);
        assert!((y - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crop_yield_with_irrigation() {
        let y = calculate_crop_yield(1.0, 1.0, 0.8, IRRIGATION_FERTILIZER_BONUS);
        let expected = 0.8 * IRRIGATION_FERTILIZER_BONUS;
        assert!((y - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crop_yield_deficit() {
        let y = calculate_crop_yield(RAINFALL_DEFICIT_MULTIPLIER, 0.5, 0.8, 1.0);
        let expected = RAINFALL_DEFICIT_MULTIPLIER * 0.5 * 0.8;
        assert!((y - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crop_yield_zero_temp() {
        let y = calculate_crop_yield(1.0, 0.0, 0.8, 1.0);
        assert!(y.abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Default state
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state() {
        let state = AgricultureState::default();
        assert!(!state.growing_season_active);
        assert!((state.crop_yield_modifier - 1.0).abs() < f32::EPSILON);
        assert!((state.rainfall_adequacy - 1.0).abs() < f32::EPSILON);
        assert!((state.temperature_suitability - 1.0).abs() < f32::EPSILON);
        assert!((state.soil_quality - BASE_SOIL_QUALITY).abs() < f32::EPSILON);
        assert!((state.fertilizer_bonus - 1.0).abs() < f32::EPSILON);
        assert!(state.frost_risk.abs() < f32::EPSILON);
        assert_eq!(state.frost_events_this_year, 0);
        assert!(state.frost_damage_total.abs() < f32::EPSILON);
        assert!(!state.has_irrigation);
        assert_eq!(state.farm_count, 0);
    }

    // -------------------------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_growing_season_boundary_temp() {
        // Just above 10C (50.18F) -> should be active in summer
        assert!(is_growing_season(10.1, Season::Summer, 0.0));
    }

    #[test]
    fn test_frost_risk_boundary_spring() {
        // Exactly 10C -> no frost risk
        let risk = calculate_frost_risk(10.0, Season::Spring);
        assert!(risk.abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_boundary_five() {
        // Exactly 5C in spring -> base risk only
        let risk = calculate_frost_risk(5.0, Season::Spring);
        assert!((risk - SPRING_FROST_BASE_RISK).abs() < 0.01);
    }

    #[test]
    fn test_rainfall_boundary_low() {
        // Exactly at low boundary -> adequate
        let r = rainfall_adequacy(20.0, false);
        assert!((r - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainfall_boundary_high() {
        // Exactly at high boundary -> adequate
        let r = rainfall_adequacy(40.0, false);
        assert!((r - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_at_boundaries() {
        assert!((temperature_suitability(10.0)).abs() < f32::EPSILON);
        assert!((temperature_suitability(15.0) - 1.0).abs() < f32::EPSILON);
        assert!((temperature_suitability(30.0) - 1.0).abs() < f32::EPSILON);
        assert!((temperature_suitability(40.0)).abs() < f32::EPSILON);
    }
}
