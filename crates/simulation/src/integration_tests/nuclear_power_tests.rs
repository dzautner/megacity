//! Integration tests for the nuclear power plant system (POWER-004).

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::nuclear_power::{
    NuclearPowerState, NUCLEAR_CAPACITY_FACTOR, NUCLEAR_CAPACITY_MW, NUCLEAR_FUEL_COST_PER_MWH,
    NUCLEAR_WASTE_KG_PER_MWH,
};
use crate::test_harness::TestCity;

/// Helper: spawn a nuclear plant entity in the TestCity at (x, y).
fn spawn_nuclear_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_nuclear(x, y));
}

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_nuclear_power_state_exists_in_new_city() {
    let city = TestCity::new();
    let state = city.resource::<NuclearPowerState>();
    assert_eq!(state.plant_count, 0);
    assert!(state.total_output_mw.abs() < f32::EPSILON);
}

// ====================================================================
// Nuclear plant increases total energy supply
// ====================================================================

#[test]
fn test_nuclear_plant_increases_energy_supply() {
    let mut city = TestCity::new();
    spawn_nuclear_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected_output = NUCLEAR_CAPACITY_MW * NUCLEAR_CAPACITY_FACTOR;
    assert!(
        grid.total_supply_mwh >= expected_output - f32::EPSILON,
        "Energy supply should include nuclear output ({expected_output} MW), got {}",
        grid.total_supply_mwh
    );
}

#[test]
fn test_multiple_nuclear_plants_stack_supply() {
    let mut city = TestCity::new();
    spawn_nuclear_plant(&mut city, 50, 50);
    spawn_nuclear_plant(&mut city, 60, 60);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected = NUCLEAR_CAPACITY_MW * NUCLEAR_CAPACITY_FACTOR * 2.0;
    assert!(
        grid.total_supply_mwh >= expected - f32::EPSILON,
        "Two nuclear plants should produce at least {expected} MW total supply, got {}",
        grid.total_supply_mwh
    );
}

// ====================================================================
// Fuel cost calculation
// ====================================================================

