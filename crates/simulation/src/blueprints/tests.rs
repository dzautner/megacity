//! Unit tests for the blueprint system.

use super::*;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::Saveable;

#[test]
fn test_blueprint_road_type_roundtrip() {
    use crate::grid::RoadType;
    let types = [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
        RoadType::Path,
    ];
    for rt in types {
        let brt: BlueprintRoadType = rt.into();
        let back: RoadType = brt.into();
        assert_eq!(rt, back);
    }
}

#[test]
fn test_blueprint_zone_type_roundtrip() {
    let types = [
        ZoneType::None,
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];
    for zt in types {
        let bzt: BlueprintZoneType = zt.into();
        let back: ZoneType = bzt.into();
        assert_eq!(zt, back);
    }
}

#[test]
fn test_blueprint_library_add_and_get() {
    let mut lib = BlueprintLibrary::default();
    assert!(lib.is_empty());
    assert_eq!(lib.count(), 0);

    let bp = Blueprint {
        name: "Test".to_string(),
        width: 10,
        height: 10,
        segments: vec![],
        zone_cells: vec![],
    };
    let idx = lib.add(bp);
    assert_eq!(idx, 0);
    assert_eq!(lib.count(), 1);
    assert!(!lib.is_empty());
    assert_eq!(lib.get(0).unwrap().name, "Test");
}

#[test]
fn test_blueprint_library_remove() {
    let mut lib = BlueprintLibrary::default();
    lib.add(Blueprint {
        name: "A".to_string(),
        width: 5,
        height: 5,
        segments: vec![],
        zone_cells: vec![],
    });
    lib.add(Blueprint {
        name: "B".to_string(),
        width: 10,
        height: 10,
        segments: vec![],
        zone_cells: vec![],
    });
    assert_eq!(lib.count(), 2);

    let removed = lib.remove(0);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name, "A");
    assert_eq!(lib.count(), 1);
    assert_eq!(lib.get(0).unwrap().name, "B");
}

#[test]
fn test_blueprint_library_remove_out_of_bounds() {
    let mut lib = BlueprintLibrary::default();
    assert!(lib.remove(0).is_none());
    assert!(lib.remove(100).is_none());
}

#[test]
fn test_saveable_roundtrip() {
    let mut lib = BlueprintLibrary::default();
    lib.add(Blueprint {
        name: "Grid Layout".to_string(),
        width: 20,
        height: 15,
        segments: vec![BlueprintSegment {
            p0: [0.0, 0.0],
            p1: [50.0, 0.0],
            p2: [100.0, 0.0],
            p3: [150.0, 0.0],
            road_type: BlueprintRoadType::Avenue,
        }],
        zone_cells: vec![
            BlueprintZoneCell {
                dx: 1,
                dy: 0,
                zone_type: BlueprintZoneType::ResidentialLow,
            },
            BlueprintZoneCell {
                dx: 2,
                dy: 1,
                zone_type: BlueprintZoneType::CommercialHigh,
            },
        ],
    });

    let bytes = lib.save_to_bytes().expect("should produce bytes");
    let restored = BlueprintLibrary::load_from_bytes(&bytes);
    assert_eq!(restored.count(), 1);
    let bp = restored.get(0).unwrap();
    assert_eq!(bp.name, "Grid Layout");
    assert_eq!(bp.width, 20);
    assert_eq!(bp.height, 15);
    assert_eq!(bp.segments.len(), 1);
    assert_eq!(bp.segments[0].road_type, BlueprintRoadType::Avenue);
    assert_eq!(bp.zone_cells.len(), 2);
    assert_eq!(
        bp.zone_cells[0].zone_type,
        BlueprintZoneType::ResidentialLow
    );
    assert_eq!(
        bp.zone_cells[1].zone_type,
        BlueprintZoneType::CommercialHigh
    );
}

#[test]
fn test_saveable_empty_returns_none() {
    let lib = BlueprintLibrary::default();
    assert!(lib.save_to_bytes().is_none());
}

#[test]
fn test_capture_empty_region() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let segments = RoadSegmentStore::default();
    let bp = Blueprint::capture(&grid, &segments, 10, 10, 5, 5, "Empty".to_string());
    assert_eq!(bp.name, "Empty");
    assert_eq!(bp.width, 5);
    assert_eq!(bp.height, 5);
    assert!(bp.segments.is_empty());
    assert!(bp.zone_cells.is_empty());
}

#[test]
fn test_capture_zone_cells() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(12, 13).zone = ZoneType::ResidentialLow;
    grid.get_mut(14, 15).zone = ZoneType::Industrial;

    let segments = RoadSegmentStore::default();
    let bp = Blueprint::capture(&grid, &segments, 10, 10, 10, 10, "Zoned".to_string());
    assert_eq!(bp.zone_cells.len(), 2);

    // Check relative coordinates
    let zc0 = &bp.zone_cells[0];
    assert_eq!(zc0.dx, 2); // 12 - 10
    assert_eq!(zc0.dy, 3); // 13 - 10
    assert_eq!(zc0.zone_type, BlueprintZoneType::ResidentialLow);

    let zc1 = &bp.zone_cells[1];
    assert_eq!(zc1.dx, 4); // 14 - 10
    assert_eq!(zc1.dy, 5); // 15 - 10
    assert_eq!(zc1.zone_type, BlueprintZoneType::Industrial);
}

#[test]
fn test_place_zones_on_grass() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut segments = RoadSegmentStore::default();
    let mut roads = RoadNetwork::default();

    let bp = Blueprint {
        name: "Test".to_string(),
        width: 5,
        height: 5,
        segments: vec![],
        zone_cells: vec![
            BlueprintZoneCell {
                dx: 0,
                dy: 0,
                zone_type: BlueprintZoneType::ResidentialLow,
            },
            BlueprintZoneCell {
                dx: 1,
                dy: 1,
                zone_type: BlueprintZoneType::CommercialLow,
            },
        ],
    };

    let result = bp.place(&mut grid, &mut segments, &mut roads, 50, 50);
    assert_eq!(result.zones_placed, 2);
    assert_eq!(grid.get(50, 50).zone, ZoneType::ResidentialLow);
    assert_eq!(grid.get(51, 51).zone, ZoneType::CommercialLow);
}

#[test]
fn test_place_zones_skip_water() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(50, 50).cell_type = CellType::Water;
    let mut segments = RoadSegmentStore::default();
    let mut roads = RoadNetwork::default();

    let bp = Blueprint {
        name: "Test".to_string(),
        width: 5,
        height: 5,
        segments: vec![],
        zone_cells: vec![BlueprintZoneCell {
            dx: 0,
            dy: 0,
            zone_type: BlueprintZoneType::ResidentialLow,
        }],
    };

    let result = bp.place(&mut grid, &mut segments, &mut roads, 50, 50);
    assert_eq!(result.zones_placed, 0);
    assert_eq!(grid.get(50, 50).zone, ZoneType::None);
}

#[test]
fn test_place_out_of_bounds_skipped() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut segments = RoadSegmentStore::default();
    let mut roads = RoadNetwork::default();

    let bp = Blueprint {
        name: "Edge".to_string(),
        width: 10,
        height: 10,
        segments: vec![],
        zone_cells: vec![BlueprintZoneCell {
            dx: 5,
            dy: 5,
            zone_type: BlueprintZoneType::Industrial,
        }],
    };

    // Place near the edge so dx=5 goes out of bounds (255 + 5 = 260 > 255)
    let result = bp.place(&mut grid, &mut segments, &mut roads, 255, 255);
    assert_eq!(result.zones_placed, 0);
}
