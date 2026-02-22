//! TEST-037: Unit Tests for Road Placement
//!
//! Tests road placement: cell type changes to Road, road type set correctly,
//! neighbors updated, road network updated, CSR graph rebuilt, and placement
//! on water fails.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::road_graph_csr::CsrGraph;
use crate::roads::{RoadNetwork, RoadNode};
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// 1. place_road changes cell_type to Road
// ---------------------------------------------------------------------------

#[test]
fn test_place_road_changes_cell_type_to_road() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    assert_eq!(grid.get(50, 50).cell_type, CellType::Grass);
    let ok = roads.place_road(&mut grid, 50, 50);
    assert!(ok, "place_road should return true on success");
    assert_eq!(
        grid.get(50, 50).cell_type,
        CellType::Road,
        "Cell type should become Road after placement"
    );
}

#[test]
fn test_place_road_typed_changes_cell_type_to_road() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    for road_type in [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
        RoadType::Path,
    ] {
        let x = 50 + road_type as usize;
        let ok = roads.place_road_typed(&mut grid, x, 50, road_type);
        assert!(ok, "place_road_typed should succeed for {:?}", road_type);
        assert_eq!(
            grid.get(x, 50).cell_type,
            CellType::Road,
            "Cell type should be Road after placing {:?}",
            road_type
        );
    }
}

// ---------------------------------------------------------------------------
// 2. road_type matches requested type
// ---------------------------------------------------------------------------

#[test]
fn test_road_type_matches_requested_local() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road_typed(&mut grid, 50, 50, RoadType::Local);
    assert_eq!(
        grid.get(50, 50).road_type,
        RoadType::Local,
        "Road type should be Local"
    );
}

#[test]
fn test_road_type_matches_requested_avenue() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road_typed(&mut grid, 50, 50, RoadType::Avenue);
    assert_eq!(
        grid.get(50, 50).road_type,
        RoadType::Avenue,
        "Road type should be Avenue"
    );
}

#[test]
fn test_road_type_matches_requested_boulevard() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road_typed(&mut grid, 50, 50, RoadType::Boulevard);
    assert_eq!(
        grid.get(50, 50).road_type,
        RoadType::Boulevard,
        "Road type should be Boulevard"
    );
}

#[test]
fn test_road_type_matches_requested_highway() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road_typed(&mut grid, 50, 50, RoadType::Highway);
    assert_eq!(
        grid.get(50, 50).road_type,
        RoadType::Highway,
        "Road type should be Highway"
    );
}

#[test]
fn test_road_type_matches_requested_oneway() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road_typed(&mut grid, 50, 50, RoadType::OneWay);
    assert_eq!(
        grid.get(50, 50).road_type,
        RoadType::OneWay,
        "Road type should be OneWay"
    );
}

#[test]
fn test_road_type_matches_requested_path() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road_typed(&mut grid, 50, 50, RoadType::Path);
    assert_eq!(
        grid.get(50, 50).road_type,
        RoadType::Path,
        "Road type should be Path"
    );
}

#[test]
fn test_road_type_matches_all_types_systematically() {
    let all_types = [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
        RoadType::Path,
    ];

    for (i, road_type) in all_types.iter().enumerate() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        let x = 50 + i;
        roads.place_road_typed(&mut grid, x, 50, *road_type);
        assert_eq!(
            grid.get(x, 50).road_type,
            *road_type,
            "Road type at ({}, 50) should be {:?}",
            x,
            road_type
        );
    }
}

// ---------------------------------------------------------------------------
// 3. road_network has edge for placed road
// ---------------------------------------------------------------------------

#[test]
fn test_road_network_has_node_after_single_placement() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    let node = RoadNode(50, 50);
    assert!(
        roads.edges.contains_key(&node),
        "RoadNetwork should have a node for placed road"
    );
}

#[test]
fn test_road_network_has_edges_for_adjacent_roads() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 51, 50);

    let node_a = RoadNode(50, 50);
    let node_b = RoadNode(51, 50);

    // A should be connected to B
    let neighbors_a = roads.neighbors(&node_a);
    assert!(
        neighbors_a.contains(&node_b),
        "Node A should be connected to adjacent Node B"
    );

    // B should be connected to A (bidirectional)
    let neighbors_b = roads.neighbors(&node_b);
    assert!(
        neighbors_b.contains(&node_a),
        "Node B should be connected to adjacent Node A"
    );
}

