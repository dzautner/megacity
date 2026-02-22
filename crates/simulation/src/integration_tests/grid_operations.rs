//! Integration tests for grid operations (TEST-010).
//!
//! Covers: world_to_grid, grid_to_world, neighbors4, in_bounds,
//! boundary cells, and edge/corner cases.

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// world_to_grid / grid_to_world roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_world_to_grid_roundtrip_origin() {
    let (wx, wy) = WorldGrid::grid_to_world(0, 0);
    let (gx, gy) = WorldGrid::world_to_grid(wx, wy);
    assert_eq!((gx as usize, gy as usize), (0, 0));
}

#[test]
fn test_world_to_grid_roundtrip_max_corner() {
    let (wx, wy) = WorldGrid::grid_to_world(255, 255);
    let (gx, gy) = WorldGrid::world_to_grid(wx, wy);
    assert_eq!((gx as usize, gy as usize), (255, 255));
}

#[test]
fn test_world_to_grid_roundtrip_center() {
    let (wx, wy) = WorldGrid::grid_to_world(128, 128);
    let (gx, gy) = WorldGrid::world_to_grid(wx, wy);
    assert_eq!((gx as usize, gy as usize), (128, 128));
}

#[test]
fn test_world_to_grid_roundtrip_edges() {
    // Test all four edge midpoints
    for &(gx, gy) in &[(0, 128), (255, 128), (128, 0), (128, 255)] {
        let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
        let (rx, ry) = WorldGrid::world_to_grid(wx, wy);
        assert_eq!(
            (rx as usize, ry as usize),
            (gx, gy),
            "Roundtrip failed for grid ({gx}, {gy})"
        );
    }
}

#[test]
fn test_world_to_grid_roundtrip_all_corners() {
    for &(gx, gy) in &[(0, 0), (255, 0), (0, 255), (255, 255)] {
        let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
        let (rx, ry) = WorldGrid::world_to_grid(wx, wy);
        assert_eq!(
            (rx as usize, ry as usize),
            (gx, gy),
            "Roundtrip failed for corner ({gx}, {gy})"
        );
    }
}

#[test]
fn test_grid_to_world_produces_cell_center() {
    // grid_to_world should produce the center of the cell
    let (wx, wy) = WorldGrid::grid_to_world(0, 0);
    assert!(
        (wx - CELL_SIZE * 0.5).abs() < f32::EPSILON,
        "Expected world x = {}, got {wx}",
        CELL_SIZE * 0.5
    );
    assert!(
        (wy - CELL_SIZE * 0.5).abs() < f32::EPSILON,
        "Expected world y = {}, got {wy}",
        CELL_SIZE * 0.5
    );
}

#[test]
fn test_grid_to_world_spacing() {
    // Adjacent cells should be CELL_SIZE apart
    let (wx0, wy0) = WorldGrid::grid_to_world(10, 20);
    let (wx1, _) = WorldGrid::grid_to_world(11, 20);
    let (_, wy1) = WorldGrid::grid_to_world(10, 21);
    assert!(
        (wx1 - wx0 - CELL_SIZE).abs() < f32::EPSILON,
        "Horizontal spacing should be {CELL_SIZE}, got {}",
        wx1 - wx0
    );
    assert!(
        (wy1 - wy0 - CELL_SIZE).abs() < f32::EPSILON,
        "Vertical spacing should be {CELL_SIZE}, got {}",
        wy1 - wy0
    );
}

#[test]
fn test_world_to_grid_floor_behavior() {
    // world_to_grid uses floor, so any position within a cell maps to it
    let (wx, wy) = WorldGrid::grid_to_world(5, 10);
    // Slightly offset from center should still map back
    let (gx, gy) = WorldGrid::world_to_grid(wx + 1.0, wy - 1.0);
    assert_eq!((gx as usize, gy as usize), (5, 10));
}

#[test]
fn test_world_to_grid_cell_boundary() {
    // At the exact start of a cell (left/bottom edge)
    let (gx, gy) = WorldGrid::world_to_grid(5.0 * CELL_SIZE, 10.0 * CELL_SIZE);
    assert_eq!((gx, gy), (5, 10));
}

#[test]
fn test_world_to_grid_negative_coords() {
    // Negative world coordinates should produce negative grid indices
    let (gx, gy) = WorldGrid::world_to_grid(-1.0, -1.0);
    assert!(gx < 0, "Negative world x should give negative grid x");
    assert!(gy < 0, "Negative world y should give negative grid y");
}

// ---------------------------------------------------------------------------
// neighbors4 tests
// ---------------------------------------------------------------------------

