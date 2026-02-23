//! Tests for the superblock module.

#[cfg(test)]
mod tests {
    use crate::superblock::constants::*;
    use crate::superblock::state::SuperblockState;
    use crate::superblock::types::{Superblock, SuperblockCell};

    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    // -------------------------------------------------------------------------
    // Superblock geometry tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_superblock_new_sorts_coordinates() {
        let sb = Superblock::new(10, 15, 5, 8, "test".to_string());
        assert_eq!(sb.x0, 5);
        assert_eq!(sb.y0, 8);
        assert_eq!(sb.x1, 10);
        assert_eq!(sb.y1, 15);
    }

    #[test]
    fn test_superblock_dimensions() {
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        assert_eq!(sb.width(), 5);
        assert_eq!(sb.height(), 5);
        assert_eq!(sb.area(), 25);
    }

    #[test]
    fn test_superblock_valid() {
        // 3x3 is minimum valid size
        let sb = Superblock::new(10, 10, 12, 12, "test".to_string());
        assert!(sb.is_valid());
    }

    #[test]
    fn test_superblock_too_small() {
        // 2x3 is too narrow
        let sb = Superblock::new(10, 10, 11, 12, "test".to_string());
        assert!(!sb.is_valid());
    }

    #[test]
    fn test_superblock_contains() {
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        assert!(sb.contains(10, 10));
        assert!(sb.contains(12, 12));
        assert!(sb.contains(14, 14));
        assert!(!sb.contains(9, 10));
        assert!(!sb.contains(10, 15));
    }

    #[test]
    fn test_superblock_perimeter() {
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        // Corners are perimeter
        assert!(sb.is_perimeter(10, 10));
        assert!(sb.is_perimeter(14, 14));
        // Edges are perimeter
        assert!(sb.is_perimeter(12, 10));
        assert!(sb.is_perimeter(10, 12));
        // Center is not perimeter
        assert!(!sb.is_perimeter(12, 12));
        // Outside is not perimeter
        assert!(!sb.is_perimeter(9, 10));
    }

    #[test]
    fn test_superblock_interior() {
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        // Center cells are interior
        assert!(sb.is_interior(11, 11));
        assert!(sb.is_interior(12, 12));
        assert!(sb.is_interior(13, 13));
        // Edges are not interior
        assert!(!sb.is_interior(10, 12));
        assert!(!sb.is_interior(14, 12));
        assert!(!sb.is_interior(12, 10));
        assert!(!sb.is_interior(12, 14));
    }

    #[test]
    fn test_superblock_3x3_has_one_interior() {
        let sb = Superblock::new(10, 10, 12, 12, "test".to_string());
        // Only center cell (11,11) is interior
        assert!(sb.is_interior(11, 11));
        // All edges/corners are perimeter
        for x in 10..=12 {
            for y in 10..=12 {
                if x == 11 && y == 11 {
                    continue;
                }
                assert!(sb.is_perimeter(x, y), "({x},{y}) should be perimeter");
            }
        }
    }

    // -------------------------------------------------------------------------
    // SuperblockState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_default_empty() {
        let state = SuperblockState::default();
        assert!(state.superblocks.is_empty());
        assert_eq!(state.total_interior_cells, 0);
        assert_eq!(state.total_coverage_cells, 0);
        assert!(state.coverage_ratio.abs() < f32::EPSILON);
    }

    #[test]
    fn test_state_add_superblock() {
        let mut state = SuperblockState::default();
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        assert!(state.add_superblock(sb));
        assert_eq!(state.superblocks.len(), 1);
        assert!(state.total_interior_cells > 0);
        assert!(state.total_coverage_cells > 0);
    }

    #[test]
    fn test_state_reject_too_small() {
        let mut state = SuperblockState::default();
        let sb = Superblock::new(10, 10, 11, 11, "tiny".to_string());
        assert!(!state.add_superblock(sb));
        assert!(state.superblocks.is_empty());
    }

    #[test]
    fn test_state_remove_superblock() {
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));
        assert!(state.remove_superblock(0));
        assert!(state.superblocks.is_empty());
        assert_eq!(state.total_interior_cells, 0);
    }

    #[test]
    fn test_state_remove_invalid_index() {
        let mut state = SuperblockState::default();
        assert!(!state.remove_superblock(0));
    }

    #[test]
    fn test_state_cell_classification() {
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));

        // Interior
        assert_eq!(state.get_cell(12, 12), SuperblockCell::Interior);
        assert!(state.is_interior(12, 12));
        assert!(state.is_in_superblock(12, 12));

        // Perimeter
        assert_eq!(state.get_cell(10, 10), SuperblockCell::Perimeter);
        assert!(!state.is_interior(10, 10));
        assert!(state.is_in_superblock(10, 10));

        // Outside
        assert_eq!(state.get_cell(5, 5), SuperblockCell::None);
        assert!(!state.is_interior(5, 5));
        assert!(!state.is_in_superblock(5, 5));
    }

    #[test]
    fn test_state_traffic_multiplier() {
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));

        // Interior gets penalty
        assert!(
            (state.traffic_multiplier(12, 12) - SUPERBLOCK_TRAFFIC_PENALTY).abs() < f32::EPSILON
        );

        // Perimeter is normal
        assert!((state.traffic_multiplier(10, 10) - 1.0).abs() < f32::EPSILON);

        // Outside is normal
        assert!((state.traffic_multiplier(5, 5) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_state_coverage_stats() {
        let mut state = SuperblockState::default();
        // 5x5 superblock = 25 total cells, 16 perimeter, 9 interior
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));

        assert_eq!(state.total_interior_cells, 9);
        assert_eq!(state.total_coverage_cells, 25);

        let expected_ratio = 25.0 / (GRID_WIDTH * GRID_HEIGHT) as f32;
        assert!((state.coverage_ratio - expected_ratio).abs() < 1e-6);
    }

    #[test]
    fn test_state_overlapping_superblocks() {
        let mut state = SuperblockState::default();
        // Two overlapping 5x5 superblocks
        state.add_superblock(Superblock::new(10, 10, 14, 14, "A".to_string()));
        state.add_superblock(Superblock::new(12, 12, 16, 16, "B".to_string()));

        // Cell (13,13) is interior of both
        assert!(state.is_interior(13, 13));

        // Cell (12,12) is perimeter of A but interior of B => should be interior
        assert!(state.is_interior(12, 12));
        // Actually (12,12) is interior of A (not on edge 10/14) and perimeter of B (on edge 12)
        // Let me trace: A: x0=10,y0=10,x1=14,y1=14. For (12,12): not on edges of A → interior of A.
        // B: x0=12,y0=12,x1=16,y1=16. For (12,12): on edges of B → perimeter of B.
        // Since A is processed first, it sets interior (2). Then B processes it as perimeter,
        // but only writes perimeter if cell_grid[idx] == 0. So (12,12) stays interior. Correct!
    }

    #[test]
    fn test_state_max_superblocks() {
        let mut state = SuperblockState::default();
        for i in 0..MAX_SUPERBLOCKS {
            let x = (i * 10) % 200;
            let y = (i / 20) * 10;
            state.add_superblock(Superblock::new(x, y, x + 4, y + 4, format!("sb{i}")));
        }
        assert_eq!(state.superblocks.len(), MAX_SUPERBLOCKS);

        // Adding one more should fail
        assert!(!state.add_superblock(Superblock::new(200, 200, 204, 204, "overflow".to_string())));
    }

    #[test]
    fn test_state_out_of_bounds_cell() {
        let state = SuperblockState::default();
        assert_eq!(state.get_cell(999, 999), SuperblockCell::None);
        assert!(!state.is_interior(999, 999));
        assert!((state.traffic_multiplier(999, 999) - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = SuperblockState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_nonempty() {
        use crate::Saveable;
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "A".to_string()));
        state.add_superblock(Superblock::new(50, 50, 55, 55, "B".to_string()));

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = SuperblockState::load_from_bytes(&bytes);

        assert_eq!(restored.superblocks.len(), 2);
        assert_eq!(restored.superblocks[0].name, "A");
        assert_eq!(restored.superblocks[1].name, "B");
        assert!(restored.is_interior(12, 12));
        assert!(restored.total_interior_cells > 0);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(SuperblockState::SAVE_KEY, "superblock_state");
    }

    // -------------------------------------------------------------------------
    // Constants verification
    // -------------------------------------------------------------------------

    #[test]
    fn test_traffic_penalty_positive() {
        assert!(SUPERBLOCK_TRAFFIC_PENALTY > 1.0);
    }

    #[test]
    fn test_happiness_bonus_positive() {
        assert!(SUPERBLOCK_HAPPINESS_BONUS > 0.0);
    }

    #[test]
    fn test_land_value_bonus_positive() {
        assert!(SUPERBLOCK_LAND_VALUE_BONUS > 0);
    }

    #[test]
    fn test_min_size_at_least_3() {
        assert!(MIN_SUPERBLOCK_SIZE >= 3);
    }
}
