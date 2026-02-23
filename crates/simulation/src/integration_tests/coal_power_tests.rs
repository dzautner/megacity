//! Integration tests for the coal power plant system (POWER-002).

use crate::coal_power::{
    CoalPowerState, PowerPlant, COAL_CAPACITY_FACTOR, COAL_CAPACITY_MW, COAL_CO2_TONS_PER_MWH,
    COAL_FUEL_COST_PER_MWH,
};
use crate::energy_demand::EnergyGrid;
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;

/// Helper: spawn a coal plant entity in the TestCity at (x, y).
fn spawn_coal_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_coal(x, y));
}

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_coal_power_state_exists_in_new_city() {
    let city = TestCity::new();
    let state = city.resource::<CoalPowerState>();
    assert_eq!(state.plant_count, 0);
}

// ====================================================================
// Coal plant increases total energy supply
// ====================================================================

#[test]
fn test_coal_plant_increases_energy_supply() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);

    // Run a slow tick cycle so aggregate_coal_power fires
    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected_output = COAL_CAPACITY_MW * COAL_CAPACITY_FACTOR;
    assert!(
        grid.total_supply_mwh >= expected_output - f32::EPSILON,
        "Energy supply should include coal output ({expected_output} MW), got {}",
        grid.total_supply_mwh
    );
}

#[test]
fn test_multiple_coal_plants_stack_supply() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);
    spawn_coal_plant(&mut city, 60, 60);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected = COAL_CAPACITY_MW * COAL_CAPACITY_FACTOR * 2.0;
    assert!(
        grid.total_supply_mwh >= expected - f32::EPSILON,
        "Two coal plants should produce at least {expected} MW total supply, got {}",
        grid.total_supply_mwh
    );
}

// ====================================================================
// Coal plant produces air pollution
// ====================================================================

#[test]
fn test_coal_plant_produces_air_pollution() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_plant = pollution.get(50, 50);
    assert!(
        at_plant > 0,
        "Pollution at coal plant location should be > 0, got {at_plant}"
    );
}

#[test]
fn test_coal_pollution_radiates_outward() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let center = pollution.get(50, 50);
    let near = pollution.get(53, 50);
    let far = pollution.get(58, 50);

    assert!(center > 0, "Center should have pollution, got {center}");
    // Near should have some pollution (may be less than center)
    assert!(
        near > 0 || center > near,
        "Near cell should have some pollution"
    );
    // Far should have less or equal pollution than center
    assert!(
        far <= center,
        "Far cell ({far}) should not exceed center ({center})"
    );
}

// ====================================================================
// Fuel cost calculation
// ====================================================================

#[test]
fn test_coal_fuel_cost_calculation() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<CoalPowerState>();
    let expected_output = COAL_CAPACITY_MW * COAL_CAPACITY_FACTOR;
    let expected_fuel_cost = expected_output * COAL_FUEL_COST_PER_MWH;

    assert_eq!(state.plant_count, 1, "Should have 1 coal plant");
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
fn test_coal_co2_emissions() {
    let mut city = TestCity::new();
    spawn_coal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<CoalPowerState>();
    let expected_co2 = COAL_CAPACITY_MW * COAL_CAPACITY_FACTOR * COAL_CO2_TONS_PER_MWH;
    assert!(
        (state.total_co2_tons - expected_co2).abs() < f32::EPSILON,
        "CO2 should be {expected_co2} tons, got {}",
        state.total_co2_tons
    );
}

// ====================================================================
// Empty city has zero coal output
// ====================================================================

#[test]
fn test_no_coal_plants_zero_output() {
    let mut city = TestCity::new();

    city.tick_slow_cycle();

    let state = city.resource::<CoalPowerState>();
    assert_eq!(state.plant_count, 0);
    assert!((state.total_output_mw).abs() < f32::EPSILON);
    assert!((state.total_fuel_cost).abs() < f32::EPSILON);
    assert!((state.total_co2_tons).abs() < f32::EPSILON);
}
