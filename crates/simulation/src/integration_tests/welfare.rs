use crate::grid::ZoneType;
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;

// ====================================================================
// Welfare system tests (issue #851)
// ====================================================================

#[test]
fn test_welfare_stats_resource_exists_in_new_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::welfare::WelfareStats>();
}

#[test]
fn test_welfare_stats_default_values_on_empty_city() {
    let city = TestCity::new();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.total_sheltered, 0);
    assert_eq!(stats.total_welfare_recipients, 0);
    assert_eq!(stats.monthly_cost, 0.0);
    assert_eq!(stats.shelter_capacity, 0);
    assert_eq!(stats.shelter_occupancy, 0);
    assert_eq!(stats.welfare_office_count, 0);
    assert_eq!(stats.shelter_count, 0);
}

#[test]
fn test_welfare_office_tracks_count_after_slow_tick() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::WelfareOffice)
        .with_service(60, 60, ServiceType::WelfareOffice);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.welfare_office_count, 2);
}

#[test]
fn test_welfare_shelter_tracks_count_after_slow_tick() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::HomelessShelter)
        .with_service(60, 60, ServiceType::HomelessShelter);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.shelter_count, 2);
}

#[test]
fn test_welfare_shelter_capacity_from_service_buildings() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::HomelessShelter)
        .with_service(60, 60, ServiceType::HomelessShelter);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    // Each shelter has 50 bed capacity
    assert_eq!(stats.shelter_capacity, 100);
}

#[test]
fn test_welfare_monthly_cost_for_offices_and_shelters() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::WelfareOffice)
        .with_service(60, 60, ServiceType::HomelessShelter);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    let expected = ServiceBuilding::monthly_maintenance(ServiceType::WelfareOffice)
        + ServiceBuilding::monthly_maintenance(ServiceType::HomelessShelter);
    assert!(
        (stats.monthly_cost - expected).abs() < 0.01,
        "monthly cost should match sum of maintenance: got {}, expected {}",
        stats.monthly_cost,
        expected
    );
}

#[test]
fn test_welfare_monthly_cost_zero_with_no_services() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();
    assert_eq!(
        city.resource::<crate::welfare::WelfareStats>().monthly_cost,
        0.0
    );
}

#[test]
fn test_welfare_no_recipients_without_offices() {
    let mut city = TestCity::new()
        .with_building(30, 30, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen((30, 30));
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.total_welfare_recipients, 0);
    assert_eq!(stats.welfare_office_count, 0);
}

#[test]
fn test_welfare_office_near_unemployed_citizen() {
    let mut city = TestCity::new()
        .with_building(30, 30, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen((30, 30))
        .with_service(31, 31, ServiceType::WelfareOffice);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.welfare_office_count, 1);
}

#[test]
fn test_welfare_multiple_offices_counts_recipients() {
    let mut city = TestCity::new()
        .with_building(30, 30, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen((30, 30))
        .with_service(31, 31, ServiceType::WelfareOffice)
        .with_service(32, 32, ServiceType::WelfareOffice);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.welfare_office_count, 2);
    assert!(stats.total_welfare_recipients <= 1);
}

#[test]
fn test_welfare_expense_scales_with_building_count() {
    let mut small = TestCity::new().with_service(30, 30, ServiceType::WelfareOffice);
    small.tick_slow_cycle();
    let cost_small = small
        .resource::<crate::welfare::WelfareStats>()
        .monthly_cost;

    let mut large = TestCity::new()
        .with_service(30, 30, ServiceType::WelfareOffice)
        .with_service(60, 60, ServiceType::WelfareOffice)
        .with_service(90, 90, ServiceType::WelfareOffice)
        .with_service(30, 90, ServiceType::HomelessShelter)
        .with_service(90, 30, ServiceType::HomelessShelter);
    large.tick_slow_cycle();
    let cost_large = large
        .resource::<crate::welfare::WelfareStats>()
        .monthly_cost;
    assert!(
        cost_large > cost_small,
        "more buildings should cost more: small={cost_small}, large={cost_large}"
    );
    let expected = 3.0 * ServiceBuilding::monthly_maintenance(ServiceType::WelfareOffice)
        + 2.0 * ServiceBuilding::monthly_maintenance(ServiceType::HomelessShelter);
    assert!((cost_large - expected).abs() < 0.01);
}

#[test]
fn test_welfare_stats_reset_between_slow_ticks() {
    let mut city = TestCity::new()
        .with_building(30, 30, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen((30, 30))
        .with_service(31, 31, ServiceType::WelfareOffice);
    city.tick_slow_cycle();
    let first = city
        .resource::<crate::welfare::WelfareStats>()
        .total_welfare_recipients;
    city.tick_slow_cycle();
    let second = city
        .resource::<crate::welfare::WelfareStats>()
        .total_welfare_recipients;
    assert!(
        second <= first,
        "recipients should not accumulate: first={first}, second={second}"
    );
}
