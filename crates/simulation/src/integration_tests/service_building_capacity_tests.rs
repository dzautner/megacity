//! Integration tests for SVC-002: Service Building Capacity Limits
//!
//! Tests staffing requirements, capacity scaling based on staffing, and
//! overcrowding penalties when demand exceeds capacity.


use crate::grid::ZoneType;
use crate::service_building_capacity::{
    tier_capacity, ServiceBuildingCapacityState, ServiceBuildingStaffing,
};
use crate::service_capacity::ServiceCapacity;
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;

/// Tick enough for staffing assignment (runs every 20 ticks).
fn tick_staffing(city: &mut TestCity) {
    city.tick(21);
}

// ====================================================================
// 1. Staffing component attached automatically
// ====================================================================

#[test]
fn test_staffing_attached_to_service_building() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    tick_staffing(&mut city);

    let world = city.world_mut();
    let count = world
        .query::<(&ServiceBuilding, &ServiceBuildingStaffing)>()
        .iter(world)
        .count();

    assert_eq!(count, 1, "Hospital should have ServiceBuildingStaffing");
}

#[test]
fn test_staffing_has_correct_requirements() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    tick_staffing(&mut city);

    let world = city.world_mut();
    let staffing: Vec<(u32, u32)> = world
        .query::<(&ServiceBuilding, &ServiceBuildingStaffing)>()
        .iter(world)
        .filter(|(s, _)| s.service_type == ServiceType::Hospital)
        .map(|(_, s)| (s.staff_required, s.max_capacity))
        .collect();

    assert_eq!(staffing.len(), 1);
    assert_eq!(staffing[0].0, 40, "Hospital requires 40 staff");
    assert_eq!(staffing[0].1, 200, "Hospital max capacity is 200 beds");
}

// ====================================================================
// 2. Hospital at 100% capacity provides full quality
// ====================================================================

#[test]
fn test_hospital_at_full_capacity_provides_full_quality() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Hospital)
        .with_building(130, 128, ZoneType::ResidentialLow, 1);

    // Spawn enough employed citizens to fully staff the hospital.
    // Hospital needs 40 staff. We need employed citizens with salary > 0.
    for i in 0..60 {
        let x = 125 + (i % 5);
        let y = 125 + (i / 5);
        city = city
            .with_building(x, y, ZoneType::CommercialLow, 1)
            .with_citizen((130, 128), (x, y));
    }

    tick_staffing(&mut city);

    let world = city.world_mut();
    let caps: Vec<(u32, u32, f32)> = world
        .query::<(&ServiceBuilding, &ServiceCapacity, &ServiceBuildingStaffing)>()
        .iter(world)
        .filter(|(s, _, _)| s.service_type == ServiceType::Hospital)
        .map(|(_, c, _st)| (c.capacity, c.current_usage, c.effectiveness()))
        .collect();

    assert_eq!(caps.len(), 1);
    // When fully staffed, capacity should be the max (200)
    // and with usage <= capacity, effectiveness should be 1.0
    let (cap, usage, eff) = caps[0];
    assert!(
        cap > 0,
        "Fully staffed hospital should have positive capacity"
    );
    if usage <= cap {
        assert!(
            (eff - 1.0).abs() < f32::EPSILON,
            "Hospital at or under capacity should have full effectiveness, got {}",
            eff
        );
    }
}

// ====================================================================
// 3. Hospital at 200% demand provides 50% quality
// ====================================================================

#[test]
fn test_hospital_overcrowded_reduces_quality() {
    // Verify the math: when current_usage = 2 * capacity, effectiveness = 0.5
    let cap = ServiceCapacity {
        capacity: 200,
        current_usage: 400,
    };
    assert!(
        (cap.effectiveness() - 0.5).abs() < f32::EPSILON,
        "200% demand should give 50% quality"
    );
}

// ====================================================================
// 4. Unstaffed school provides 0 education coverage
// ====================================================================

#[test]
fn test_unstaffed_building_provides_zero_service() {
    // A city with a school but no employed citizens -> unstaffed -> capacity = 0
    let mut city = TestCity::new().with_service(128, 128, ServiceType::ElementarySchool);
    tick_staffing(&mut city);

    let world = city.world_mut();
    let caps: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceCapacity, &ServiceBuildingStaffing)>()
        .iter(world)
        .filter(|(s, _, _)| s.service_type == ServiceType::ElementarySchool)
        .map(|(_, c, _)| c.capacity)
        .collect();

    assert_eq!(caps.len(), 1);
    assert_eq!(
        caps[0], 0,
        "Unstaffed school should have 0 effective capacity"
    );
}

// ====================================================================
// 5. Partially staffed building has reduced capacity
// ====================================================================

#[test]
fn test_partially_staffed_reduces_capacity() {
    let staffing = ServiceBuildingStaffing {
        staff_required: 40,
        staff_assigned: 20,
        max_capacity: 200,
    };
    assert_eq!(
        staffing.effective_capacity(),
        100,
        "Half-staffed hospital should have 100 bed capacity"
    );
}

// ====================================================================
// 6. Tier capacities match spec
// ====================================================================

