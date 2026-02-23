//! TEST-034: Save/Load Round-Trip Integration Tests (Issue #813)
//!
//! Verifies that a city's state survives a save/load roundtrip:
//! - Grid cells: cell_type, zone, road_type
//! - Citizen count
//! - Treasury
//! - Building count
//! - Road segment count
//! - Service and utility entities

use crate::buildings::Building;
use crate::citizen::{
    CitizenDetails, CitizenState, CitizenStateComp, Gender, Needs, Personality, Position, Velocity,
};
use crate::economy::CityBudget;
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::utilities::{UtilitySource, UtilityType};
use crate::SaveableRegistry;

// ---------------------------------------------------------------------------
// Helper: Snapshot of city state for comparison
// ---------------------------------------------------------------------------

struct CitySnapshot {
    treasury: f64,
    tax_rate: f32,
    citizen_count: usize,
    building_count: usize,
    segment_count: usize,
    road_cell_count: usize,
    clock_day: u32,
    clock_hour: f32,
    /// Grid cell snapshots: (x, y, cell_type, zone, road_type)
    cell_samples: Vec<(usize, usize, CellType, ZoneType, RoadType)>,
}

fn snapshot_city(city: &mut TestCity) -> CitySnapshot {
    let positions = [
        (10, 10),
        (15, 10),
        (20, 10),
        (12, 11),
        (18, 11),
        (24, 11),
        (15, 9),
        (28, 10),
        (50, 50),
    ];
    let cell_samples = positions
        .iter()
        .map(|&(x, y)| {
            let c = city.cell(x, y);
            (x, y, c.cell_type, c.zone, c.road_type)
        })
        .collect();

    CitySnapshot {
        treasury: city.budget().treasury,
        tax_rate: city.budget().tax_rate,
        citizen_count: city.citizen_count(),
        building_count: city.building_count(),
        segment_count: city.segment_count(),
        road_cell_count: city.road_cell_count(),
        clock_day: city.clock().day,
        clock_hour: city.clock().hour,
        cell_samples,
    }
}

fn assert_snapshot_matches(label: &str, before: &CitySnapshot, after: &CitySnapshot) {
    assert!(
        (before.treasury - after.treasury).abs() < 1e-6,
        "{label}: treasury {}, expected {}",
        after.treasury,
        before.treasury,
    );
    assert!(
        (before.tax_rate - after.tax_rate).abs() < f32::EPSILON,
        "{label}: tax_rate {}, expected {}",
        after.tax_rate,
        before.tax_rate,
    );
    assert_eq!(
        before.citizen_count, after.citizen_count,
        "{label}: citizen count"
    );
    assert_eq!(
        before.building_count, after.building_count,
        "{label}: building count"
    );
    assert_eq!(
        before.segment_count, after.segment_count,
        "{label}: segment count"
    );
    assert_eq!(
        before.road_cell_count, after.road_cell_count,
        "{label}: road cell count"
    );
    assert_eq!(before.clock_day, after.clock_day, "{label}: clock day");
    assert!(
        (before.clock_hour - after.clock_hour).abs() < 0.01,
        "{label}: clock hour {}, expected {}",
        after.clock_hour,
        before.clock_hour,
    );
    for (b, a) in before.cell_samples.iter().zip(after.cell_samples.iter()) {
        assert_eq!(b.2, a.2, "{label}: cell_type at ({}, {})", b.0, b.1);
        assert_eq!(b.3, a.3, "{label}: zone at ({}, {})", b.0, b.1);
        assert_eq!(b.4, a.4, "{label}: road_type at ({}, {})", b.0, b.1);
    }
}

// ---------------------------------------------------------------------------
// Test: Full city roundtrip via SaveableRegistry
// ---------------------------------------------------------------------------

/// Build a city with roads, zones, buildings, citizens, services, and a
/// utility. Save and load via SaveableRegistry. Verify grid cells, citizen
/// count, treasury, building count, and road segment count all match.
#[test]
fn test_save_load_roundtrip_full_city_all_criteria() {
    let mut city = TestCity::new()
        .with_budget(75_000.0)
        .with_time(10.0)
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_road(20, 5, 20, 15, RoadType::Avenue)
        .with_zone_rect(12, 11, 16, 13, ZoneType::ResidentialLow)
        .with_zone_rect(22, 11, 26, 13, ZoneType::CommercialLow)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(14, 11, ZoneType::ResidentialLow, 2)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_building(24, 11, ZoneType::Industrial, 1)
        .with_service(15, 9, ServiceType::FireStation)
        .with_utility(28, 10, UtilityType::PowerPlant)
        .with_citizen((12, 11), (18, 11))
        .with_citizen((14, 11), (24, 11))
        .with_citizen((12, 11), (24, 11));

    let before = snapshot_city(&mut city);
    assert_eq!(before.citizen_count, 3);
    assert_eq!(before.building_count, 4);
    assert!(before.treasury > 74_000.0);
    assert!(before.segment_count >= 2);
    assert!(before.road_cell_count > 0);

    // Save via SaveableRegistry.
    let extensions = {
        let w = city.world_mut();
        let r = w.resource::<SaveableRegistry>();
        r.save_all(w)
    };

    // Load via SaveableRegistry.
    {
        let w = city.world_mut();
        let r = w.remove_resource::<SaveableRegistry>().unwrap();
        r.load_all(w, &extensions);
        w.insert_resource(r);
    }

    let after = snapshot_city(&mut city);
    assert_snapshot_matches("full_city_roundtrip", &before, &after);
}

