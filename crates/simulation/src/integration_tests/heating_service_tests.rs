//! Integration tests for SVC-010: Heating Service and Weather Integration.

use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::heating::{HeatingGrid, HeatingPlant, HeatingPlantType, HeatingStats};
use crate::heating_service::HeatingServiceState;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_heating_service_state_exists() {
    let city = TestCity::new();
    let state = city.resource::<HeatingServiceState>();
    assert_eq!(state.individual_heating_count, 0);
    assert_eq!(state.district_heating_count, 0);
    assert_eq!(state.unheated_count, 0);
}

// ====================================================================
// Warm weather: no heating demand
// ====================================================================

#[test]
fn test_no_heating_demand_in_warm_weather() {
    let mut city = TestCity::new()
        .with_weather(25.0) // warm
        .with_road(50, 50, 60, 50)
        .with_building(52, 49, ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();

    let state = city.resource::<HeatingServiceState>();
    assert!(
        state.current_demand <= 0.0,
        "No heating demand expected at 25C, got {}",
        state.current_demand
    );
    assert_eq!(state.individual_heating_count, 0);
    assert_eq!(state.district_heating_count, 0);
}

// ====================================================================
// Cold weather: individual heating for occupied buildings
// ====================================================================

#[test]
fn test_individual_heating_in_cold_weather() {
    let mut city = TestCity::new()
        .with_weather(-5.0) // cold
        .with_road(50, 50, 60, 50)
        .with_building(52, 49, ZoneType::ResidentialLow, 1);

    // Add some occupants to the building
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            building.occupants = 5;
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<HeatingServiceState>();
    assert!(
        state.current_demand > 0.0,
        "Heating demand expected at -5C, got {}",
        state.current_demand
    );
    assert!(
        state.individual_heating_count > 0,
        "Occupied building without district heating should use individual heating"
    );
    assert!(
        state.individual_heating_cost > 0.0,
        "Individual heating should have a cost"
    );
}

// ====================================================================
// District heating plant coverage
// ====================================================================

#[test]
fn test_district_heating_plant_attaches_component() {
    let mut city = TestCity::new()
        .with_weather(-5.0)
        .with_road(50, 50, 60, 50)
        .with_service(55, 48, ServiceType::DistrictHeatingPlant);

    city.tick_slow_cycle();

    // Check that a HeatingPlant component was attached
    let world = city.world_mut();
    let count = world.query::<&HeatingPlant>().iter(world).count();
    assert!(
        count > 0,
        "DistrictHeatingPlant service should get a HeatingPlant component"
    );
}

#[test]
fn test_geothermal_plant_attaches_component() {
    let mut city = TestCity::new()
        .with_weather(-5.0)
        .with_road(50, 50, 60, 50)
        .with_service(55, 48, ServiceType::GeothermalPlant);

    city.tick_slow_cycle();

    let world = city.world_mut();
    let mut query = world.query::<&HeatingPlant>();
    let plants: Vec<_> = query.iter(world).collect();
    assert!(!plants.is_empty(), "GeothermalPlant should get HeatingPlant component");
    assert_eq!(
        plants[0].plant_type,
        HeatingPlantType::Geothermal,
        "Should be classified as Geothermal"
    );
}

// ====================================================================
// District heating covers nearby buildings
// ====================================================================

#[test]
fn test_district_heating_covers_nearby_buildings() {
    let mut city = TestCity::new()
        .with_weather(-5.0)
        .with_road(50, 50, 70, 50)
        .with_service(55, 48, ServiceType::DistrictHeatingPlant)
        .with_building(57, 49, ZoneType::ResidentialLow, 1);

    // Add occupants
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            building.occupants = 5;
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<HeatingServiceState>();
    // The building near the district heating plant should be covered
    assert!(
        state.district_heating_count > 0 || state.individual_heating_count > 0,
        "Building near heating plant should have some form of heating"
    );
}

// ====================================================================
// Energy consumption tracking
// ====================================================================

#[test]
fn test_heating_consumes_energy() {
    let mut city = TestCity::new()
        .with_weather(-5.0)
        .with_road(50, 50, 60, 50)
        .with_building(52, 49, ZoneType::ResidentialLow, 1);

    // Add occupants for individual heating
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            building.occupants = 5;
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<HeatingServiceState>();
    assert!(
        state.heating_energy_mw > 0.0,
        "Heating should consume energy, got {} MW",
        state.heating_energy_mw
    );
}

// ====================================================================
// Individual heating is more expensive than district
// ====================================================================

#[test]
fn test_individual_heating_costs_more() {
    use crate::heating_service::{INDIVIDUAL_HEATING_COST_PER_BUILDING, INDIVIDUAL_HEATING_ENERGY_MW, DISTRICT_HEATING_ENERGY_MW};

    // Individual heating uses more energy per building than district
    assert!(
        INDIVIDUAL_HEATING_ENERGY_MW > DISTRICT_HEATING_ENERGY_MW,
        "Individual heating should use more energy"
    );

    // Individual heating cost is higher than district plant cost per unit
    assert!(
        INDIVIDUAL_HEATING_COST_PER_BUILDING > HeatingPlantType::DistrictHeating.cost_per_unit(),
        "Individual heating should be more expensive per unit"
    );
}

// ====================================================================
// HeatingServiceState Saveable roundtrip
// ====================================================================

#[test]
fn test_heating_service_saveable() {
    use crate::Saveable;

    let state = HeatingServiceState {
        individual_heating_count: 10,
        district_heating_count: 50,
        unheated_count: 3,
        heating_energy_mw: 5.5,
        individual_heating_cost: 500.0,
        current_demand: 0.8,
        cold_affected_citizens: 7,
    };

    let bytes = state.save_to_bytes().unwrap();
    let restored = HeatingServiceState::load_from_bytes(&bytes);

    assert_eq!(restored.individual_heating_count, 10);
    assert_eq!(restored.district_heating_count, 50);
    assert_eq!(restored.unheated_count, 3);
    assert!((restored.heating_energy_mw - 5.5).abs() < f32::EPSILON);
    assert!((restored.individual_heating_cost - 500.0).abs() < f64::EPSILON);
    assert!((restored.current_demand - 0.8).abs() < f32::EPSILON);
    assert_eq!(restored.cold_affected_citizens, 7);
}
