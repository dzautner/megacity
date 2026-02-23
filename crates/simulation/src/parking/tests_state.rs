//! Tests for parking policy state, computation functions, and saveable trait.

use super::*;

use crate::grid::ZoneType;

// -------------------------------------------------------------------------
// Effective parking ratio tests
// -------------------------------------------------------------------------

#[test]
fn test_default_policy_full_ratio() {
    let state = ParkingPolicyState::default();
    let ratio = effective_parking_ratio(ZoneType::ResidentialLow, &state);
    assert!((ratio - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_eliminate_minimums_zero_ratio() {
    let state = ParkingPolicyState {
        eliminate_minimums: true,
        parking_maximum: false,
    };
    let ratio = effective_parking_ratio(ZoneType::ResidentialHigh, &state);
    assert!(ratio.abs() < f32::EPSILON);
}

#[test]
fn test_parking_maximum_halves_ratio() {
    let state = ParkingPolicyState {
        eliminate_minimums: false,
        parking_maximum: true,
    };
    let ratio = effective_parking_ratio(ZoneType::ResidentialHigh, &state);
    // 2.0 * 0.5 = 1.0
    assert!((ratio - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_eliminate_minimums_overrides_maximum() {
    let state = ParkingPolicyState {
        eliminate_minimums: true,
        parking_maximum: true,
    };
    let ratio = effective_parking_ratio(ZoneType::CommercialHigh, &state);
    assert!(ratio.abs() < f32::EPSILON);
}

// -------------------------------------------------------------------------
// Required parking spaces tests
// -------------------------------------------------------------------------

#[test]
fn test_required_spaces_residential_low() {
    let state = ParkingPolicyState::default();
    // 10 units * 1.0 ratio = 10 spaces
    let spaces = required_parking_spaces(ZoneType::ResidentialLow, 10, &state);
    assert_eq!(spaces, 10);
}

#[test]
fn test_required_spaces_residential_high() {
    let state = ParkingPolicyState::default();
    // 50 units * 2.0 ratio = 100 spaces
    let spaces = required_parking_spaces(ZoneType::ResidentialHigh, 50, &state);
    assert_eq!(spaces, 100);
}

#[test]
fn test_required_spaces_commercial() {
    let state = ParkingPolicyState::default();
    // 30 units * 3.3 ratio = 99 spaces
    let spaces = required_parking_spaces(ZoneType::CommercialHigh, 30, &state);
    assert_eq!(spaces, 99);
}

#[test]
fn test_required_spaces_industrial() {
    let state = ParkingPolicyState::default();
    // 20 units * 2.0 ratio = 40 spaces
    let spaces = required_parking_spaces(ZoneType::Industrial, 20, &state);
    assert_eq!(spaces, 40);
}

#[test]
fn test_required_spaces_zero_with_eliminated_minimums() {
    let state = ParkingPolicyState {
        eliminate_minimums: true,
        parking_maximum: false,
    };
    let spaces = required_parking_spaces(ZoneType::ResidentialHigh, 100, &state);
    assert_eq!(spaces, 0);
}

#[test]
fn test_required_spaces_reduced_with_maximum() {
    let state_default = ParkingPolicyState::default();
    let state_max = ParkingPolicyState {
        eliminate_minimums: false,
        parking_maximum: true,
    };
    let spaces_default = required_parking_spaces(ZoneType::Industrial, 20, &state_default);
    let spaces_max = required_parking_spaces(ZoneType::Industrial, 20, &state_max);
    assert!(spaces_max < spaces_default);
    // 20 * 2.0 * 0.5 = 20 (ceiling)
    assert_eq!(spaces_max, 20);
}

#[test]
fn test_required_spaces_zero_capacity() {
    let state = ParkingPolicyState::default();
    let spaces = required_parking_spaces(ZoneType::ResidentialLow, 0, &state);
    assert_eq!(spaces, 0);
}

#[test]
fn test_required_spaces_none_zone() {
    let state = ParkingPolicyState::default();
    let spaces = required_parking_spaces(ZoneType::None, 100, &state);
    assert_eq!(spaces, 0);
}

// -------------------------------------------------------------------------
// Parking construction cost tests
// -------------------------------------------------------------------------

#[test]
fn test_parking_cost_residential_low() {
    let state = ParkingPolicyState::default();
    // 10 units * 1.0 ratio = 10 spaces * $5K = $50K
    let cost = parking_construction_cost(ZoneType::ResidentialLow, 10, &state);
    assert!((cost - 50_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_parking_cost_residential_high() {
    let state = ParkingPolicyState::default();
    // 50 units * 2.0 ratio = 100 spaces * $20K = $2M
    let cost = parking_construction_cost(ZoneType::ResidentialHigh, 50, &state);
    assert!((cost - 2_000_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_parking_cost_zero_with_eliminated_minimums() {
    let state = ParkingPolicyState {
        eliminate_minimums: true,
        parking_maximum: false,
    };
    let cost = parking_construction_cost(ZoneType::ResidentialHigh, 100, &state);
    assert!(cost.abs() < f64::EPSILON);
}

#[test]
fn test_parking_cost_reduced_with_maximum() {
    let state_default = ParkingPolicyState::default();
    let state_max = ParkingPolicyState {
        eliminate_minimums: false,
        parking_maximum: true,
    };
    let cost_default = parking_construction_cost(ZoneType::ResidentialHigh, 50, &state_default);
    let cost_max = parking_construction_cost(ZoneType::ResidentialHigh, 50, &state_max);
    assert!(cost_max < cost_default);
}

// -------------------------------------------------------------------------
// Ratio multiplier tests
// -------------------------------------------------------------------------

#[test]
fn test_ratio_multiplier_default() {
    let state = ParkingPolicyState::default();
    assert!((ratio_multiplier(&state) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_ratio_multiplier_eliminated() {
    let state = ParkingPolicyState {
        eliminate_minimums: true,
        parking_maximum: false,
    };
    assert!(ratio_multiplier(&state).abs() < f32::EPSILON);
}

#[test]
fn test_ratio_multiplier_maximum() {
    let state = ParkingPolicyState {
        eliminate_minimums: false,
        parking_maximum: true,
    };
    assert!((ratio_multiplier(&state) - PARKING_MAXIMUM_FRACTION).abs() < f32::EPSILON);
}

// -------------------------------------------------------------------------
// Default state tests
// -------------------------------------------------------------------------

#[test]
fn test_default_policy_state() {
    let state = ParkingPolicyState::default();
    assert!(!state.eliminate_minimums);
    assert!(!state.parking_maximum);
}

#[test]
fn test_default_effects() {
    let effects = ParkingEffects::default();
    assert_eq!(effects.total_required_spaces, 0);
    assert!(effects.total_parking_cost.abs() < f64::EPSILON);
    assert!(effects.effective_ratio_multiplier.abs() < f32::EPSILON);
    assert!(!effects.minimums_eliminated);
    assert!(!effects.maximum_active);
    assert_eq!(effects.buildings_affected, 0);
}

// -------------------------------------------------------------------------
// Saveable trait tests
// -------------------------------------------------------------------------

#[test]
fn test_saveable_skips_default() {
    use crate::Saveable;
    let state = ParkingPolicyState::default();
    assert!(state.save_to_bytes().is_none());
}

#[test]
fn test_saveable_saves_when_active() {
    use crate::Saveable;
    let state = ParkingPolicyState {
        eliminate_minimums: true,
        parking_maximum: false,
    };
    assert!(state.save_to_bytes().is_some());
}

#[test]
fn test_saveable_roundtrip() {
    use crate::Saveable;
    let state = ParkingPolicyState {
        eliminate_minimums: true,
        parking_maximum: true,
    };
    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = ParkingPolicyState::load_from_bytes(&bytes);
    assert_eq!(restored.eliminate_minimums, state.eliminate_minimums);
    assert_eq!(restored.parking_maximum, state.parking_maximum);
}

#[test]
fn test_saveable_key() {
    use crate::Saveable;
    assert_eq!(ParkingPolicyState::SAVE_KEY, "parking_policy");
}

// -------------------------------------------------------------------------
// Policy integration-style tests
// -------------------------------------------------------------------------

#[test]
fn test_eliminate_minimums_removes_all_costs() {
    let state = ParkingPolicyState {
        eliminate_minimums: true,
        parking_maximum: false,
    };
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
        let cost = parking_construction_cost(zone, 100, &state);
        assert!(
            cost.abs() < f64::EPSILON,
            "Zone {:?} should have zero parking cost with minimums eliminated",
            zone
        );
    }
}

#[test]
fn test_parking_maximum_reduces_but_nonzero() {
    let state = ParkingPolicyState {
        eliminate_minimums: false,
        parking_maximum: true,
    };
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
        let cost_max = parking_construction_cost(zone, 100, &state);
        let cost_default = parking_construction_cost(zone, 100, &ParkingPolicyState::default());
        assert!(
            cost_max < cost_default,
            "Zone {:?} parking cost with maximum (${}) should be less than default (${})",
            zone,
            cost_max,
            cost_default
        );
        assert!(
            cost_max > 0.0,
            "Zone {:?} parking cost with maximum should still be positive",
            zone
        );
    }
}
