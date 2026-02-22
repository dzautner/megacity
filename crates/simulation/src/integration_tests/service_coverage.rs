//! TEST-007: Unit Tests for Service Coverage Grid
//!
//! Tests service coverage BFS/radius calculation. Verifies cells within radius
//! have coverage, cells outside do not, and that overlapping service buildings
//! produce correct bitflag unions.

use crate::happiness::{
    ServiceCoverageGrid, COVERAGE_EDUCATION, COVERAGE_ENTERTAINMENT, COVERAGE_FIRE,
    COVERAGE_HEALTH, COVERAGE_PARK, COVERAGE_POLICE, COVERAGE_TELECOM, COVERAGE_TRANSPORT,
};
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;

// ====================================================================
// Helper: run one slow cycle to trigger update_service_coverage
// ====================================================================

fn tick_coverage(city: &mut TestCity) {
    city.tick_slow_cycles(1);
}

// ====================================================================
// 1. Single service building covers correct radius
// ====================================================================

#[test]
fn test_fire_station_covers_cells_within_radius() {
    // FireStation radius = 20.0 * CELL_SIZE = 320.0
    // In cells: 320 / 16 = 20 cells
    let mut city = TestCity::new().with_service(128, 128, ServiceType::FireStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    // Cell at the building itself should be covered
    let idx_center = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_fire(idx_center),
        "Fire station should cover its own cell"
    );

    // Cell 10 cells away (well within 20-cell radius)
    let idx_near = ServiceCoverageGrid::idx(138, 128);
    assert!(
        cov.has_fire(idx_near),
        "Cell 10 cells away should be covered by fire station"
    );

    // Cell 19 cells away (still within radius — 19*16 = 304 < 320)
    let idx_edge = ServiceCoverageGrid::idx(147, 128);
    assert!(
        cov.has_fire(idx_edge),
        "Cell 19 cells away should be covered by fire station"
    );
}

#[test]
fn test_hospital_covers_cells_within_radius() {
    // Hospital radius = 25.0 * CELL_SIZE = 400.0
    // In cells: 400 / 16 = 25 cells
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_center = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_health(idx_center),
        "Hospital should cover its own cell"
    );

    // 15 cells away — within 25-cell radius
    let idx_mid = ServiceCoverageGrid::idx(143, 128);
    assert!(
        cov.has_health(idx_mid),
        "Cell 15 cells away should have health coverage"
    );

    // 24 cells away — still within radius (24*16 = 384 < 400)
    let idx_near_edge = ServiceCoverageGrid::idx(152, 128);
    assert!(
        cov.has_health(idx_near_edge),
        "Cell 24 cells away should have health coverage"
    );
}

#[test]
fn test_police_station_covers_cells_within_radius() {
    // PoliceStation radius = 20.0 * CELL_SIZE = 320.0
    let mut city = TestCity::new().with_service(128, 128, ServiceType::PoliceStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_center = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_police(idx_center),
        "Police station should cover its own cell"
    );

    let idx_near = ServiceCoverageGrid::idx(128, 138);
    assert!(
        cov.has_police(idx_near),
        "Cell 10 cells away (Y) should be covered"
    );
}

#[test]
fn test_elementary_school_covers_education() {
    // ElementarySchool radius = 15.0 * CELL_SIZE = 240.0
    let mut city = TestCity::new().with_service(128, 128, ServiceType::ElementarySchool);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_center = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_education(idx_center),
        "Elementary school should provide education coverage at its own cell"
    );

    let idx_near = ServiceCoverageGrid::idx(135, 128);
    assert!(
        cov.has_education(idx_near),
        "Cell 7 cells away should have education coverage"
    );
}

#[test]
fn test_small_park_covers_park_area() {
    // SmallPark radius = 8.0 * CELL_SIZE = 128.0 → 8 cells
    let mut city = TestCity::new().with_service(128, 128, ServiceType::SmallPark);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_center = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_park(idx_center),
        "Small park should cover its own cell"
    );

    // 5 cells away — within 8-cell radius
    let idx_near = ServiceCoverageGrid::idx(133, 128);
    assert!(
        cov.has_park(idx_near),
        "Cell 5 cells away should have park coverage"
    );
}

// ====================================================================
// 2. Cells just outside radius have no coverage
// ====================================================================

