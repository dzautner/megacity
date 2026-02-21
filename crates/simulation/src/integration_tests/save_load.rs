use crate::buildings::Building;
use crate::citizen::CitizenState;
use crate::economy::CityBudget;
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::utilities::{UtilitySource, UtilityType};

/// Test that citizen component data survives a serde serialize/deserialize
/// roundtrip, verifying that personality, needs, details, position, velocity,
/// and activity timer all match after decoding. This validates the same
/// serialization path that the save system relies on (serde derives).
#[test]
fn test_save_load_roundtrip_citizen_fidelity() {
    use crate::citizen::{
        CitizenDetails, CitizenStateComp, Family, Gender, HomeLocation, Needs, PathCache,
        Personality, Position, Velocity, WorkLocation,
    };
    use crate::movement::ActivityTimer;

    // Build a small city with specific citizen data.
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    // Spawn a citizen with known, non-default values.
    let world = city.world_mut();

    let home_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(12, 11).building_id.unwrap()
    };
    let work_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(18, 11).building_id.unwrap()
    };

    world.spawn((
        crate::citizen::Citizen,
        Position { x: 200.5, y: 180.3 },
        Velocity { x: 1.5, y: -0.7 },
        HomeLocation {
            grid_x: 12,
            grid_y: 11,
            building: home_entity,
        },
        WorkLocation {
            grid_x: 18,
            grid_y: 11,
            building: work_entity,
        },
        CitizenStateComp(CitizenState::Working),
        PathCache::new(vec![
            crate::roads::RoadNode(12, 10),
            crate::roads::RoadNode(15, 10),
            crate::roads::RoadNode(18, 10),
        ]),
        CitizenDetails {
            age: 42,
            gender: Gender::Female,
            education: 3,
            happiness: 72.5,
            health: 88.3,
            salary: 6500.0,
            savings: 15000.0,
        },
        Personality {
            ambition: 0.85,
            sociability: 0.3,
            materialism: 0.65,
            resilience: 0.92,
        },
        Needs {
            hunger: 55.0,
            energy: 70.0,
            social: 42.0,
            fun: 38.0,
            comfort: 65.0,
        },
        Family::default(),
        ActivityTimer(150),
    ));

    // Collect citizen data before "save".
    let world = city.world_mut();
    let mut query = world.query::<(
        &CitizenDetails,
        &CitizenStateComp,
        &HomeLocation,
        &WorkLocation,
        &PathCache,
        &Velocity,
        &Position,
        &Personality,
        &Needs,
        &ActivityTimer,
    )>();

    // Serialize each component via serde_json (the same serde path the save system
    // uses, since all citizen components derive Serialize/Deserialize).
    let mut serialized_data = Vec::new();
    for (details, state, home, work, path, vel, pos, pers, needs, timer) in query.iter(world) {
        let details_json = serde_json::to_string(details).unwrap();
        let state_json = serde_json::to_string(&state.0).unwrap();
        let path_json = serde_json::to_string(path).unwrap();
        let vel_json = serde_json::to_string(vel).unwrap();
        let pos_json = serde_json::to_string(pos).unwrap();
        let pers_json = serde_json::to_string(pers).unwrap();
        let needs_json = serde_json::to_string(needs).unwrap();

        serialized_data.push((
            details_json,
            state_json,
            (home.grid_x, home.grid_y),
            (work.grid_x, work.grid_y),
            path_json,
            vel_json,
            pos_json,
            pers_json,
            needs_json,
            timer.0,
        ));
    }

    assert_eq!(serialized_data.len(), 1, "should have exactly one citizen");
    let saved = &serialized_data[0];

    // Deserialize (simulating load).
    let details_after: CitizenDetails = serde_json::from_str(&saved.0).unwrap();
    let state_after: CitizenState = serde_json::from_str(&saved.1).unwrap();
    let path_after: PathCache = serde_json::from_str(&saved.4).unwrap();
    let vel_after: Velocity = serde_json::from_str(&saved.5).unwrap();
    let pos_after: Position = serde_json::from_str(&saved.6).unwrap();
    let pers_after: Personality = serde_json::from_str(&saved.7).unwrap();
    let needs_after: Needs = serde_json::from_str(&saved.8).unwrap();

    // Assert all citizen details match.
    assert_eq!(details_after.age, 42, "age mismatch after roundtrip");
    assert!(
        matches!(details_after.gender, Gender::Female),
        "gender mismatch after roundtrip"
    );
    assert_eq!(
        details_after.education, 3,
        "education mismatch after roundtrip"
    );
    assert!(
        (details_after.happiness - 72.5).abs() < f32::EPSILON,
        "happiness mismatch: {}",
        details_after.happiness
    );
    assert!(
        (details_after.health - 88.3).abs() < 0.01,
        "health mismatch: {}",
        details_after.health
    );
    assert!(
        (details_after.salary - 6500.0).abs() < f32::EPSILON,
        "salary mismatch after roundtrip"
    );
    assert!(
        (details_after.savings - 15000.0).abs() < f32::EPSILON,
        "savings mismatch after roundtrip"
    );

    // Assert state.
    assert_eq!(
        state_after,
        CitizenState::Working,
        "state mismatch after roundtrip"
    );

    // Assert personality.
    assert!(
        (pers_after.ambition - 0.85).abs() < f32::EPSILON,
        "ambition mismatch after roundtrip"
    );
    assert!(
        (pers_after.sociability - 0.3).abs() < f32::EPSILON,
        "sociability mismatch after roundtrip"
    );
    assert!(
        (pers_after.materialism - 0.65).abs() < f32::EPSILON,
        "materialism mismatch after roundtrip"
    );
    assert!(
        (pers_after.resilience - 0.92).abs() < f32::EPSILON,
        "resilience mismatch after roundtrip"
    );

    // Assert needs.
    assert!(
        (needs_after.hunger - 55.0).abs() < f32::EPSILON,
        "hunger mismatch after roundtrip"
    );
    assert!(
        (needs_after.energy - 70.0).abs() < f32::EPSILON,
        "energy mismatch after roundtrip"
    );
    assert!(
        (needs_after.social - 42.0).abs() < f32::EPSILON,
        "social mismatch after roundtrip"
    );
    assert!(
        (needs_after.fun - 38.0).abs() < f32::EPSILON,
        "fun mismatch after roundtrip"
    );
    assert!(
        (needs_after.comfort - 65.0).abs() < f32::EPSILON,
        "comfort mismatch after roundtrip"
    );

    // Assert position.
    assert!(
        (pos_after.x - 200.5).abs() < f32::EPSILON,
        "pos_x mismatch: {}",
        pos_after.x
    );
    assert!(
        (pos_after.y - 180.3).abs() < 0.01,
        "pos_y mismatch: {}",
        pos_after.y
    );

    // Assert velocity.
    assert!(
        (vel_after.x - 1.5).abs() < f32::EPSILON,
        "vel_x mismatch after roundtrip"
    );
    assert!(
        (vel_after.y - (-0.7)).abs() < f32::EPSILON,
        "vel_y mismatch after roundtrip"
    );

    // Assert path cache.
    assert_eq!(
        path_after.waypoints.len(),
        3,
        "path length mismatch after roundtrip"
    );
    assert_eq!(
        (path_after.waypoints[0].0, path_after.waypoints[0].1),
        (12, 10),
        "path[0] mismatch"
    );
    assert_eq!(
        (path_after.waypoints[1].0, path_after.waypoints[1].1),
        (15, 10),
        "path[1] mismatch"
    );
    assert_eq!(
        (path_after.waypoints[2].0, path_after.waypoints[2].1),
        (18, 10),
        "path[2] mismatch"
    );
    assert_eq!(path_after.current_index, 0, "path current_index mismatch");

    // Assert home/work grid positions survived.
    assert_eq!(saved.2, (12, 11), "home position mismatch after roundtrip");
    assert_eq!(saved.3, (18, 11), "work position mismatch after roundtrip");

    // Assert activity timer.
    assert_eq!(saved.9, 150, "activity timer mismatch after roundtrip");
}