#[test]
fn test_nuclear_fuel_cost_calculation() {
    let mut city = TestCity::new();
    spawn_nuclear_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<NuclearPowerState>();
    let expected_output = NUCLEAR_CAPACITY_MW * NUCLEAR_CAPACITY_FACTOR;
    let expected_fuel_cost = expected_output * NUCLEAR_FUEL_COST_PER_MWH;

    assert_eq!(state.plant_count, 1, "Should have 1 nuclear plant");
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
// Radioactive waste production
// ====================================================================

#[test]
fn test_nuclear_produces_radioactive_waste() {
    let mut city = TestCity::new();
    spawn_nuclear_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<NuclearPowerState>();
    let expected_output = NUCLEAR_CAPACITY_MW * NUCLEAR_CAPACITY_FACTOR;
    let expected_waste = expected_output * NUCLEAR_WASTE_KG_PER_MWH;

    assert!(
        (state.total_radioactive_waste_kg - expected_waste).abs() < f32::EPSILON,
        "Radioactive waste should be {expected_waste} kg, got {}",
        state.total_radioactive_waste_kg
    );
    assert!(
        state.total_radioactive_waste_kg > 0.0,
        "Nuclear should produce radioactive waste"
    );
}

#[test]
fn test_nuclear_waste_accumulates_over_cycles() {
    let mut city = TestCity::new();
    spawn_nuclear_plant(&mut city, 50, 50);

    city.tick_slow_cycle();
    let waste_after_1 = city.resource::<NuclearPowerState>().cumulative_radioactive_waste_kg;

    city.tick_slow_cycle();
    let waste_after_2 = city.resource::<NuclearPowerState>().cumulative_radioactive_waste_kg;

    assert!(
        waste_after_2 > waste_after_1,
        "Cumulative waste should increase: {} -> {}",
        waste_after_1,
        waste_after_2
    );
}

// ====================================================================
// Zero emissions (air pollution and CO2)
// ====================================================================

#[test]
fn test_nuclear_zero_co2_emissions() {
    // Nuclear should produce zero CO2 — verified via the constant
    assert!(
        crate::nuclear_power::NUCLEAR_CO2_TONS_PER_MWH.abs() < f32::EPSILON,
        "Nuclear CO2 rate should be zero"
    );
}

#[test]
fn test_nuclear_zero_air_pollution_constant() {
    assert!(
        crate::nuclear_power::NUCLEAR_AIR_POLLUTION_Q.abs() < f32::EPSILON,
        "Nuclear air pollution Q should be zero"
    );
}

// ====================================================================
// Empty city has zero nuclear output
// ====================================================================

#[test]
fn test_no_nuclear_plants_zero_output() {
    let mut city = TestCity::new();

    city.tick_slow_cycle();

    let state = city.resource::<NuclearPowerState>();
    assert_eq!(state.plant_count, 0);
    assert!(state.total_output_mw.abs() < f32::EPSILON);
    assert!(state.total_fuel_cost.abs() < f32::EPSILON);
    assert!(state.total_radioactive_waste_kg.abs() < f32::EPSILON);
}

// ====================================================================
// Nuclear and other plants coexist
// ====================================================================

#[test]
fn test_nuclear_and_coal_plants_coexist() {
    let mut city = TestCity::new();
    spawn_nuclear_plant(&mut city, 50, 50);
    city.world_mut().spawn(PowerPlant::new_coal(60, 60));

    city.tick_slow_cycle();

    let nuclear_state = city.resource::<NuclearPowerState>();
    assert_eq!(
        nuclear_state.plant_count, 1,
        "Should count only nuclear plants"
    );

    let grid = city.resource::<EnergyGrid>();
    let nuclear_output = NUCLEAR_CAPACITY_MW * NUCLEAR_CAPACITY_FACTOR;
    let coal_output =
        crate::coal_power::COAL_CAPACITY_MW * crate::coal_power::COAL_CAPACITY_FACTOR;
    let expected_total = nuclear_output + coal_output;
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
fn test_nuclear_plant_has_correct_type() {
    let plant = PowerPlant::new_nuclear(10, 20);
    assert_eq!(plant.plant_type, PowerPlantType::Nuclear);
    assert_ne!(plant.plant_type, PowerPlantType::Coal);
    assert_ne!(plant.plant_type, PowerPlantType::NaturalGas);
    assert_ne!(plant.plant_type, PowerPlantType::WindTurbine);
    assert_ne!(plant.plant_type, PowerPlantType::WasteToEnergy);
}

// ====================================================================
// Nuclear has highest capacity among generators
// ====================================================================

#[test]
fn test_nuclear_highest_capacity() {
    assert!(
        NUCLEAR_CAPACITY_MW > crate::coal_power::COAL_CAPACITY_MW,
        "Nuclear capacity ({} MW) should exceed coal ({} MW)",
        NUCLEAR_CAPACITY_MW,
        crate::coal_power::COAL_CAPACITY_MW
    );
    assert!(
        NUCLEAR_CAPACITY_MW > crate::gas_power::GAS_CAPACITY_MW,
        "Nuclear capacity ({} MW) should exceed gas ({} MW)",
        NUCLEAR_CAPACITY_MW,
        crate::gas_power::GAS_CAPACITY_MW
    );
}

// ====================================================================
// Nuclear dispatches after renewables but before coal/gas
// ====================================================================

#[test]
fn test_nuclear_fuel_cost_between_renewables_and_coal() {
    // Nuclear fuel cost ($15) should be higher than renewables ($0)
    // but lower than coal ($30) and gas ($40)
    assert!(
        NUCLEAR_FUEL_COST_PER_MWH > 0.0,
        "Nuclear fuel cost should be positive"
    );
    assert!(
        NUCLEAR_FUEL_COST_PER_MWH < crate::coal_power::COAL_FUEL_COST_PER_MWH,
        "Nuclear fuel cost ({}) should be less than coal ({})",
        NUCLEAR_FUEL_COST_PER_MWH,
        crate::coal_power::COAL_FUEL_COST_PER_MWH
    );
    assert!(
        NUCLEAR_FUEL_COST_PER_MWH < crate::gas_power::GAS_FUEL_COST_PER_MWH,
        "Nuclear fuel cost ({}) should be less than gas ({})",
        NUCLEAR_FUEL_COST_PER_MWH,
        crate::gas_power::GAS_FUEL_COST_PER_MWH
    );
}
