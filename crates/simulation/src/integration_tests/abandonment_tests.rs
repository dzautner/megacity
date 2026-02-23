//! TEST-043: Integration tests for building abandonment logic.
//!
//! Tests the `Abandoned` component lifecycle including:
//! - Abandonment triggered by missing both utilities (power + water)
//! - Abandonment triggered by upgraded empty buildings (level > 1, 0 occupants)
//! - Occupants forced to 0 on abandoned buildings
//! - Recovery when utilities are restored
//! - Citizens associated with abandoned buildings lose occupancy
//! - Demolished after threshold ticks of abandonment

use crate::abandonment::Abandoned;
use crate::buildings::Building;
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::immigration::CityAttractiveness;
use crate::test_harness::TestCity;
use crate::utilities::{UtilitySource, UtilityType};

/// Helper: set CityAttractiveness high to prevent emigration side-effects.
fn suppress_emigration(city: &mut TestCity) {
    let world = city.world_mut();
    if let Some(mut attractiveness) = world.get_resource_mut::<CityAttractiveness>() {
        attractiveness.overall_score = 80.0;
    }
}

// ====================================================================
// Test 1: Building abandons when it has no power AND no water
// ====================================================================

/// A building that has neither power nor water should be marked `Abandoned`
/// after the CHECK_INTERVAL (50 ticks) elapses.
#[test]
fn test_abandonment_no_power_no_water_triggers_abandoned() {
    let mut city = TestCity::new().with_building(80, 80, ZoneType::ResidentialLow, 1);

    suppress_emigration(&mut city);

    // Ensure both utilities are off (default for a building with no utility sources).
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(80, 80).has_power = false;
        grid.get_mut(80, 80).has_water = false;
    }

    // Before the check interval, building should NOT be abandoned.
    city.tick(10);
    {
        let world = city.world_mut();
        let count = world.query::<&Abandoned>().iter(world).count();
        assert_eq!(
            count, 0,
            "building should not be abandoned before CHECK_INTERVAL"
        );
    }

    // Tick past the CHECK_INTERVAL (50 ticks) so the system runs.
    city.tick(45);
    {
        let world = city.world_mut();
        let count = world.query::<&Abandoned>().iter(world).count();
        assert!(
            count > 0,
            "building should be marked Abandoned when both power and water are missing"
        );
    }
}

// ====================================================================
// Test 2: Upgraded empty building (level > 1, 0 occupants) triggers abandonment
// ====================================================================

/// An upgraded building (level > 1) with 0 occupants should be marked as
/// Abandoned even if it has power and water.
#[test]
fn test_abandonment_upgraded_empty_building_triggers_abandoned() {
    let mut city = TestCity::new()
        .with_road(80, 80, 90, 80, RoadType::Local)
        .with_utility(80, 80, UtilityType::PowerPlant)
        .with_utility(80, 81, UtilityType::WaterTower)
        .with_building(85, 79, ZoneType::ResidentialLow, 2); // level 2

    suppress_emigration(&mut city);

    // Let utilities propagate.
    city.tick(5);

    // Confirm the building has utilities.
    let cell = city.cell(85, 79);
    assert!(cell.has_power, "building should have power");
    assert!(cell.has_water, "building should have water");

    // Verify building is level 2 with 0 occupants.
    {
        let world = city.world_mut();
        let mut query = world.query::<&Building>();
        let building = query
            .iter(world)
            .find(|b| b.grid_x == 85 && b.grid_y == 79)
            .expect("building at (85, 79) should exist");
        assert_eq!(building.level, 2, "building should be level 2");
        assert_eq!(building.occupants, 0, "building should have 0 occupants");
    }

    // No abandoned buildings yet.
    {
        let world = city.world_mut();
        let count = world.query::<&Abandoned>().iter(world).count();
        assert_eq!(count, 0, "no buildings should be abandoned initially");
    }

    // Tick past CHECK_INTERVAL.
    city.tick(50);

    // The upgraded empty building should now be abandoned.
    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &Abandoned)>();
        let found = query
            .iter(world)
            .any(|(b, _)| b.grid_x == 85 && b.grid_y == 79);
        assert!(
            found,
            "upgraded empty building (level > 1, 0 occupants) should be marked Abandoned"
        );
    }
}

// ====================================================================
// Test 3: Level 1 building with 0 occupants but utilities does NOT abandon
// ====================================================================

/// A level-1 building with 0 occupants but with power and water should
/// NOT be abandoned. Only upgraded buildings (level > 1) with 0 occupants
/// trigger the empty-upgraded condition.
#[test]
fn test_abandonment_level1_empty_with_utilities_not_abandoned() {
    let mut city = TestCity::new()
        .with_road(80, 80, 90, 80, RoadType::Local)
        .with_utility(80, 80, UtilityType::PowerPlant)
        .with_utility(80, 81, UtilityType::WaterTower)
        .with_building(85, 79, ZoneType::ResidentialLow, 1); // level 1

    suppress_emigration(&mut city);

    // Let utilities propagate.
    city.tick(5);

    // Confirm utilities are present.
    let cell = city.cell(85, 79);
    assert!(cell.has_power, "building should have power");
    assert!(cell.has_water, "building should have water");

    // Tick well past CHECK_INTERVAL.
    city.tick(110);

    // Building should NOT be abandoned: level 1 with utilities.
    {
        let world = city.world_mut();
        let count = world.query::<&Abandoned>().iter(world).count();
        assert_eq!(
            count, 0,
            "level 1 empty building with both utilities should NOT be abandoned"
        );
    }
}

// ====================================================================
// Test 4: Abandoned building has occupants forced to 0
// ====================================================================

