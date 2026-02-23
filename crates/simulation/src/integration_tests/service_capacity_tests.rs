//! Integration tests for SERV-001: Service Capacity Limits
//!
//! Tests that service buildings have correct default capacities, that utilization
//! increases as nearby buildings gain occupants, and that over-capacity reduces
//! effectiveness gracefully.

use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::service_capacity::{default_capacity, ServiceCapacity, ServiceCapacityStats};
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;

// ====================================================================
// Helper: tick enough to attach capacities and update usage
// ====================================================================

fn tick_capacity(city: &mut TestCity) {
    // 10 ticks to trigger the update_service_usage system
    city.tick(10);
}

// ====================================================================
// 1. Default capacity values
// ====================================================================

#[test]
fn test_hospital_has_correct_default_capacity() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    tick_capacity(&mut city);

    let world = city.world_mut();
    let caps: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .filter(|(s, _)| s.service_type == ServiceType::Hospital)
        .map(|(_, c)| c.capacity)
        .collect();

    assert_eq!(caps.len(), 1, "Should have exactly one hospital");
    assert_eq!(caps[0], 200, "Hospital default capacity should be 200");
}

#[test]
fn test_elementary_school_has_correct_default_capacity() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::ElementarySchool);
    tick_capacity(&mut city);

    let world = city.world_mut();
    let caps: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .filter(|(s, _)| s.service_type == ServiceType::ElementarySchool)
        .map(|(_, c)| c.capacity)
        .collect();

    assert_eq!(caps[0], 300);
}

#[test]
fn test_high_school_has_correct_default_capacity() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::HighSchool);
    tick_capacity(&mut city);

    let world = city.world_mut();
    let caps: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .filter(|(s, _)| s.service_type == ServiceType::HighSchool)
        .map(|(_, c)| c.capacity)
        .collect();

    assert_eq!(caps[0], 600);
}

#[test]
fn test_university_has_correct_default_capacity() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::University);
    tick_capacity(&mut city);

    let world = city.world_mut();
    let caps: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .filter(|(s, _)| s.service_type == ServiceType::University)
        .map(|(_, c)| c.capacity)
        .collect();

    assert_eq!(caps[0], 2000);
}

#[test]
fn test_fire_station_has_correct_default_capacity() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::FireStation);
    tick_capacity(&mut city);

    let world = city.world_mut();
    let caps: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .filter(|(s, _)| s.service_type == ServiceType::FireStation)
        .map(|(_, c)| c.capacity)
        .collect();

    assert_eq!(caps[0], 500);
}

#[test]
fn test_police_station_has_correct_default_capacity() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::PoliceStation);
    tick_capacity(&mut city);

    let world = city.world_mut();
    let caps: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .filter(|(s, _)| s.service_type == ServiceType::PoliceStation)
        .map(|(_, c)| c.capacity)
        .collect();

    assert_eq!(caps[0], 500);
}

// ====================================================================
// 2. All service types covered by default_capacity
// ====================================================================

#[test]
fn test_all_service_types_have_nonzero_capacity() {
    let types = [
        ServiceType::Hospital,
        ServiceType::MedicalClinic,
        ServiceType::MedicalCenter,
        ServiceType::Kindergarten,
        ServiceType::ElementarySchool,
        ServiceType::HighSchool,
        ServiceType::University,
        ServiceType::Library,
        ServiceType::FireStation,
        ServiceType::FireHouse,
        ServiceType::FireHQ,
        ServiceType::PoliceStation,
        ServiceType::PoliceKiosk,
        ServiceType::PoliceHQ,
        ServiceType::Prison,
        ServiceType::SmallPark,
        ServiceType::LargePark,
        ServiceType::Playground,
        ServiceType::Plaza,
        ServiceType::SportsField,
        ServiceType::Stadium,
        ServiceType::Landfill,
        ServiceType::RecyclingCenter,
        ServiceType::Incinerator,
        ServiceType::TransferStation,
        ServiceType::Cemetery,
        ServiceType::Crematorium,
        ServiceType::CityHall,
        ServiceType::Museum,
        ServiceType::Cathedral,
        ServiceType::TVStation,
        ServiceType::BusDepot,
        ServiceType::TrainStation,
        ServiceType::SubwayStation,
        ServiceType::TramDepot,
        ServiceType::FerryPier,
        ServiceType::SmallAirstrip,
        ServiceType::RegionalAirport,
        ServiceType::InternationalAirport,
        ServiceType::CellTower,
        ServiceType::DataCenter,
        ServiceType::HomelessShelter,
        ServiceType::WelfareOffice,
        ServiceType::PostOffice,
        ServiceType::MailSortingCenter,
        ServiceType::WaterTreatmentPlant,
        ServiceType::WellPump,
        ServiceType::HeatingBoiler,
        ServiceType::DistrictHeatingPlant,
        ServiceType::GeothermalPlant,
    ];

    for st in types {
        let cap = default_capacity(st);
        assert!(cap > 0, "{:?} should have non-zero default capacity", st);
    }
}

// ====================================================================
// 3. Capacity component attached automatically
// ====================================================================

