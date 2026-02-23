//! Property-based tests for grid coordinate and cell invariants (TEST-013a).
//!
//! Uses manual randomized testing with `rand` to verify:
//! - Coordinate roundtrip: grid_to_world -> world_to_grid is identity
//! - Intra-cell world coords map to the correct grid cell
//! - grid_to_world lands at cell center
//! - Cell field validity on default grids
//! - Index bijectivity, in_bounds consistency, neighbors4 correctness
//! - Zone and road type classification invariants

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::roads::RoadNetwork;

const NUM_SAMPLES: usize = 5000;
const SEED: u64 = 0xDEAD_BEEF_CAFE_1337;

fn all_zone_types() -> [ZoneType; 9] {
    [
        ZoneType::None,
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ]
}

fn all_road_types() -> [RoadType; 6] {
    [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
        RoadType::Path,
    ]
}

// ---------------------------------------------------------------------------
// 1. Coordinate Roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_prop_coordinate_roundtrip_random() {
    let mut rng = StdRng::seed_from_u64(SEED);
    for _ in 0..NUM_SAMPLES {
        let gx = rng.gen_range(0..GRID_WIDTH);
        let gy = rng.gen_range(0..GRID_HEIGHT);
        let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
        let (rx, ry) = WorldGrid::world_to_grid(wx, wy);
        assert_eq!(
            (rx as usize, ry as usize),
            (gx, gy),
            "Roundtrip failed: grid ({gx},{gy}) -> world ({wx},{wy}) -> grid ({rx},{ry})"
        );
    }
}

#[test]
fn test_prop_coordinate_roundtrip_exhaustive_edges() {
    for x in 0..GRID_WIDTH {
        for &y in &[0, GRID_HEIGHT - 1] {
            let (wx, wy) = WorldGrid::grid_to_world(x, y);
            let (rx, ry) = WorldGrid::world_to_grid(wx, wy);
            assert_eq!((rx as usize, ry as usize), (x, y));
        }
    }
    for y in 1..GRID_HEIGHT - 1 {
        for &x in &[0, GRID_WIDTH - 1] {
            let (wx, wy) = WorldGrid::grid_to_world(x, y);
            let (rx, ry) = WorldGrid::world_to_grid(wx, wy);
            assert_eq!((rx as usize, ry as usize), (x, y));
        }
    }
}

#[test]
fn test_prop_world_coord_within_cell_maps_to_same_grid() {
    let mut rng = StdRng::seed_from_u64(SEED + 1);
    for _ in 0..NUM_SAMPLES {
        let gx = rng.gen_range(0..GRID_WIDTH);
        let gy = rng.gen_range(0..GRID_HEIGHT);
        let offset_x: f32 = rng.gen_range(0.01..CELL_SIZE - 0.01);
        let offset_y: f32 = rng.gen_range(0.01..CELL_SIZE - 0.01);
        let wx = gx as f32 * CELL_SIZE + offset_x;
        let wy = gy as f32 * CELL_SIZE + offset_y;
        let (rx, ry) = WorldGrid::world_to_grid(wx, wy);
        assert_eq!(
            (rx as usize, ry as usize),
            (gx, gy),
            "Intra-cell mapping failed: grid ({gx},{gy}), offset ({offset_x:.3},{offset_y:.3})"
        );
    }
}

#[test]
fn test_prop_grid_to_world_lands_at_cell_center() {
    let mut rng = StdRng::seed_from_u64(SEED + 2);
    for _ in 0..NUM_SAMPLES {
        let gx = rng.gen_range(0..GRID_WIDTH);
        let gy = rng.gen_range(0..GRID_HEIGHT);
        let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
        let expected_x = gx as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let expected_y = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        assert!((wx - expected_x).abs() < f32::EPSILON);
        assert!((wy - expected_y).abs() < f32::EPSILON);
    }
}

// ---------------------------------------------------------------------------
// 2. Default Cell Validity
// ---------------------------------------------------------------------------

#[test]
fn test_prop_default_grid_cells_are_valid() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut rng = StdRng::seed_from_u64(SEED + 3);
    for _ in 0..NUM_SAMPLES {
        let x = rng.gen_range(0..GRID_WIDTH);
        let y = rng.gen_range(0..GRID_HEIGHT);
        let cell = grid.get(x, y);
        assert_eq!(cell.cell_type, CellType::Grass);
        assert_eq!(cell.zone, ZoneType::None);
        assert!(cell.building_id.is_none());
    }
}

#[test]
fn test_prop_road_cells_have_road_type() {
    let mut rng = StdRng::seed_from_u64(SEED + 4);
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let road_types = all_road_types();
    let mut placed = Vec::new();
    for _ in 0..500 {
        let x = rng.gen_range(1..GRID_WIDTH - 1);
        let y = rng.gen_range(1..GRID_HEIGHT - 1);
        let rt = road_types[rng.gen_range(0..road_types.len())];
        if roads.place_road_typed(&mut grid, x, y, rt) {
            placed.push((x, y, rt));
        }
    }
    for (x, y, rt) in &placed {
        let cell = grid.get(*x, *y);
        assert_eq!(cell.cell_type, CellType::Road);
        assert_eq!(cell.road_type, *rt);
        assert!(roads.is_road(*x, *y));
    }
}