// ---------------------------------------------------------------------------
// Test: Grid cell state survives serde roundtrip
// ---------------------------------------------------------------------------

/// Verify grid cell properties (cell_type, zone, road_type) survive serde.
#[test]
fn test_save_load_roundtrip_grid_cells_serde() {
    let city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Boulevard)
        .with_zone(12, 11, ZoneType::ResidentialHigh)
        .with_zone(14, 11, ZoneType::CommercialHigh)
        .with_zone(16, 11, ZoneType::Industrial)
        .with_zone(18, 11, ZoneType::Office);

    let positions = [(10, 10), (15, 10), (12, 11), (14, 11), (16, 11), (18, 11)];
    for (x, y) in positions {
        let cell = city.cell(x, y);
        let ct: CellType =
            serde_json::from_str(&serde_json::to_string(&cell.cell_type).unwrap()).unwrap();
        let zone: ZoneType =
            serde_json::from_str(&serde_json::to_string(&cell.zone).unwrap()).unwrap();
        let rt: RoadType =
            serde_json::from_str(&serde_json::to_string(&cell.road_type).unwrap()).unwrap();
        assert_eq!(ct, cell.cell_type, "cell_type at ({x}, {y})");
        assert_eq!(zone, cell.zone, "zone at ({x}, {y})");
        assert_eq!(rt, cell.road_type, "road_type at ({x}, {y})");
    }

    assert_eq!(city.cell(10, 10).cell_type, CellType::Road);
    assert_eq!(city.cell(10, 10).road_type, RoadType::Boulevard);
    assert_eq!(city.cell(12, 11).zone, ZoneType::ResidentialHigh);
    assert_eq!(city.cell(14, 11).zone, ZoneType::CommercialHigh);
    assert_eq!(city.cell(16, 11).zone, ZoneType::Industrial);
    assert_eq!(city.cell(18, 11).zone, ZoneType::Office);
}

// ---------------------------------------------------------------------------
// Test: Treasury survives serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_save_load_roundtrip_treasury_serde() {
    let city = TestCity::new().with_budget(123_456.78);
    let budget = city.budget();
    let json = serde_json::to_string(budget).unwrap();
    let restored: CityBudget = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.treasury - 123_456.78).abs() < 1e-6,
        "treasury mismatch"
    );
    assert!(
        (restored.tax_rate - budget.tax_rate).abs() < f32::EPSILON,
        "tax_rate mismatch"
    );
    assert_eq!(restored.last_collection_day, budget.last_collection_day);
}

// ---------------------------------------------------------------------------
// Test: Road segment count survives roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_save_load_roundtrip_road_segment_count() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_road(20, 5, 20, 15, RoadType::Avenue)
        .with_road(30, 10, 30, 20, RoadType::Boulevard);

    let seg_before = city.segment_count();
    let road_before = city.road_cell_count();
    assert!(seg_before >= 3, "expected >=3 segments, got {seg_before}");
    assert!(road_before > 0);

    let extensions = {
        let w = city.world_mut();
        let r = w.resource::<SaveableRegistry>();
        r.save_all(w)
    };
    {
        let w = city.world_mut();
        let r = w.remove_resource::<SaveableRegistry>().unwrap();
        r.load_all(w, &extensions);
        w.insert_resource(r);
    }

    assert_eq!(city.segment_count(), seg_before, "segment count changed");
    assert_eq!(
        city.road_cell_count(),
        road_before,
        "road cell count changed"
    );
}

// ---------------------------------------------------------------------------
// Test: Citizen data fidelity through serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_save_load_roundtrip_citizen_data_fidelity() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_citizen((12, 11), (18, 11));

    assert_eq!(city.citizen_count(), 1);

    let world = city.world_mut();
    let mut q = world.query::<(
        &CitizenDetails,
        &CitizenStateComp,
        &Personality,
        &Needs,
        &Position,
        &Velocity,
    )>();
    let (d, s, p, n, pos, vel) = q.iter(world).next().expect("should have 1 citizen");

    let d2: CitizenDetails = serde_json::from_str(&serde_json::to_string(d).unwrap()).unwrap();
    let s2: CitizenState = serde_json::from_str(&serde_json::to_string(&s.0).unwrap()).unwrap();
    let p2: Personality = serde_json::from_str(&serde_json::to_string(p).unwrap()).unwrap();
    let n2: Needs = serde_json::from_str(&serde_json::to_string(n).unwrap()).unwrap();
    let pos2: Position = serde_json::from_str(&serde_json::to_string(pos).unwrap()).unwrap();
    let vel2: Velocity = serde_json::from_str(&serde_json::to_string(vel).unwrap()).unwrap();

    assert_eq!(d2.age, 30);
    assert!(matches!(d2.gender, Gender::Male));
    assert_eq!(d2.education, 2);
    assert!((d2.happiness - 60.0).abs() < f32::EPSILON);
    assert_eq!(s2, CitizenState::AtHome);
    assert!((p2.ambition - 0.5).abs() < f32::EPSILON);
    assert!((n2.hunger - Needs::default().hunger).abs() < f32::EPSILON);
    assert!(
        pos2.x != 0.0 || pos2.y != 0.0,
        "position should be non-zero"
    );
    assert!(vel2.x.abs() < f32::EPSILON && vel2.y.abs() < f32::EPSILON);
}