#[test]
fn test_neighbors4_center_returns_four() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(128, 128);
    assert_eq!(count, 4, "Center cell should have 4 neighbors");
    let valid = &neighbors[..count];
    assert!(valid.contains(&(127, 128)), "Missing left neighbor");
    assert!(valid.contains(&(129, 128)), "Missing right neighbor");
    assert!(valid.contains(&(128, 127)), "Missing top neighbor");
    assert!(valid.contains(&(128, 129)), "Missing bottom neighbor");
}

#[test]
fn test_neighbors4_top_left_corner_returns_two() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(0, 0);
    assert_eq!(count, 2, "Top-left corner should have 2 neighbors");
    let valid = &neighbors[..count];
    assert!(valid.contains(&(1, 0)), "Missing right neighbor");
    assert!(valid.contains(&(0, 1)), "Missing bottom neighbor");
}

#[test]
fn test_neighbors4_top_right_corner_returns_two() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(255, 0);
    assert_eq!(count, 2, "Top-right corner should have 2 neighbors");
    let valid = &neighbors[..count];
    assert!(valid.contains(&(254, 0)), "Missing left neighbor");
    assert!(valid.contains(&(255, 1)), "Missing bottom neighbor");
}

#[test]
fn test_neighbors4_bottom_left_corner_returns_two() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(0, 255);
    assert_eq!(count, 2, "Bottom-left corner should have 2 neighbors");
    let valid = &neighbors[..count];
    assert!(valid.contains(&(1, 255)), "Missing right neighbor");
    assert!(valid.contains(&(0, 254)), "Missing top neighbor");
}

#[test]
fn test_neighbors4_bottom_right_corner_returns_two() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(255, 255);
    assert_eq!(count, 2, "Bottom-right corner should have 2 neighbors");
    let valid = &neighbors[..count];
    assert!(valid.contains(&(254, 255)), "Missing left neighbor");
    assert!(valid.contains(&(255, 254)), "Missing top neighbor");
}

#[test]
fn test_neighbors4_top_edge_returns_three() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(128, 0);
    assert_eq!(count, 3, "Top edge cell should have 3 neighbors");
    let valid = &neighbors[..count];
    assert!(valid.contains(&(127, 0)), "Missing left neighbor");
    assert!(valid.contains(&(129, 0)), "Missing right neighbor");
    assert!(valid.contains(&(128, 1)), "Missing bottom neighbor");
}

#[test]
fn test_neighbors4_bottom_edge_returns_three() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(128, 255);
    assert_eq!(count, 3, "Bottom edge cell should have 3 neighbors");
    let valid = &neighbors[..count];
    assert!(valid.contains(&(127, 255)), "Missing left neighbor");
    assert!(valid.contains(&(129, 255)), "Missing right neighbor");
    assert!(valid.contains(&(128, 254)), "Missing top neighbor");
}

#[test]
fn test_neighbors4_left_edge_returns_three() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(0, 128);
    assert_eq!(count, 3, "Left edge cell should have 3 neighbors");
    let valid = &neighbors[..count];
    assert!(valid.contains(&(1, 128)), "Missing right neighbor");
    assert!(valid.contains(&(0, 127)), "Missing top neighbor");
    assert!(valid.contains(&(0, 129)), "Missing bottom neighbor");
}

#[test]
fn test_neighbors4_right_edge_returns_three() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(255, 128);
    assert_eq!(count, 3, "Right edge cell should have 3 neighbors");
    let valid = &neighbors[..count];
    assert!(valid.contains(&(254, 128)), "Missing left neighbor");
    assert!(valid.contains(&(255, 127)), "Missing top neighbor");
    assert!(valid.contains(&(255, 129)), "Missing bottom neighbor");
}

#[test]
fn test_neighbors4_no_diagonal_neighbors() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let (neighbors, count) = grid.neighbors4(128, 128);
    let valid = &neighbors[..count];
    // Diagonal neighbors should NOT be in the result
    assert!(
        !valid.contains(&(127, 127)),
        "neighbors4 should not include diagonals"
    );
    assert!(
        !valid.contains(&(129, 129)),
        "neighbors4 should not include diagonals"
    );
    assert!(
        !valid.contains(&(127, 129)),
        "neighbors4 should not include diagonals"
    );
    assert!(
        !valid.contains(&(129, 127)),
        "neighbors4 should not include diagonals"
    );
}

// ---------------------------------------------------------------------------
// in_bounds tests
// ---------------------------------------------------------------------------

#[test]
fn test_in_bounds_valid_origin() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    assert!(grid.in_bounds(0, 0), "(0,0) should be in bounds");
}

#[test]
fn test_in_bounds_valid_max() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    assert!(grid.in_bounds(255, 255), "(255,255) should be in bounds");
}

