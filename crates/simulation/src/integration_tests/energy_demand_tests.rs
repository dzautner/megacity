//! Integration tests for the energy demand system (POWER-001).

use crate::energy_demand::{EnergyConsumer, EnergyGrid, LoadPriority};
use crate::grid::ZoneType;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

#[test]
fn test_energy_demand_building_gets_consumer_component() {
    let mut city = TestCity::new().with_building(10, 10, ZoneType::ResidentialLow, 1);

    // Run a tick to let the attach system run
    city.tick(1);

    let world = city.world_mut();
    let count = world.query::<&EnergyConsumer>().iter(world).count();
    assert!(
        count >= 1,
        "Expected at least 1 EnergyConsumer, got {count}"
    );
}

#[test]
fn test_energy_demand_service_gets_consumer_component() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Hospital);

    city.tick(1);

    let world = city.world_mut();
    let consumers: Vec<&EnergyConsumer> = world.query::<&EnergyConsumer>().iter(world).collect();

    assert!(
        !consumers.is_empty(),
        "Hospital should have an EnergyConsumer component"
    );

    let hospital_consumer = consumers
        .iter()
        .find(|c| (c.base_demand_kwh - 200_000.0).abs() < f32::EPSILON);
    assert!(
        hospital_consumer.is_some(),
        "Hospital should have 200,000 kWh/month base demand"
    );
}

#[test]
fn test_energy_demand_aggregation_nonzero_after_ticks() {
    let mut city = TestCity::new()
        .with_building(10, 10, ZoneType::ResidentialLow, 1)
        .with_building(12, 10, ZoneType::Industrial, 1);

    // Run enough ticks for attach + aggregation (aggregation runs every 4 ticks)
    city.tick(8);

    let grid = city.resource::<EnergyGrid>();
    assert!(
        grid.total_demand_mwh > 0.0,
        "Total demand should be positive after ticks, got {}",
        grid.total_demand_mwh
    );
    assert!(
        grid.consumer_count >= 2,
        "Should have at least 2 consumers, got {}",
        grid.consumer_count
    );
}

#[test]
fn test_energy_demand_industrial_exceeds_residential() {
    let mut city = TestCity::new().with_building(10, 10, ZoneType::Industrial, 1);
    city.tick(8);

    let world = city.world_mut();
    let industrial_consumers: Vec<&EnergyConsumer> = world
        .query::<&EnergyConsumer>()
        .iter(world)
        .filter(|c| (c.base_demand_kwh - 50_000.0).abs() < f32::EPSILON)
        .collect();

    assert!(
        !industrial_consumers.is_empty(),
        "Industrial building should have 50,000 kWh consumer"
    );
}

#[test]
fn test_energy_demand_on_peak_vs_off_peak() {
    // On-peak city (hour=15)
    let mut on_peak = TestCity::new()
        .with_building(10, 10, ZoneType::CommercialHigh, 1)
        .with_time(15.0);
    on_peak.tick(8);
    let on_peak_demand = on_peak.resource::<EnergyGrid>().total_demand_mwh;

    // Off-peak city (hour=2)
    let mut off_peak = TestCity::new()
        .with_building(10, 10, ZoneType::CommercialHigh, 1)
        .with_time(2.0);
    off_peak.tick(8);
    let off_peak_demand = off_peak.resource::<EnergyGrid>().total_demand_mwh;

    assert!(
        on_peak_demand > off_peak_demand,
        "On-peak demand ({on_peak_demand}) should exceed off-peak ({off_peak_demand})"
    );
}

#[test]
fn test_energy_demand_empty_city_is_zero() {
    let mut city = TestCity::new();
    city.tick(8);

    let grid = city.resource::<EnergyGrid>();
    assert!(
        grid.total_demand_mwh.abs() < f32::EPSILON,
        "Empty city should have zero demand"
    );
    assert_eq!(grid.consumer_count, 0);
}

#[test]
fn test_energy_consumer_priority_hospital_is_critical() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Hospital);
    city.tick(1);

    let world = city.world_mut();
    let consumers: Vec<&EnergyConsumer> = world.query::<&EnergyConsumer>().iter(world).collect();

    let critical = consumers
        .iter()
        .find(|c| c.priority == LoadPriority::Critical);
    assert!(critical.is_some(), "Hospital should have Critical priority");
}

#[test]
fn test_energy_demand_reserve_margin_no_supply() {
    let mut city = TestCity::new().with_building(10, 10, ZoneType::ResidentialLow, 1);
    city.tick(8);

    let grid = city.resource::<EnergyGrid>();
    // With demand but no supply, reserve margin should be -1.0
    assert!(
        grid.reserve_margin < 0.0,
        "Reserve margin should be negative with no supply, got {}",
        grid.reserve_margin
    );
}