/// Test that tearing down a city (despawning all entities, resetting resources)
/// results in a clean slate with no leftover entities.
#[test]
fn test_new_game_teardown_clean_slate() {
    use crate::citizen::Citizen;

    // Build a city with roads, buildings, and citizens.
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_building(24, 11, ZoneType::Industrial, 1)
        .with_citizen((12, 11), (18, 11))
        .with_citizen((12, 11), (24, 11))
        .with_service(15, 11, ServiceType::FireStation)
        .with_utility(20, 11, UtilityType::PowerPlant);

    // Run a few ticks so systems process.
    city.tick(5);

    // Verify city is populated.
    assert!(city.citizen_count() >= 2, "city should have citizens");
    assert!(city.building_count() >= 3, "city should have buildings");

    // Simulate "new game" by despawning all entities and resetting resources.
    // (The actual NewGameEvent is handled by SavePlugin in the app crate,
    //  but we test the core teardown logic directly.)
    let world = city.world_mut();

    // Despawn all citizens.
    let citizen_entities: Vec<bevy::prelude::Entity> = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<Citizen>>()
        .iter(world)
        .collect();
    for entity in citizen_entities {
        world.despawn(entity);
    }

    // Despawn all buildings.
    let building_entities: Vec<bevy::prelude::Entity> = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<Building>>()
        .iter(world)
        .collect();
    for entity in building_entities {
        world.despawn(entity);
    }

    // Despawn all service buildings.
    let service_entities: Vec<bevy::prelude::Entity> = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ServiceBuilding>>()
        .iter(world)
        .collect();
    for entity in service_entities {
        world.despawn(entity);
    }

    // Despawn all utility sources.
    let utility_entities: Vec<bevy::prelude::Entity> = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<UtilitySource>>()
        .iter(world)
        .collect();
    for entity in utility_entities {
        world.despawn(entity);
    }

    // Reset resources to defaults (simulating new-game reset).
    let width = world.resource::<WorldGrid>().width;
    let height = world.resource::<WorldGrid>().height;
    *world.resource_mut::<WorldGrid>() = WorldGrid::new(width, height);
    *world.resource_mut::<RoadNetwork>() = RoadNetwork::default();
    *world.resource_mut::<RoadSegmentStore>() = RoadSegmentStore::default();
    world.resource_mut::<CityBudget>().treasury = 50_000.0;
    world.resource_mut::<CityBudget>().tax_rate = 0.10;
    world.resource_mut::<CityBudget>().last_collection_day = 0;
    world.resource_mut::<GameClock>().day = 1;
    world.resource_mut::<GameClock>().hour = 8.0;

    // Verify clean slate.
    assert_eq!(
        city.citizen_count(),
        0,
        "should have 0 citizens after new game"
    );
    assert_eq!(
        city.building_count(),
        0,
        "should have 0 buildings after new game"
    );
    assert_eq!(
        city.road_cell_count(),
        0,
        "should have 0 road cells after new game"
    );
    assert!(
        (city.budget().treasury - 50_000.0).abs() < f64::EPSILON,
        "treasury should be reset to 50000, got {}",
        city.budget().treasury
    );
    assert_eq!(city.clock().day, 1, "day should be reset to 1");

    // Verify simulation can still tick without panics after teardown.
    city.tick(10);

    // Still clean: no citizens spontaneously appear without zones/buildings.
    assert_eq!(
        city.citizen_count(),
        0,
        "should still have 0 citizens after ticking on clean slate"
    );
}

