//! Integration tests for Wind Turbine Farm Power Plant (POWER-006).
//!
//! Note: The `update_wind` system modifies `WindState.speed` on each slow tick,
//! so integration tests verify behavior using the actual wind speed recorded in
//! `WindPowerState.current_wind_speed` rather than the value we set.

use crate::coal_power::PowerPlant;
use crate::energy_demand::EnergyGrid;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::wind::WindState;
use crate::wind_power::{wind_power_output, WindPowerState, WIND_FARM_NAMEPLATE_MW};

#[test]
fn test_wind_power_plant_component_attached() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WindTurbine);

    // Run enough ticks for component attachment
    city.tick_slow_cycle();

    let world = city.world_mut();
    let count = world.query::<&PowerPlant>().iter(world).count();
    assert_eq!(
        count, 1,
        "PowerPlant component should be attached to wind turbine utility"
    );
}

#[test]
fn test_wind_power_output_matches_cubic_curve() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WindTurbine);

    // Run slow cycles to attach components and produce output
    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let wind_state = city.resource::<WindPowerState>();
    let actual_speed = wind_state.current_wind_speed;
    let actual_output = wind_state.total_output_mw;
    let expected_output = wind_power_output(WIND_FARM_NAMEPLATE_MW, actual_speed);

    assert!(
        (actual_output - expected_output).abs() < 0.01,
        "output ({actual_output} MW) should match cubic formula \
         ({expected_output} MW) at wind speed {actual_speed}"
    );
}

#[test]
fn test_wind_farm_contributes_to_energy_grid() {
    let mut city = TestCity::new()
        .with_utility(50, 50, UtilityType::WindTurbine)
        .with_utility(60, 60, UtilityType::WindTurbine);

    // Run slow cycles to attach components and produce output
    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let wind_state = city.resource::<WindPowerState>();
    assert_eq!(wind_state.farm_count, 2, "should have 2 wind farms");

    let energy = city.resource::<EnergyGrid>();
    // Energy grid should include at least the wind contribution
    assert!(
        energy.total_supply_mwh >= wind_state.total_output_mw - 0.5,
        "energy grid total_supply_mwh ({}) should include wind contribution ({} MW)",
        energy.total_supply_mwh,
        wind_state.total_output_mw
    );
}

#[test]
fn test_wind_power_state_tracks_farm_count() {
    let mut city = TestCity::new()
        .with_utility(50, 50, UtilityType::WindTurbine)
        .with_utility(70, 70, UtilityType::WindTurbine)
        .with_utility(90, 90, UtilityType::WindTurbine);

    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let wind_state = city.resource::<WindPowerState>();
    assert_eq!(wind_state.farm_count, 3, "should have 3 wind farms");
}

#[test]
fn test_wind_power_unit_below_cut_in() {
    // Pure function test: verify zero output below cut-in speed
    let output = wind_power_output(100.0, 0.05);
    assert_eq!(output, 0.0, "output should be zero below cut-in speed");
}

#[test]
fn test_wind_power_unit_above_cut_out() {
    // Pure function test: verify zero output above cut-out speed
    let output = wind_power_output(100.0, 0.96);
    assert_eq!(output, 0.0, "output should be zero above cut-out speed");
}

#[test]
fn test_wind_power_unit_cubic_at_half_speed() {
    // Pure function test: verify cubic curve at 0.5
    let output = wind_power_output(100.0, 0.5);
    let expected = 100.0 * 0.5_f32.powi(3); // 12.5 MW
    assert!(
        (output - expected).abs() < 0.001,
        "expected {expected} MW, got {output} MW"
    );
}

#[test]
fn test_wind_power_unit_at_cut_in_boundary() {
    // Exactly at cut-in (0.1) should produce power
    let output = wind_power_output(100.0, 0.1);
    assert!(output > 0.0, "output at exact cut-in should be positive");
    let expected = 100.0 * 0.1_f32.powi(3);
    assert!(
        (output - expected).abs() < 0.001,
        "expected {expected} MW, got {output} MW"
    );
}

#[test]
fn test_wind_power_unit_at_cut_out_boundary() {
    // Exactly at cut-out (0.95) should still produce power
    let output = wind_power_output(100.0, 0.95);
    assert!(output > 0.0, "output at exact cut-out should be positive");
    let expected = 100.0 * 0.95_f32.powi(3);
    assert!(
        (output - expected).abs() < 0.001,
        "expected {expected} MW, got {output} MW"
    );
}

#[test]
fn test_wind_power_nameplate_capacity() {
    // Verify wind farms use the correct nameplate capacity
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WindTurbine);

    city.tick_slow_cycle();

    let world = city.world_mut();
    let plant = world.query::<&PowerPlant>().iter(world).next().unwrap();
    assert!(
        (plant.capacity_mw - WIND_FARM_NAMEPLATE_MW).abs() < f32::EPSILON,
        "wind farm should have {WIND_FARM_NAMEPLATE_MW} MW nameplate, got {} MW",
        plant.capacity_mw
    );
}

#[test]
fn test_wind_power_zero_fuel_cost() {
    // Verify wind farms have zero fuel cost
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WindTurbine);

    city.tick_slow_cycle();

    let world = city.world_mut();
    let plant = world.query::<&PowerPlant>().iter(world).next().unwrap();
    assert!(
        plant.fuel_cost.abs() < f32::EPSILON,
        "wind farm fuel cost should be $0/MWh, got ${}",
        plant.fuel_cost
    );
}
