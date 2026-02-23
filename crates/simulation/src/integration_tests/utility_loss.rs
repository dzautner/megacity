use crate::buildings::Building;
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::test_harness::TestCity;
use crate::utilities::{UtilitySource, UtilityType};
use crate::weather::Weather;
use crate::TestSafetyNet;

/// Helper: remove TestSafetyNet so abandonment systems run in these tests.
fn enable_destructive_systems(city: &mut TestCity) {
    city.world_mut().remove_resource::<TestSafetyNet>();
}

// ====================================================================
// TEST-055: Utility Loss -> Abandonment Chain
// ====================================================================

/// Verify that a building with both power and water initially has coverage,
/// and that removing utility sources causes the building to lose coverage
/// after propagation re-runs.
#[test]
fn test_utility_loss_building_loses_power_and_water() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::PowerPlant)
        .with_utility(50, 51, UtilityType::WaterTower)
        .with_building(55, 49, ZoneType::ResidentialLow, 1);
    enable_destructive_systems(&mut city);

    city.tick(5);

    let cell = city.cell(55, 49);
    assert!(
        cell.has_power,
        "building cell should have power before removal"
    );
    assert!(
        cell.has_water,
        "building cell should have water before removal"
    );

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

    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.temperature += 0.1;
    }

    city.tick(5);

    let cell = city.cell(55, 49);
    assert!(
        !cell.has_power,
        "building cell should lose power after source removal"
    );
    assert!(
        !cell.has_water,
        "building cell should lose water after source removal"
    );
}

/// Core abandonment flow: a building that loses both power and water
/// should become abandoned within CHECK_INTERVAL (50) ticks.
#[test]
fn test_utility_loss_triggers_abandonment() {
    use crate::abandonment::Abandoned;

    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::PowerPlant)
        .with_utility(50, 51, UtilityType::WaterTower)
        .with_building(55, 49, ZoneType::ResidentialLow, 1);
    enable_destructive_systems(&mut city);

    city.tick(5);

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.grid_x == 55 && building.grid_y == 49 {
                building.occupants = 5;
            }
        }
    }

    {
        let world = city.world_mut();
        let abandoned_count = world.query::<&Abandoned>().iter(world).count();
        assert_eq!(
            abandoned_count, 0,
            "no buildings should be abandoned initially"
        );
    }

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

    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.temperature += 0.1;
    }

    city.tick(55);

    let abandoned_count = {
        let world = city.world_mut();
        world.query::<&Abandoned>().iter(world).count()
    };
    assert!(
        abandoned_count > 0,
        "building should be marked abandoned after losing both utilities"
    );
}

/// Verify that an abandoned building has its occupants forced to 0.
#[test]
fn test_abandoned_building_evicts_occupants() {
    use crate::abandonment::Abandoned;

    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::PowerPlant)
        .with_utility(50, 51, UtilityType::WaterTower)
        .with_building(55, 49, ZoneType::ResidentialLow, 1);
    enable_destructive_systems(&mut city);

    city.tick(5);

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.grid_x == 55 && building.grid_y == 49 {
                building.occupants = 8;
            }
        }
    }

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

    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.temperature += 0.1;
    }

    city.tick(55);

    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &Abandoned)>();
        let mut found_abandoned = false;
        for (building, _abandoned) in query.iter(world) {
            if building.grid_x == 55 && building.grid_y == 49 {
                assert_eq!(
                    building.occupants, 0,
                    "abandoned building should have 0 occupants"
                );
                found_abandoned = true;
            }
        }
        assert!(found_abandoned, "building at (55, 49) should be abandoned");
    }
}

/// Verify that an abandoned building recovers when both utilities are restored.
#[test]
fn test_abandoned_building_recovers_when_utilities_restored() {
    use crate::abandonment::Abandoned;

    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::PowerPlant)
        .with_utility(50, 51, UtilityType::WaterTower)
        .with_building(55, 49, ZoneType::ResidentialLow, 1);
    enable_destructive_systems(&mut city);

    city.tick(5);

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(55, 49).has_power = false;
        grid.get_mut(55, 49).has_water = false;
    }

    city.tick(55);

    {
        let world = city.world_mut();
        let abandoned_count = world.query::<&Abandoned>().iter(world).count();
        assert!(
            abandoned_count > 0,
            "building should be abandoned after losing utilities"
        );
    }

    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.temperature += 0.1;
    }

    city.tick(5);

    {
        let world = city.world_mut();
        let abandoned_count = world.query::<&Abandoned>().iter(world).count();
        assert_eq!(
            abandoned_count, 0,
            "building should recover after utilities are restored"
        );
    }
}

