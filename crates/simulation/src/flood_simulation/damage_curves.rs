//! Depth-damage curve data and interpolation for flood damage estimation.
//!
//! Depth-damage curves translate flood depth into a fractional damage value for
//! each zone type (Residential, Commercial, Industrial). These curves are used
//! to calculate building damage based on flood depth and zone type.

use crate::grid::ZoneType;

// =============================================================================
// Constants
// =============================================================================

/// Number of water-spreading iterations per slow tick.
pub(crate) const SPREAD_ITERATIONS: usize = 5;

/// Fraction of excess water distributed to each lower-elevation neighbor per iteration.
pub(crate) const SPREAD_RATE: f32 = 0.25;

/// Natural drain rate per tick (feet removed from all cells).
pub(crate) const NATURAL_DRAIN_RATE: f32 = 0.01;

/// Additional drain rate per tick for cells covered by storm drainage infrastructure (feet).
pub(crate) const STORM_DRAIN_RATE: f32 = 0.05;

/// Minimum flood depth (feet) for a cell to count as "flooded".
pub(crate) const FLOOD_DEPTH_THRESHOLD: f32 = 0.5;

/// Overflow cell count above which flooding is triggered.
/// Storm drainage overflow_cells must exceed this value to initiate flooding.
pub(crate) const OVERFLOW_TRIGGER_THRESHOLD: u32 = 10;

/// Conversion factor from stormwater runoff grid units to flood depth in feet.
/// Stormwater runoff is stored as `rainfall_intensity * imperviousness * CELL_AREA`.
/// We normalise into feet of standing water.
pub(crate) const RUNOFF_TO_FEET: f32 = 0.001;

/// Base property value per building capacity unit, used for damage cost estimation.
pub(crate) const BASE_PROPERTY_VALUE_PER_CAPACITY: f64 = 1000.0;

// =============================================================================
// Depth-damage curve data
// =============================================================================

/// Depth breakpoints (in feet) for the depth-damage curves.
pub(crate) const DEPTH_BREAKPOINTS: [f32; 5] = [0.0, 1.0, 3.0, 6.0, 10.0];

/// Damage fractions for Residential zones at each depth breakpoint.
pub(crate) const RESIDENTIAL_DAMAGE: [f32; 5] = [0.0, 0.10, 0.35, 0.65, 0.90];

/// Damage fractions for Commercial zones at each depth breakpoint.
pub(crate) const COMMERCIAL_DAMAGE: [f32; 5] = [0.0, 0.05, 0.20, 0.50, 0.80];

/// Damage fractions for Industrial zones at each depth breakpoint.
pub(crate) const INDUSTRIAL_DAMAGE: [f32; 5] = [0.0, 0.03, 0.15, 0.40, 0.70];

// =============================================================================
// Depth-damage curve lookup
// =============================================================================

/// Linearly interpolate the damage fraction for a given `depth` (feet) using the
/// provided breakpoint and damage arrays.
///
/// Depths below the first breakpoint return 0.0; depths above the last breakpoint
/// return the maximum damage fraction.
pub fn interpolate_damage(depth: f32, breakpoints: &[f32; 5], damages: &[f32; 5]) -> f32 {
    if depth <= breakpoints[0] {
        return damages[0];
    }
    for i in 1..breakpoints.len() {
        if depth <= breakpoints[i] {
            let t = (depth - breakpoints[i - 1]) / (breakpoints[i] - breakpoints[i - 1]);
            return damages[i - 1] + t * (damages[i] - damages[i - 1]);
        }
    }
    // Beyond the last breakpoint: return maximum damage
    damages[breakpoints.len() - 1]
}

