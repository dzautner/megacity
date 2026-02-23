//! SVC-005: Integration tests for Police Service Multi-Tier System.

use crate::crime::CrimeGrid;
use crate::police_tiers::{PoliceTier, PoliceTiersState};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

fn tick_slow(city: &mut TestCity) {
    city.tick_slow_cycles(1);
}

// ====================================================================
// 1. Default / empty state
// ====================================================================

#[test]
fn test_police_tiers_default_in_empty_city() {
    let mut city = TestCity::new();
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert_eq!(s.total_buildings(), 0);
    assert!(!s.coordination_active);
    assert!((s.coordination_multiplier - 1.0).abs() < 0.001);
    assert!((s.city_coverage - 0.0).abs() < 0.001);
}

// ====================================================================
// 2. Building count tracking per tier
// ====================================================================

#[test]
fn test_kiosk_counted_as_kiosk_tier() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::PoliceKiosk)
        .with_service(80, 80, ServiceType::PoliceKiosk);
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert_eq!(s.kiosk_stats.building_count, 2);
    assert_eq!(s.station_stats.building_count, 0);
    assert_eq!(s.hq_stats.building_count, 0);
    assert_eq!(s.total_buildings(), 2);
}

#[test]
fn test_station_counted_as_station_tier() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::PoliceStation);
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert_eq!(s.station_stats.building_count, 1);
}

#[test]
fn test_hq_counted_as_hq_tier() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::PoliceHQ);
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert_eq!(s.hq_stats.building_count, 1);
}

#[test]
fn test_mixed_tiers_counted_separately() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::PoliceKiosk)
        .with_service(60, 60, ServiceType::PoliceStation)
        .with_service(90, 90, ServiceType::PoliceHQ);
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert_eq!(s.kiosk_stats.building_count, 1);
    assert_eq!(s.station_stats.building_count, 1);
    assert_eq!(s.hq_stats.building_count, 1);
    assert_eq!(s.total_buildings(), 3);
}

// ====================================================================
// 3. Coordination bonus
// ====================================================================

#[test]
fn test_coordination_inactive_without_hq() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::PoliceKiosk)
        .with_service(80, 80, ServiceType::PoliceStation);
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert!(!s.coordination_active);
    assert!((s.coordination_multiplier - 1.0).abs() < 0.001);
}

#[test]
fn test_coordination_active_with_hq() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::PoliceKiosk)
        .with_service(80, 80, ServiceType::PoliceHQ);
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert!(s.coordination_active);
    assert!(s.coordination_multiplier > 1.0);
}

// ====================================================================
// 4. Coverage radius (HQ covers more than station, station more than kiosk)
// ====================================================================

#[test]
fn test_hq_covers_more_cells_than_station() {
    // Place each at the same location in separate cities.
    let mut city_hq = TestCity::new().with_service(128, 128, ServiceType::PoliceHQ);
    tick_slow(&mut city_hq);
    let hq_cells = city_hq
        .resource::<PoliceTiersState>()
        .hq_stats
        .cells_covered;

    let mut city_station = TestCity::new().with_service(128, 128, ServiceType::PoliceStation);
    tick_slow(&mut city_station);
    let station_cells = city_station
        .resource::<PoliceTiersState>()
        .station_stats
        .cells_covered;

    assert!(
        hq_cells > station_cells,
        "HQ cells ({hq_cells}) should be > Station cells ({station_cells})"
    );
}

#[test]
fn test_station_covers_more_cells_than_kiosk() {
    let mut city_station = TestCity::new().with_service(128, 128, ServiceType::PoliceStation);
    tick_slow(&mut city_station);
    let station_cells = city_station
        .resource::<PoliceTiersState>()
        .station_stats
        .cells_covered;

    let mut city_kiosk = TestCity::new().with_service(128, 128, ServiceType::PoliceKiosk);
    tick_slow(&mut city_kiosk);
    let kiosk_cells = city_kiosk
        .resource::<PoliceTiersState>()
        .kiosk_stats
        .cells_covered;

    assert!(
        station_cells > kiosk_cells,
        "Station cells ({station_cells}) should be > Kiosk cells ({kiosk_cells})"
    );
}

