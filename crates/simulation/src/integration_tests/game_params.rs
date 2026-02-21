use crate::buildings::Building;
use crate::grid::{RoadType, ZoneType};
use crate::test_harness::TestCity;

// ====================================================================
// GameParams data-driven parameters tests
// ====================================================================

#[test]
fn test_game_params_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::game_params::GameParams>();
}

#[test]
fn test_game_params_defaults_match_original_constants() {
    let city = TestCity::new();
    let params = city.resource::<crate::game_params::GameParams>();

    assert!(
        (params.economy.starting_treasury - 10_000.0).abs() < f64::EPSILON,
        "starting_treasury should be 10000"
    );
    assert!(
        (params.economy.default_tax_rate - 0.10).abs() < f32::EPSILON,
        "default_tax_rate should be 0.10"
    );
    assert_eq!(params.economy.tax_collection_interval_days, 30);
    assert!((params.citizen.speed - 48.0).abs() < f32::EPSILON);
    assert_eq!(params.citizen.shopping_duration_ticks, 30);
    assert_eq!(params.citizen.leisure_duration_ticks, 60);
    assert_eq!(params.building.construction_ticks, 100);
    assert_eq!(params.building.spawn_interval_ticks, 2);

    let local = params.road_params(RoadType::Local);
    assert!((local.speed - 30.0).abs() < f32::EPSILON);
    assert!((local.cost - 10.0).abs() < f64::EPSILON);
    assert_eq!(local.capacity, 20);
}

#[test]
fn test_game_params_saveable_roundtrip() {
    use crate::game_params::GameParams;
    use crate::Saveable;

    let mut params = GameParams::default();
    params.economy.starting_treasury = 99_999.0;
    params.citizen.speed = 200.0;

    let bytes = params.save_to_bytes().expect("should encode");
    let restored = GameParams::load_from_bytes(&bytes);

    assert!((restored.economy.starting_treasury - 99_999.0).abs() < f64::EPSILON);
    assert!((restored.citizen.speed - 200.0).abs() < f32::EPSILON);
}

#[test]
fn test_game_params_modifying_construction_ticks() {
    use crate::buildings::UnderConstruction;
    use crate::game_params::GameParams;

    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_utility(5, 5, crate::utilities::UtilityType::PowerPlant)
        .with_utility(6, 6, crate::utilities::UtilityType::WaterTower);

    {
        let world = city.world_mut();
        let mut params = world.resource_mut::<GameParams>();
        params.building.construction_ticks = 2;
    }

    let mut city = city
        .with_zone_rect(11, 9, 19, 9, ZoneType::ResidentialLow)
        .with_zone_rect(11, 11, 19, 11, ZoneType::ResidentialLow);

    city.tick(20);

    let world = city.world_mut();
    let still_constructing: usize = world.query::<&UnderConstruction>().iter(world).count();
    let buildings: usize = world.query::<&Building>().iter(world).count();

    if buildings > 0 {
        assert!(
            still_constructing < buildings,
            "With 2-tick construction, most of {} buildings should be done, but {} still constructing",
            buildings,
            still_constructing
        );
    }
}

#[test]
fn test_game_params_zone_demand_bootstrap() {
    use crate::game_params::GameParams;
    use crate::zones::ZoneDemand;

    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);

    {
        let world = city.world_mut();
        let mut params = world.resource_mut::<GameParams>();
        params.zone_demand.bootstrap_demand = 0.9;
    }

    city.tick_slow_cycle();

    let demand = city.resource::<ZoneDemand>();
    assert!(
        demand.residential > 0.0,
        "With bootstrap_demand=0.9 and roads, residential demand should be positive, got {}",
        demand.residential
    );
}

#[test]
fn test_game_params_road_params_lookup() {
    use crate::game_params::GameParams;

    let city = TestCity::new();
    let params = city.resource::<GameParams>();

    let road_types = [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
        RoadType::Path,
    ];
    for rt in road_types {
        let rp = params.road_params(rt);
        assert!(rp.speed > 0.0, "{:?} should have positive speed", rt);
        assert!(rp.cost > 0.0, "{:?} should have positive cost", rt);
    }
}