#[test]
fn test_road_network_connected_line() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Place a line of 5 roads: (50,50) through (54,50)
    for x in 50..=54 {
        roads.place_road(&mut grid, x, 50);
    }

    // Interior nodes should have exactly 2 neighbors
    for x in 51..=53 {
        let neighbors = roads.neighbors(&RoadNode(x, 50));
        assert_eq!(
            neighbors.len(),
            2,
            "Interior road at ({}, 50) should have 2 neighbors, got {}",
            x,
            neighbors.len()
        );
    }

    // Endpoints should have exactly 1 neighbor
    assert_eq!(
        roads.neighbors(&RoadNode(50, 50)).len(),
        1,
        "Left endpoint should have 1 neighbor"
    );
    assert_eq!(
        roads.neighbors(&RoadNode(54, 50)).len(),
        1,
        "Right endpoint should have 1 neighbor"
    );
}

#[test]
fn test_road_network_is_road_returns_true() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    assert!(
        roads.is_road(50, 50),
        "is_road should return true for placed road"
    );
    assert!(
        !roads.is_road(60, 60),
        "is_road should return false for non-road cell"
    );
}

// ---------------------------------------------------------------------------
// 4. Adjacent cells become road-adjacent (neighbors updated)
// ---------------------------------------------------------------------------

#[test]
fn test_adjacent_cells_connected_when_road_placed_nearby() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Place first road
    roads.place_road(&mut grid, 50, 50);
    assert_eq!(
        roads.neighbors(&RoadNode(50, 50)).len(),
        0,
        "Isolated road should have no neighbors"
    );

    // Place adjacent road
    roads.place_road(&mut grid, 50, 51);
    assert_eq!(
        roads.neighbors(&RoadNode(50, 50)).len(),
        1,
        "Road at (50,50) should now have 1 neighbor after placing (50,51)"
    );
    assert_eq!(
        roads.neighbors(&RoadNode(50, 51)).len(),
        1,
        "Road at (50,51) should have 1 neighbor"
    );
}

#[test]
fn test_non_adjacent_roads_not_connected() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 52, 50); // gap at (51, 50)

    assert_eq!(
        roads.neighbors(&RoadNode(50, 50)).len(),
        0,
        "Roads with a gap between them should not be connected"
    );
    assert_eq!(
        roads.neighbors(&RoadNode(52, 50)).len(),
        0,
        "Roads with a gap between them should not be connected"
    );
}

#[test]
fn test_filling_gap_connects_roads() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 52, 50);

    // Before filling the gap
    assert_eq!(roads.neighbors(&RoadNode(50, 50)).len(), 0);
    assert_eq!(roads.neighbors(&RoadNode(52, 50)).len(), 0);

    // Fill the gap
    roads.place_road(&mut grid, 51, 50);

    // After filling, all three should be connected
    assert_eq!(
        roads.neighbors(&RoadNode(50, 50)).len(),
        1,
        "(50,50) should connect to (51,50)"
    );
    assert_eq!(
        roads.neighbors(&RoadNode(51, 50)).len(),
        2,
        "(51,50) should connect to both (50,50) and (52,50)"
    );
    assert_eq!(
        roads.neighbors(&RoadNode(52, 50)).len(),
        1,
        "(52,50) should connect to (51,50)"
    );
}

#[test]
fn test_cross_intersection_neighbors() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Build a cross: center at (50,50)
    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 49, 50);
    roads.place_road(&mut grid, 51, 50);
    roads.place_road(&mut grid, 50, 49);
    roads.place_road(&mut grid, 50, 51);

    let center_neighbors = roads.neighbors(&RoadNode(50, 50));
    assert_eq!(
        center_neighbors.len(),
        4,
        "Center of cross should have 4 neighbors"
    );
    assert!(center_neighbors.contains(&RoadNode(49, 50)));
    assert!(center_neighbors.contains(&RoadNode(51, 50)));
    assert!(center_neighbors.contains(&RoadNode(50, 49)));
    assert!(center_neighbors.contains(&RoadNode(50, 51)));
}