/// Verify that an abandoned building is demolished after DEMOLISH_THRESHOLD ticks.
#[test]
fn test_abandoned_building_demolished_after_threshold() {
    use crate::abandonment::Abandoned;

    let mut city = TestCity::new().with_building(55, 55, ZoneType::ResidentialLow, 1);
    enable_destructive_systems(&mut city);

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(55, 55).has_power = false;
        grid.get_mut(55, 55).has_water = false;
    }

    city.tick(55);

    {
        let world = city.world_mut();
        let abandoned_count = world.query::<&Abandoned>().iter(world).count();
        assert!(abandoned_count > 0, "building should be abandoned");
    }

    city.tick(510);

    let building_count = city.building_count();
    assert_eq!(
        building_count, 0,
        "building should be demolished after abandonment threshold"
    );

    let cell = city.cell(55, 55);
    assert!(
        cell.building_id.is_none(),
        "grid cell building_id should be cleared after demolition"
    );
    assert_eq!(
        cell.zone,
        ZoneType::None,
        "grid cell zone should be cleared after demolition"
    );
}

/// Full chain test with citizens: building with citizens loses utilities,
/// becomes abandoned, citizens are evicted (occupants = 0).
#[test]
fn test_utility_loss_abandonment_chain_with_citizens() {
    use crate::abandonment::Abandoned;

    let mut city = TestCity::new()
        .with_road(50, 50, 65, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::PowerPlant)
        .with_utility(50, 51, UtilityType::WaterTower)
        .with_building(55, 49, ZoneType::ResidentialLow, 1)
        .with_building(60, 49, ZoneType::CommercialLow, 1)
        .with_citizen((55, 49), (60, 49))
        .with_citizen((55, 49), (60, 49))
        .with_citizen((55, 49), (60, 49));
    enable_destructive_systems(&mut city);

    city.tick(5);

    assert!(
        city.cell(55, 49).has_power,
        "home building should have power"
    );
    assert!(
        city.cell(55, 49).has_water,
        "home building should have water"
    );

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.grid_x == 55 && building.grid_y == 49 {
                building.occupants = 3;
            }
        }
    }

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

    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.temperature += 0.1;
    }

    city.tick(55);

    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &Abandoned)>();
        let mut found_home_abandoned = false;
        for (building, _) in query.iter(world) {
            if building.grid_x == 55 && building.grid_y == 49 {
                assert_eq!(
                    building.occupants, 0,
                    "abandoned home building should have 0 occupants (citizens evicted)"
                );
                found_home_abandoned = true;
            }
        }
        assert!(
            found_home_abandoned,
            "home building should be marked abandoned after utility loss"
        );
    }
}

/// Verify that losing only power (but retaining water) does NOT trigger
/// abandonment, since the condition requires BOTH to be missing.
#[test]
fn test_partial_utility_loss_no_abandonment() {
    use crate::abandonment::Abandoned;

    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::PowerPlant)
        .with_utility(50, 51, UtilityType::WaterTower)
        .with_building(55, 49, ZoneType::ResidentialLow, 1);
    enable_destructive_systems(&mut city);

    city.tick(5);

    let power_entity: Option<bevy::prelude::Entity> = {
        let world = city.world_mut();
        world
            .query::<(bevy::prelude::Entity, &UtilitySource)>()
            .iter(world)
            .find(|(_, s)| s.utility_type == UtilityType::PowerPlant)
            .map(|(e, _)| e)
    };
    if let Some(entity) = power_entity {
        city.world_mut().despawn(entity);
    }

    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.temperature += 0.1;
    }

    city.tick(55);

    let cell = city.cell(55, 49);
    assert!(!cell.has_power, "building cell should have lost power");
    assert!(cell.has_water, "building cell should still have water");

    let abandoned_count = {
        let world = city.world_mut();
        world.query::<&Abandoned>().iter(world).count()
    };
    assert_eq!(
        abandoned_count, 0,
        "building should NOT be abandoned when only one utility is missing"
    );
}