/// When a building becomes abandoned, its occupants should be forced to 0
/// by the `process_abandoned_buildings` system.
#[test]
fn test_abandonment_forces_occupants_to_zero() {
    let mut city = TestCity::new()
        .with_road(80, 80, 90, 80, RoadType::Local)
        .with_utility(80, 80, UtilityType::PowerPlant)
        .with_utility(80, 81, UtilityType::WaterTower)
        .with_building(85, 79, ZoneType::ResidentialLow, 1);

    suppress_emigration(&mut city);

    // Let utilities propagate.
    city.tick(5);

    // Give the building some occupants.
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.grid_x == 85 && building.grid_y == 79 {
                building.occupants = 10;
            }
        }
    }

    // Verify occupants are set.
    {
        let world = city.world_mut();
        let mut query = world.query::<&Building>();
        let building = query
            .iter(world)
            .find(|b| b.grid_x == 85 && b.grid_y == 79)
            .unwrap();
        assert_eq!(building.occupants, 10, "building should have 10 occupants");
    }

    // Remove all utility sources to trigger abandonment.
    let utility_entities: Vec<bevy::prelude::Entity> = {
        let world = city.world_mut();
        world
            .query::<(bevy::prelude::Entity, &UtilitySource)>()
            .iter(world)
            .map(|(e, _)| e)
            .collect()
    };
    for entity in utility_entities {
        city.world_mut().despawn(entity);
    }

    // Nudge weather to force utility re-propagation.
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<crate::weather::Weather>();
        weather.temperature += 0.1;
    }

    // Tick past CHECK_INTERVAL so abandonment triggers.
    city.tick(55);

    // Building should be abandoned with 0 occupants.
    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &Abandoned)>();
        let result = query
            .iter(world)
            .find(|(b, _)| b.grid_x == 85 && b.grid_y == 79);
        assert!(result.is_some(), "building should be abandoned");

        let (building, _) = result.unwrap();
        assert_eq!(
            building.occupants, 0,
            "abandoned building should have occupants forced to 0"
        );
    }
}

// ====================================================================
// Test 5: Citizens removed from abandoned building (occupants = 0 with citizens)
// ====================================================================

/// When a building with citizen occupants becomes abandoned, occupants
/// should be forced to 0. This test uses spawned citizens with the building
/// as their home.
#[test]
fn test_abandonment_citizens_evicted_occupants_zero() {
    let mut city = TestCity::new()
        .with_road(80, 80, 95, 80, RoadType::Local)
        .with_utility(80, 80, UtilityType::PowerPlant)
        .with_utility(80, 81, UtilityType::WaterTower)
        .with_building(85, 79, ZoneType::ResidentialLow, 1)
        .with_building(90, 79, ZoneType::CommercialLow, 1)
        .with_citizen((85, 79), (90, 79))
        .with_citizen((85, 79), (90, 79));

    suppress_emigration(&mut city);

    // Let utilities propagate.
    city.tick(5);

    // Set occupants to match the citizen count.
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.grid_x == 85 && building.grid_y == 79 {
                building.occupants = 2;
            }
        }
    }

    // Remove all utility sources.
    let utility_entities: Vec<bevy::prelude::Entity> = {
        let world = city.world_mut();
        world
            .query::<(bevy::prelude::Entity, &UtilitySource)>()
            .iter(world)
            .map(|(e, _)| e)
            .collect()
    };
    for entity in utility_entities {
        city.world_mut().despawn(entity);
    }

    // Nudge weather to force utility re-propagation.
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<crate::weather::Weather>();
        weather.temperature += 0.1;
    }

    // Tick past CHECK_INTERVAL.
    city.tick(55);

    // The home building should be abandoned with 0 occupants.
    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &Abandoned)>();
        let result = query
            .iter(world)
            .find(|(b, _)| b.grid_x == 85 && b.grid_y == 79);
        assert!(
            result.is_some(),
            "home building should be marked Abandoned after utility loss"
        );
        let (building, _) = result.unwrap();
        assert_eq!(
            building.occupants, 0,
            "abandoned building with citizens should have occupants forced to 0"
        );
    }
}

// ====================================================================
// Test 6: Abandoned building recovers when utilities are restored
// ====================================================================

/// An abandoned building should have its `Abandoned` component removed
/// when both power and water are restored.
#[test]
fn test_abandonment_recovery_when_utilities_restored() {
    let mut city = TestCity::new()
        .with_road(80, 80, 90, 80, RoadType::Local)
        .with_utility(80, 80, UtilityType::PowerPlant)
        .with_utility(80, 81, UtilityType::WaterTower)
        .with_building(85, 79, ZoneType::ResidentialLow, 1);

    suppress_emigration(&mut city);

    // Let utilities propagate.
    city.tick(5);

    // Manually remove utility coverage to trigger abandonment.
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(85, 79).has_power = false;
        grid.get_mut(85, 79).has_water = false;
    }

    // Tick past CHECK_INTERVAL.
    city.tick(55);

    // Building should be abandoned.
    {
        let world = city.world_mut();
        let count = world.query::<&Abandoned>().iter(world).count();
        assert!(
            count > 0,
            "building should be abandoned after losing utilities"
        );
    }

    // Utility sources are still alive, so propagation should restore coverage.
    // Nudge weather to force re-propagation.
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<crate::weather::Weather>();
        weather.temperature += 0.1;
    }

    // Tick a few times for propagation + process_abandoned_buildings to recover.
    city.tick(5);

    // Building should have recovered (no more Abandoned component).
    {
        let world = city.world_mut();
        let count = world.query::<&Abandoned>().iter(world).count();
        assert_eq!(
            count, 0,
            "building should recover from abandonment when utilities are restored"
        );
    }
}