/// Test that citizens do not lose state when transitioning through LOD tiers
/// (Full -> Abstract -> Full). The LOD system preserves all components and
/// only adds/removes a CompressedCitizen marker.
#[test]
fn test_lod_roundtrip_no_state_loss() {
    use crate::citizen::{
        Citizen, CitizenDetails, CitizenStateComp, Family, Gender, HomeLocation, Needs, PathCache,
        Personality, Position, Velocity, WorkLocation,
    };
    use crate::lod::{CompressedCitizen, LodTier};
    use crate::movement::ActivityTimer;

    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    // Spawn a citizen with known values AND a LodTier::Full component.
    let world = city.world_mut();
    let home_entity = world
        .resource::<WorldGrid>()
        .get(12, 11)
        .building_id
        .unwrap();
    let work_entity = world
        .resource::<WorldGrid>()
        .get(18, 11)
        .building_id
        .unwrap();

    let citizen_entity = world
        .spawn((
            Citizen,
            LodTier::Full,
            Position { x: 200.0, y: 180.0 },
            Velocity { x: 0.5, y: -0.3 },
            HomeLocation {
                grid_x: 12,
                grid_y: 11,
                building: home_entity,
            },
            WorkLocation {
                grid_x: 18,
                grid_y: 11,
                building: work_entity,
            },
            CitizenStateComp(CitizenState::Working),
            PathCache::new(vec![crate::roads::RoadNode(15, 10)]),
            CitizenDetails {
                age: 35,
                gender: Gender::Male,
                education: 2,
                happiness: 68.0,
                health: 92.0,
                salary: 4500.0,
                savings: 12000.0,
            },
            Personality {
                ambition: 0.7,
                sociability: 0.4,
                materialism: 0.55,
                resilience: 0.8,
            },
            Needs {
                hunger: 60.0,
                energy: 75.0,
                social: 50.0,
                fun: 45.0,
                comfort: 70.0,
            },
            Family::default(),
            ActivityTimer(99),
        ))
        .id();

    // Simulate Full -> Abstract transition: change LodTier and insert
    // CompressedCitizen manually (the compress_abstract_citizens system runs
    // in Update, which TestCity::tick() does not execute).
    let world = city.world_mut();
    {
        let state = world.get::<CitizenStateComp>(citizen_entity).unwrap().0;
        let details = world.get::<CitizenDetails>(citizen_entity).unwrap();
        let home = world.get::<HomeLocation>(citizen_entity).unwrap();
        let compressed = CompressedCitizen::new(
            home.grid_x as u8,
            home.grid_y as u8,
            state,
            details.age,
            details.happiness as u8,
            0,
            0,
        );
        world
            .entity_mut(citizen_entity)
            .insert((LodTier::Abstract, compressed));
    }

    // Verify citizen got CompressedCitizen marker.
    assert!(
        world.get::<CompressedCitizen>(citizen_entity).is_some(),
        "citizen should have CompressedCitizen component in Abstract tier"
    );

    // Verify all original components are still intact while in Abstract tier.
    {
        let details = world.get::<CitizenDetails>(citizen_entity).unwrap();
        assert_eq!(details.age, 35, "age should be preserved in Abstract tier");
        assert!(
            (details.salary - 4500.0).abs() < f32::EPSILON,
            "salary should be preserved in Abstract tier"
        );

        let personality = world.get::<Personality>(citizen_entity).unwrap();
        assert!(
            (personality.ambition - 0.7).abs() < f32::EPSILON,
            "ambition should be preserved in Abstract tier"
        );

        let needs = world.get::<Needs>(citizen_entity).unwrap();
        assert!(
            (needs.hunger - 60.0).abs() < f32::EPSILON,
            "hunger should be preserved in Abstract tier: {}",
            needs.hunger
        );
    }

    // Simulate Abstract -> Full transition: change LodTier and remove
    // CompressedCitizen (what decompress_active_citizens does).
    world
        .entity_mut(citizen_entity)
        .insert(LodTier::Full)
        .remove::<CompressedCitizen>();

    // Verify CompressedCitizen marker was removed.
    assert!(
        world.get::<CompressedCitizen>(citizen_entity).is_none(),
        "citizen should NOT have CompressedCitizen after returning to Full tier"
    );

    // Verify core components survived the Full -> Abstract -> Full roundtrip.
    let details = world.get::<CitizenDetails>(citizen_entity).unwrap();
    assert_eq!(details.age, 35, "age lost in LOD roundtrip");
    assert!(
        matches!(details.gender, Gender::Male),
        "gender lost in LOD roundtrip"
    );
    assert_eq!(details.education, 2, "education lost in LOD roundtrip");
    assert!(
        (details.salary - 4500.0).abs() < f32::EPSILON,
        "salary lost in LOD roundtrip: {}",
        details.salary
    );
    assert!(
        (details.savings - 12000.0).abs() < f32::EPSILON,
        "savings lost in LOD roundtrip: {}",
        details.savings
    );

    let personality = world.get::<Personality>(citizen_entity).unwrap();
    assert!(
        (personality.ambition - 0.7).abs() < f32::EPSILON,
        "ambition lost in LOD roundtrip"
    );
    assert!(
        (personality.sociability - 0.4).abs() < f32::EPSILON,
        "sociability lost in LOD roundtrip"
    );
    assert!(
        (personality.materialism - 0.55).abs() < f32::EPSILON,
        "materialism lost in LOD roundtrip"
    );
    assert!(
        (personality.resilience - 0.8).abs() < f32::EPSILON,
        "resilience lost in LOD roundtrip"
    );

    // Needs should be exactly preserved since no ticks were run.
    let needs = world.get::<Needs>(citizen_entity).unwrap();
    assert!(
        (needs.hunger - 60.0).abs() < f32::EPSILON,
        "hunger lost in LOD roundtrip: {}",
        needs.hunger
    );
    assert!(
        (needs.energy - 75.0).abs() < f32::EPSILON,
        "energy lost in LOD roundtrip: {}",
        needs.energy
    );
    assert!(
        (needs.social - 50.0).abs() < f32::EPSILON,
        "social lost in LOD roundtrip: {}",
        needs.social
    );
    assert!(
        (needs.fun - 45.0).abs() < f32::EPSILON,
        "fun lost in LOD roundtrip: {}",
        needs.fun
    );
    assert!(
        (needs.comfort - 70.0).abs() < f32::EPSILON,
        "comfort lost in LOD roundtrip: {}",
        needs.comfort
    );

    // Verify home/work locations survived.
    let home = world.get::<HomeLocation>(citizen_entity).unwrap();
    assert_eq!(home.grid_x, 12, "home_x lost in LOD roundtrip");
    assert_eq!(home.grid_y, 11, "home_y lost in LOD roundtrip");

    let work = world.get::<WorkLocation>(citizen_entity).unwrap();
    assert_eq!(work.grid_x, 18, "work_x lost in LOD roundtrip");
    assert_eq!(work.grid_y, 11, "work_y lost in LOD roundtrip");
}