#[test]
fn test_diagonal_roads_not_connected() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Diagonal placement: only 4-connectivity is used
    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 51, 51);

    assert_eq!(
        roads.neighbors(&RoadNode(50, 50)).len(),
        0,
        "Diagonally adjacent roads should NOT be connected (4-connectivity)"
    );
    assert_eq!(
        roads.neighbors(&RoadNode(51, 51)).len(),
        0,
        "Diagonally adjacent roads should NOT be connected (4-connectivity)"
    );
}

// ---------------------------------------------------------------------------
// 5. Place road on water fails
// ---------------------------------------------------------------------------

#[test]
fn test_place_road_on_water_fails() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    grid.get_mut(50, 50).cell_type = CellType::Water;

    let ok = roads.place_road(&mut grid, 50, 50);
    assert!(!ok, "place_road should return false on Water cell");
    assert_eq!(
        grid.get(50, 50).cell_type,
        CellType::Water,
        "Water cell should remain Water after failed placement"
    );
    assert!(
        !roads.is_road(50, 50),
        "RoadNetwork should NOT have a node on Water"
    );
}

#[test]
fn test_place_road_typed_on_water_fails() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    grid.get_mut(50, 50).cell_type = CellType::Water;

    for road_type in [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
        RoadType::Path,
    ] {
        let ok = roads.place_road_typed(&mut grid, 50, 50, road_type);
        assert!(
            !ok,
            "place_road_typed should return false for {:?} on Water",
            road_type
        );
    }

    assert_eq!(
        grid.get(50, 50).cell_type,
        CellType::Water,
        "Water cell should remain unchanged"
    );
}

// ---------------------------------------------------------------------------
// 6. Duplicate road placement fails
// ---------------------------------------------------------------------------

#[test]
fn test_duplicate_road_placement_returns_false() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    assert!(roads.place_road(&mut grid, 50, 50));
    assert!(
        !roads.place_road(&mut grid, 50, 50),
        "Placing road on existing road cell should return false"
    );

    // Cell should still be road
    assert_eq!(grid.get(50, 50).cell_type, CellType::Road);
}

// ---------------------------------------------------------------------------
// 7. Out-of-bounds placement fails
// ---------------------------------------------------------------------------

#[test]
fn test_place_road_out_of_bounds_fails() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // These coordinates are out of bounds (>= 256)
    let ok = roads.place_road(&mut grid, GRID_WIDTH, 0);
    assert!(!ok, "Placing road at x=GRID_WIDTH should fail");

    let ok = roads.place_road(&mut grid, 0, GRID_HEIGHT);
    assert!(!ok, "Placing road at y=GRID_HEIGHT should fail");
}

// ---------------------------------------------------------------------------
// 8. Intersection detection
// ---------------------------------------------------------------------------

#[test]
fn test_intersection_not_detected_for_straight_line() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Straight line: no node should have 3+ neighbors
    for x in 50..=55 {
        roads.place_road(&mut grid, x, 50);
    }

    for x in 50..=55 {
        assert!(
            !roads.intersections.contains(&RoadNode(x, 50)),
            "Node ({}, 50) in a straight line should NOT be an intersection",
            x
        );
    }
}

#[test]
fn test_intersection_detected_at_t_junction() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // T-junction: horizontal line + one vertical branch
    roads.place_road(&mut grid, 49, 50);
    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 51, 50);
    roads.place_road(&mut grid, 50, 51);

    assert!(
        roads.intersections.contains(&RoadNode(50, 50)),
        "Center of T-junction should be detected as intersection"
    );
}

#[test]
fn test_intersection_detected_at_cross() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Cross: 4 branches meeting at center
    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 49, 50);
    roads.place_road(&mut grid, 51, 50);
    roads.place_road(&mut grid, 50, 49);
    roads.place_road(&mut grid, 50, 51);

    assert!(
        roads.intersections.contains(&RoadNode(50, 50)),
        "Center of cross should be detected as intersection"
    );
}

// ---------------------------------------------------------------------------
// 9. Road removal clears cell and disconnects network
// ---------------------------------------------------------------------------

#[test]
fn test_remove_road_resets_cell_type() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    assert_eq!(grid.get(50, 50).cell_type, CellType::Road);

    roads.remove_road(&mut grid, 50, 50);
    assert_eq!(
        grid.get(50, 50).cell_type,
        CellType::Grass,
        "Cell should revert to Grass after road removal"
    );
}