// ====================================================================
// 5. Crime reduction effectiveness
// ====================================================================

#[test]
fn test_hq_reduces_more_crime_than_kiosk() {
    // Seed crime grid with uniform crime, then compare reduction.
    let mut city_hq = TestCity::new().with_service(128, 128, ServiceType::PoliceHQ);
    {
        let w = city_hq.world_mut();
        let mut cg = w.resource_mut::<CrimeGrid>();
        for level in &mut cg.levels {
            *level = 50;
        }
    }
    tick_slow(&mut city_hq);
    let hq_reduced = city_hq
        .resource::<PoliceTiersState>()
        .hq_stats
        .crime_reduced;

    let mut city_kiosk = TestCity::new().with_service(128, 128, ServiceType::PoliceKiosk);
    {
        let w = city_kiosk.world_mut();
        let mut cg = w.resource_mut::<CrimeGrid>();
        for level in &mut cg.levels {
            *level = 50;
        }
    }
    tick_slow(&mut city_kiosk);
    let kiosk_reduced = city_kiosk
        .resource::<PoliceTiersState>()
        .kiosk_stats
        .crime_reduced;

    assert!(
        hq_reduced > kiosk_reduced,
        "HQ should reduce more crime ({hq_reduced}) than Kiosk ({kiosk_reduced})"
    );
}

// ====================================================================
// 6. City coverage ratio
// ====================================================================

#[test]
fn test_city_coverage_nonzero_with_police() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::PoliceStation);
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert!(
        s.city_coverage > 0.0,
        "Should have nonzero coverage, got {}",
        s.city_coverage
    );
}

#[test]
fn test_city_coverage_zero_without_police() {
    let mut city = TestCity::new();
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert!(
        (s.city_coverage - 0.0).abs() < 0.001,
        "No police = 0 coverage"
    );
}

// ====================================================================
// 7. Maintenance cost tracking
// ====================================================================

#[test]
fn test_maintenance_tracked_per_tier() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::PoliceKiosk)
        .with_service(60, 60, ServiceType::PoliceStation)
        .with_service(90, 90, ServiceType::PoliceHQ);
    tick_slow(&mut city);
    let s = city.resource::<PoliceTiersState>();
    assert!(
        s.kiosk_stats.total_maintenance > 0.0,
        "Kiosk maintenance should be tracked"
    );
    assert!(
        s.station_stats.total_maintenance > s.kiosk_stats.total_maintenance,
        "Station maintenance > kiosk"
    );
    assert!(
        s.hq_stats.total_maintenance > s.station_stats.total_maintenance,
        "HQ maintenance > station"
    );
}

// ====================================================================
// 8. Tier properties
// ====================================================================

#[test]
fn test_tier_properties_valid() {
    for tier in [PoliceTier::Kiosk, PoliceTier::Station, PoliceTier::Headquarters] {
        assert!(tier.coverage_radius() > 0);
        assert!(tier.crime_reduction() > 0);
        assert!(tier.response_time() > 0);
        assert!(tier.maintenance_cost() > 0.0);
    }
}

// ====================================================================
// 9. Saveable roundtrip (integration)
// ====================================================================

#[test]
fn test_police_tiers_state_persists() {
    use crate::Saveable;
    let mut s = PoliceTiersState::default();
    s.kiosk_stats.building_count = 4;
    s.station_stats.crime_reduced = 100;
    s.hq_stats.cells_covered = 500;
    s.coordination_active = true;
    s.city_coverage = 0.35;
    let bytes = s.save_to_bytes().expect("should serialize");
    let r = PoliceTiersState::load_from_bytes(&bytes);
    assert_eq!(r.kiosk_stats.building_count, 4);
    assert_eq!(r.station_stats.crime_reduced, 100);
    assert_eq!(r.hq_stats.cells_covered, 500);
    assert!(r.coordination_active);
    assert!((r.city_coverage - 0.35).abs() < 0.001);
}
