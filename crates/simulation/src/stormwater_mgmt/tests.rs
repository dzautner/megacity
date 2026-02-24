//! Unit tests for stormwater management module.

#[cfg(test)]
mod tests {
    use crate::stormwater_mgmt::state::StormwaterMgmtState;
    use crate::Saveable;

    #[test]
    fn test_stormwater_mgmt_state_default() {
        let state = StormwaterMgmtState::default();
        assert_eq!(state.green_infra_absorbed, 0.0);
        assert_eq!(state.flood_damaged_roads, 0);
        assert_eq!(state.displaced_citizens, 0);
        assert_eq!(state.avg_flood_risk, 0.0);
        assert_eq!(state.high_risk_cells, 0);
    }

    #[test]
    fn test_stormwater_mgmt_state_default_skips_save() {
        let state = StormwaterMgmtState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "default state should skip save"
        );
    }

    #[test]
    fn test_stormwater_mgmt_state_with_data_saves() {
        let mut state = StormwaterMgmtState::default();
        state.green_infra_absorbed = 100.0;
        state.flood_damaged_roads = 5;
        assert!(
            state.save_to_bytes().is_some(),
            "non-default state should save"
        );
    }

    #[test]
    fn test_stormwater_mgmt_state_roundtrip() {
        let mut state = StormwaterMgmtState::default();
        state.green_infra_absorbed = 42.5;
        state.flood_damaged_roads = 10;
        state.displaced_citizens = 200;
        state.avg_flood_risk = 120.0;
        state.high_risk_cells = 5000;

        let bytes = state.save_to_bytes().unwrap();
        let restored = StormwaterMgmtState::load_from_bytes(&bytes);

        assert!((restored.green_infra_absorbed - 42.5).abs() < 0.01);
        assert_eq!(restored.flood_damaged_roads, 10);
        assert_eq!(restored.displaced_citizens, 200);
        assert!((restored.avg_flood_risk - 120.0).abs() < 0.01);
        assert_eq!(restored.high_risk_cells, 5000);
    }
}