#[test]
fn test_fire_station_does_not_cover_outside_radius() {
    // FireStation radius = 20.0 * CELL_SIZE = 320.0
    // 21 cells away = 336.0 > 320.0 → outside
    let mut city = TestCity::new().with_service(128, 128, ServiceType::FireStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_outside = ServiceCoverageGrid::idx(149, 128);
    assert!(
        !cov.has_fire(idx_outside),
        "Cell 21 cells away should NOT be covered by fire station"
    );

    // Far away — definitely outside
    let idx_far = ServiceCoverageGrid::idx(200, 128);
    assert!(
        !cov.has_fire(idx_far),
        "Cell 72 cells away should NOT be covered by fire station"
    );
}

#[test]
fn test_hospital_does_not_cover_outside_radius() {
    // Hospital radius = 25.0 * CELL_SIZE = 400.0
    // 26 cells away = 416.0 > 400.0 → outside
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_outside = ServiceCoverageGrid::idx(154, 128);
    assert!(
        !cov.has_health(idx_outside),
        "Cell 26 cells away should NOT have health coverage"
    );
}

#[test]
fn test_police_station_does_not_cover_outside_radius() {
    // PoliceStation radius = 20.0 * CELL_SIZE = 320.0
    let mut city = TestCity::new().with_service(128, 128, ServiceType::PoliceStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_outside = ServiceCoverageGrid::idx(128, 149);
    assert!(
        !cov.has_police(idx_outside),
        "Cell 21 cells away (Y-axis) should NOT have police coverage"
    );
}

#[test]
fn test_small_park_does_not_cover_outside_radius() {
    // SmallPark radius = 8.0 * CELL_SIZE = 128.0
    // 9 cells away = 144.0 > 128.0 → outside
    let mut city = TestCity::new().with_service(128, 128, ServiceType::SmallPark);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_outside = ServiceCoverageGrid::idx(137, 128);
    assert!(
        !cov.has_park(idx_outside),
        "Cell 9 cells away should NOT have park coverage"
    );
}

#[test]
fn test_diagonal_outside_radius() {
    // FireStation radius = 20.0 * CELL_SIZE = 320.0
    // Diagonal 15,15 away = sqrt(15^2+15^2)*16 = 15*1.414*16 = 339.4 > 320 → outside
    let mut city = TestCity::new().with_service(128, 128, ServiceType::FireStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_diag = ServiceCoverageGrid::idx(143, 143);
    assert!(
        !cov.has_fire(idx_diag),
        "Cell at diagonal (15,15) should NOT be covered — Euclidean distance exceeds radius"
    );
}

#[test]
fn test_diagonal_inside_radius() {
    // FireStation radius = 20.0 * CELL_SIZE = 320.0
    // Diagonal 14,14 away = sqrt(14^2+14^2)*16 = 14*1.414*16 = 316.8 < 320 → inside
    let mut city = TestCity::new().with_service(128, 128, ServiceType::FireStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_diag = ServiceCoverageGrid::idx(142, 142);
    assert!(
        cov.has_fire(idx_diag),
        "Cell at diagonal (14,14) should be covered — Euclidean distance within radius"
    );
}

// ====================================================================
// 3. Overlapping coverage from multiple buildings
// ====================================================================

#[test]
fn test_overlapping_fire_stations() {
    // Two fire stations placed so their coverage areas overlap in the middle
    let mut city = TestCity::new()
        .with_service(110, 128, ServiceType::FireStation)
        .with_service(146, 128, ServiceType::FireStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    // Center between the two stations — both should cover it
    let idx_mid = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_fire(idx_mid),
        "Midpoint between two fire stations should have fire coverage"
    );

    // Near first station only
    let idx_near_first = ServiceCoverageGrid::idx(100, 128);
    assert!(
        cov.has_fire(idx_near_first),
        "Cell near first station should have fire coverage"
    );

    // Near second station only
    let idx_near_second = ServiceCoverageGrid::idx(156, 128);
    assert!(
        cov.has_fire(idx_near_second),
        "Cell near second station should have fire coverage"
    );
}

#[test]
fn test_overlapping_different_services() {
    // Place a hospital and a police station at the same location
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Hospital)
        .with_service(128, 128, ServiceType::PoliceStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(cov.has_health(idx), "Should have health coverage");
    assert!(cov.has_police(idx), "Should have police coverage");
    assert!(!cov.has_fire(idx), "Should NOT have fire coverage");
    assert!(
        !cov.has_education(idx),
        "Should NOT have education coverage"
    );
}

#[test]
fn test_multiple_overlapping_services_all_flags() {
    // Place one of each coverage type at the same location
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::Hospital)
        .with_service(128, 128, ServiceType::PoliceStation)
        .with_service(128, 128, ServiceType::FireStation)
        .with_service(128, 128, ServiceType::ElementarySchool)
        .with_service(128, 128, ServiceType::SmallPark)
        .with_service(128, 128, ServiceType::Stadium)
        .with_service(128, 128, ServiceType::CellTower)
        .with_service(128, 128, ServiceType::BusDepot);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);

    assert!(cov.has_health(idx), "Should have health coverage");
    assert!(cov.has_police(idx), "Should have police coverage");
    assert!(cov.has_fire(idx), "Should have fire coverage");
    assert!(cov.has_education(idx), "Should have education coverage");
    assert!(cov.has_park(idx), "Should have park coverage");
    assert!(
        cov.has_entertainment(idx),
        "Should have entertainment coverage"
    );
    assert!(cov.has_telecom(idx), "Should have telecom coverage");
    assert!(cov.has_transport(idx), "Should have transport coverage");

    // Verify all 8 bits are set
    let all_bits = COVERAGE_HEALTH
        | COVERAGE_EDUCATION
        | COVERAGE_POLICE
        | COVERAGE_PARK
        | COVERAGE_ENTERTAINMENT
        | COVERAGE_TELECOM
        | COVERAGE_TRANSPORT
        | COVERAGE_FIRE;
    assert_eq!(
        cov.flags[idx], all_bits,
        "All 8 coverage bits should be set"
    );
}

// ====================================================================
// 4. All service types (fire, police, health, education, garbage/parks)
// ====================================================================

#[test]
fn test_fire_house_provides_fire_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::FireHouse);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(cov.has_fire(idx), "FireHouse should provide fire coverage");
}