#[test]
fn test_in_bounds_rejects_width() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    assert!(
        !grid.in_bounds(GRID_WIDTH, 0),
        "x=GRID_WIDTH should be out of bounds"
    );
}

#[test]
fn test_in_bounds_rejects_height() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    assert!(
        !grid.in_bounds(0, GRID_HEIGHT),
        "y=GRID_HEIGHT should be out of bounds"
    );
}

#[test]
fn test_in_bounds_rejects_both_out() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    assert!(
        !grid.in_bounds(GRID_WIDTH, GRID_HEIGHT),
        "(GRID_WIDTH, GRID_HEIGHT) should be out of bounds"
    );
}

#[test]
fn test_in_bounds_rejects_large_values() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    assert!(
        !grid.in_bounds(1000, 1000),
        "Very large indices should be out of bounds"
    );
}

#[test]
fn test_in_bounds_boundary_cells() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    // All four corners
    assert!(grid.in_bounds(0, 0));
    assert!(grid.in_bounds(255, 0));
    assert!(grid.in_bounds(0, 255));
    assert!(grid.in_bounds(255, 255));
    // Just outside each corner
    assert!(!grid.in_bounds(256, 0));
    assert!(!grid.in_bounds(0, 256));
    assert!(!grid.in_bounds(256, 256));
}

// ---------------------------------------------------------------------------
// index / get / get_mut tests
// ---------------------------------------------------------------------------

#[test]
fn test_index_linearization() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    assert_eq!(grid.index(0, 0), 0);
    assert_eq!(grid.index(1, 0), 1);
    assert_eq!(grid.index(0, 1), GRID_WIDTH);
    assert_eq!(grid.index(255, 255), GRID_WIDTH * GRID_HEIGHT - 1);
}

#[test]
fn test_get_and_get_mut_same_cell() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(10, 20).cell_type = CellType::Road;
    assert_eq!(grid.get(10, 20).cell_type, CellType::Road);
}

#[test]
fn test_default_cell_properties() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let cell = grid.get(0, 0);
    assert_eq!(cell.cell_type, CellType::Grass);
    assert_eq!(cell.zone, ZoneType::None);
    assert_eq!(cell.road_type, RoadType::Local);
    assert!(cell.building_id.is_none());
    assert!(!cell.has_power);
    assert!(!cell.has_water);
    assert!((cell.elevation - 0.0).abs() < f32::EPSILON);
}

// ---------------------------------------------------------------------------
// WorldGrid::new dimensions
// ---------------------------------------------------------------------------

#[test]
fn test_grid_new_dimensions() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    assert_eq!(grid.width, GRID_WIDTH);
    assert_eq!(grid.height, GRID_HEIGHT);
    assert_eq!(grid.cells.len(), GRID_WIDTH * GRID_HEIGHT);
}

#[test]
fn test_grid_new_small_custom() {
    // Verify the grid works for non-standard sizes too
    let grid = WorldGrid::new(10, 20);
    assert_eq!(grid.width, 10);
    assert_eq!(grid.height, 20);
    assert_eq!(grid.cells.len(), 200);
    assert!(grid.in_bounds(9, 19));
    assert!(!grid.in_bounds(10, 20));
}

#[test]
fn test_small_grid_neighbors4_corner() {
    let grid = WorldGrid::new(3, 3);
    // Corner of a 3x3 grid
    let (neighbors, count) = grid.neighbors4(0, 0);
    assert_eq!(count, 2);
    let valid = &neighbors[..count];
    assert!(valid.contains(&(1, 0)));
    assert!(valid.contains(&(0, 1)));
}

#[test]
fn test_small_grid_neighbors4_center() {
    let grid = WorldGrid::new(3, 3);
    let (_, count) = grid.neighbors4(1, 1);
    assert_eq!(count, 4, "Center of 3x3 grid should have 4 neighbors");
}

// ---------------------------------------------------------------------------
// Integration tests using TestCity harness
// ---------------------------------------------------------------------------

#[test]
fn test_testcity_grid_is_256x256() {
    let city = TestCity::new();
    let grid = city.grid();
    assert_eq!(grid.width, 256);
    assert_eq!(grid.height, 256);
}

#[test]
fn test_testcity_empty_grid_all_grass() {
    let city = TestCity::new();
    let grid = city.grid();
    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = grid.get(x, y);
            assert_eq!(
                cell.cell_type,
                CellType::Grass,
                "Cell ({x}, {y}) should be Grass in empty city"
            );
        }
    }
}

