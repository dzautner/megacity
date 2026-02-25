//! Integration tests for the geothermal power plant system (POWER-013).

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::geothermal_power::{
    GeothermalPowerState, GEOTHERMAL_CAPACITY_FACTOR, GEOTHERMAL_CAPACITY_MW,
};
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;

/// Helper: spawn a geothermal plant entity in the TestCity at (x, y).
fn spawn_geothermal_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_geothermal(x, y));
}

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_geothermal_power_state_exists_in_new_city() {
    let city = TestCity::new();
    let state = city.resource::<GeothermalPowerState>();
    assert_eq!(state.plant_count, 0);
    assert!((state.total_output_mw).abs() < f32::EPSILON);
}

// ====================================================================
// Geothermal plant increases total energy supply
// ====================================================================

#[test]
fn test_geothermal_plant_increases_energy_supply() {
    let mut city = TestCity::new();
    spawn_geothermal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected_output = GEOTHERMAL_CAPACITY_MW * GEOTHERMAL_CAPACITY_FACTOR;
    assert!(
        grid.total_supply_mwh >= expected_output - f32::EPSILON,
        "Energy supply should include geothermal output ({expected_output} MW), got {}",
        grid.total_supply_mwh
    );
}

#[test]
fn test_multiple_geothermal_plants_stack_supply() {
    let mut city = TestCity::new();
    spawn_geothermal_plant(&mut city, 50, 50);
    spawn_geothermal_plant(&mut city, 60, 60);
    spawn_geothermal_plant(&mut city, 70, 70);

    city.tick_slow_cycle();

    let grid = city.resource::<EnergyGrid>();
    let expected = GEOTHERMAL_CAPACITY_MW * GEOTHERMAL_CAPACITY_FACTOR * 3.0;
    assert!(
        grid.total_supply_mwh >= expected - f32::EPSILON,
        "Three geothermal plants should produce at least {expected} MW total, got {}",
        grid.total_supply_mwh
    );
}

// ====================================================================
// Geothermal plant produces NO air pollution
// ====================================================================

#[test]
fn test_geothermal_plant_no_air_pollution() {
    let mut city = TestCity::new();
    spawn_geothermal_plant(&mut city, 128, 128);

    // Record baseline pollution
    let baseline = city.resource::<PollutionGrid>().get(128, 128);

    city.tick_slow_cycle();

    let after = city.resource::<PollutionGrid>().get(128, 128);
    // Geothermal has zero emissions (Q=0), so pollution at the plant
    // should not increase from baseline.
    assert!(
        after <= baseline + 1,
        "Geothermal should not add significant pollution: baseline={baseline}, after={after}"
    );
}

// ====================================================================
// Geothermal plant has zero fuel cost
// ====================================================================

#[test]
fn test_geothermal_zero_fuel_cost() {
    let mut city = TestCity::new();
    spawn_geothermal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<GeothermalPowerState>();
    assert_eq!(state.plant_count, 1, "Should have 1 geothermal plant");

    // Verify the plant itself has zero fuel cost
    let world = city.world_mut();
    let plants: Vec<_> = world
        .query::<&PowerPlant>()
        .iter(world)
        .filter(|p| p.plant_type == PowerPlantType::Geothermal)
        .collect();
    assert_eq!(plants.len(), 1);
    assert_eq!(
        plants[0].fuel_cost, 0.0,
        "Geothermal fuel cost must be zero"
    );
}

// ====================================================================
// Constant baseload output
// ====================================================================

#[test]
fn test_geothermal_constant_baseload_output() {
    let mut city = TestCity::new();
    spawn_geothermal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();
    let output_1 = city.resource::<GeothermalPowerState>().total_output_mw;

    city.tick_slow_cycle();
    let output_2 = city.resource::<GeothermalPowerState>().total_output_mw;

    let expected = GEOTHERMAL_CAPACITY_MW * GEOTHERMAL_CAPACITY_FACTOR;
    assert!(
        (output_1 - expected).abs() < f32::EPSILON,
        "First cycle output should be {expected}, got {output_1}"
    );
    assert!(
        (output_2 - expected).abs() < f32::EPSILON,
        "Second cycle output should be {expected}, got {output_2}"
    );
    assert!(
        (output_1 - output_2).abs() < f32::EPSILON,
        "Baseload output should be constant: cycle1={output_1}, cycle2={output_2}"
    );
}

// ====================================================================
// Empty city has zero geothermal output
// ====================================================================

#[test]
fn test_no_geothermal_plants_zero_output() {
    let mut city = TestCity::new();

    city.tick_slow_cycle();

    let state = city.resource::<GeothermalPowerState>();
    assert_eq!(state.plant_count, 0);
    assert!((state.total_output_mw).abs() < f32::EPSILON);
}

// ====================================================================
// State update reflects correct count and output
// ====================================================================

#[test]
fn test_geothermal_state_tracks_plant_count() {
    let mut city = TestCity::new();
    spawn_geothermal_plant(&mut city, 50, 50);
    spawn_geothermal_plant(&mut city, 60, 60);

    city.tick_slow_cycle();

    let state = city.resource::<GeothermalPowerState>();
    assert_eq!(
        state.plant_count, 2,
        "Should track 2 geothermal plants"
    );

    let expected_output = GEOTHERMAL_CAPACITY_MW * GEOTHERMAL_CAPACITY_FACTOR * 2.0;
    assert!(
        (state.total_output_mw - expected_output).abs() < f32::EPSILON,
        "Total output should be {expected_output}, got {}",
        state.total_output_mw
    );
}
