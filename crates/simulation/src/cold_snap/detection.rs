use crate::weather::ClimateZone;

use super::types::{
    ColdSnapTier, COLD_SNAP_ABSOLUTE_THRESHOLD_C, COLD_SNAP_CONSECUTIVE_DAYS,
    COLD_SNAP_SEASONAL_DEVIATION_C, HOMELESS_MORTALITY_THRESHOLD_C,
};

// =============================================================================
// Cold day detection and tier classification
// =============================================================================

/// Determine whether the current temperature qualifies as "cold" relative to
/// the absolute threshold or the seasonal average.
///
/// A day is cold if either:
/// - Temperature is below -12C (absolute threshold), OR
/// - Temperature is more than 11C below the seasonal average
pub fn is_cold_day(temp_c: f32, seasonal_avg: f32) -> bool {
    temp_c < COLD_SNAP_ABSOLUTE_THRESHOLD_C
        || temp_c < (seasonal_avg - COLD_SNAP_SEASONAL_DEVIATION_C)
}

/// Compute the seasonal average temperature for the current season and climate zone.
///
/// Uses the midpoint of the season's min/max temperature range.
pub fn seasonal_average_temp(season: crate::weather::Season, zone: ClimateZone) -> f32 {
    let (t_min, t_max) = season.temperature_range_for_zone(zone);
    (t_min + t_max) / 2.0
}

/// Classify the cold snap tier from consecutive cold days and current temperature.
pub fn cold_snap_tier(consecutive_days: u32, temp_c: f32) -> ColdSnapTier {
    if consecutive_days >= COLD_SNAP_CONSECUTIVE_DAYS {
        if temp_c < -23.0 {
            ColdSnapTier::Emergency
        } else {
            ColdSnapTier::Warning
        }
    } else if consecutive_days >= 1 {
        ColdSnapTier::Watch
    } else {
        ColdSnapTier::Normal
    }
}

// =============================================================================
// Effect modifiers
// =============================================================================

/// Calculate heating demand modifier based on cold snap tier and temperature.
///
/// During a cold snap, heating demand surges +80-150% above normal:
/// - Watch: +0% (monitoring only)
/// - Warning: +80% (1.8x)
/// - Emergency: +150% (2.5x)
///
/// Additionally, for non-active cold snaps at sub-zero temps, a mild
/// increase is applied proportional to how far below 0C.
pub fn heating_demand_modifier(tier: ColdSnapTier, temp_c: f32) -> f32 {
    match tier {
        ColdSnapTier::Normal => {
            if temp_c < 0.0 {
                // Mild increase when below freezing but no cold snap
                (1.0 + (-temp_c) * 0.02).min(1.3)
            } else {
                1.0
            }
        }
        ColdSnapTier::Watch => {
            if temp_c < 0.0 {
                (1.0 + (-temp_c) * 0.03).min(1.5)
            } else {
                1.0
            }
        }
        ColdSnapTier::Warning => 1.8,
        ColdSnapTier::Emergency => 2.5,
    }
}