#[test]
fn test_testcity_road_placement_updates_grid() {
    let city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    let grid = city.grid();
    // At least some cells along the road should be marked as road
    let mut road_found = false;
    for x in 10..=20 {
        if grid.get(x, 10).cell_type == CellType::Road {
            road_found = true;
            break;
        }
    }
    assert!(road_found, "Road placement should mark cells as Road");
}

#[test]
fn test_testcity_zone_placement_updates_grid() {
    let city = TestCity::new().with_zone(50, 50, ZoneType::ResidentialLow);
    city.assert_zone(50, 50, ZoneType::ResidentialLow);
}

#[test]
fn test_testcity_zone_rect_updates_grid() {
    let city = TestCity::new().with_zone_rect(10, 10, 15, 15, ZoneType::CommercialHigh);
    let grid = city.grid();
    for y in 10..=15 {
        for x in 10..=15 {
            assert_eq!(
                grid.get(x, y).zone,
                ZoneType::CommercialHigh,
                "Cell ({x}, {y}) should be CommercialHigh"
            );
        }
    }
}

#[test]
fn test_testcity_building_placement_sets_building_id() {
    let city = TestCity::new().with_building(30, 30, ZoneType::ResidentialLow, 1);
    city.assert_has_building(30, 30);
}

#[test]
fn test_testcity_grid_coord_roundtrip_via_harness() {
    let _city = TestCity::new();
    // Verify the roundtrip works for several sample points
    for gx in [0_usize, 50, 128, 200, 255] {
        for gy in [0_usize, 50, 128, 200, 255] {
            let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
            let (rx, ry) = WorldGrid::world_to_grid(wx, wy);
            assert_eq!(
                (rx as usize, ry as usize),
                (gx, gy),
                "Roundtrip failed for ({gx}, {gy})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// RoadType properties
// ---------------------------------------------------------------------------

#[test]
fn test_road_type_speed_positive() {
    let types = [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
        RoadType::Path,
    ];
    for rt in types {
        assert!(rt.speed() > 0.0, "{rt:?} should have positive speed");
    }
}

#[test]
fn test_road_type_cost_positive() {
    let types = [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
        RoadType::Path,
    ];
    for rt in types {
        assert!(rt.cost() > 0.0, "{rt:?} should have positive cost");
    }
}

#[test]
fn test_road_type_path_no_vehicles() {
    assert!(
        !RoadType::Path.allows_vehicles(),
        "Path should not allow vehicles"
    );
}

#[test]
fn test_road_type_all_except_path_allow_vehicles() {
    for rt in [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
    ] {
        assert!(rt.allows_vehicles(), "{rt:?} should allow vehicles");
    }
}

#[test]
fn test_road_type_zoning_allowed() {
    assert!(RoadType::Local.allows_zoning());
    assert!(RoadType::Avenue.allows_zoning());
    assert!(RoadType::Boulevard.allows_zoning());
    assert!(!RoadType::Highway.allows_zoning());
    assert!(!RoadType::OneWay.allows_zoning());
    assert!(!RoadType::Path.allows_zoning());
}

// ---------------------------------------------------------------------------
// ZoneType properties
// ---------------------------------------------------------------------------

#[test]
fn test_zone_type_residential_classification() {
    assert!(ZoneType::ResidentialLow.is_residential());
    assert!(ZoneType::ResidentialMedium.is_residential());
    assert!(ZoneType::ResidentialHigh.is_residential());
    assert!(!ZoneType::CommercialLow.is_residential());
    assert!(!ZoneType::Industrial.is_residential());
    assert!(!ZoneType::None.is_residential());
}

#[test]
fn test_zone_type_commercial_classification() {
    assert!(ZoneType::CommercialLow.is_commercial());
    assert!(ZoneType::CommercialHigh.is_commercial());
    assert!(!ZoneType::ResidentialLow.is_commercial());
    assert!(!ZoneType::Industrial.is_commercial());
}

#[test]
fn test_zone_type_job_zone_classification() {
    assert!(ZoneType::CommercialLow.is_job_zone());
    assert!(ZoneType::CommercialHigh.is_job_zone());
    assert!(ZoneType::Industrial.is_job_zone());
    assert!(ZoneType::Office.is_job_zone());
    assert!(ZoneType::MixedUse.is_job_zone());
    assert!(!ZoneType::ResidentialLow.is_job_zone());
    assert!(!ZoneType::None.is_job_zone());
}

#[test]
fn test_zone_type_none_max_level_is_zero() {
    assert_eq!(ZoneType::None.max_level(), 0);
}

#[test]
fn test_zone_type_max_levels_nonzero_for_real_zones() {
    let real_zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];
    for zone in real_zones {
        assert!(
            zone.max_level() > 0,
            "{zone:?} should have positive max level"
        );
    }
}
