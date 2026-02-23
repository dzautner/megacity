//! Tests for the search/filter feature.

use super::helpers::{citizen_name, education_label, zone_label};
use super::types::SearchState;

use bevy::prelude::*;
use simulation::citizen::Gender;
use simulation::grid::ZoneType;

#[test]
fn test_zone_label_all_variants() {
    assert_eq!(zone_label(ZoneType::None), "None");
    assert_eq!(zone_label(ZoneType::ResidentialLow), "Residential (Low)");
    assert_eq!(zone_label(ZoneType::ResidentialMedium), "Residential (Med)");
    assert_eq!(zone_label(ZoneType::ResidentialHigh), "Residential (High)");
    assert_eq!(zone_label(ZoneType::CommercialLow), "Commercial (Low)");
    assert_eq!(zone_label(ZoneType::CommercialHigh), "Commercial (High)");
    assert_eq!(zone_label(ZoneType::Industrial), "Industrial");
    assert_eq!(zone_label(ZoneType::Office), "Office");
    assert_eq!(zone_label(ZoneType::MixedUse), "Mixed Use");
}

#[test]
fn test_education_label() {
    assert_eq!(education_label(0), "None");
    assert_eq!(education_label(1), "Elementary");
    assert_eq!(education_label(2), "High School");
    assert_eq!(education_label(3), "University");
    assert_eq!(education_label(4), "Advanced");
}

#[test]
fn test_citizen_name_deterministic() {
    let entity = Entity::from_raw(42);
    let name1 = citizen_name(entity, Gender::Male);
    let name2 = citizen_name(entity, Gender::Male);
    assert_eq!(name1, name2, "Names should be deterministic");
}

#[test]
fn test_citizen_name_gender_difference() {
    let entity = Entity::from_raw(0);
    let male_name = citizen_name(entity, Gender::Male);
    let female_name = citizen_name(entity, Gender::Female);
    assert_ne!(male_name, female_name, "Male/female names should differ");
}

#[test]
fn test_search_state_default() {
    let state = SearchState::default();
    assert!(!state.visible);
    assert!(state.query.is_empty());
    assert!(state.search_buildings);
    assert!(state.search_citizens);
    assert!(state.building_results.is_empty());
    assert!(state.citizen_results.is_empty());
}
