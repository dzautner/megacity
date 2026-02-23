//! Unit tests for NimbyState, ZoneSnapshot, serialization, constants,
//! and personality-related opinion tests.

#[cfg(test)]
mod tests {
    use crate::citizen::Personality;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::ZoneType;
    use crate::nimby::opinion::calculate_opinion;
    use crate::nimby::types::{
        zone_type_name, NimbyState, ZoneChangeEvent, ZoneSnapshot,
        CONSTRUCTION_SLOWDOWN_PER_OPPOSITION, EMINENT_DOMAIN_HAPPINESS_PENALTY,
        HAPPINESS_PENALTY_PER_OPPOSITION, MAX_CONSTRUCTION_SLOWDOWN, MAX_NIMBY_HAPPINESS_PENALTY,
        MAX_ZONE_CHANGES, OPINION_DURATION_TICKS, PROTEST_COOLDOWN_TICKS, PROTEST_THRESHOLD,
        REACTION_RADIUS,
    };
    use crate::Saveable;

    // -------------------------------------------------------------------------
    // NimbyState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_nimby_state_default() {
        let state = NimbyState::default();
        assert!(state.zone_changes.is_empty());
        assert_eq!(state.opposition_grid.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(state.active_protests, 0);
        assert_eq!(state.total_changes_processed, 0);
        assert_eq!(state.opposition_at(0, 0), 0.0);
        assert_eq!(state.opposition_at(128, 128), 0.0);
    }

    #[test]
    fn test_nimby_state_set_get_opposition() {
        let mut state = NimbyState::default();
        state.opposition_grid[10 * GRID_WIDTH + 10] = 25.0;
        assert_eq!(state.opposition_at(10, 10), 25.0);
        assert_eq!(state.opposition_at(0, 0), 0.0);
    }

    // -------------------------------------------------------------------------
    // ZoneSnapshot tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zone_snapshot_default() {
        let snapshot = ZoneSnapshot::default();
        assert_eq!(snapshot.zones.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(snapshot.zones[0], ZoneType::None);
    }

    // -------------------------------------------------------------------------
    // Saveable implementation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skip_default() {
        let state = NimbyState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = NimbyState::default();
        state.zone_changes.push(ZoneChangeEvent {
            grid_x: 10,
            grid_y: 20,
            old_zone: ZoneType::None,
            new_zone: ZoneType::Industrial,
            created_tick: 100,
            remaining_ticks: 150,
            protest_triggered: true,
            protest_cooldown: 50,
        });
        state.total_changes_processed = 5;

        let bytes = state
            .save_to_bytes()
            .expect("non-default state should save");
        let restored = NimbyState::load_from_bytes(&bytes);

        assert_eq!(restored.zone_changes.len(), 1);
        assert_eq!(restored.zone_changes[0].grid_x, 10);
        assert_eq!(restored.zone_changes[0].grid_y, 20);
        assert_eq!(restored.zone_changes[0].new_zone, ZoneType::Industrial);
        assert_eq!(restored.zone_changes[0].remaining_ticks, 150);
        assert!(restored.zone_changes[0].protest_triggered);
        assert_eq!(restored.zone_changes[0].protest_cooldown, 50);
        assert_eq!(restored.total_changes_processed, 5);
        // Opposition grid is recomputed each tick, not saved
        assert_eq!(restored.opposition_at(10, 20), 0.0);
    }

    // -------------------------------------------------------------------------
    // Zone type name tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zone_type_name_coverage() {
        assert!(!zone_type_name(ZoneType::None).is_empty());
        assert!(!zone_type_name(ZoneType::ResidentialLow).is_empty());
        assert!(!zone_type_name(ZoneType::ResidentialMedium).is_empty());
        assert!(!zone_type_name(ZoneType::ResidentialHigh).is_empty());
        assert!(!zone_type_name(ZoneType::CommercialLow).is_empty());
        assert!(!zone_type_name(ZoneType::CommercialHigh).is_empty());
        assert!(!zone_type_name(ZoneType::Industrial).is_empty());
        assert!(!zone_type_name(ZoneType::Office).is_empty());
        assert!(!zone_type_name(ZoneType::MixedUse).is_empty());
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(REACTION_RADIUS > 0);
        assert!(MAX_ZONE_CHANGES > 0);
        assert!(OPINION_DURATION_TICKS > 0);
        assert!(HAPPINESS_PENALTY_PER_OPPOSITION > 0.0);
        assert!(MAX_NIMBY_HAPPINESS_PENALTY > 0.0);
        assert!(PROTEST_THRESHOLD > 0.0);
        assert!(CONSTRUCTION_SLOWDOWN_PER_OPPOSITION > 0.0);
        assert!(MAX_CONSTRUCTION_SLOWDOWN > 0);
        assert!(EMINENT_DOMAIN_HAPPINESS_PENALTY > 0.0);
        assert!(PROTEST_COOLDOWN_TICKS > 0);
    }

    // -------------------------------------------------------------------------
    // Personality-related opinion tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_high_income_opposes_more() {
        let personality = Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.5,
        };
        let mid_income = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &personality,
            false,
            false,
            0.10,
        );
        let high_income = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            3,
            &personality,
            false,
            false,
            0.10,
        );
        assert!(
            high_income > mid_income,
            "high income should oppose more: high={}, mid={}",
            high_income,
            mid_income
        );
    }

    #[test]
    fn test_materialistic_personality_opposes_more() {
        let low_mat = Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.2,
            resilience: 0.5,
        };
        let high_mat = Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.9,
            resilience: 0.5,
        };
        let low_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &low_mat,
            false,
            false,
            0.10,
        );
        let high_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &high_mat,
            false,
            false,
            0.10,
        );
        assert!(
            high_opinion > low_opinion,
            "materialistic should oppose more: high={}, low={}",
            high_opinion,
            low_opinion
        );
    }

    #[test]
    fn test_resilient_personality_opposes_less() {
        let low_res = Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.2,
        };
        let high_res = Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.9,
        };
        let low_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &low_res,
            false,
            false,
            0.10,
        );
        let high_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &high_res,
            false,
            false,
            0.10,
        );
        assert!(
            high_opinion < low_opinion,
            "resilient should oppose less: high={}, low={}",
            high_opinion,
            low_opinion
        );
    }
}