/// Returns the damage fraction for a given `depth` (feet) and `zone` type.
///
/// Residential, Commercial, and Industrial zones each have distinct curves.
/// Office and MixedUse zones use the Commercial curve. All other zones (None,
/// unzoned) return 0.0 damage.
pub fn depth_damage_fraction(depth: f32, zone: ZoneType) -> f32 {
    if zone.is_residential() {
        interpolate_damage(depth, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE)
    } else if zone.is_commercial() || matches!(zone, ZoneType::Office | ZoneType::MixedUse) {
        interpolate_damage(depth, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE)
    } else if matches!(zone, ZoneType::Industrial) {
        interpolate_damage(depth, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE)
    } else {
        0.0
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Depth-damage curve interpolation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_interpolate_damage_at_breakpoints() {
        // At each exact breakpoint, the result should match the damage table
        for i in 0..DEPTH_BREAKPOINTS.len() {
            let d = interpolate_damage(
                DEPTH_BREAKPOINTS[i],
                &DEPTH_BREAKPOINTS,
                &RESIDENTIAL_DAMAGE,
            );
            assert!(
                (d - RESIDENTIAL_DAMAGE[i]).abs() < f32::EPSILON,
                "Residential damage at {} ft should be {}, got {}",
                DEPTH_BREAKPOINTS[i],
                RESIDENTIAL_DAMAGE[i],
                d
            );
        }
    }

    #[test]
    fn test_interpolate_damage_between_breakpoints() {
        // At 2.0 ft (midpoint between 1.0 and 3.0), residential should interpolate
        // between 0.10 and 0.35 => 0.10 + 0.5 * 0.25 = 0.225
        let d = interpolate_damage(2.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            (d - 0.225).abs() < 0.001,
            "Residential damage at 2.0 ft should be ~0.225, got {}",
            d
        );
    }

    #[test]
    fn test_interpolate_damage_below_zero() {
        let d = interpolate_damage(-1.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            d.abs() < f32::EPSILON,
            "Damage at negative depth should be 0.0, got {}",
            d
        );
    }

    #[test]
    fn test_interpolate_damage_above_max_breakpoint() {
        // Above 10 ft should return the max damage (0.90 for residential)
        let d = interpolate_damage(15.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            (d - 0.90).abs() < f32::EPSILON,
            "Residential damage above 10 ft should be 0.90, got {}",
            d
        );
    }

    #[test]
    fn test_interpolate_damage_at_zero_depth() {
        let d = interpolate_damage(0.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            d.abs() < f32::EPSILON,
            "Damage at 0 ft should be 0.0, got {}",
            d
        );
    }

    // -------------------------------------------------------------------------
    // Commercial depth-damage curve tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_commercial_damage_at_breakpoints() {
        for i in 0..DEPTH_BREAKPOINTS.len() {
            let d =
                interpolate_damage(DEPTH_BREAKPOINTS[i], &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
            assert!(
                (d - COMMERCIAL_DAMAGE[i]).abs() < f32::EPSILON,
                "Commercial damage at {} ft should be {}, got {}",
                DEPTH_BREAKPOINTS[i],
                COMMERCIAL_DAMAGE[i],
                d
            );
        }
    }

    #[test]
    fn test_commercial_damage_interpolation_midpoint() {
        // At 4.5 ft (midpoint between 3.0 and 6.0), commercial should be
        // 0.20 + 0.5 * (0.50 - 0.20) = 0.20 + 0.15 = 0.35
        let d = interpolate_damage(4.5, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
        assert!(
            (d - 0.35).abs() < 0.001,
            "Commercial damage at 4.5 ft should be ~0.35, got {}",
            d
        );
    }

    // -------------------------------------------------------------------------
    // Industrial depth-damage curve tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_industrial_damage_at_breakpoints() {
        for i in 0..DEPTH_BREAKPOINTS.len() {
            let d =
                interpolate_damage(DEPTH_BREAKPOINTS[i], &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
            assert!(
                (d - INDUSTRIAL_DAMAGE[i]).abs() < f32::EPSILON,
                "Industrial damage at {} ft should be {}, got {}",
                DEPTH_BREAKPOINTS[i],
                INDUSTRIAL_DAMAGE[i],
                d
            );
        }
    }

    #[test]
    fn test_industrial_damage_above_max() {
        let d = interpolate_damage(20.0, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
        assert!(
            (d - 0.70).abs() < f32::EPSILON,
            "Industrial damage above 10 ft should be 0.70, got {}",
            d
        );
    }

    // -------------------------------------------------------------------------
    // Zone-type damage dispatch tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_depth_damage_fraction_residential_zones() {
        for zone in [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
        ] {
            let d = depth_damage_fraction(6.0, zone);
            assert!(
                (d - 0.65).abs() < f32::EPSILON,
                "Residential damage at 6 ft for {:?} should be 0.65, got {}",
                zone,
                d
            );
        }
    }

    #[test]
    fn test_depth_damage_fraction_commercial_zones() {
        for zone in [ZoneType::CommercialLow, ZoneType::CommercialHigh] {
            let d = depth_damage_fraction(6.0, zone);
            assert!(
                (d - 0.50).abs() < f32::EPSILON,
                "Commercial damage at 6 ft for {:?} should be 0.50, got {}",
                zone,
                d
            );
        }
    }

    #[test]
    fn test_depth_damage_fraction_industrial() {
        let d = depth_damage_fraction(6.0, ZoneType::Industrial);
        assert!(
            (d - 0.40).abs() < f32::EPSILON,
            "Industrial damage at 6 ft should be 0.40, got {}",
            d
        );
    }

    #[test]
    fn test_depth_damage_fraction_office_uses_commercial_curve() {
        let office = depth_damage_fraction(3.0, ZoneType::Office);
        let commercial = depth_damage_fraction(3.0, ZoneType::CommercialHigh);
        assert!(
            (office - commercial).abs() < f32::EPSILON,
            "Office should use commercial curve: office={}, commercial={}",
            office,
            commercial
        );
    }

    #[test]
    fn test_depth_damage_fraction_mixed_use_uses_commercial_curve() {
        let mixed = depth_damage_fraction(3.0, ZoneType::MixedUse);
        let commercial = depth_damage_fraction(3.0, ZoneType::CommercialLow);
        assert!(
            (mixed - commercial).abs() < f32::EPSILON,
            "MixedUse should use commercial curve: mixed={}, commercial={}",
            mixed,
            commercial
        );
    }

    #[test]
    fn test_depth_damage_fraction_none_zone_is_zero() {
        let d = depth_damage_fraction(10.0, ZoneType::None);
        assert!(
            d.abs() < f32::EPSILON,
            "None zone should have 0 damage, got {}",
            d
        );
    }

    // -------------------------------------------------------------------------
    // Damage monotonicity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_residential_damage_monotonically_increasing() {
        let mut prev = 0.0_f32;
        for depth_tenths in 0..=120 {
            let depth = depth_tenths as f32 * 0.1;
            let d = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
            assert!(
                d >= prev - f32::EPSILON,
                "Residential damage should be monotonically increasing: {} at {} ft < {} at prev depth",
                d,
                depth,
                prev
            );
            prev = d;
        }
    }

    #[test]
    fn test_commercial_damage_monotonically_increasing() {
        let mut prev = 0.0_f32;
        for depth_tenths in 0..=120 {
            let depth = depth_tenths as f32 * 0.1;
            let d = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
            assert!(
                d >= prev - f32::EPSILON,
                "Commercial damage should be monotonically increasing at {} ft",
                depth
            );
            prev = d;
        }
    }

    #[test]
    fn test_industrial_damage_monotonically_increasing() {
        let mut prev = 0.0_f32;
        for depth_tenths in 0..=120 {
            let depth = depth_tenths as f32 * 0.1;
            let d = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
            assert!(
                d >= prev - f32::EPSILON,
                "Industrial damage should be monotonically increasing at {} ft",
                depth
            );
            prev = d;
        }
    }

    // -------------------------------------------------------------------------
    // Residential > Commercial > Industrial damage ordering
    // -------------------------------------------------------------------------

    #[test]
    fn test_damage_ordering_residential_gt_commercial_gt_industrial() {
        for depth_tenths in 1..=100 {
            let depth = depth_tenths as f32 * 0.1;
            let res = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
            let com = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
            let ind = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
            assert!(
                res >= com - f32::EPSILON,
                "Residential damage ({}) should >= commercial ({}) at {} ft",
                res,
                com,
                depth
            );
            assert!(
                com >= ind - f32::EPSILON,
                "Commercial damage ({}) should >= industrial ({}) at {} ft",
                com,
                ind,
                depth
            );
        }
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_all_constants_positive() {
        assert!(SPREAD_RATE > 0.0);
        assert!(SPREAD_RATE <= 1.0);
        assert!(NATURAL_DRAIN_RATE > 0.0);
        assert!(STORM_DRAIN_RATE > 0.0);
        assert!(FLOOD_DEPTH_THRESHOLD > 0.0);
        assert!(OVERFLOW_TRIGGER_THRESHOLD > 0);
        assert!(RUNOFF_TO_FEET > 0.0);
        assert!(BASE_PROPERTY_VALUE_PER_CAPACITY > 0.0);
        assert!(SPREAD_ITERATIONS > 0);
    }

    #[test]
    fn test_damage_curves_start_at_zero() {
        assert!(RESIDENTIAL_DAMAGE[0].abs() < f32::EPSILON);
        assert!(COMMERCIAL_DAMAGE[0].abs() < f32::EPSILON);
        assert!(INDUSTRIAL_DAMAGE[0].abs() < f32::EPSILON);
    }

    #[test]
    fn test_damage_curves_max_below_one() {
        let last = DEPTH_BREAKPOINTS.len() - 1;
        assert!(RESIDENTIAL_DAMAGE[last] <= 1.0);
        assert!(COMMERCIAL_DAMAGE[last] <= 1.0);
        assert!(INDUSTRIAL_DAMAGE[last] <= 1.0);
    }

    #[test]
    fn test_depth_breakpoints_are_monotonically_increasing() {
        for i in 1..DEPTH_BREAKPOINTS.len() {
            assert!(
                DEPTH_BREAKPOINTS[i] > DEPTH_BREAKPOINTS[i - 1],
                "Breakpoints must be monotonically increasing: {} <= {}",
                DEPTH_BREAKPOINTS[i],
                DEPTH_BREAKPOINTS[i - 1]
            );
        }
    }

    // -------------------------------------------------------------------------
    // Interpolation edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_interpolation_at_exactly_1ft() {
        let res = interpolate_damage(1.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            (res - 0.10).abs() < f32::EPSILON,
            "Residential at 1 ft should be 0.10, got {}",
            res
        );
        let com = interpolate_damage(1.0, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
        assert!(
            (com - 0.05).abs() < f32::EPSILON,
            "Commercial at 1 ft should be 0.05, got {}",
            com
        );
        let ind = interpolate_damage(1.0, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
        assert!(
            (ind - 0.03).abs() < f32::EPSILON,
            "Industrial at 1 ft should be 0.03, got {}",
            ind
        );
    }

    #[test]
    fn test_interpolation_at_8ft() {
        // 8 ft is between 6 ft and 10 ft. t = (8-6)/(10-6) = 0.5
        // Residential: 0.65 + 0.5 * (0.90 - 0.65) = 0.65 + 0.125 = 0.775
        let res = interpolate_damage(8.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            (res - 0.775).abs() < 0.001,
            "Residential at 8 ft should be ~0.775, got {}",
            res
        );

        // Commercial: 0.50 + 0.5 * (0.80 - 0.50) = 0.50 + 0.15 = 0.65
        let com = interpolate_damage(8.0, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
        assert!(
            (com - 0.65).abs() < 0.001,
            "Commercial at 8 ft should be ~0.65, got {}",
            com
        );

        // Industrial: 0.40 + 0.5 * (0.70 - 0.40) = 0.40 + 0.15 = 0.55
        let ind = interpolate_damage(8.0, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
        assert!(
            (ind - 0.55).abs() < 0.001,
            "Industrial at 8 ft should be ~0.55, got {}",
            ind
        );
    }
}