#[test]
fn test_capacity_component_attached_to_service_building() {
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::Hospital)
        .with_service(120, 100, ServiceType::FireStation);
    tick_capacity(&mut city);

    let world = city.world_mut();
    let count = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .count();

    assert_eq!(
        count, 2,
        "Both service buildings should have ServiceCapacity"
    );
}

// ====================================================================
// 4. Usage increases with nearby population
// ====================================================================

#[test]
fn test_usage_zero_when_no_buildings_nearby() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    tick_capacity(&mut city);

    let world = city.world_mut();
    let usage: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .map(|(_, c)| c.current_usage)
        .collect();

    assert_eq!(usage[0], 0, "Usage should be 0 with no nearby buildings");
}

#[test]
fn test_usage_increases_with_building_occupants() {
    // Place a hospital and a residential building within its radius
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Hospital)
        .with_building(130, 128, ZoneType::Residential, 1);

    // Set occupants on the building
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            building.occupants = 50;
        }
    }

    tick_capacity(&mut city);

    let world = city.world_mut();
    let usages: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .map(|(_, c)| c.current_usage)
        .collect();

    assert_eq!(
        usages[0], 50,
        "Usage should reflect the building's occupants"
    );
}

#[test]
fn test_usage_sums_multiple_buildings() {
    // Place a hospital and multiple residential buildings within its radius
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Hospital)
        .with_building(130, 128, ZoneType::Residential, 1)
        .with_building(126, 128, ZoneType::Residential, 1);

    // Set occupants on both buildings
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            building.occupants = 30;
        }
    }

    tick_capacity(&mut city);

    let world = city.world_mut();
    let usages: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity)>()
        .iter(world)
        .map(|(_, c)| c.current_usage)
        .collect();

    assert_eq!(
        usages[0], 60,
        "Usage should sum occupants from all nearby buildings (30+30)"
    );
}

// ====================================================================
// 5. Over-capacity reduces effectiveness
// ====================================================================

#[test]
fn test_over_capacity_reduces_effectiveness() {
    let cap = ServiceCapacity {
        capacity: 100,
        current_usage: 200,
    };
    let eff = cap.effectiveness();
    assert!(
        (eff - 0.5).abs() < f32::EPSILON,
        "200% utilization should give 0.5 effectiveness, got {}",
        eff
    );
}

#[test]
fn test_graceful_degradation_not_binary() {
    // Verify that effectiveness degrades smoothly, not as a step function
    let cap_100pct = ServiceCapacity {
        capacity: 100,
        current_usage: 100,
    };
    let cap_150pct = ServiceCapacity {
        capacity: 100,
        current_usage: 150,
    };
    let cap_200pct = ServiceCapacity {
        capacity: 100,
        current_usage: 200,
    };
    let cap_300pct = ServiceCapacity {
        capacity: 100,
        current_usage: 300,
    };

    let eff_100 = cap_100pct.effectiveness();
    let eff_150 = cap_150pct.effectiveness();
    let eff_200 = cap_200pct.effectiveness();
    let eff_300 = cap_300pct.effectiveness();

    assert!(eff_100 > eff_150, "100% should be more effective than 150%");
    assert!(eff_150 > eff_200, "150% should be more effective than 200%");
    assert!(eff_200 > eff_300, "200% should be more effective than 300%");

    // All should be > 0 (never fully disabled)
    assert!(eff_300 > 0.0, "Even 300% should have some effectiveness");
}

// ====================================================================
// 6. Aggregate stats resource
// ====================================================================

#[test]
fn test_capacity_stats_populated() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Hospital)
        .with_service(128, 130, ServiceType::FireStation);
    tick_capacity(&mut city);

    let stats = city.resource::<ServiceCapacityStats>();
    assert!(
        !stats.categories.is_empty(),
        "Stats should have category entries after tick"
    );

    // Find health category
    let health = stats.categories.iter().find(|c| c.category == "Health");
    assert!(health.is_some(), "Should have a Health category");
    assert_eq!(
        health.unwrap().total_capacity,
        200,
        "Health capacity should be 200 (one hospital)"
    );
}

// ====================================================================
// 7. Dynamically spawned service gets capacity
// ====================================================================

#[test]
fn test_dynamically_spawned_service_gets_capacity() {
    let mut city = TestCity::new();
    tick_capacity(&mut city);

    // Verify no capacity components exist yet
    {
        let world = city.world_mut();
        let count = world.query::<&ServiceCapacity>().iter(world).count();
        assert_eq!(count, 0, "No capacity before any service spawned");
    }

    // Spawn a service building dynamically
    {
        let world = city.world_mut();
        world.spawn(ServiceBuilding {
            service_type: ServiceType::PoliceStation,
            grid_x: 128,
            grid_y: 128,
            radius: ServiceBuilding::coverage_radius(ServiceType::PoliceStation),
        });
    }

    tick_capacity(&mut city);

    let world = city.world_mut();
    let caps: Vec<u32> = world
        .query::<&ServiceCapacity>()
        .iter(world)
        .map(|c| c.capacity)
        .collect();

    assert_eq!(caps.len(), 1, "Should have one capacity component");
    assert_eq!(caps[0], 500, "Police station should have capacity 500");
}
