//! Integration tests for Wind Turbine Farm Power Plant (POWER-006).

use crate::coal_power::PowerPlant;
use crate::energy_demand::EnergyGrid;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::wind::WindState;
use crate::wind_power::{WindPowerState, CUT_IN_SPEED, CUT_OUT_SPEED};

#[test]
fn test_wind_power_zero_output_below_cut_in() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WindTurbine);

    // Set wind speed below cut-in threshold
    city.world_mut().resource_mut::<WindState>().speed = 0.05;

    // Run enough ticks for the slow timer to fire and components to attach
    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let wind_state = city.resource::<WindPowerState>();
    assert_eq!(
        wind_state.total_output_mw, 0.0,
        "wind output should be zero below cut-in speed ({CUT_IN_SPEED})"
    );
}

#[test]
fn test_wind_power_cubic_scaling() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WindTurbine);

    // Set wind speed to 0.5 (within operating range)
    city.world_mut().resource_mut::<WindState>().speed = 0.5;

    // Run enough ticks for the slow timer to fire and components to attach
    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let wind_state = city.resource::<WindPowerState>();
    let expected = 100.0 * 0.5_f32.powi(3); // 12.5 MW
    assert!(
        (wind_state.total_output_mw - expected).abs() < 0.1,
        "expected ~{expected} MW at 0.5 wind speed, got {} MW",
        wind_state.total_output_mw
    );
}

#[test]
fn test_wind_power_shutdown_above_cut_out() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WindTurbine);

    // Set wind speed above cut-out threshold
    city.world_mut().resource_mut::<WindState>().speed = 0.96;

    // Run enough ticks for the slow timer to fire and components to attach
    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let wind_state = city.resource::<WindPowerState>();
    assert_eq!(
        wind_state.total_output_mw, 0.0,
        "wind output should be zero above cut-out speed ({CUT_OUT_SPEED})"
    );
}

#[test]
fn test_wind_farm_contributes_to_energy_grid() {
    let mut city = TestCity::new()
        .with_utility(50, 50, UtilityType::WindTurbine)
        .with_utility(60, 60, UtilityType::WindTurbine);

    // Set wind speed to 0.8 (strong, within operating range)
    city.world_mut().resource_mut::<WindState>().speed = 0.8;

    // Run enough ticks for the slow timer to fire and components to attach
    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let wind_state = city.resource::<WindPowerState>();
    let expected_per_farm = 100.0 * 0.8_f32.powi(3); // 51.2 MW each
    let expected_total = expected_per_farm * 2.0; // ~102.4 MW

    assert_eq!(wind_state.farm_count, 2, "should have 2 wind farms");
    assert!(
        (wind_state.total_output_mw - expected_total).abs() < 0.5,
        "total wind output should be ~{expected_total} MW from 2 farms, got {} MW",
        wind_state.total_output_mw
    );

    let energy = city.resource::<EnergyGrid>();
    assert!(
        energy.total_supply_mwh >= expected_total - 0.5,
        "energy grid total_supply_mwh should include wind contribution, got {} MW",
        energy.total_supply_mwh
    );
}

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
fn test_wind_power_at_exact_cut_in() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WindTurbine);

    // Set wind speed exactly at cut-in (should produce power)
    city.world_mut().resource_mut::<WindState>().speed = CUT_IN_SPEED;

    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let wind_state = city.resource::<WindPowerState>();
    let expected = 100.0 * CUT_IN_SPEED.powi(3);
    assert!(
        (wind_state.total_output_mw - expected).abs() < 0.01,
        "at exact cut-in speed, output should be {expected} MW, got {} MW",
        wind_state.total_output_mw
    );
}

#[test]
fn test_wind_power_at_exact_cut_out() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WindTurbine);

    // Set wind speed exactly at cut-out (should still produce power)
    city.world_mut().resource_mut::<WindState>().speed = CUT_OUT_SPEED;

    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let wind_state = city.resource::<WindPowerState>();
    let expected = 100.0 * CUT_OUT_SPEED.powi(3);
    assert!(
        (wind_state.total_output_mw - expected).abs() < 0.1,
        "at exact cut-out speed, output should be {expected} MW, got {} MW",
        wind_state.total_output_mw
    );
}