/// Test that the SaveableRegistry correctly roundtrips extension data through
/// save_all / load_all, and that reset_all restores defaults.
#[test]
fn test_extension_map_roundtrip_via_registry() {
    use crate::SaveableRegistry;

    // Create a minimal Bevy app with a SaveableRegistry.
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    // Define a test resource that implements Saveable.
    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct TestExtensionResource {
        value_a: u32,
        value_b: String,
    }

    impl crate::Saveable for TestExtensionResource {
        const SAVE_KEY: &'static str = "test_extension_res";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            // Use serde_json for a simple, human-debuggable encoding.
            serde_json::to_vec(&(self.value_a, &self.value_b)).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (a, b): (u32, String) = serde_json::from_slice(bytes).unwrap_or_default();
            Self {
                value_a: a,
                value_b: b,
            }
        }
    }

    // Register the test resource.
    app.init_resource::<TestExtensionResource>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<TestExtensionResource>();
    }

    // Set custom values.
    {
        let mut res = app.world_mut().resource_mut::<TestExtensionResource>();
        res.value_a = 42;
        res.value_b = "roundtrip_test".to_string();
    }

    // Save extensions via registry.
    let extensions = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };

    // Verify extension was saved under the correct key.
    assert!(
        extensions.contains_key("test_extension_res"),
        "extension should be saved under key 'test_extension_res'"
    );
    assert_eq!(
        extensions.len(),
        1,
        "should have exactly one extension entry"
    );

    // Verify the bytes are non-empty.
    let saved_bytes = extensions.get("test_extension_res").unwrap();
    assert!(
        !saved_bytes.is_empty(),
        "saved extension bytes should be non-empty"
    );

    // Reset the resource to default (simulating fresh world before load).
    app.world_mut()
        .insert_resource(TestExtensionResource::default());
    {
        let res = app.world().resource::<TestExtensionResource>();
        assert_eq!(res.value_a, 0, "resource should be at default before load");
        assert!(
            res.value_b.is_empty(),
            "resource should be at default before load"
        );
    }

    // Load extensions back via registry.
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    // Verify restored values match what was saved.
    let res = app.world().resource::<TestExtensionResource>();
    assert_eq!(
        res.value_a, 42,
        "value_a should be restored from extension map"
    );
    assert_eq!(
        res.value_b, "roundtrip_test",
        "value_b should be restored from extension map"
    );

    // Test reset_all (simulating new game teardown).
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.reset_all(app.world_mut());
        app.world_mut().insert_resource(registry);
    }
    let res = app.world().resource::<TestExtensionResource>();
    assert_eq!(
        res.value_a, 0,
        "value_a should be reset to default after reset_all"
    );
    assert!(
        res.value_b.is_empty(),
        "value_b should be empty after reset_all"
    );
}