#[test]
fn test_fire_hq_provides_fire_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::FireHQ);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(cov.has_fire(idx), "FireHQ should provide fire coverage");
}

#[test]
fn test_police_kiosk_provides_police_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::PoliceKiosk);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_police(idx),
        "PoliceKiosk should provide police coverage"
    );
}

#[test]
fn test_police_hq_provides_police_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::PoliceHQ);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_police(idx),
        "PoliceHQ should provide police coverage"
    );
}

#[test]
fn test_medical_clinic_provides_health_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::MedicalClinic);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_health(idx),
        "MedicalClinic should provide health coverage"
    );
}

#[test]
fn test_medical_center_provides_health_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::MedicalCenter);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_health(idx),
        "MedicalCenter should provide health coverage"
    );
}

#[test]
fn test_high_school_provides_education_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::HighSchool);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_education(idx),
        "HighSchool should provide education coverage"
    );
}

#[test]
fn test_university_provides_education_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::University);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_education(idx),
        "University should provide education coverage"
    );
}

#[test]
fn test_library_provides_education_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Library);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_education(idx),
        "Library should provide education coverage"
    );
}

#[test]
fn test_kindergarten_provides_education_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Kindergarten);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_education(idx),
        "Kindergarten should provide education coverage"
    );
}

#[test]
fn test_large_park_provides_park_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::LargePark);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(cov.has_park(idx), "LargePark should provide park coverage");
}

#[test]
fn test_playground_provides_park_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Playground);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(cov.has_park(idx), "Playground should provide park coverage");
}

#[test]
fn test_stadium_provides_entertainment_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Stadium);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_entertainment(idx),
        "Stadium should provide entertainment coverage"
    );
}

#[test]
fn test_plaza_provides_entertainment_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Plaza);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_entertainment(idx),
        "Plaza should provide entertainment coverage"
    );
}

#[test]
fn test_sports_field_provides_entertainment_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::SportsField);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_entertainment(idx),
        "SportsField should provide entertainment coverage"
    );
}

#[test]
fn test_cell_tower_provides_telecom_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::CellTower);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_telecom(idx),
        "CellTower should provide telecom coverage"
    );
}

#[test]
fn test_data_center_provides_telecom_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::DataCenter);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_telecom(idx),
        "DataCenter should provide telecom coverage"
    );
}

