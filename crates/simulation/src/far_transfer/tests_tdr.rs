//! Unit tests for TDR transfers, stat district helpers, and saveable roundtrip.

#[cfg(test)]
mod tests {
    use crate::config::GRID_WIDTH;
    use crate::districts::{DISTRICTS_X, DISTRICTS_Y, DISTRICT_SIZE};

    use crate::far_transfer::systems::stat_district_for_grid;
    use crate::far_transfer::types::*;

    // -------------------------------------------------------------------------
    // TDR integration test: transfer FAR from park to adjacent development
    // -------------------------------------------------------------------------

    #[test]
    fn test_tdr_park_to_adjacent_site() {
        // Simulate: park in district (0,0) provides FAR, building in (0,0) receives it
        let mut state = FarTransferState::default();

        // Park provides FAR to district 0
        state.district_available_far[0] = PARK_UNUSED_FAR_PER_CELL;

        // Building in same district should be able to receive FAR
        let remaining = state.available_far_for_district(0);
        assert!(
            remaining > 0.0,
            "park should provide available FAR: {}",
            remaining
        );

        // Simulate transfer
        let transfer = remaining.min(MAX_TRANSFER_FAR_PER_CELL);
        state.district_transferred_far[0] += transfer;
        state.transferred_far[5 * GRID_WIDTH + 5] = transfer;

        // Verify accounting: source FAR is debited
        assert!(
            state.available_far_for_district(0).abs() < f32::EPSILON,
            "transferred FAR should be debited from source"
        );

        // Verify receiving cell has the transferred FAR
        assert!(
            (state.transferred_at(5, 5) - transfer).abs() < f32::EPSILON,
            "receiving cell should have transferred FAR"
        );
    }

    // -------------------------------------------------------------------------
    // Stat district helper tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_stat_district_for_grid_origin() {
        let (dx, dy) = stat_district_for_grid(0, 0);
        assert_eq!((dx, dy), (0, 0));
    }

    #[test]
    fn test_stat_district_for_grid_middle() {
        let (dx, dy) = stat_district_for_grid(128, 128);
        assert_eq!(dx, 128 / DISTRICT_SIZE);
        assert_eq!(dy, 128 / DISTRICT_SIZE);
    }

    #[test]
    fn test_stat_district_for_grid_max() {
        let (dx, dy) = stat_district_for_grid(255, 255);
        assert!(dx < DISTRICTS_X);
        assert!(dy < DISTRICTS_Y);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = FarTransferState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_bonus_active() {
        use crate::Saveable;
        let mut state = FarTransferState::default();
        state.total_bonus_far = 1.0;
        state.bonus_far[0] = 1.0;
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_saves_when_transfer_active() {
        use crate::Saveable;
        let mut state = FarTransferState::default();
        state.total_transferred_far = 2.0;
        state.transferred_far[0] = 2.0;
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = FarTransferState::default();
        state.bonus_far[100] = 0.6;
        state.transferred_far[200] = 1.5;
        state.bonus_flags[100] = bonus_type_to_bit(FarBonusType::AffordableHousing);
        state.district_available_far[0] = 5.0;
        state.district_transferred_far[0] = 2.0;
        state.total_bonus_far = 0.6;
        state.total_transferred_far = 1.5;

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = FarTransferState::load_from_bytes(&bytes);

        assert!((restored.bonus_far[100] - 0.6).abs() < f32::EPSILON);
        assert!((restored.transferred_far[200] - 1.5).abs() < f32::EPSILON);
        assert_eq!(
            restored.bonus_flags[100],
            bonus_type_to_bit(FarBonusType::AffordableHousing)
        );
        assert!((restored.district_available_far[0] - 5.0).abs() < f32::EPSILON);
        assert!((restored.district_transferred_far[0] - 2.0).abs() < f32::EPSILON);
        assert!((restored.total_bonus_far - 0.6).abs() < f32::EPSILON);
        assert!((restored.total_transferred_far - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(FarTransferState::SAVE_KEY, "far_transfer");
    }
}
