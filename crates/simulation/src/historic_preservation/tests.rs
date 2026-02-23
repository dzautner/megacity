//! Tests for historic preservation.

use super::*;

// -------------------------------------------------------------------------
// Default state tests
// -------------------------------------------------------------------------

#[test]
fn test_default_state() {
    let state = HistoricPreservationState::default();
    assert!(state.preserved_districts.is_empty());
    assert!(state.removal_penalties.is_empty());
    assert_eq!(state.historic_tourism_visitors, 0);
}

// -------------------------------------------------------------------------
// Designation and removal tests
// -------------------------------------------------------------------------

#[test]
fn test_designate_district() {
    let mut state = HistoricPreservationState::default();
    state.designate(0);
    assert!(state.is_preserved(0));
    assert!(!state.is_preserved(1));
}

#[test]
fn test_designate_multiple_districts() {
    let mut state = HistoricPreservationState::default();
    state.designate(0);
    state.designate(3);
    state.designate(5);
    assert!(state.is_preserved(0));
    assert!(state.is_preserved(3));
    assert!(state.is_preserved(5));
    assert!(!state.is_preserved(1));
    assert_eq!(state.preserved_districts.len(), 3);
}

#[test]
fn test_designate_idempotent() {
    let mut state = HistoricPreservationState::default();
    state.designate(0);
    state.designate(0);
    assert_eq!(state.preserved_districts.len(), 1);
}

#[test]
fn test_remove_designation() {
    let mut state = HistoricPreservationState::default();
    state.designate(0);
    state.remove(0);
    assert!(!state.is_preserved(0));
    assert_eq!(state.removal_penalties.len(), 1);
}

#[test]
fn test_remove_nonexistent_no_penalty() {
    let mut state = HistoricPreservationState::default();
    state.remove(5); // never designated
    assert!(state.removal_penalties.is_empty());
}

// -------------------------------------------------------------------------
// Cell preservation tests
// -------------------------------------------------------------------------

#[test]
fn test_is_cell_preserved() {
    let mut state = HistoricPreservationState::default();
    let mut district_map = DistrictMap::default();

    // Assign cell (10, 10) to district 0
    district_map.assign_cell_to_district(10, 10, 0);

    // Not preserved yet
    assert!(!state.is_cell_preserved(10, 10, &district_map));

    // Designate district 0
    state.designate(0);
    assert!(state.is_cell_preserved(10, 10, &district_map));

    // Cell not in any district
    assert!(!state.is_cell_preserved(100, 100, &district_map));
}

// -------------------------------------------------------------------------
// Building protection tests
// -------------------------------------------------------------------------

#[test]
fn test_is_building_protected() {
    let mut state = HistoricPreservationState::default();
    let mut district_map = DistrictMap::default();

    district_map.assign_cell_to_district(10, 10, 0);
    assert!(!is_building_protected(10, 10, &state, &district_map));

    state.designate(0);
    assert!(is_building_protected(10, 10, &state, &district_map));
}

#[test]
fn test_building_not_protected_outside_district() {
    let mut state = HistoricPreservationState::default();
    let district_map = DistrictMap::default();

    state.designate(0);
    // Cell (200, 200) is not assigned to any district
    assert!(!is_building_protected(200, 200, &state, &district_map));
}

// -------------------------------------------------------------------------
// Land value bonus tests
// -------------------------------------------------------------------------

#[test]
fn test_historic_land_value_bonus_calculation() {
    // 10% of 100 = 10
    assert_eq!(historic_land_value_bonus(100), 10);
    // 10% of 50 = 5
    assert_eq!(historic_land_value_bonus(50), 5);
    // 10% of 0 = 0
    assert_eq!(historic_land_value_bonus(0), 0);
    // 10% of 255 = 25
    assert_eq!(historic_land_value_bonus(255), 25);
}

// -------------------------------------------------------------------------
// Tourism tests
// -------------------------------------------------------------------------

#[test]
fn test_historic_tourism_none() {
    assert_eq!(calculate_historic_tourism(0), 0);
}

#[test]
fn test_historic_tourism_one_district() {
    assert_eq!(
        calculate_historic_tourism(1),
        HISTORIC_TOURISM_VISITORS_PER_DISTRICT
    );
}

