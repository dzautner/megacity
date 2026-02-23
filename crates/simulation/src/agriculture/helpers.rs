use crate::weather::Season;

use super::types::{AUTUMN_FROST_BASE_RISK, SPRING_FROST_BASE_RISK};
use super::types::{
    FROST_RISK_THRESHOLD, GROWING_TEMP_THRESHOLD_F, RAINFALL_ADEQUATE_HIGH, RAINFALL_ADEQUATE_LOW,
    RAINFALL_DEFICIT_MULTIPLIER, RAINFALL_EXCESS_MULTIPLIER,
};

// =============================================================================
// Helper functions (pure, testable)
// =============================================================================

/// Convert Celsius to Fahrenheit.
pub(crate) fn celsius_to_fahrenheit(c: f32) -> f32 {
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
