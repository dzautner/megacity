//! Unit tests for energy demand calculations.

use super::systems::{compute_demand_mw, time_of_use_multiplier};
use super::types::{EnergyConsumer, EnergyGrid, LoadPriority};
use crate::grid::ZoneType;
use crate::services::ServiceType;

#[test]
fn test_time_of_use_off_peak() {
    assert!((time_of_use_multiplier(0.0) - 0.6).abs() < f32::EPSILON);
    assert!((time_of_use_multiplier(3.0) - 0.6).abs() < f32::EPSILON);
    assert!((time_of_use_multiplier(5.5) - 0.6).abs() < f32::EPSILON);
    assert!((time_of_use_multiplier(22.0) - 0.6).abs() < f32::EPSILON);
    assert!((time_of_use_multiplier(23.5) - 0.6).abs() < f32::EPSILON);
}

#[test]
fn test_time_of_use_mid_peak() {
    assert!((time_of_use_multiplier(6.0) - 1.0).abs() < f32::EPSILON);
    assert!((time_of_use_multiplier(10.0) - 1.0).abs() < f32::EPSILON);
    assert!((time_of_use_multiplier(13.5) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_time_of_use_on_peak() {
    assert!((time_of_use_multiplier(14.0) - 1.5).abs() < f32::EPSILON);
    assert!((time_of_use_multiplier(18.0) - 1.5).abs() < f32::EPSILON);
    assert!((time_of_use_multiplier(21.5) - 1.5).abs() < f32::EPSILON);
}

#[test]
fn test_compute_demand_residential() {
    let demand = compute_demand_mw(1_000.0, 1.0, 1.0, 1.0);
    let expected = 1_000.0 / 720.0 / 1000.0;
    assert!(
        (demand - expected).abs() < 0.0001,
        "Expected {expected}, got {demand}"
    );
}

#[test]
fn test_compute_demand_with_on_peak() {
    let demand = compute_demand_mw(1_000.0, 1.5, 1.0, 1.0);
    let expected = 1_000.0 * 1.5 / 720.0 / 1000.0;
    assert!(
        (demand - expected).abs() < 0.0001,
        "Expected {expected}, got {demand}"
    );
}

#[test]
fn test_compute_demand_with_hvac_and_season() {
    let demand = compute_demand_mw(1_000.0, 1.0, 1.3, 1.4);
    let expected = 1_000.0 * 1.3 * 1.4 / 720.0 / 1000.0;
    assert!(
        (demand - expected).abs() < 0.0001,
        "Expected {expected}, got {demand}"
    );
}

#[test]
fn test_compute_demand_data_center() {
    let demand = compute_demand_mw(500_000.0, 1.5, 1.0, 1.0);
    let expected = 500_000.0 * 1.5 / 720.0 / 1000.0;
    assert!(
        (demand - expected).abs() < 0.01,
        "Expected {expected}, got {demand}"
    );
}

#[test]
fn test_compute_demand_hospital() {
    let demand = compute_demand_mw(200_000.0, 1.0, 1.0, 1.0);
    let expected = 200_000.0 / 720.0 / 1000.0;
    assert!(
        (demand - expected).abs() < 0.01,
        "Expected {expected}, got {demand}"
    );
}

#[test]
fn test_base_demand_for_zones() {
    assert!(
        (EnergyConsumer::base_demand_for_zone(ZoneType::ResidentialLow) - 1_000.0).abs()
            < f32::EPSILON
    );
    assert!(
        (EnergyConsumer::base_demand_for_zone(ZoneType::ResidentialMedium) - 1_000.0).abs()
            < f32::EPSILON
    );
    assert!(
        (EnergyConsumer::base_demand_for_zone(ZoneType::ResidentialHigh) - 1_000.0).abs()
            < f32::EPSILON
    );
    assert!(
        (EnergyConsumer::base_demand_for_zone(ZoneType::CommercialLow) - 3_000.0).abs()
            < f32::EPSILON
    );
    assert!(
        (EnergyConsumer::base_demand_for_zone(ZoneType::CommercialHigh) - 15_000.0).abs()
            < f32::EPSILON
    );
    assert!(
        (EnergyConsumer::base_demand_for_zone(ZoneType::Industrial) - 50_000.0).abs()
            < f32::EPSILON
    );
    assert!((EnergyConsumer::base_demand_for_zone(ZoneType::None) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_base_demand_for_services() {
    assert!(
        (EnergyConsumer::base_demand_for_service(ServiceType::Hospital) - 200_000.0).abs()
            < f32::EPSILON
    );
    assert!(
        (EnergyConsumer::base_demand_for_service(ServiceType::DataCenter) - 500_000.0).abs()
            < f32::EPSILON
    );
}

#[test]
fn test_priority_for_zones() {
    assert_eq!(
        EnergyConsumer::priority_for_zone(ZoneType::ResidentialLow),
        LoadPriority::High
    );
    assert_eq!(
        EnergyConsumer::priority_for_zone(ZoneType::Industrial),
        LoadPriority::Normal
    );
    assert_eq!(
        EnergyConsumer::priority_for_zone(ZoneType::None),
        LoadPriority::Low
    );
}

#[test]
fn test_priority_for_services() {
    assert_eq!(
        EnergyConsumer::priority_for_service(ServiceType::Hospital),
        LoadPriority::Critical
    );
    assert_eq!(
        EnergyConsumer::priority_for_service(ServiceType::DataCenter),
        LoadPriority::Critical
    );
    assert_eq!(
        EnergyConsumer::priority_for_service(ServiceType::SmallPark),
        LoadPriority::Low
    );
    assert_eq!(
        EnergyConsumer::priority_for_service(ServiceType::Museum),
        LoadPriority::Normal
    );
}

#[test]
fn test_energy_grid_default() {
    let grid = EnergyGrid::default();
    assert_eq!(grid.total_demand_mwh, 0.0);
    assert_eq!(grid.total_supply_mwh, 0.0);
    assert_eq!(grid.reserve_margin, 1.0);
    assert_eq!(grid.consumer_count, 0);
}

#[test]
fn test_energy_grid_saveable_roundtrip() {
    use crate::Saveable;
    let grid = EnergyGrid {
        total_demand_mwh: 42.5,
        total_supply_mwh: 60.0,
        reserve_margin: 0.29,
        consumer_count: 100,
    };
    let bytes = grid.save_to_bytes().expect("should serialize");
    let restored = EnergyGrid::load_from_bytes(&bytes);
    assert!((restored.total_demand_mwh - 42.5).abs() < 0.01);
    assert!((restored.total_supply_mwh - 60.0).abs() < 0.01);
    assert_eq!(restored.consumer_count, 100);
}

#[test]
fn test_reserve_margin_with_surplus() {
    let supply = 100.0_f32;
    let demand = 80.0_f32;
    let margin = (supply - demand) / supply;
    assert!((margin - 0.2).abs() < 0.001);
}

#[test]
fn test_reserve_margin_with_deficit() {
    let supply = 50.0_f32;
    let demand = 80.0_f32;
    let margin = (supply - demand) / supply;
    assert!((margin - (-0.6)).abs() < 0.001);
}

#[test]
fn test_off_peak_demand_is_lower_than_on_peak() {
    let base = 10_000.0;
    let off_peak = compute_demand_mw(base, 0.6, 1.0, 1.0);
    let on_peak = compute_demand_mw(base, 1.5, 1.0, 1.0);
    assert!(
        off_peak < on_peak,
        "Off-peak demand ({off_peak}) should be lower than on-peak ({on_peak})"
    );
    let ratio = off_peak / on_peak;
    assert!((ratio - 0.4).abs() < 0.001);
}

#[test]
fn test_hvac_modifier_increases_demand() {
    let base = 10_000.0;
    let mild = compute_demand_mw(base, 1.0, 1.0, 1.0);
    let cold = compute_demand_mw(base, 1.0, 1.6, 1.0);
    assert!(
        cold > mild,
        "Cold weather demand ({cold}) should exceed mild ({mild})"
    );
    assert!((cold / mild - 1.6).abs() < 0.001);
}

#[test]
fn test_industrial_demand_exceeds_residential() {
    let res = compute_demand_mw(1_000.0, 1.0, 1.0, 1.0);
    let ind = compute_demand_mw(50_000.0, 1.0, 1.0, 1.0);
    assert!(
        ind > res,
        "Industrial ({ind}) should exceed residential ({res})"
    );
    assert!((ind / res - 50.0).abs() < 0.01);
}

#[test]
fn test_zero_base_demand_yields_zero() {
    let demand = compute_demand_mw(0.0, 1.5, 1.6, 1.4);
    assert!(demand.abs() < f32::EPSILON);
}