#[test]
fn test_prop_non_road_cells_not_in_network() {
    let mut rng = StdRng::seed_from_u64(SEED + 5);
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    for _ in 0..200 {
        let x = rng.gen_range(0..GRID_WIDTH);
        let y = rng.gen_range(0..GRID_HEIGHT);
        roads.place_road(&mut grid, x, y);
    }
    for _ in 0..NUM_SAMPLES {
        let x = rng.gen_range(0..GRID_WIDTH);
        let y = rng.gen_range(0..GRID_HEIGHT);
        if grid.get(x, y).cell_type != CellType::Road {
            assert!(!roads.is_road(x, y));
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Index / Bounds Invariants
// ---------------------------------------------------------------------------

#[test]
fn test_prop_index_bijection() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut rng = StdRng::seed_from_u64(SEED + 12);
    for _ in 0..NUM_SAMPLES {
        let x1 = rng.gen_range(0..GRID_WIDTH);
        let y1 = rng.gen_range(0..GRID_HEIGHT);
        let x2 = rng.gen_range(0..GRID_WIDTH);
        let y2 = rng.gen_range(0..GRID_HEIGHT);
        let idx1 = grid.index(x1, y1);
        let idx2 = grid.index(x2, y2);
        if (x1, y1) == (x2, y2) {
            assert_eq!(idx1, idx2);
        } else {
            assert_ne!(idx1, idx2);
        }
    }
}

#[test]
fn test_prop_in_bounds_matches_range() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut rng = StdRng::seed_from_u64(SEED + 13);
    for _ in 0..NUM_SAMPLES {
        let x = rng.gen_range(0..GRID_WIDTH + 10);
        let y = rng.gen_range(0..GRID_HEIGHT + 10);
        let expected = x < GRID_WIDTH && y < GRID_HEIGHT;
        assert_eq!(grid.in_bounds(x, y), expected);
    }
}

// ---------------------------------------------------------------------------
// 4. Neighbors4 Invariants
// ---------------------------------------------------------------------------

#[test]
fn test_prop_neighbors4_are_cardinal_and_in_bounds() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut rng = StdRng::seed_from_u64(SEED + 14);
    for _ in 0..NUM_SAMPLES {
        let x = rng.gen_range(0..GRID_WIDTH);
        let y = rng.gen_range(0..GRID_HEIGHT);
        let (neighbors, count) = grid.neighbors4(x, y);
        let expected_count = 4
            - (x == 0) as usize
            - (x == GRID_WIDTH - 1) as usize
            - (y == 0) as usize
            - (y == GRID_HEIGHT - 1) as usize;
        assert_eq!(count, expected_count);
        for i in 0..count {
            let (nx, ny) = neighbors[i];
            assert!(grid.in_bounds(nx, ny));
            let dx = (x as i32 - nx as i32).unsigned_abs();
            let dy = (y as i32 - ny as i32).unsigned_abs();
            assert!((dx == 1 && dy == 0) || (dx == 0 && dy == 1));
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Zone / Road Type Classification
// ---------------------------------------------------------------------------

#[test]
fn test_prop_zone_type_classification_is_consistent() {
    for zone in all_zone_types() {
        if zone == ZoneType::None {
            assert!(!zone.is_residential());
            assert!(!zone.is_commercial());
            assert!(!zone.is_job_zone());
            assert!(!zone.is_mixed_use());
            assert_eq!(zone.max_level(), 0);
            continue;
        }
        if zone.is_residential() {
            assert!(!zone.is_commercial());
        }
        if zone.is_commercial() {
            assert!(!zone.is_residential());
        }
        if zone.is_commercial() || zone == ZoneType::Industrial || zone == ZoneType::Office {
            assert!(zone.is_job_zone());
        }
        if zone.is_residential() && !zone.is_mixed_use() {
            assert!(!zone.is_job_zone());
        }
        assert!(zone.max_level() >= 1);
        assert!(zone.default_far() > 0.0);
    }
}

#[test]
fn test_prop_road_type_properties_are_positive() {
    for rt in all_road_types() {
        assert!(rt.speed() > 0.0);
        assert!(rt.cost() > 0.0);
        assert!(rt.maintenance_cost() > 0.0);
        assert!(rt.capacity() > 0);
        assert!(rt.width_cells() >= 1);
        if let Some(uc) = rt.upgrade_cost() {
            assert!(uc > 0.0);
        }
    }
}

#[test]
fn test_prop_road_upgrade_chain_terminates() {
    for start_rt in all_road_types() {
        let mut current = Some(start_rt);
        let mut steps = 0;
        while let Some(rt) = current {
            current = rt.upgrade_tier();
            steps += 1;
            assert!(steps <= 10);
        }
    }
}
