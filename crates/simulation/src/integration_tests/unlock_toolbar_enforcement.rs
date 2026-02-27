//! Integration tests for toolbar unlock enforcement (PLAY-P0-01, issue #1744).
//!
//! These tests verify that the UnlockState correctly gates service and
//! utility types that correspond to toolbar tools.

use crate::services::ServiceType;
use crate::unlocks::{UnlockNode, UnlockState};
use crate::utilities::UtilityType;

// ====================================================================
// Toolbar enforcement: locked utilities cannot pass unlock check
// ====================================================================

#[test]
fn test_locked_utility_solar_farm_blocked_at_start() {
    let state = UnlockState::default();
    assert!(
        !state.is_utility_unlocked(UtilityType::SolarFarm),
        "Solar farm should be locked at game start"
    );
}

#[test]
fn test_locked_utility_nuclear_plant_blocked_at_start() {
    let state = UnlockState::default();
    assert!(
        !state.is_utility_unlocked(UtilityType::NuclearPlant),
        "Nuclear plant should be locked at game start"
    );
}

#[test]
fn test_locked_utility_sewage_plant_blocked_at_start() {
    let state = UnlockState::default();
    assert!(
        !state.is_utility_unlocked(UtilityType::SewagePlant),
        "Sewage plant should be locked at game start"
    );
}

#[test]
fn test_starter_utilities_unlocked_at_start() {
    let state = UnlockState::default();
    assert!(
        state.is_utility_unlocked(UtilityType::PowerPlant),
        "Coal power plant should be unlocked at start"
    );
    assert!(
        state.is_utility_unlocked(UtilityType::WaterTower),
        "Water tower should be unlocked at start"
    );
    assert!(
        state.is_utility_unlocked(UtilityType::PumpingStation),
        "Pumping station should be unlocked at start"
    );
}

// ====================================================================
// Toolbar enforcement: locked services cannot pass unlock check
// ====================================================================

#[test]
fn test_locked_service_fire_station_blocked_at_start() {
    let state = UnlockState::default();
    assert!(
        !state.is_service_unlocked(ServiceType::FireStation),
        "Fire station should be locked at game start"
    );
}

#[test]
fn test_locked_service_police_station_blocked_at_start() {
    let state = UnlockState::default();
    assert!(
        !state.is_service_unlocked(ServiceType::PoliceStation),
        "Police station should be locked at game start"
    );
}

#[test]
fn test_locked_service_hospital_blocked_at_start() {
    let state = UnlockState::default();
    assert!(
        !state.is_service_unlocked(ServiceType::Hospital),
        "Hospital should be locked at game start"
    );
}

#[test]
fn test_locked_service_international_airport_blocked_at_start() {
    let state = UnlockState::default();
    assert!(
        !state.is_service_unlocked(ServiceType::InternationalAirport),
        "International airport should be locked at game start"
    );
}

// ====================================================================
// After purchasing unlock, service/utility passes check
// ====================================================================

#[test]
fn test_unlocked_fire_station_after_purchase() {
    let mut state = UnlockState::default();
    state.development_points = 10;
    assert!(state.purchase(UnlockNode::FireService));
    assert!(
        state.is_service_unlocked(ServiceType::FireStation),
        "Fire station should be available after purchasing FireService"
    );
    assert!(
        state.is_service_unlocked(ServiceType::FireHouse),
        "Fire house should be available after purchasing FireService"
    );
}

#[test]
fn test_unlocked_solar_farm_after_purchase() {
    let mut state = UnlockState::default();
    state.development_points = 10;
    assert!(state.purchase(UnlockNode::SolarPower));
    assert!(
        state.is_utility_unlocked(UtilityType::SolarFarm),
        "Solar farm should be available after purchasing SolarPower"
    );
}

#[test]
fn test_unlocked_nuclear_plant_after_purchase() {
    let mut state = UnlockState::default();
    state.development_points = 10;
    assert!(state.purchase(UnlockNode::NuclearPower));
    assert!(
        state.is_utility_unlocked(UtilityType::NuclearPlant),
        "Nuclear plant should be available after purchasing NuclearPower"
    );
}

// ====================================================================
// All starter nodes should be unlocked by default
// ====================================================================

#[test]
fn test_all_starter_nodes_unlocked_by_default() {
    let state = UnlockState::default();
    let starters = [
        UnlockNode::BasicRoads,
        UnlockNode::ResidentialZoning,
        UnlockNode::CommercialZoning,
        UnlockNode::IndustrialZoning,
        UnlockNode::BasicPower,
        UnlockNode::BasicWater,
    ];
    for node in &starters {
        assert!(
            state.is_unlocked(*node),
            "Starter node {:?} should be unlocked by default",
            node
        );
    }
}

#[test]
fn test_non_starter_nodes_locked_by_default() {
    let state = UnlockState::default();
    let locked_nodes = [
        UnlockNode::HealthCare,
        UnlockNode::DeathCare,
        UnlockNode::BasicSanitation,
        UnlockNode::FireService,
        UnlockNode::PoliceService,
        UnlockNode::ElementaryEducation,
        UnlockNode::HighSchoolEducation,
        UnlockNode::SmallParks,
        UnlockNode::PublicTransport,
        UnlockNode::Landmarks,
        UnlockNode::HighDensityResidential,
        UnlockNode::HighDensityCommercial,
        UnlockNode::OfficeZoning,
        UnlockNode::UniversityEducation,
        UnlockNode::SolarPower,
        UnlockNode::WindPower,
        UnlockNode::NuclearPower,
        UnlockNode::InternationalAirports,
    ];
    for node in &locked_nodes {
        assert!(
            !state.is_unlocked(*node),
            "Node {:?} should be locked by default",
            node
        );
    }
}
