//! Integration tests for the Combined Heat and Power (CHP) system (POWER-021).

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::combined_heat_power::{
    ChpState, CHP_EFFICIENCY_BONUS, CHP_HEAT_OUTPUT_RATIO, CHP_UPGRADE_COST,
};
use crate::heating::HeatingGrid;
use crate::test_harness::TestCity;

/// Helper: spawn a coal power plant and register it for CHP upgrade.
fn spawn_chp_coal_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_coal(x, y));
    let mut chp = world.resource_mut::<ChpState>();
    chp.upgraded_positions.push((x, y));
}

/// Helper: spawn a gas power plant and register it for CHP upgrade.
fn spawn_chp_gas_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_gas(x, y));
    let mut chp = world.resource_mut::<ChpState>();
    chp.upgraded_positions.push((x, y));
}

/// Helper: spawn a biomass power plant and register it for CHP upgrade.
fn spawn_chp_biomass_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_biomass(x, y));
    let mut chp = world.resource_mut::<ChpState>();
    chp.upgraded_positions.push((x, y));
}

/// Helper: spawn a coal power plant without CHP (for comparison).
fn spawn_non_chp_coal_plant(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    world.spawn(PowerPlant::new_coal(x, y));
}

/// Helper: place roads in a rectangular area for BFS propagation.
fn place_road_area(city: &mut TestCity, x0: usize, y0: usize, x1: usize, y1: usize) {
    let world = city.world_mut();
    world.resource_scope(|world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
        world.resource_scope(|_world, mut roads: bevy::prelude::Mut<crate::roads::RoadNetwork>| {
            for y in y0..=y1 {
                for x in x0..=x1 {
                    roads.place_road(&mut grid, x, y);
                }
            }
        });
    });
}

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_chp_state_exists_in_new_city() {
    let city = TestCity::new();
    let state = city.resource::<ChpState>();
    assert_eq!(state.upgrade_count, 0);
    assert!(state.upgraded_positions.is_empty());
}

// ====================================================================
// CHP plant provides heating coverage
// ====================================================================

#[test]
fn test_chp_coal_plant_provides_heating() {
    let mut city = TestCity::new().with_weather(-5.0);
    spawn_chp_coal_plant(&mut city, 50, 50);
    place_road_area(&mut city, 45, 45, 55, 55);

    city.tick_slow_cycle();

    let heating = city.resource::<HeatingGrid>();
    assert!(
        heating.is_heated(50, 50),
        "CHP plant should provide heating at its location"
    );
}

#[test]
fn test_non_chp_plant_no_heating() {
    let mut city = TestCity::new().with_weather(-5.0);
    spawn_non_chp_coal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let heating = city.resource::<HeatingGrid>();
    assert!(
        !heating.is_heated(50, 50),
        "Non-CHP plant should not provide heating"
    );
}

// ====================================================================
// CHP heat propagates to nearby cells
// ====================================================================

#[test]
fn test_chp_heat_propagates_nearby() {
    let mut city = TestCity::new().with_weather(-5.0);
    spawn_chp_coal_plant(&mut city, 50, 50);
    place_road_area(&mut city, 40, 40, 60, 60);

    city.tick_slow_cycle();

    let heating = city.resource::<HeatingGrid>();
    assert!(
        heating.is_heated(52, 50),
        "Cell 2 away from CHP plant should be heated"
    );
    let heat_near = heating.get(51, 50);
    let heat_far = heating.get(55, 50);
    assert!(
        heat_near >= heat_far,
        "Heat should decay: near={heat_near}, far={heat_far}"
    );
}

// ====================================================================
// CHP stats tracking
// ====================================================================

#[test]
fn test_chp_stats_update_with_cold_weather() {
    let mut city = TestCity::new().with_weather(-5.0);
    spawn_chp_coal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<ChpState>();
    assert_eq!(state.upgrade_count, 1, "Should track 1 CHP-upgraded plant");
    assert!(
        state.total_heat_output_mw > 0.0,
        "Heat output should be positive in cold weather, got {}",
        state.total_heat_output_mw
    );
    assert!(
        state.total_efficiency_bonus_mw > 0.0,
        "Efficiency bonus should be positive, got {}",
        state.total_efficiency_bonus_mw
    );
}

#[test]
fn test_chp_stats_zero_in_warm_weather() {
    let mut city = TestCity::new().with_weather(25.0);
    spawn_chp_coal_plant(&mut city, 50, 50);

    city.tick_slow_cycle();

    let state = city.resource::<ChpState>();
    assert_eq!(state.upgrade_count, 1);
    assert!(
        state.total_heat_output_mw.abs() < f32::EPSILON,
        "Heat output should be zero in warm weather, got {}",
        state.total_heat_output_mw
    );
}

// ====================================================================
// Multiple CHP plant types
// ====================================================================

#[test]
fn test_multiple_chp_plant_types() {
    let mut city = TestCity::new().with_weather(-5.0);
    spawn_chp_coal_plant(&mut city, 50, 50);
    spawn_chp_gas_plant(&mut city, 70, 70);
    spawn_chp_biomass_plant(&mut city, 90, 90);

    city.tick_slow_cycle();

    let state = city.resource::<ChpState>();
    assert_eq!(
        state.upgrade_count, 3,
        "Should track 3 CHP-upgraded plants"
    );
    assert!(
        state.total_heat_output_mw > 0.0,
        "Total heat should be positive"
    );
}

// ====================================================================
// CHP eligibility discrimination
// ====================================================================

#[test]
fn test_chp_ignores_ineligible_plant_types() {
    use crate::combined_heat_power::is_chp_eligible;

    assert!(is_chp_eligible(PowerPlantType::Coal));
    assert!(is_chp_eligible(PowerPlantType::NaturalGas));
    assert!(is_chp_eligible(PowerPlantType::Biomass));
    assert!(is_chp_eligible(PowerPlantType::WasteToEnergy));
    assert!(!is_chp_eligible(PowerPlantType::WindTurbine));
    assert!(!is_chp_eligible(PowerPlantType::Nuclear));
    assert!(!is_chp_eligible(PowerPlantType::HydroDam));
}

// ====================================================================
// CHP constants validation
// ====================================================================

#[test]
fn test_chp_upgrade_cost() {
    assert!(
        (CHP_UPGRADE_COST - 20_000_000.0).abs() < f64::EPSILON,
        "CHP upgrade should cost $20M"
    );
}

#[test]
fn test_chp_efficiency_bonus() {
    assert!(
        (CHP_EFFICIENCY_BONUS - 0.15).abs() < f32::EPSILON,
        "CHP should provide +15% efficiency"
    );
}

#[test]
fn test_chp_heat_ratio() {
    assert!(
        (CHP_HEAT_OUTPUT_RATIO - 0.5).abs() < f32::EPSILON,
        "Heat output should be 0.5x electricity"
    );
}

// ====================================================================
// Empty city has no CHP output
// ====================================================================

#[test]
fn test_no_chp_plants_zero_output() {
    let mut city = TestCity::new().with_weather(-5.0);

    city.tick_slow_cycle();

    let state = city.resource::<ChpState>();
    assert_eq!(state.upgrade_count, 0);
    assert!(state.total_heat_output_mw.abs() < f32::EPSILON);
    assert!(state.total_efficiency_bonus_mw.abs() < f32::EPSILON);
}