#[test]
fn test_remove_road_clears_zone_and_building() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    // Manually set zone/building to test they get cleared
    grid.get_mut(50, 50).zone = ZoneType::ResidentialLow;

    roads.remove_road(&mut grid, 50, 50);
    assert_eq!(
        grid.get(50, 50).zone,
        ZoneType::None,
        "Zone should be cleared after road removal"
    );
    assert!(
        grid.get(50, 50).building_id.is_none(),
        "Building ID should be cleared after road removal"
    );
}

#[test]
fn test_remove_road_disconnects_neighbors() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 51, 50);
    roads.place_road(&mut grid, 52, 50);

    // Remove the middle road
    roads.remove_road(&mut grid, 51, 50);

    assert!(
        !roads.is_road(51, 50),
        "Removed road should no longer be in the network"
    );
    assert_eq!(
        roads.neighbors(&RoadNode(50, 50)).len(),
        0,
        "Left neighbor should be disconnected"
    );
    assert_eq!(
        roads.neighbors(&RoadNode(52, 50)).len(),
        0,
        "Right neighbor should be disconnected"
    );
}

#[test]
fn test_remove_road_tracks_recently_removed() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    roads.remove_road(&mut grid, 50, 50);

    assert_eq!(
        roads.recently_removed.len(),
        1,
        "recently_removed should track the removed node"
    );
    assert_eq!(roads.recently_removed[0], RoadNode(50, 50));
}

#[test]
fn test_drain_removed_clears_list() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 51, 50);
    roads.remove_road(&mut grid, 50, 50);
    roads.remove_road(&mut grid, 51, 50);

    let removed = roads.drain_removed();
    assert_eq!(removed.len(), 2, "drain_removed should return all removed");
    assert!(
        roads.recently_removed.is_empty(),
        "recently_removed should be empty after drain"
    );
}

#[test]
fn test_remove_nonexistent_road_returns_false() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    let ok = roads.remove_road(&mut grid, 50, 50);
    assert!(!ok, "Removing non-existent road should return false");
}

// ---------------------------------------------------------------------------
// 10. Intersection status updated after removal
// ---------------------------------------------------------------------------

#[test]
fn test_intersection_cleared_after_branch_removal() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Build a T-junction at (50,50)
    roads.place_road(&mut grid, 49, 50);
    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 51, 50);
    roads.place_road(&mut grid, 50, 51);

    assert!(
        roads.intersections.contains(&RoadNode(50, 50)),
        "Should be intersection before removal"
    );

    // Remove one branch
    roads.remove_road(&mut grid, 50, 51);

    assert!(
        !roads.intersections.contains(&RoadNode(50, 50)),
        "Should no longer be intersection after branch removal (2 neighbors left)"
    );
}

// ---------------------------------------------------------------------------
// 11. CSR graph rebuilt from road network
// ---------------------------------------------------------------------------

#[test]
fn test_csr_graph_contains_placed_roads() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 51, 50);
    roads.place_road(&mut grid, 52, 50);

    let csr = CsrGraph::from_road_network(&roads);

    assert_eq!(
        csr.node_count(),
        3,
        "CSR should have 3 nodes for 3 placed roads"
    );
    assert!(
        csr.nodes.contains(&RoadNode(50, 50)),
        "CSR should contain node (50,50)"
    );
    assert!(
        csr.nodes.contains(&RoadNode(51, 50)),
        "CSR should contain node (51,50)"
    );
    assert!(
        csr.nodes.contains(&RoadNode(52, 50)),
        "CSR should contain node (52,50)"
    );
}

#[test]
fn test_csr_graph_reflects_connectivity() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 51, 50);

    let csr = CsrGraph::from_road_network(&roads);

    // Both nodes should have exactly 1 edge each (connected to each other)
    assert_eq!(csr.node_count(), 2);
    // Total edges should be 2 (one in each direction)
    assert_eq!(
        csr.edges.len(),
        2,
        "CSR should have 2 edges for a bidirectional connection"
    );
}

#[test]
fn test_csr_graph_empty_after_all_roads_removed() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 51, 50);
    roads.remove_road(&mut grid, 50, 50);
    roads.remove_road(&mut grid, 51, 50);

    let csr = CsrGraph::from_road_network(&roads);
    assert_eq!(
        csr.node_count(),
        0,
        "CSR should have 0 nodes after removing all roads"
    );
}

// ---------------------------------------------------------------------------
// 12. TestCity harness: road placement via with_road
// ---------------------------------------------------------------------------

