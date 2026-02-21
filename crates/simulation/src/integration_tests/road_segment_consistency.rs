use crate::grid::{CellType, RoadType};
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Road segment store / grid cache consistency
// ---------------------------------------------------------------------------

/// After adding road segments, verify segment rasterized cells match grid.
#[test]
fn test_road_segment_grid_cache_consistency_after_placement() {
    use std::collections::HashSet;

    let city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_road(30, 10, 30, 30, RoadType::Avenue)
        .with_road(10, 30, 30, 30, RoadType::Highway);

    let grid = city.grid();
    let segments = city.road_segments();

    let mut segment_cells: HashSet<(usize, usize)> = HashSet::new();
    for seg in &segments.segments {
        for &(cx, cy) in &seg.rasterized_cells {
            segment_cells.insert((cx, cy));
        }
    }

    for &(cx, cy) in &segment_cells {
        if grid.in_bounds(cx, cy) {
            assert_eq!(
                grid.get(cx, cy).cell_type,
                CellType::Road,
                "Segment claims cell ({}, {}) is rasterized, but grid says {:?}",
                cx,
                cy,
                grid.get(cx, cy).cell_type,
            );
        }
    }

    if !segment_cells.is_empty() {
        let min_x = segment_cells.iter().map(|c| c.0).min().unwrap();
        let max_x = segment_cells.iter().map(|c| c.0).max().unwrap();
        let min_y = segment_cells.iter().map(|c| c.1).min().unwrap();
        let max_y = segment_cells.iter().map(|c| c.1).max().unwrap();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if grid.in_bounds(x, y) && grid.get(x, y).cell_type == CellType::Road {
                    assert!(
                        segment_cells.contains(&(x, y)),
                        "Grid has road at ({}, {}) but no segment claims it",
                        x,
                        y,
                    );
                }
            }
        }
    }
}

/// After adding road segments, verify road network has nodes for all road cells.
#[test]
fn test_road_network_nodes_match_grid_roads_after_segment_placement() {
    let city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_road(20, 10, 20, 20, RoadType::Local);

    let grid = city.grid();
    let network = city.road_network();

    for y in 0..grid.height {
        for x in 0..grid.width {
            if grid.get(x, y).cell_type == CellType::Road {
                let node = crate::roads::RoadNode(x, y);
                assert!(
                    network.edges.contains_key(&node),
                    "Grid has road at ({}, {}) but RoadNetwork has no node for it",
                    x,
                    y,
                );
            }
        }
    }
}