#[test]
fn test_historic_tourism_multiple_districts() {
    assert_eq!(
        calculate_historic_tourism(3),
        3 * HISTORIC_TOURISM_VISITORS_PER_DISTRICT
    );
}

// -------------------------------------------------------------------------
// Removal penalty tests
// -------------------------------------------------------------------------

#[test]
fn test_removal_penalty_initial() {
    let mut state = HistoricPreservationState::default();
    state.designate(0);
    state.remove(0);

    let penalty = state.removal_happiness_penalty();
    assert!(
        (penalty - PRESERVATION_REMOVAL_HAPPINESS_PENALTY).abs() < f32::EPSILON,
        "initial penalty should be full: {}",
        penalty
    );
}

#[test]
fn test_removal_penalty_decays() {
    let mut state = HistoricPreservationState::default();
    state.designate(0);
    state.remove(0);

    // Simulate half decay
    state.removal_penalties[0].1 = REMOVAL_PENALTY_DURATION_TICKS / 2;
    let penalty = state.removal_happiness_penalty();
    let expected = PRESERVATION_REMOVAL_HAPPINESS_PENALTY * 0.5;
    assert!(
        (penalty - expected).abs() < 0.1,
        "half-decayed penalty should be ~{}: got {}",
        expected,
        penalty
    );
}

#[test]
fn test_removal_penalty_none_when_empty() {
    let state = HistoricPreservationState::default();
    assert_eq!(state.removal_happiness_penalty(), 0.0);
}

#[test]
fn test_multiple_removal_penalties_stack() {
    let mut state = HistoricPreservationState::default();
    state.designate(0);
    state.designate(1);
    state.remove(0);
    state.remove(1);

    let penalty = state.removal_happiness_penalty();
    let expected = PRESERVATION_REMOVAL_HAPPINESS_PENALTY * 2.0;
    assert!(
        (penalty - expected).abs() < f32::EPSILON,
        "two removals should double penalty: got {}",
        penalty
    );
}

// -------------------------------------------------------------------------
// Saveable trait tests
// -------------------------------------------------------------------------

#[test]
fn test_saveable_skips_default() {
    use crate::Saveable;
    let state = HistoricPreservationState::default();
    assert!(state.save_to_bytes().is_none());
}

#[test]
fn test_saveable_saves_when_active() {
    use crate::Saveable;
    let mut state = HistoricPreservationState::default();
    state.designate(0);
    assert!(state.save_to_bytes().is_some());
}

#[test]
fn test_saveable_roundtrip() {
    use crate::Saveable;
    let mut state = HistoricPreservationState::default();
    state.designate(0);
    state.designate(3);
    state
        .removal_penalties
        .push((5, REMOVAL_PENALTY_DURATION_TICKS));
    state.historic_tourism_visitors = 400;

    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = HistoricPreservationState::load_from_bytes(&bytes);

    assert!(restored.is_preserved(0));
    assert!(restored.is_preserved(3));
    assert!(!restored.is_preserved(1));
    assert_eq!(restored.removal_penalties.len(), 1);
    assert_eq!(restored.removal_penalties[0].0, 5);
    assert_eq!(
        restored.removal_penalties[0].1,
        REMOVAL_PENALTY_DURATION_TICKS
    );
}

#[test]
fn test_saveable_key() {
    use crate::Saveable;
    assert_eq!(HistoricPreservationState::SAVE_KEY, "historic_preservation");
}

#[test]
fn test_saveable_saves_with_penalty_only() {
    use crate::Saveable;
    let mut state = HistoricPreservationState::default();
    // No preserved districts, but there's an active removal penalty
    state
        .removal_penalties
        .push((0, REMOVAL_PENALTY_DURATION_TICKS));
    assert!(state.save_to_bytes().is_some());
}

// -------------------------------------------------------------------------
// Constants validation tests
// -------------------------------------------------------------------------

#[test]
fn test_constants_are_reasonable() {
    assert!(HISTORIC_LAND_VALUE_BONUS > 0.0);
    assert!(HISTORIC_LAND_VALUE_BONUS < 1.0);
    assert!(HISTORIC_TOURISM_VISITORS_PER_DISTRICT > 0);
    assert!(PRESERVATION_REMOVAL_HAPPINESS_PENALTY > 0.0);
    assert!(REMOVAL_PENALTY_DURATION_TICKS > 0);
}