#[test]
fn test_testcity_with_road_creates_road_cells() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let grid = city.grid();
    let mut road_count = 0;
    for y in 100..=110 {
        if grid.get(100, y).cell_type == CellType::Road {
            road_count += 1;
        }
    }
    assert!(
        road_count > 0,
        "TestCity::with_road should create road cells in the grid"
    );
}

#[test]
fn test_testcity_with_road_updates_network() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let network = city.road_network();
    assert!(
        !network.edges.is_empty(),
        "TestCity::with_road should populate the RoadNetwork"
    );
}

#[test]
fn test_testcity_with_road_creates_segments() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    assert_eq!(
        city.segment_count(),
        1,
        "TestCity::with_road should create 1 segment"
    );
}

#[test]
fn test_testcity_with_road_road_type_preserved() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Avenue);

    assert_eq!(
        city.segment_road_type(0),
        Some(RoadType::Avenue),
        "Segment should retain the requested road type"
    );

    // Check at least one grid cell has the correct road type
    let grid = city.grid();
    let mut found_avenue = false;
    for y in 100..=110 {
        if grid.get(100, y).cell_type == CellType::Road
            && grid.get(100, y).road_type == RoadType::Avenue
        {
            found_avenue = true;
            break;
        }
    }
    assert!(found_avenue, "Grid cells should have RoadType::Avenue");
}

#[test]
fn test_testcity_rebuild_csr_populates_graph() {
    let city = TestCity::new()
        .with_road(100, 100, 110, 100, RoadType::Local)
        .rebuild_csr();

    let csr = city.resource::<CsrGraph>();
    assert!(
        csr.node_count() > 0,
        "CSR graph should have nodes after rebuild"
    );
}

// ---------------------------------------------------------------------------
// 13. Road placement does not affect water adjacent to road
// ---------------------------------------------------------------------------

#[test]
fn test_road_adjacent_to_water_does_not_modify_water() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    grid.get_mut(50, 49).cell_type = CellType::Water;
    roads.place_road(&mut grid, 50, 50);

    assert_eq!(
        grid.get(50, 49).cell_type,
        CellType::Water,
        "Water cell adjacent to placed road should remain Water"
    );
    assert_eq!(
        grid.get(50, 50).cell_type,
        CellType::Road,
        "Road should be placed successfully next to Water"
    );
}

// ---------------------------------------------------------------------------
// 14. Boundary / edge cell placement
// ---------------------------------------------------------------------------

#[test]
fn test_place_road_at_grid_corners() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Top-left corner
    assert!(roads.place_road(&mut grid, 0, 0));
    assert_eq!(grid.get(0, 0).cell_type, CellType::Road);

    // Bottom-right corner
    assert!(roads.place_road(&mut grid, GRID_WIDTH - 1, GRID_HEIGHT - 1));
    assert_eq!(
        grid.get(GRID_WIDTH - 1, GRID_HEIGHT - 1).cell_type,
        CellType::Road
    );

    // Top-right corner
    assert!(roads.place_road(&mut grid, GRID_WIDTH - 1, 0));
    assert_eq!(grid.get(GRID_WIDTH - 1, 0).cell_type, CellType::Road);

    // Bottom-left corner
    assert!(roads.place_road(&mut grid, 0, GRID_HEIGHT - 1));
    assert_eq!(grid.get(0, GRID_HEIGHT - 1).cell_type, CellType::Road);
}

#[test]
fn test_place_road_at_edge_has_correct_neighbor_count() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Place a road at the left edge with neighbors above and below
    roads.place_road(&mut grid, 0, 50);
    roads.place_road(&mut grid, 0, 51);
    roads.place_road(&mut grid, 1, 50);

    let neighbors = roads.neighbors(&RoadNode(0, 50));
    assert_eq!(
        neighbors.len(),
        2,
        "Edge road with 2 adjacent roads should have 2 neighbors (not 3, since one side is boundary)"
    );
}

// ---------------------------------------------------------------------------
// 15. Default road_type for place_road (no type arg)
// ---------------------------------------------------------------------------

#[test]
fn test_place_road_defaults_to_local() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    roads.place_road(&mut grid, 50, 50);
    assert_eq!(
        grid.get(50, 50).road_type,
        RoadType::Local,
        "place_road() without type should default to Local"
    );
}
