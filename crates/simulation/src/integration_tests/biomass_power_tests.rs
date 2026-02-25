//! Integration tests for the biomass power plant system (POWER-017).

use crate::biomass_power::{
    BiomassPowerState, BIOMASS_CAPACITY_FACTOR, BIOMASS_CAPACITY_MW, BIOMASS_CO2_TONS_PER_MWH,
    BIOMASS_FUEL_COST_PER_MWH,
};
use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;

/// Helper: spawn a biomass plant entity in the TestCity at (x, y).
fn spawn_biomass_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_biomass(x, y));
}

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_biomass_power_state_exists_in_new_city() {
    let city = TestCity::new();
    let state = city.resource::<BiomassPowerState>();
    assert_eq!(state.plant_count, 0);
}

// ====================================================================
// Biomass plant increases total energy supply
// ====================================================================

#[test]
fn test_biomass_plant_increases_energy_supply() {
    let mut city = TestCity::new();
    spawn_biomass_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected_output = BIOMASS_CAPACITY_MW * BIOMASS_CAPACITY_FACTOR;
    assert!(
        grid.total_supply_mwh >= expected_output - f32::EPSILON,
        "Energy supply should include biomass output ({expected_output} MW), got {}",
        grid.total_supply_mwh
    );
}

#[test]
fn test_multiple_biomass_plants_stack_supply() {
    let mut city = TestCity::new();
    spawn_biomass_plant(&mut city, 50, 50);
    spawn_biomass_plant(&mut city, 60, 60);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected = BIOMASS_CAPACITY_MW * BIOMASS_CAPACITY_FACTOR * 2.0;
    assert!(
        grid.total_supply_mwh >= expected - f32::EPSILON,
        "Two biomass plants should produce at least {expected} MW total supply, got {}",
        grid.total_supply_mwh
    );
}

// ====================================================================
// Biomass plant produces air pollution
// ====================================================================

#[test]
fn test_biomass_plant_produces_air_pollution() {
    let mut city = TestCity::new();
    spawn_biomass_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_plant = pollution.get(50, 50);
    assert!(
        at_plant > 0,
        "Pollution at biomass plant location should be > 0, got {at_plant}"
    );
}

#[test]
fn test_biomass_pollution_less_than_coal() {
    // Biomass plant pollution
    let mut biomass_city = TestCity::new();
    spawn_biomass_plant(&mut biomass_city, 50, 50);
    biomass_city.tick_slow_cycle();
    let biomass_pollution = biomass_city.resource::<PollutionGrid>().get(50, 50);

    // Coal plant pollution
    let mut coal_city = TestCity::new();
    coal_city.world_mut().spawn(PowerPlant::new_coal(50, 50));
    coal_city.tick_slow_cycle();
    let coal_pollution = coal_city.resource::<PollutionGrid>().get(50, 50);

    assert!(
        biomass_pollution < coal_pollution,
        "Biomass pollution ({biomass_pollution}) should be less than coal ({coal_pollution})"
    );
}

// ====================================================================
// Fuel cost calculation
// ====================================================================

#[test]
fn test_biomass_fuel_cost_calculation() {
    let mut city = TestCity::new();
    spawn_biomass_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<BiomassPowerState>();
    let expected_output = BIOMASS_CAPACITY_MW * BIOMASS_CAPACITY_FACTOR;
    let expected_fuel_cost = expected_output * BIOMASS_FUEL_COST_PER_MWH;

    assert_eq!(state.plant_count, 1, "Should have 1 biomass plant");
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
fn test_biomass_co2_emissions() {
    let mut city = TestCity::new();
    spawn_biomass_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<BiomassPowerState>();
    let expected_co2 = BIOMASS_CAPACITY_MW * BIOMASS_CAPACITY_FACTOR * BIOMASS_CO2_TONS_PER_MWH;
    assert!(
        (state.total_co2_tons - expected_co2).abs() < f32::EPSILON,
        "CO2 should be {expected_co2} tons, got {}",
        state.total_co2_tons
    );
}

#[test]
fn test_biomass_co2_lower_than_coal() {
    assert!(
        BIOMASS_CO2_TONS_PER_MWH < crate::coal_power::COAL_CO2_TONS_PER_MWH,
        "Biomass CO2 rate ({}) should be lower than coal ({})",
        BIOMASS_CO2_TONS_PER_MWH,
        crate::coal_power::COAL_CO2_TONS_PER_MWH
    );
}

// ====================================================================
// Empty city has zero biomass output
// ====================================================================

#[test]
fn test_no_biomass_plants_zero_output() {
    let mut city = TestCity::new();

    city.tick_slow_cycle();

    let state = city.resource::<BiomassPowerState>();
    assert_eq!(state.plant_count, 0);
    assert!((state.total_output_mw).abs() < f32::EPSILON);
    assert!((state.total_fuel_cost).abs() < f32::EPSILON);
    assert!((state.total_co2_tons).abs() < f32::EPSILON);
}

// ====================================================================
// Biomass and other plants coexist
// ====================================================================

#[test]
fn test_biomass_and_coal_plants_coexist() {
    let mut city = TestCity::new();
    spawn_biomass_plant(&mut city, 50, 50);
    city.world_mut().spawn(PowerPlant::new_coal(60, 60));

    city.tick_slow_cycle();

    let biomass_state = city.resource::<BiomassPowerState>();
    assert_eq!(
        biomass_state.plant_count, 1,
        "Should count only biomass plants"
    );

    let grid = city.resource::<EnergyGrid>();
    let biomass_output = BIOMASS_CAPACITY_MW * BIOMASS_CAPACITY_FACTOR;
    let coal_output =
        crate::coal_power::COAL_CAPACITY_MW * crate::coal_power::COAL_CAPACITY_FACTOR;
    let expected_total = biomass_output + coal_output;
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
fn test_biomass_plant_has_correct_type() {
    let plant = PowerPlant::new_biomass(10, 20);
    assert_eq!(plant.plant_type, PowerPlantType::Biomass);
    assert_ne!(plant.plant_type, PowerPlantType::Coal);
    assert_ne!(plant.plant_type, PowerPlantType::NaturalGas);
    assert_ne!(plant.plant_type, PowerPlantType::WasteToEnergy);
}

// ====================================================================
// Capacity and capacity factor
// ====================================================================

#[test]
fn test_biomass_capacity_values() {
    assert!(
        (BIOMASS_CAPACITY_MW - 25.0).abs() < f32::EPSILON,
        "Biomass capacity should be 25 MW"
    );
    assert!(
        (BIOMASS_CAPACITY_FACTOR - 0.80).abs() < f32::EPSILON,
        "Biomass capacity factor should be 0.80"
    );
}