// ---------------------------------------------------------------------------
// Test: Building data survives serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_save_load_roundtrip_building_data_serde() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(14, 11, ZoneType::ResidentialLow, 3)
        .with_building(18, 11, ZoneType::CommercialLow, 2)
        .with_building(24, 11, ZoneType::Industrial, 1);

    assert_eq!(city.building_count(), 4);

    let world = city.world_mut();
    let mut q = world.query::<&Building>();
    let mut buildings: Vec<_> = q.iter(world).cloned().collect();
    buildings.sort_by_key(|b| (b.grid_x, b.grid_y));

    for b in &buildings {
        let r: Building = serde_json::from_str(&serde_json::to_string(b).unwrap()).unwrap();
        assert_eq!(
            b.zone_type, r.zone_type,
            "zone at ({},{})",
            b.grid_x, b.grid_y
        );
        assert_eq!(b.level, r.level, "level at ({},{})", b.grid_x, b.grid_y);
        assert_eq!(b.grid_x, r.grid_x);
        assert_eq!(b.grid_y, r.grid_y);
        assert_eq!(b.capacity, r.capacity);
    }

    assert_eq!(buildings[0].zone_type, ZoneType::ResidentialLow);
    assert_eq!(buildings[0].level, 1);
    assert_eq!(buildings[1].level, 3);
}

// ---------------------------------------------------------------------------
// Test: Services and utilities survive serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_save_load_roundtrip_services_and_utilities_serde() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_service(15, 9, ServiceType::FireStation)
        .with_service(20, 9, ServiceType::PoliceStation)
        .with_utility(28, 10, UtilityType::PowerPlant)
        .with_utility(30, 10, UtilityType::WaterTower);

    let world = city.world_mut();

    let mut sq = world.query::<&ServiceBuilding>();
    let services: Vec<_> = sq.iter(world).cloned().collect();
    assert_eq!(services.len(), 2, "should have 2 services");
    for s in &services {
        let r: ServiceBuilding = serde_json::from_str(&serde_json::to_string(s).unwrap()).unwrap();
        assert_eq!(s.service_type, r.service_type);
        assert_eq!(s.grid_x, r.grid_x);
        assert_eq!(s.grid_y, r.grid_y);
    }

    let mut uq = world.query::<&UtilitySource>();
    let utilities: Vec<_> = uq.iter(world).cloned().collect();
    assert_eq!(utilities.len(), 2, "should have 2 utilities");
    for u in &utilities {
        let r: UtilitySource = serde_json::from_str(&serde_json::to_string(u).unwrap()).unwrap();
        assert_eq!(u.utility_type, r.utility_type);
        assert_eq!(u.grid_x, r.grid_x);
        assert_eq!(u.grid_y, r.grid_y);
        assert_eq!(u.range, r.range);
    }
}

// ---------------------------------------------------------------------------
// Test: Multiple roundtrips preserve determinism
// ---------------------------------------------------------------------------

#[test]
fn test_save_load_roundtrip_multiple_cycles_deterministic() {
    let mut city = TestCity::new()
        .with_budget(50_000.0)
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_road(20, 5, 20, 15, RoadType::Avenue)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_building(24, 11, ZoneType::Industrial, 1)
        .with_service(15, 9, ServiceType::FireStation)
        .with_utility(28, 10, UtilityType::PowerPlant)
        .with_citizen((12, 11), (18, 11))
        .with_citizen((12, 11), (24, 11));

    let baseline = snapshot_city(&mut city);

    for cycle in 0..10 {
        let ext = {
            let w = city.world_mut();
            let r = w.resource::<SaveableRegistry>();
            r.save_all(w)
        };
        {
            let w = city.world_mut();
            let r = w.remove_resource::<SaveableRegistry>().unwrap();
            r.load_all(w, &ext);
            w.insert_resource(r);
        }
        let current = snapshot_city(&mut city);
        assert_snapshot_matches(&format!("cycle_{cycle}"), &baseline, &current);
    }
}

// ---------------------------------------------------------------------------
// Test: GameClock survives serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_save_load_roundtrip_game_clock_serde() {
    let city = TestCity::new().with_time(14.5);
    let clock = city.clock();
    let json = serde_json::to_string(clock).unwrap();
    let restored: GameClock = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.day, clock.day);
    assert!((restored.hour - 14.5).abs() < 0.01);
    assert_eq!(restored.speed, clock.speed);
}
