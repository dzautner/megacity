//! Integration tests for the natural gas combined-cycle power plant system (POWER-003).

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::gas_power::{
    GasPowerState, GAS_CAPACITY_FACTOR, GAS_CAPACITY_MW, GAS_CO2_TONS_PER_MWH,
    GAS_FUEL_COST_PER_MWH,
};
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;

/// Helper: spawn a gas plant entity in the TestCity at (x, y).
fn spawn_gas_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_gas(x, y));
}

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_gas_power_state_exists_in_new_city() {
    let city = TestCity::new();
    let state = city.resource::<GasPowerState>();
    assert_eq!(state.plant_count, 0);
}

// ====================================================================
// Gas plant increases total energy supply
// ====================================================================

#[test]
fn test_gas_plant_increases_energy_supply() {
    let mut city = TestCity::new();
    spawn_gas_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected_output = GAS_CAPACITY_MW * GAS_CAPACITY_FACTOR;
    assert!(
        grid.total_supply_mwh >= expected_output - f32::EPSILON,
        "Energy supply should include gas output ({expected_output} MW), got {}",
        grid.total_supply_mwh
    );
}

#[test]
fn test_multiple_gas_plants_stack_supply() {
    let mut city = TestCity::new();
    spawn_gas_plant(&mut city, 50, 50);
    spawn_gas_plant(&mut city, 60, 60);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected = GAS_CAPACITY_MW * GAS_CAPACITY_FACTOR * 2.0;
    assert!(
        grid.total_supply_mwh >= expected - f32::EPSILON,
        "Two gas plants should produce at least {expected} MW total supply, got {}",
        grid.total_supply_mwh
    );
}

// ====================================================================
// Gas plant produces air pollution (less than coal)
// ====================================================================

#[test]
fn test_gas_plant_produces_air_pollution() {
    let mut city = TestCity::new();
    spawn_gas_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_plant = pollution.get(50, 50);
    assert!(
        at_plant > 0,
        "Pollution at gas plant location should be > 0, got {at_plant}"
    );
}

#[test]
fn test_gas_pollution_less_than_coal() {
    // Gas plant pollution
    let mut gas_city = TestCity::new();
    spawn_gas_plant(&mut gas_city, 50, 50);
    gas_city.tick_slow_cycle();
    let gas_pollution = gas_city.resource::<PollutionGrid>().get(50, 50);

    // Coal plant pollution
    let mut coal_city = TestCity::new();
    coal_city.world_mut().spawn(PowerPlant::new_coal(50, 50));
    coal_city.tick_slow_cycle();
    let coal_pollution = coal_city.resource::<PollutionGrid>().get(50, 50);

    assert!(
        gas_pollution < coal_pollution,
        "Gas pollution ({gas_pollution}) should be less than coal ({coal_pollution})"
    );
}

#[test]
fn test_gas_pollution_radiates_outward() {
    let mut city = TestCity::new();
    spawn_gas_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let center = pollution.get(50, 50);
    let near = pollution.get(53, 50);
    let far = pollution.get(57, 50);

    assert!(center > 0, "Center should have pollution, got {center}");
    assert!(
        near > 0 || center > near,
        "Near cell should have some pollution"
    );
    assert!(
        far <= center,
        "Far cell ({far}) should not exceed center ({center})"
    );
}

// ====================================================================
// Fuel cost calculation
// ====================================================================

#[test]
fn test_gas_fuel_cost_calculation() {
    let mut city = TestCity::new();
    spawn_gas_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<GasPowerState>();
    let expected_output = GAS_CAPACITY_MW * GAS_CAPACITY_FACTOR;
    let expected_fuel_cost = expected_output * GAS_FUEL_COST_PER_MWH;

    assert_eq!(state.plant_count, 1, "Should have 1 gas plant");
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
fn test_gas_co2_emissions() {
    let mut city = TestCity::new();
    spawn_gas_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<GasPowerState>();
    let expected_co2 = GAS_CAPACITY_MW * GAS_CAPACITY_FACTOR * GAS_CO2_TONS_PER_MWH;
    assert!(
        (state.total_co2_tons - expected_co2).abs() < f32::EPSILON,
        "CO2 should be {expected_co2} tons, got {}",
        state.total_co2_tons
    );
}

#[test]
fn test_gas_co2_lower_than_coal() {
    // Gas CO2 per MW output should be lower than coal
    assert!(
        GAS_CO2_TONS_PER_MWH < crate::coal_power::COAL_CO2_TONS_PER_MWH,
        "Gas CO2 rate ({}) should be lower than coal ({})",
        GAS_CO2_TONS_PER_MWH,
        crate::coal_power::COAL_CO2_TONS_PER_MWH
    );
}

// ====================================================================
// Empty city has zero gas output
// ====================================================================

#[test]
fn test_no_gas_plants_zero_output() {
    let mut city = TestCity::new();

    city.tick_slow_cycle();

    let state = city.resource::<GasPowerState>();
    assert_eq!(state.plant_count, 0);
    assert!((state.total_output_mw).abs() < f32::EPSILON);
    assert!((state.total_fuel_cost).abs() < f32::EPSILON);
    assert!((state.total_co2_tons).abs() < f32::EPSILON);
}

// ====================================================================
// Gas and coal plants coexist
// ====================================================================

#[test]
fn test_gas_and_coal_plants_coexist() {
    let mut city = TestCity::new();
    spawn_gas_plant(&mut city, 50, 50);
    city.world_mut().spawn(PowerPlant::new_coal(60, 60));

    city.tick_slow_cycle();

    let gas_state = city.resource::<GasPowerState>();
    assert_eq!(gas_state.plant_count, 1, "Should count only gas plants");

    let grid = city.resource::<EnergyGrid>();
    let gas_output = GAS_CAPACITY_MW * GAS_CAPACITY_FACTOR;
    let coal_output =
        crate::coal_power::COAL_CAPACITY_MW * crate::coal_power::COAL_CAPACITY_FACTOR;
    let expected_total = gas_output + coal_output;
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
fn test_gas_plant_has_correct_type() {
    let plant = PowerPlant::new_gas(10, 20);
    assert_eq!(plant.plant_type, PowerPlantType::NaturalGas);
    assert_ne!(plant.plant_type, PowerPlantType::Coal);
}
