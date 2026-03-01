//! Integration tests for water coverage seep radius (issue #1959).
//!
//! Verifies that zones placed up to 2 cells from a road still receive
//! water coverage from the BFS propagation in `utilities.rs`.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::CellType;
use crate::roads::RoadNetwork;
use crate::utilities::{UtilitySource, UtilityType};

// ====================================================================
// Zone 1 cell from road gets water (baseline)
// ====================================================================

/// A grass cell directly adjacent to a road with a water tower should
/// receive water coverage. This is the baseline behavior.
#[test]
fn test_water_covers_zone_1_cell_from_road() {
    let mut grid = crate::grid::WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Place a horizontal road at y=50
    for x in 50..=60 {
        roads.place_road(&mut grid, x, 50);
    }

    // Grass cell at (55, 49) -- 1 cell north of road
    assert_eq!(grid.get(55, 49).cell_type, CellType::Grass);

    let source = UtilitySource {
        utility_type: UtilityType::WaterTower,
        grid_x: 50,
        grid_y: 50,
        range: 20,
    };

    let mut visited = vec![false; GRID_WIDTH * GRID_HEIGHT];
    crate::utilities::bfs_propagate_pub(&mut grid, &source, source.range, &mut visited);

    assert!(
        grid.get(55, 49).has_water,
        "Grass cell 1 cell from road should have water"
    );
}

// ====================================================================
// Zone 2 cells from road gets water (the fix)
// ====================================================================

/// A grass cell 2 cells away from a road (Manhattan distance) should
/// now receive water coverage with the extended seep radius.
#[test]
fn test_water_covers_zone_2_cells_from_road() {
    let mut grid = crate::grid::WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Place a horizontal road at y=50
    for x in 50..=60 {
        roads.place_road(&mut grid, x, 50);
    }

    // Grass cell at (55, 48) -- 2 cells north of road
    assert_eq!(grid.get(55, 48).cell_type, CellType::Grass);

    let source = UtilitySource {
        utility_type: UtilityType::WaterTower,
        grid_x: 50,
        grid_y: 50,
        range: 20,
    };

    let mut visited = vec![false; GRID_WIDTH * GRID_HEIGHT];
    crate::utilities::bfs_propagate_pub(&mut grid, &source, source.range, &mut visited);

    assert!(
        grid.get(55, 48).has_water,
        "Grass cell 2 cells from road should have water (seep radius fix)"
    );
}

// ====================================================================
// Zone 3 cells from road does NOT get water
// ====================================================================

/// A grass cell 3 cells away from a road should NOT receive water
/// coverage, as the seep radius is 2.
#[test]
fn test_water_does_not_cover_zone_3_cells_from_road() {
    let mut grid = crate::grid::WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Place a horizontal road at y=50
    for x in 50..=60 {
        roads.place_road(&mut grid, x, 50);
    }

    // Grass cell at (55, 47) -- 3 cells north of road
    assert_eq!(grid.get(55, 47).cell_type, CellType::Grass);

    let source = UtilitySource {
        utility_type: UtilityType::WaterTower,
        grid_x: 50,
        grid_y: 50,
        range: 20,
    };

    let mut visited = vec![false; GRID_WIDTH * GRID_HEIGHT];
    crate::utilities::bfs_propagate_pub(&mut grid, &source, source.range, &mut visited);

    assert!(
        !grid.get(55, 47).has_water,
        "Grass cell 3 cells from road should NOT have water"
    );
}

// ====================================================================
// Diagonal seep at Manhattan distance 2
// ====================================================================

/// A grass cell at diagonal offset (1, 1) from the nearest road cell
/// (Manhattan distance 2) should receive water coverage.
#[test]
fn test_water_covers_diagonal_at_manhattan_2() {
    let mut grid = crate::grid::WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Place a single road cell so we can precisely control distances
    roads.place_road(&mut grid, 50, 50);

    // (51, 51) is at Manhattan distance |1|+|1| = 2 from (50, 50)
    assert_eq!(grid.get(51, 51).cell_type, CellType::Grass);

    let source = UtilitySource {
        utility_type: UtilityType::WaterTower,
        grid_x: 50,
        grid_y: 50,
        range: 20,
    };

    let mut visited = vec![false; GRID_WIDTH * GRID_HEIGHT];
    crate::utilities::bfs_propagate_pub(&mut grid, &source, source.range, &mut visited);

    assert!(
        grid.get(51, 51).has_water,
        "Grass cell at diagonal Manhattan distance 2 from road should have water"
    );

    // (52, 50) is at Manhattan distance 2 straight east -- should also have water
    assert!(
        grid.get(52, 50).has_water,
        "Grass cell 2 cells east of road should have water"
    );

    // (52, 51) is at Manhattan distance 3 -- should NOT have water
    assert!(
        !grid.get(52, 51).has_water,
        "Grass cell at Manhattan distance 3 should NOT have water"
    );
}
