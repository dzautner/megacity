//! Integration tests for the oil-fired power plant system (POWER-015).

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::oil_power::{
    OilPowerState, OIL_CAPACITY_FACTOR, OIL_CAPACITY_MW, OIL_CO2_TONS_PER_MWH,
    OIL_FUEL_COST_PER_MWH,
};
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;

/// Helper: spawn an oil plant entity in the TestCity at (x, y).
fn spawn_oil_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_oil(x, y));
}

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_oil_power_state_exists_in_new_city() {
    let city = TestCity::new();
    let state = city.resource::<OilPowerState>();
    assert_eq!(state.plant_count, 0);
}

// ====================================================================
// Oil plant increases total energy supply
// ====================================================================

#[test]
fn test_oil_plant_increases_energy_supply() {
    let mut city = TestCity::new();
    spawn_oil_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected_output = OIL_CAPACITY_MW * OIL_CAPACITY_FACTOR;
    assert!(
        grid.total_supply_mwh >= expected_output - f32::EPSILON,
        "Energy supply should include oil output ({expected_output} MW), got {}",
        grid.total_supply_mwh
    );
}

#[test]
fn test_multiple_oil_plants_stack_supply() {
    let mut city = TestCity::new();
    spawn_oil_plant(&mut city, 50, 50);
    spawn_oil_plant(&mut city, 60, 60);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected = OIL_CAPACITY_MW * OIL_CAPACITY_FACTOR * 2.0;
    assert!(
        grid.total_supply_mwh >= expected - f32::EPSILON,
        "Two oil plants should produce at least {expected} MW total supply, got {}",
        grid.total_supply_mwh
    );
}

// ====================================================================
// Oil plant produces air pollution (high)
// ====================================================================

#[test]
fn test_oil_plant_produces_air_pollution() {
    let mut city = TestCity::new();
    spawn_oil_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_plant = pollution.get(50, 50);
    assert!(
        at_plant > 0,
        "Pollution at oil plant location should be > 0, got {at_plant}"
    );
}

#[test]
fn test_oil_pollution_less_than_coal() {
    // Oil plant pollution (Q=75) should be less than coal (Q=100)
    let mut oil_city = TestCity::new();
    spawn_oil_plant(&mut oil_city, 50, 50);
    oil_city.tick_slow_cycle();
    let oil_pollution = oil_city.resource::<PollutionGrid>().get(50, 50);

    let mut coal_city = TestCity::new();
    coal_city.world_mut().spawn(PowerPlant::new_coal(50, 50));
    coal_city.tick_slow_cycle();
    let coal_pollution = coal_city.resource::<PollutionGrid>().get(50, 50);

    assert!(
        oil_pollution < coal_pollution,
        "Oil pollution ({oil_pollution}) should be less than coal ({coal_pollution})"
    );
}

#[test]
fn test_oil_pollution_more_than_gas() {
    // Oil plant pollution (Q=75) should be more than gas (Q=35)
    let mut oil_city = TestCity::new();
    spawn_oil_plant(&mut oil_city, 50, 50);
    oil_city.tick_slow_cycle();
    let oil_pollution = oil_city.resource::<PollutionGrid>().get(50, 50);

    let mut gas_city = TestCity::new();
    gas_city.world_mut().spawn(PowerPlant::new_gas(50, 50));
    gas_city.tick_slow_cycle();
    let gas_pollution = gas_city.resource::<PollutionGrid>().get(50, 50);

    assert!(
        oil_pollution > gas_pollution,
        "Oil pollution ({oil_pollution}) should be more than gas ({gas_pollution})"
    );
}

// ====================================================================
// Fuel cost calculation
// ====================================================================

#[test]
fn test_oil_fuel_cost_calculation() {
    let mut city = TestCity::new();
    spawn_oil_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<OilPowerState>();
    let expected_output = OIL_CAPACITY_MW * OIL_CAPACITY_FACTOR;
    let expected_fuel_cost = expected_output * OIL_FUEL_COST_PER_MWH;

    assert_eq!(state.plant_count, 1, "Should have 1 oil plant");
    assert!(
        (state.total_output_mw - expected_output).abs() < f32::EPSILON,
        "Total output should be {expected_output}, got {}",
        state.total_output_mw
    );
    assert!(
        (state.total_fuel_cost - expected_fuel_cost).abs() < 0.01,
        "Total fuel cost should be {expected_fuel_cost}, got {}",
        state.total_fuel_cost
    );
}

// ====================================================================
// CO2 emissions
// ====================================================================

#[test]
fn test_oil_co2_emissions() {
    let mut city = TestCity::new();
    spawn_oil_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<OilPowerState>();
    let expected_co2 = OIL_CAPACITY_MW * OIL_CAPACITY_FACTOR * OIL_CO2_TONS_PER_MWH;
    assert!(
        (state.total_co2_tons - expected_co2).abs() < f32::EPSILON,
        "CO2 should be {expected_co2} tons, got {}",
        state.total_co2_tons
    );
}

// ====================================================================
// Empty city has zero oil output
// ====================================================================

#[test]
fn test_no_oil_plants_zero_output() {
    let mut city = TestCity::new();

    city.tick_slow_cycle();

    let state = city.resource::<OilPowerState>();
    assert_eq!(state.plant_count, 0);
    assert!((state.total_output_mw).abs() < f32::EPSILON);
    assert!((state.total_fuel_cost).abs() < f32::EPSILON);
    assert!((state.total_co2_tons).abs() < f32::EPSILON);
}

// ====================================================================
// Oil and other plants coexist
// ====================================================================

#[test]
fn test_oil_and_coal_plants_coexist() {
    let mut city = TestCity::new();
    spawn_oil_plant(&mut city, 50, 50);
    city.world_mut().spawn(PowerPlant::new_coal(60, 60));

    city.tick_slow_cycle();

    let oil_state = city.resource::<OilPowerState>();
    assert_eq!(oil_state.plant_count, 1, "Should count only oil plants");

    let grid = city.resource::<EnergyGrid>();
    let oil_output = OIL_CAPACITY_MW * OIL_CAPACITY_FACTOR;
    let coal_output =
        crate::coal_power::COAL_CAPACITY_MW * crate::coal_power::COAL_CAPACITY_FACTOR;
    let expected_total = oil_output + coal_output;
    assert!(
        grid.total_supply_mwh >= expected_total - f32::EPSILON,
        "Combined supply should be at least {expected_total} MW, got {}",
        grid.total_supply_mwh
    );
}

// ====================================================================
// PowerPlant type discrimination
// ====================================================================

#[test]
fn test_oil_plant_has_correct_type() {
    let plant = PowerPlant::new_oil(10, 20);
    assert_eq!(plant.plant_type, PowerPlantType::Oil);
    assert_ne!(plant.plant_type, PowerPlantType::Coal);
    assert_ne!(plant.plant_type, PowerPlantType::NaturalGas);
}

// ====================================================================
// Oil is most expensive fossil fuel
// ====================================================================

#[test]
fn test_oil_is_most_expensive_fossil_fuel() {
    assert!(
        OIL_FUEL_COST_PER_MWH > crate::coal_power::COAL_FUEL_COST_PER_MWH,
        "Oil (${OIL_FUEL_COST_PER_MWH}/MWh) should be more expensive than coal"
    );
    assert!(
        OIL_FUEL_COST_PER_MWH > crate::gas_power::GAS_FUEL_COST_PER_MWH,
        "Oil (${OIL_FUEL_COST_PER_MWH}/MWh) should be more expensive than gas"
    );
}