#[test]
fn test_bus_depot_provides_transport_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::BusDepot);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_transport(idx),
        "BusDepot should provide transport coverage"
    );
}

#[test]
fn test_train_station_provides_transport_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::TrainStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_transport(idx),
        "TrainStation should provide transport coverage"
    );
}

#[test]
fn test_subway_station_provides_transport_coverage() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::SubwayStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_transport(idx),
        "SubwayStation should provide transport coverage"
    );
}

// ====================================================================
// 5. Coverage bitflags are correctly set
// ====================================================================

#[test]
fn test_coverage_bitflags_are_independent() {
    // Placing a hospital should NOT set fire or police bits
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);

    assert!(cov.has_health(idx), "Should have health bit set");
    assert!(!cov.has_fire(idx), "Should NOT have fire bit set");
    assert!(!cov.has_police(idx), "Should NOT have police bit set");
    assert!(!cov.has_education(idx), "Should NOT have education bit set");
    assert!(!cov.has_park(idx), "Should NOT have park bit set");
    assert!(
        !cov.has_entertainment(idx),
        "Should NOT have entertainment bit set"
    );
    assert!(!cov.has_telecom(idx), "Should NOT have telecom bit set");
    assert!(!cov.has_transport(idx), "Should NOT have transport bit set");

    // Raw flag should be exactly COVERAGE_HEALTH
    assert_eq!(
        cov.flags[idx], COVERAGE_HEALTH,
        "Only COVERAGE_HEALTH bit should be set"
    );
}

#[test]
fn test_coverage_bitflags_union_correctly() {
    // Place police + fire — should have exactly those two bits
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::PoliceStation)
        .with_service(128, 128, ServiceType::FireStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);

    let expected = COVERAGE_POLICE | COVERAGE_FIRE;
    assert_eq!(
        cov.flags[idx], expected,
        "Should have exactly police + fire bits set, got {:#010b}",
        cov.flags[idx]
    );
}

#[test]
fn test_no_coverage_at_empty_cell() {
    // An empty city should have no coverage anywhere
    let mut city = TestCity::new();
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    // Check a few random cells
    for (x, y) in [(0, 0), (128, 128), (255, 255), (50, 200)] {
        let idx = ServiceCoverageGrid::idx(x, y);
        assert_eq!(
            cov.flags[idx], 0,
            "Cell ({x},{y}) should have no coverage flags in empty city"
        );
    }
}

#[test]
fn test_coverage_radius_varies_by_service_tier() {
    // FireHouse has smaller radius (12 cells) than FireStation (20 cells)
    // Place both at 128,128, check a cell at 15 cells away:
    // - FireStation (20 cells) should cover it
    // - FireHouse (12 cells) should NOT cover it

    // Test FireStation covers 15 cells away
    let mut city_station = TestCity::new().with_service(128, 128, ServiceType::FireStation);
    tick_coverage(&mut city_station);
    let cov_station = city_station.resource::<ServiceCoverageGrid>();
    let idx_15 = ServiceCoverageGrid::idx(143, 128);
    assert!(
        cov_station.has_fire(idx_15),
        "FireStation should cover cell 15 cells away"
    );

    // Test FireHouse does NOT cover 15 cells away (radius is 12 cells)
    let mut city_house = TestCity::new().with_service(128, 128, ServiceType::FireHouse);
    tick_coverage(&mut city_house);
    let cov_house = city_house.resource::<ServiceCoverageGrid>();
    assert!(
        !cov_house.has_fire(idx_15),
        "FireHouse should NOT cover cell 15 cells away (radius is only 12 cells)"
    );

    // But FireHouse should cover 11 cells away
    let idx_11 = ServiceCoverageGrid::idx(139, 128);
    assert!(
        cov_house.has_fire(idx_11),
        "FireHouse should cover cell 11 cells away"
    );
}

