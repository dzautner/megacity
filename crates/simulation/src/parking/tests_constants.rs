//! Tests for parking constants, ratios, and per-zone cost tables.

use crate::grid::ZoneType;

use super::*;

// -------------------------------------------------------------------------
// Parking ratio tests
// -------------------------------------------------------------------------

#[test]
fn test_residential_low_ratio() {
    assert!((parking_ratio(ZoneType::ResidentialLow) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_residential_medium_ratio() {
    assert!((parking_ratio(ZoneType::ResidentialMedium) - 1.5).abs() < f32::EPSILON);
}

#[test]
fn test_residential_high_ratio() {
    assert!((parking_ratio(ZoneType::ResidentialHigh) - 2.0).abs() < f32::EPSILON);
}

#[test]
fn test_commercial_low_ratio() {
    assert!((parking_ratio(ZoneType::CommercialLow) - 3.3).abs() < 0.01);
}

#[test]
fn test_commercial_high_ratio() {
    assert!((parking_ratio(ZoneType::CommercialHigh) - 3.3).abs() < 0.01);
}

#[test]
fn test_industrial_ratio() {
    assert!((parking_ratio(ZoneType::Industrial) - 2.0).abs() < f32::EPSILON);
}

#[test]
fn test_office_ratio() {
    assert!((parking_ratio(ZoneType::Office) - 2.5).abs() < f32::EPSILON);
}

#[test]
fn test_mixed_use_ratio() {
    assert!((parking_ratio(ZoneType::MixedUse) - 2.5).abs() < f32::EPSILON);
}

#[test]
fn test_none_zone_ratio() {
    assert!((parking_ratio(ZoneType::None)).abs() < f32::EPSILON);
}

// -------------------------------------------------------------------------
// Parking cost per space tests
// -------------------------------------------------------------------------

#[test]
fn test_residential_low_cost() {
    assert!((parking_cost_per_space(ZoneType::ResidentialLow) - 5_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_residential_high_cost() {
    assert!((parking_cost_per_space(ZoneType::ResidentialHigh) - 20_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_industrial_cost() {
    assert!((parking_cost_per_space(ZoneType::Industrial) - 5_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_office_cost() {
    assert!((parking_cost_per_space(ZoneType::Office) - 15_000.0).abs() < f64::EPSILON);
}

// -------------------------------------------------------------------------
// Constant verification tests
// -------------------------------------------------------------------------

#[test]
fn test_constant_values() {
    assert_eq!(PARKING_COST_LOW_DENSITY, 5_000.0);
    assert_eq!(PARKING_COST_MEDIUM_DENSITY, 10_000.0);
    assert_eq!(PARKING_COST_HIGH_DENSITY, 20_000.0);
    assert_eq!(PARKING_COST_INDUSTRIAL, 5_000.0);
    assert_eq!(PARKING_COST_OFFICE, 15_000.0);
    assert_eq!(PARKING_MAXIMUM_FRACTION, 0.5);
}

// -------------------------------------------------------------------------
// Integration-style tests
// -------------------------------------------------------------------------

#[test]
fn test_all_zones_have_parking_ratios() {
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];
    for zone in zones {
        assert!(
            parking_ratio(zone) > 0.0,
            "Zone {:?} should have a positive parking ratio",
            zone
        );
    }
}

#[test]
fn test_all_zones_have_parking_costs() {
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];
    for zone in zones {
        assert!(
            parking_cost_per_space(zone) > 0.0,
            "Zone {:?} should have a positive parking cost",
            zone
        );
    }
}

#[test]
fn test_cost_in_valid_range() {
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];
    for zone in zones {
        let cost = parking_cost_per_space(zone);
        assert!(
            (5_000.0..=20_000.0).contains(&cost),
            "Zone {:?} parking cost ${} should be between $5K and $20K",
            zone,
            cost
        );
    }
}

#[test]
fn test_high_density_more_expensive_than_low() {
    assert!(
        parking_cost_per_space(ZoneType::ResidentialHigh)
            > parking_cost_per_space(ZoneType::ResidentialLow)
    );
}

#[test]
fn test_residential_ratios_in_range() {
    // Issue spec: Residential 1-2 per unit
    assert!(parking_ratio(ZoneType::ResidentialLow) >= 1.0);
    assert!(parking_ratio(ZoneType::ResidentialLow) <= 2.0);
    assert!(parking_ratio(ZoneType::ResidentialMedium) >= 1.0);
    assert!(parking_ratio(ZoneType::ResidentialMedium) <= 2.0);
    assert!(parking_ratio(ZoneType::ResidentialHigh) >= 1.0);
    assert!(parking_ratio(ZoneType::ResidentialHigh) <= 2.0);
}