/// Calculate homeless mortality rate per 100k per day.
///
/// Exponential curve below -18C: `2.0 * exp(0.3 * (threshold - temp))`
/// Returns 0.0 when temperature is above the threshold.
pub fn homeless_mortality(temp_c: f32) -> f32 {
    if temp_c >= HOMELESS_MORTALITY_THRESHOLD_C {
        return 0.0;
    }
    let excess = HOMELESS_MORTALITY_THRESHOLD_C - temp_c;
    2.0 * (0.3 * excess).exp()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Cold day detection tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_cold_day_absolute() {
        // Below -12C is always cold regardless of seasonal average
        assert!(is_cold_day(-13.0, 0.0));
        assert!(is_cold_day(-20.0, -5.0));
        // At or above -12C, depends on seasonal average
        assert!(!is_cold_day(-12.0, -5.0)); // -12 is not < -12 (absolute) and -12 > -16 (seasonal)
    }

    #[test]
    fn test_is_cold_day_seasonal_deviation() {
        // 11C below seasonal average of 5C = -6C threshold
        // -7C is below -6C, so it's cold
        assert!(is_cold_day(-7.0, 5.0));
        // -5C is above -6C, so not cold (and above -12C absolute)
        assert!(!is_cold_day(-5.0, 5.0));
    }

    #[test]
    fn test_is_cold_day_warm_season_deviation() {
        // In summer with avg 25C: deviation threshold = 25 - 11 = 14C
        // 13C is below 14C, so it's a cold day (unusual cold spell in summer)
        assert!(is_cold_day(13.0, 25.0));
        // 15C is above 14C, not cold
        assert!(!is_cold_day(15.0, 25.0));
    }

    // -----------------------------------------------------------------------
    // Seasonal average temperature tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_seasonal_average_temperate_winter() {
        let avg = seasonal_average_temp(crate::weather::Season::Winter, ClimateZone::Temperate);
        // Temperate winter: t_min=-8, t_max=6, avg=-1.0
        assert!(
            (avg - (-1.0)).abs() < 0.01,
            "Temperate winter avg should be ~-1.0, got {}",
            avg
        );
    }

    #[test]
    fn test_seasonal_average_subarctic_winter() {
        let avg = seasonal_average_temp(crate::weather::Season::Winter, ClimateZone::Subarctic);
        // Subarctic winter: t_min=-34, t_max=-12, avg=-23.0
        assert!(
            (avg - (-23.0)).abs() < 0.01,
            "Subarctic winter avg should be ~-23.0, got {}",
            avg
        );
    }

    // -----------------------------------------------------------------------
    // Cold snap tier tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_tier_normal() {
        assert_eq!(cold_snap_tier(0, 5.0), ColdSnapTier::Normal);
        assert_eq!(cold_snap_tier(0, -10.0), ColdSnapTier::Normal);
    }

    #[test]
    fn test_tier_watch() {
        assert_eq!(cold_snap_tier(1, -15.0), ColdSnapTier::Watch);
        assert_eq!(cold_snap_tier(2, -20.0), ColdSnapTier::Watch);
    }

    #[test]
    fn test_tier_warning() {
        assert_eq!(cold_snap_tier(3, -15.0), ColdSnapTier::Warning);
        assert_eq!(cold_snap_tier(5, -20.0), ColdSnapTier::Warning);
        assert_eq!(cold_snap_tier(10, -10.0), ColdSnapTier::Warning);
    }

    #[test]
    fn test_tier_emergency() {
        assert_eq!(cold_snap_tier(3, -24.0), ColdSnapTier::Emergency);
        assert_eq!(cold_snap_tier(5, -30.0), ColdSnapTier::Emergency);
        // At exactly -23C, still Warning (threshold is < -23)
        assert_eq!(cold_snap_tier(3, -23.0), ColdSnapTier::Warning);
    }

    // -----------------------------------------------------------------------
    // Heating demand modifier tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_heating_normal_above_zero() {
        let modifier = heating_demand_modifier(ColdSnapTier::Normal, 10.0);
        assert!(
            (modifier - 1.0).abs() < f32::EPSILON,
            "Above zero, normal tier should be 1.0, got {}",
            modifier
        );
    }

    #[test]
    fn test_heating_normal_below_zero() {
        let modifier = heating_demand_modifier(ColdSnapTier::Normal, -5.0);
        // 1.0 + 5 * 0.02 = 1.10
        assert!(
            (modifier - 1.1).abs() < 0.01,
            "Normal tier at -5C should be ~1.1, got {}",
            modifier
        );
    }

    #[test]
    fn test_heating_normal_capped() {
        let modifier = heating_demand_modifier(ColdSnapTier::Normal, -30.0);
        // 1.0 + 30 * 0.02 = 1.6, but capped at 1.3
        assert!(
            (modifier - 1.3).abs() < f32::EPSILON,
            "Normal tier heating cap should be 1.3, got {}",
            modifier
        );
    }

    #[test]
    fn test_heating_warning() {
        let modifier = heating_demand_modifier(ColdSnapTier::Warning, -15.0);
        assert!(
            (modifier - 1.8).abs() < f32::EPSILON,
            "Warning tier should be 1.8, got {}",
            modifier
        );
    }

    #[test]
    fn test_heating_emergency() {
        let modifier = heating_demand_modifier(ColdSnapTier::Emergency, -25.0);
        assert!(
            (modifier - 2.5).abs() < f32::EPSILON,
            "Emergency tier should be 2.5, got {}",
            modifier
        );
    }

    // -----------------------------------------------------------------------
    // Homeless mortality tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_homeless_mortality_above_threshold() {
        assert!(
            homeless_mortality(-17.0).abs() < f32::EPSILON,
            "Above -18C should have zero mortality"
        );
        assert!(
            homeless_mortality(0.0).abs() < f32::EPSILON,
            "Above freezing should have zero mortality"
        );
    }

    #[test]
    fn test_homeless_mortality_at_threshold() {
        assert!(
            homeless_mortality(-18.0).abs() < f32::EPSILON,
            "At exactly -18C should have zero mortality"
        );
    }

    #[test]
    fn test_homeless_mortality_below_threshold() {
        let rate = homeless_mortality(-20.0);
        // 2.0 * exp(0.3 * 2) = 2.0 * exp(0.6) ~ 3.644
        let expected = 2.0 * (0.3_f32 * 2.0).exp();
        assert!(
            (rate - expected).abs() < 0.01,
            "At -20C expected ~{}, got {}",
            expected,
            rate
        );
    }

    #[test]
    fn test_homeless_mortality_exponential_growth() {
        let m20 = homeless_mortality(-20.0);
        let m25 = homeless_mortality(-25.0);
        let m30 = homeless_mortality(-30.0);
        assert!(
            m25 > m20 * 2.0,
            "Mortality should grow fast: -25C={} vs -20C={}",
            m25,
            m20
        );
        assert!(
            m30 > m25 * 2.0,
            "Mortality should grow fast: -30C={} vs -25C={}",
            m30,
            m25
        );
    }
}
