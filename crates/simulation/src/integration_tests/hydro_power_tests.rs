//! Integration tests for Hydroelectric Dam Power Plant (POWER-007).

use crate::coal_power::PowerPlant;
use crate::energy_demand::EnergyGrid;
use crate::hydro_power::{seasonal_capacity_factor, HydroPowerState, HYDRO_NAMEPLATE_MW};
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::weather::{Season, Weather};

#[test]
fn test_hydro_dam_component_attached() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::HydroDam);

    // Run enough ticks for component attachment
    city.tick_slow_cycle();

    let world = city.world_mut();
    let count = world.query::<&PowerPlant>().iter(world).count();
    assert_eq!(
        count, 1,
        "PowerPlant component should be attached to hydro dam utility"
    );
}

#[test]
fn test_hydro_dam_nameplate_capacity() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::HydroDam);

    city.tick_slow_cycle();

    let world = city.world_mut();
    let plant = world.query::<&PowerPlant>().iter(world).next().unwrap();
    assert!(
        (plant.capacity_mw - HYDRO_NAMEPLATE_MW).abs() < f32::EPSILON,
        "hydro dam should have {HYDRO_NAMEPLATE_MW} MW nameplate, got {} MW",
        plant.capacity_mw
    );
}

#[test]
fn test_hydro_dam_zero_fuel_cost() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::HydroDam);

    city.tick_slow_cycle();

    let world = city.world_mut();
    let plant = world.query::<&PowerPlant>().iter(world).next().unwrap();
    assert!(
        plant.fuel_cost.abs() < f32::EPSILON,
        "hydro dam fuel cost should be $0/MWh, got ${}",
        plant.fuel_cost
    );
}

#[test]
fn test_hydro_power_generation_occurs() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::HydroDam);

    // Run slow cycles to attach components and produce output
    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let hydro_state = city.resource::<HydroPowerState>();
    assert_eq!(hydro_state.dam_count, 1, "should have 1 hydro dam");
    assert!(
        hydro_state.total_output_mw > 0.0,
        "hydro dam should produce power, got {} MW",
        hydro_state.total_output_mw
    );
}

#[test]
fn test_hydro_output_matches_seasonal_formula() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::HydroDam);

    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let weather = city.resource::<Weather>();
    let current_season = weather.season;
    let expected_factor = seasonal_capacity_factor(current_season);
    let expected_output = HYDRO_NAMEPLATE_MW * expected_factor;

    let hydro_state = city.resource::<HydroPowerState>();
    assert!(
        (hydro_state.total_output_mw - expected_output).abs() < 0.01,
        "output ({} MW) should match seasonal formula ({} MW) for {:?}",
        hydro_state.total_output_mw,
        expected_output,
        current_season
    );
}

#[test]
fn test_hydro_dam_contributes_to_energy_grid() {
    let mut city = TestCity::new()
        .with_utility(50, 50, UtilityType::HydroDam)
        .with_utility(60, 60, UtilityType::HydroDam);

    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let hydro_state = city.resource::<HydroPowerState>();
    assert_eq!(hydro_state.dam_count, 2, "should have 2 hydro dams");

    let energy = city.resource::<EnergyGrid>();
    assert!(
        energy.total_supply_mwh >= hydro_state.total_output_mw - 0.5,
        "energy grid total_supply_mwh ({}) should include hydro contribution ({} MW)",
        energy.total_supply_mwh,
        hydro_state.total_output_mw
    );
}

#[test]
fn test_hydro_state_tracks_dam_count() {
    let mut city = TestCity::new()
        .with_utility(50, 50, UtilityType::HydroDam)
        .with_utility(70, 70, UtilityType::HydroDam)
        .with_utility(90, 90, UtilityType::HydroDam);

    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let hydro_state = city.resource::<HydroPowerState>();
    assert_eq!(hydro_state.dam_count, 3, "should have 3 hydro dams");
}

#[test]
fn test_hydro_seasonal_variation_spring_vs_summer() {
    // Spring should produce more than summer
    let spring_factor = seasonal_capacity_factor(Season::Spring);
    let summer_factor = seasonal_capacity_factor(Season::Summer);

    let spring_output = HYDRO_NAMEPLATE_MW * spring_factor;
    let summer_output = HYDRO_NAMEPLATE_MW * summer_factor;

    assert!(
        spring_output > summer_output,
        "spring output ({spring_output} MW) should exceed summer output ({summer_output} MW)"
    );
}

#[test]
fn test_hydro_capacity_factor_stored_in_state() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::HydroDam);

    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let hydro_state = city.resource::<HydroPowerState>();
    assert!(
        hydro_state.current_capacity_factor > 0.0,
        "capacity factor should be set, got {}",
        hydro_state.current_capacity_factor
    );

    let weather = city.resource::<Weather>();
    let expected = seasonal_capacity_factor(weather.season);
    assert!(
        (hydro_state.current_capacity_factor - expected).abs() < f32::EPSILON,
        "stored capacity factor ({}) should match seasonal value ({}) for {:?}",
        hydro_state.current_capacity_factor,
        expected,
        weather.season
    );
}
