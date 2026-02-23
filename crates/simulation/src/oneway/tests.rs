#[cfg(test)]
mod tests {
    use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::WorldGrid;
    use crate::oneway::{OneWayDirection, OneWayDirectionMap};
    use crate::road_graph_csr::{csr_find_path, CsrGraph};
    use crate::road_segments::{RoadSegmentStore, SegmentId};
    use crate::roads::{RoadNetwork, RoadNode};
    use crate::Saveable;
    use bevy::prelude::*;

    #[test]
    fn test_toggle_cycles_directions() {
        let mut map = OneWayDirectionMap::default();
        let id = SegmentId(42);

        assert_eq!(map.get(id), None);

        map.toggle(id);
        assert_eq!(map.get(id), Some(OneWayDirection::Forward));

        map.toggle(id);
        assert_eq!(map.get(id), Some(OneWayDirection::Reverse));

        map.toggle(id);
        assert_eq!(map.get(id), None);
    }

    #[test]
    fn test_generation_increments_on_change() {
        let mut map = OneWayDirectionMap::default();
        let id = SegmentId(1);

        assert_eq!(map.generation, 0);
        map.set(id, OneWayDirection::Forward);
        assert_eq!(map.generation, 1);
        map.remove(id);
        assert_eq!(map.generation, 2);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut map = OneWayDirectionMap::default();
        map.set(SegmentId(10), OneWayDirection::Forward);
        map.set(SegmentId(20), OneWayDirection::Reverse);

        let bytes = map.save_to_bytes().unwrap();
        let loaded = OneWayDirectionMap::load_from_bytes(&bytes);

        assert_eq!(loaded.get(SegmentId(10)), Some(OneWayDirection::Forward));
        assert_eq!(loaded.get(SegmentId(20)), Some(OneWayDirection::Reverse));
        assert_eq!(loaded.directions.len(), 2);
    }

    #[test]
    fn test_saveable_empty_returns_none() {
        let map = OneWayDirectionMap::default();
        assert!(map.save_to_bytes().is_none());
    }

    #[test]
    fn test_oneway_forward_blocks_reverse_path() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();

        // Build a straight road from (5,10) to (15,10) using segments
        let from = Vec2::new(5.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let to = Vec2::new(15.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let (seg_id, _cells) = store.add_straight_segment(
            from,
            to,
            crate::grid::RoadType::Local,
            24.0,
            &mut grid,
            &mut roads,
        );

        // Without one-way: path should exist in both directions
        let csr_bidir = CsrGraph::from_road_network(&roads);
        let forward_path = csr_find_path(&csr_bidir, RoadNode(5, 10), RoadNode(15, 10));
        let reverse_path = csr_find_path(&csr_bidir, RoadNode(15, 10), RoadNode(5, 10));
        assert!(
            forward_path.is_some(),
            "Forward path should exist bidirectional"
        );
        assert!(
            reverse_path.is_some(),
            "Reverse path should exist bidirectional"
        );

        // Set one-way Forward (start -> end, i.e. 5,10 -> 15,10)
        let mut oneway_map = OneWayDirectionMap::default();
        oneway_map.set(seg_id, OneWayDirection::Forward);

        // Build blocked edges
        let mut blocked = std::collections::HashSet::new();
        for segment in &store.segments {
            if let Some(direction) = oneway_map.get(segment.id) {
                let cells = &segment.rasterized_cells;
                for window in cells.windows(2) {
                    let a = RoadNode(window[0].0, window[0].1);
                    let b = RoadNode(window[1].0, window[1].1);
                    match direction {
                        OneWayDirection::Forward => {
                            blocked.insert((b, a));
                        }
                        OneWayDirection::Reverse => {
                            blocked.insert((a, b));
                        }
                    }
                }
            }
        }

        let csr_oneway = CsrGraph::from_road_network_filtered(&roads, &blocked);

        // Forward path should still exist
        let forward_path = csr_find_path(&csr_oneway, RoadNode(5, 10), RoadNode(15, 10));
        assert!(
            forward_path.is_some(),
            "Forward path should exist with one-way forward"
        );

        // Reverse path should be blocked
        let reverse_path = csr_find_path(&csr_oneway, RoadNode(15, 10), RoadNode(5, 10));
        assert!(
            reverse_path.is_none(),
            "Reverse path should be blocked with one-way forward"
        );
    }

    #[test]
    fn test_oneway_reverse_blocks_forward_path() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();

        let from = Vec2::new(5.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let to = Vec2::new(15.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let (seg_id, _cells) = store.add_straight_segment(
            from,
            to,
            crate::grid::RoadType::Local,
            24.0,
            &mut grid,
            &mut roads,
        );

        let mut oneway_map = OneWayDirectionMap::default();
        oneway_map.set(seg_id, OneWayDirection::Reverse);

        let mut blocked = std::collections::HashSet::new();
        for segment in &store.segments {
            if let Some(direction) = oneway_map.get(segment.id) {
                let cells = &segment.rasterized_cells;
                for window in cells.windows(2) {
                    let a = RoadNode(window[0].0, window[0].1);
                    let b = RoadNode(window[1].0, window[1].1);
                    match direction {
                        OneWayDirection::Forward => {
                            blocked.insert((b, a));
                        }
                        OneWayDirection::Reverse => {
                            blocked.insert((a, b));
                        }
                    }
                }
            }
        }

        let csr_oneway = CsrGraph::from_road_network_filtered(&roads, &blocked);

        // Forward path should be blocked
        let forward_path = csr_find_path(&csr_oneway, RoadNode(5, 10), RoadNode(15, 10));
        assert!(
            forward_path.is_none(),
            "Forward path should be blocked with one-way reverse"
        );

        // Reverse path should exist
        let reverse_path = csr_find_path(&csr_oneway, RoadNode(15, 10), RoadNode(5, 10));
        assert!(
            reverse_path.is_some(),
            "Reverse path should exist with one-way reverse"
        );
    }

    #[test]
    fn test_removing_oneway_restores_bidirectional() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();

        let from = Vec2::new(5.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let to = Vec2::new(15.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let (seg_id, _cells) = store.add_straight_segment(
            from,
            to,
            crate::grid::RoadType::Local,
            24.0,
            &mut grid,
            &mut roads,
        );

        let mut oneway_map = OneWayDirectionMap::default();
        oneway_map.set(seg_id, OneWayDirection::Forward);
        oneway_map.remove(seg_id);

        // No blocked edges since we removed the one-way
        let blocked = std::collections::HashSet::new();
        let csr = CsrGraph::from_road_network_filtered(&roads, &blocked);

        let forward = csr_find_path(&csr, RoadNode(5, 10), RoadNode(15, 10));
        let reverse = csr_find_path(&csr, RoadNode(15, 10), RoadNode(5, 10));
        assert!(
            forward.is_some(),
            "Forward path should exist after removing one-way"
        );
        assert!(
            reverse.is_some(),
            "Reverse path should exist after removing one-way"
        );
    }
}
