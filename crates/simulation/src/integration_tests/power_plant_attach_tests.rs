//! PLAY-P0-06: Verify power plants placed via UI register as generators.
//!
//! When a player places a power plant via the UI, `place_utility_source` spawns
//! only a `UtilitySource` entity. The `attach_*_power_plants` systems must
//! detect these entities and insert the `PowerPlant` component so that the
//! energy dispatch system recognises them as generators.

use crate::coal_power::{CoalPowerState, PowerPlant, PowerPlantType, COAL_CAPACITY_MW};
use crate::nuclear_power::NuclearPowerState;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

// ====================================================================
// Coal power plant attach
// ====================================================================

#[test]
fn test_coal_utility_source_gets_power_plant_attached() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::PowerPlant);

    // Before ticking, the entity should only have UtilitySource, not PowerPlant
    let world = city.world_mut();
    let count_before = world
        .query::<&PowerPlant>()
        .iter(world)
        .filter(|p| p.plant_type == PowerPlantType::Coal)
        .count();
    assert_eq!(
        count_before, 0,
        "Before tick, coal UtilitySource should not yet have PowerPlant"
    );

    // Run a slow tick cycle to trigger attach_coal_power_plants
    city.tick_slow_cycle();

    let world = city.world_mut();
    let count_after = world
        .query::<&PowerPlant>()
        .iter(world)
        .filter(|p| p.plant_type == PowerPlantType::Coal)
        .count();
    assert_eq!(
        count_after, 1,
        "After tick, coal UtilitySource should have PowerPlant attached"
    );
}

#[test]
fn test_coal_utility_source_contributes_to_energy_supply() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::PowerPlant);

    // Run enough ticks for attach + aggregate
    city.tick_slow_cycles(2);

    let state = city.resource::<CoalPowerState>();
    assert_eq!(state.plant_count, 1, "Should count 1 coal plant");
    assert!(
        state.total_output_mw > 0.0,
        "Coal plant should produce power, got {} MW",
        state.total_output_mw
    );
}

#[test]
fn test_coal_utility_source_registers_in_dispatch() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::PowerPlant);

    city.tick_slow_cycles(2);

    // Verify the PowerPlant entity has capacity set correctly
    let world = city.world_mut();
    let capacities: Vec<f32> = world
        .query::<&PowerPlant>()
        .iter(world)
        .filter(|p| p.plant_type == PowerPlantType::Coal)
        .map(|p| p.capacity_mw)
        .collect();

    assert_eq!(capacities.len(), 1, "Should have exactly 1 coal PowerPlant");
    assert!(
        (capacities[0] - COAL_CAPACITY_MW).abs() < f32::EPSILON,
        "Coal plant capacity should be {} MW, got {}",
        COAL_CAPACITY_MW,
        capacities[0]
    );
}

// ====================================================================
// Nuclear power plant attach
// ====================================================================

#[test]
fn test_nuclear_utility_source_gets_power_plant_attached() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::NuclearPlant);

    city.tick_slow_cycle();

    let world = city.world_mut();
    let count = world
        .query::<&PowerPlant>()
        .iter(world)
        .filter(|p| p.plant_type == PowerPlantType::Nuclear)
        .count();
    assert_eq!(
        count, 1,
        "After tick, nuclear UtilitySource should have PowerPlant attached"
    );
}

#[test]
fn test_nuclear_utility_source_contributes_to_energy_supply() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::NuclearPlant);

    city.tick_slow_cycles(2);

    let state = city.resource::<NuclearPowerState>();
    assert_eq!(state.plant_count, 1, "Should count 1 nuclear plant");
    assert!(
        state.total_output_mw > 0.0,
        "Nuclear plant should produce power, got {} MW",
        state.total_output_mw
    );
}

// ====================================================================
// Multiple power plants
// ====================================================================

#[test]
fn test_multiple_utility_sources_all_get_power_plant() {
    let mut city = TestCity::new()
        .with_utility(40, 40, UtilityType::PowerPlant)
        .with_utility(60, 60, UtilityType::PowerPlant)
        .with_utility(80, 80, UtilityType::NuclearPlant);

    city.tick_slow_cycle();

    let world = city.world_mut();
    let coal_count = world
        .query::<&PowerPlant>()
        .iter(world)
        .filter(|p| p.plant_type == PowerPlantType::Coal)
        .count();
    let nuclear_count = world
        .query::<&PowerPlant>()
        .iter(world)
        .filter(|p| p.plant_type == PowerPlantType::Nuclear)
        .count();

    assert_eq!(coal_count, 2, "Should have 2 coal PowerPlants");
    assert_eq!(nuclear_count, 1, "Should have 1 nuclear PowerPlant");
}

// ====================================================================
// Idempotent: attach does not duplicate
// ====================================================================

#[test]
fn test_attach_is_idempotent_no_duplicate_power_plant() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::PowerPlant);

    // Run multiple slow tick cycles
    city.tick_slow_cycles(5);

    let world = city.world_mut();
    let count = world
        .query::<&PowerPlant>()
        .iter(world)
        .filter(|p| p.plant_type == PowerPlantType::Coal)
        .count();
    assert_eq!(
        count, 1,
        "Should still have exactly 1 PowerPlant after multiple ticks, got {count}"
    );
}

// ====================================================================
// Non-power utilities should NOT get PowerPlant
// ====================================================================

#[test]
fn test_water_tower_does_not_get_power_plant() {
    let mut city = TestCity::new().with_utility(50, 50, UtilityType::WaterTower);

    city.tick_slow_cycle();

    let world = city.world_mut();
    let count = world.query::<&PowerPlant>().iter(world).count();
    assert_eq!(
        count, 0,
        "Water tower should not receive PowerPlant component"
    );
}
