#[cfg(test)]
mod tests {
    use super::super::types::INTERSECTION_SNAP_RADIUS;
    use simulation::config::CELL_SIZE;
    use simulation::road_segments::{SegmentNode, SegmentNodeId};

    /// Helper: create a snap resource and test snapping logic directly.
    fn find_snap_target(
        cursor_pos: bevy::math::Vec2,
        nodes: &[SegmentNode],
    ) -> Option<bevy::math::Vec2> {
        let mut best_dist = INTERSECTION_SNAP_RADIUS;
        let mut best_pos: Option<bevy::math::Vec2> = None;
        for node in nodes {
            let dist = (node.position - cursor_pos).length();
            if dist < best_dist {
                best_dist = dist;
                best_pos = Some(node.position);
            }
        }
        best_pos
    }

    #[test]
    fn test_intersection_snap_within_radius() {
        let node_pos = bevy::math::Vec2::new(100.0, 200.0);
        let nodes = vec![SegmentNode {
            id: SegmentNodeId(0),
            position: node_pos,
            connected_segments: vec![],
        }];

        // Cursor within 1 cell distance (CELL_SIZE = 16.0)
        let cursor_pos = bevy::math::Vec2::new(110.0, 200.0); // 10 units away < 16
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, Some(node_pos));
    }

    #[test]
    fn test_intersection_snap_outside_radius() {
        let node_pos = bevy::math::Vec2::new(100.0, 200.0);
        let nodes = vec![SegmentNode {
            id: SegmentNodeId(0),
            position: node_pos,
            connected_segments: vec![],
        }];

        // Cursor more than 1 cell away
        let cursor_pos = bevy::math::Vec2::new(120.0, 200.0); // 20 units away > 16
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, None);
    }

    #[test]
    fn test_intersection_snap_picks_closest_node() {
        let node_a = bevy::math::Vec2::new(100.0, 200.0);
        let node_b = bevy::math::Vec2::new(108.0, 200.0);
        let nodes = vec![
            SegmentNode {
                id: SegmentNodeId(0),
                position: node_a,
                connected_segments: vec![],
            },
            SegmentNode {
                id: SegmentNodeId(1),
                position: node_b,
                connected_segments: vec![],
            },
        ];

        // Cursor at 105, equidistant-ish but closer to node_b
        let cursor_pos = bevy::math::Vec2::new(106.0, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, Some(node_b)); // 2 units away vs 6 units
    }

    #[test]
    fn test_intersection_snap_no_nodes() {
        let nodes: Vec<SegmentNode> = vec![];
        let cursor_pos = bevy::math::Vec2::new(100.0, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, None);
    }

    #[test]
    fn test_intersection_snap_exact_position() {
        let node_pos = bevy::math::Vec2::new(100.0, 200.0);
        let nodes = vec![SegmentNode {
            id: SegmentNodeId(0),
            position: node_pos,
            connected_segments: vec![],
        }];

        // Cursor exactly at node position
        let cursor_pos = bevy::math::Vec2::new(100.0, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, Some(node_pos));
    }

    #[test]
    fn test_intersection_snap_at_boundary() {
        let node_pos = bevy::math::Vec2::new(100.0, 200.0);
        let nodes = vec![SegmentNode {
            id: SegmentNodeId(0),
            position: node_pos,
            connected_segments: vec![],
        }];

        // Cursor at exactly CELL_SIZE distance (should NOT snap since we use strict <)
        let cursor_pos = bevy::math::Vec2::new(100.0 + CELL_SIZE, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, None);

        // Just inside the radius
        let cursor_pos = bevy::math::Vec2::new(100.0 + CELL_SIZE - 0.1, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, Some(node_pos));
    }
}