#[test]
fn test_hospital_tier_capacities() {
    assert_eq!(tier_capacity(ServiceType::MedicalClinic), 50);
    assert_eq!(tier_capacity(ServiceType::Hospital), 200);
    assert_eq!(tier_capacity(ServiceType::MedicalCenter), 500);
}

#[test]
fn test_school_tier_capacities() {
    assert_eq!(tier_capacity(ServiceType::ElementarySchool), 300);
    assert_eq!(tier_capacity(ServiceType::HighSchool), 1500);
    assert_eq!(tier_capacity(ServiceType::University), 5000);
}

#[test]
fn test_fire_tier_capacities() {
    assert_eq!(tier_capacity(ServiceType::FireHouse), 2);
    assert_eq!(tier_capacity(ServiceType::FireStation), 5);
    assert_eq!(tier_capacity(ServiceType::FireHQ), 10);
}

#[test]
fn test_police_tier_capacities() {
    assert_eq!(tier_capacity(ServiceType::PoliceKiosk), 10);
    assert_eq!(tier_capacity(ServiceType::PoliceStation), 30);
    assert_eq!(tier_capacity(ServiceType::PoliceHQ), 100);
}

// ====================================================================
// 7. Staffing stats resource populated
// ====================================================================

#[test]
fn test_staffing_stats_populated() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Hospital)
        .with_service(128, 130, ServiceType::FireStation);
    tick_staffing(&mut city);

    let state = city.resource::<ServiceBuildingCapacityState>();
    assert!(
        !state.categories.is_empty(),
        "Staffing stats should be populated"
    );

    let health = state.categories.iter().find(|c| c.category == "Health");
    assert!(health.is_some(), "Should have Health category");
    let health = health.unwrap();
    assert_eq!(
        health.total_staff_required, 40,
        "Hospital needs 40 staff"
    );
    assert_eq!(
        health.total_max_capacity, 200,
        "Hospital max capacity is 200"
    );
}

// ====================================================================
// 8. Staff drawn from employed citizens
// ====================================================================

#[test]
fn test_staff_assigned_from_employed_citizens() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);

    // Spawn employed citizens (with salary > 0 via with_citizen)
    for i in 0..50 {
        let x = 120 + (i % 10);
        let y = 120 + (i / 10);
        city = city
            .with_building(x, y, ZoneType::CommercialLow, 1)
            .with_citizen((x, y), (x, y));
    }

    tick_staffing(&mut city);

    let world = city.world_mut();
    let assigned: Vec<u32> = world
        .query::<(&ServiceBuilding, &ServiceBuildingStaffing)>()
        .iter(world)
        .filter(|(s, _)| s.service_type == ServiceType::Hospital)
        .map(|(_, s)| s.staff_assigned)
        .collect();

    assert_eq!(assigned.len(), 1);
    assert!(
        assigned[0] > 0,
        "Hospital should have some staff assigned when employed citizens exist"
    );
}

// ====================================================================
// 9. Multiple buildings share staff proportionally
// ====================================================================

#[test]
fn test_staff_distributed_proportionally() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Hospital)       // requires 40
        .with_service(128, 140, ServiceType::ElementarySchool); // requires 20

    // Spawn 30 employed citizens (not enough for both)
    for i in 0..30 {
        let x = 120 + (i % 10);
        let y = 120 + (i / 10);
        city = city
            .with_building(x, y, ZoneType::CommercialLow, 1)
            .with_citizen((x, y), (x, y));
    }

    tick_staffing(&mut city);

    let world = city.world_mut();
    let staffing: Vec<(ServiceType, u32, u32)> = world
        .query::<(&ServiceBuilding, &ServiceBuildingStaffing)>()
        .iter(world)
        .map(|(s, st)| (s.service_type, st.staff_required, st.staff_assigned))
        .collect();

    let hospital = staffing.iter().find(|(st, _, _)| *st == ServiceType::Hospital);
    let school = staffing.iter().find(|(st, _, _)| *st == ServiceType::ElementarySchool);

    assert!(hospital.is_some());
    assert!(school.is_some());

    let (_, _, h_assigned) = hospital.unwrap();
    let (_, _, s_assigned) = school.unwrap();

    // Hospital requires 40 (2/3 of total 60), school requires 20 (1/3)
    // With 30 employed, hospital should get ~20, school ~10
    assert!(
        *h_assigned > *s_assigned,
        "Hospital (req=40) should get more staff than school (req=20), got h={} s={}",
        h_assigned,
        s_assigned
    );
}

// ====================================================================
// 10. Dynamically spawned service gets staffing
// ====================================================================

#[test]
fn test_dynamically_spawned_service_gets_staffing() {
    let mut city = TestCity::new();
    tick_staffing(&mut city);

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

    tick_staffing(&mut city);

    let world = city.world_mut();
    let staffing: Vec<(u32, u32)> = world
        .query::<(&ServiceBuilding, &ServiceBuildingStaffing)>()
        .iter(world)
        .map(|(_, s)| (s.staff_required, s.max_capacity))
        .collect();

    assert_eq!(staffing.len(), 1);
    assert_eq!(staffing[0].0, 20, "Police station requires 20 staff");
    assert_eq!(staffing[0].1, 30, "Police station max capacity is 30 officers");
}