#[test]
fn test_coverage_radius_euclidean_not_manhattan() {
    // FireStation radius = 20 cells (320.0 world units)
    // A point at (14, 14) from center has Euclidean distance ~19.8 cells → inside
    // A point at (15, 15) from center has Euclidean distance ~21.2 cells → outside
    // Manhattan distance for (14,14) is 28, and (15,15) is 30, so if it were
    // Manhattan-based both would be well outside the 20-cell "radius".

    let mut city = TestCity::new().with_service(128, 128, ServiceType::FireStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    // (14,14) diagonal → Euclidean = sqrt(14^2+14^2)*16 = ~316.8 < 320 → covered
    let idx_inside = ServiceCoverageGrid::idx(142, 142);
    assert!(
        cov.has_fire(idx_inside),
        "Euclidean coverage: (14,14) diagonal should be inside fire radius"
    );

    // (15,15) diagonal → Euclidean = sqrt(15^2+15^2)*16 = ~339.4 > 320 → NOT covered
    let idx_outside = ServiceCoverageGrid::idx(143, 143);
    assert!(
        !cov.has_fire(idx_outside),
        "Euclidean coverage: (15,15) diagonal should be outside fire radius"
    );
}

// ====================================================================
// 6. Edge cases
// ====================================================================

#[test]
fn test_service_at_grid_corner_covers_partial_area() {
    // Place a fire station at corner (0, 0) — coverage should only extend
    // into valid grid cells (no out-of-bounds access)
    let mut city = TestCity::new().with_service(0, 0, ServiceType::FireStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx_origin = ServiceCoverageGrid::idx(0, 0);
    assert!(
        cov.has_fire(idx_origin),
        "Corner service should cover its own cell"
    );

    let idx_adjacent = ServiceCoverageGrid::idx(5, 5);
    assert!(
        cov.has_fire(idx_adjacent),
        "Cell (5,5) should be covered from corner service"
    );
}

#[test]
fn test_service_at_grid_max_corner() {
    // Place at far corner (255, 255)
    let mut city = TestCity::new().with_service(255, 255, ServiceType::PoliceStation);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    let idx = ServiceCoverageGrid::idx(255, 255);
    assert!(
        cov.has_police(idx),
        "Service at (255,255) should cover its own cell"
    );

    let idx_near = ServiceCoverageGrid::idx(245, 255);
    assert!(
        cov.has_police(idx_near),
        "Cell 10 cells away from (255,255) should be covered"
    );
}

#[test]
fn test_prison_has_zero_radius_no_spatial_coverage() {
    // Prison has 0 radius (city-wide effect, not spatial)
    // and maps to COVERAGE_POLICE bit
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Prison);
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();

    // With 0 radius, ceil(0/16) = 0, so the loop -0..=0 runs with dx=0,dy=0
    // Distance = 0 <= 0 → it covers only its own cell
    let idx_center = ServiceCoverageGrid::idx(128, 128);
    assert!(
        cov.has_police(idx_center),
        "Prison should cover its own cell (zero radius, but distance 0 <= 0)"
    );

    let idx_adjacent = ServiceCoverageGrid::idx(129, 128);
    assert!(
        !cov.has_police(idx_adjacent),
        "Prison should NOT cover adjacent cells (zero radius)"
    );
}

#[test]
fn test_coverage_recalculates_when_service_added() {
    // Start with no services, verify empty, add a service, tick, verify coverage
    let mut city = TestCity::new();
    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(128, 128);
    assert_eq!(cov.flags[idx], 0, "No coverage before any service placed");

    // Spawn a service building dynamically
    {
        let world = city.world_mut();
        world.spawn(ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: 128,
            grid_y: 128,
            radius: ServiceBuilding::coverage_radius(ServiceType::Hospital),
        });
    }

    tick_coverage(&mut city);

    let cov = city.resource::<ServiceCoverageGrid>();
    assert!(
        cov.has_health(idx),
        "Coverage should update after service is spawned"
    );
}

#[test]
fn test_coverage_radius_matches_static_method() {
    // Verify that the TestCity harness sets radius from coverage_radius()
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::Hospital)
        .with_service(110, 100, ServiceType::FireStation)
        .with_service(120, 100, ServiceType::PoliceKiosk);

    let world = city.world_mut();
    let services: Vec<(ServiceType, f32)> = world
        .query::<&ServiceBuilding>()
        .iter(world)
        .map(|s| (s.service_type, s.radius))
        .collect();

    for (stype, radius) in services {
        let expected = ServiceBuilding::coverage_radius(stype);
        assert!(
            (radius - expected).abs() < f32::EPSILON,
            "{:?} should have radius {expected}, got {radius}",
            stype
        );
    }
}
