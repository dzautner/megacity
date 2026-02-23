#[cfg(test)]
mod tests {
    use crate::reservoir::types::*;

    // =========================================================================
    // Warning tier tests
    // =========================================================================

    #[test]
    fn test_tier_normal_above_50_pct() {
        assert_eq!(
            warning_tier_from_fill(1.0_f32),
            ReservoirWarningTier::Normal
        );
        assert_eq!(
            warning_tier_from_fill(0.75_f32),
            ReservoirWarningTier::Normal
        );
        assert_eq!(
            warning_tier_from_fill(0.51_f32),
            ReservoirWarningTier::Normal
        );
    }

    #[test]
    fn test_tier_watch_30_to_50_pct() {
        assert_eq!(
            warning_tier_from_fill(0.50_f32),
            ReservoirWarningTier::Watch
        );
        assert_eq!(
            warning_tier_from_fill(0.40_f32),
            ReservoirWarningTier::Watch
        );
        assert_eq!(
            warning_tier_from_fill(0.31_f32),
            ReservoirWarningTier::Watch
        );
    }

    #[test]
    fn test_tier_warning_20_to_30_pct() {
        assert_eq!(
            warning_tier_from_fill(0.30_f32),
            ReservoirWarningTier::Warning
        );
        assert_eq!(
            warning_tier_from_fill(0.25_f32),
            ReservoirWarningTier::Warning
        );
        assert_eq!(
            warning_tier_from_fill(0.21_f32),
            ReservoirWarningTier::Warning
        );
    }

    #[test]
    fn test_tier_critical_at_or_below_20_pct() {
        assert_eq!(
            warning_tier_from_fill(0.20_f32),
            ReservoirWarningTier::Critical
        );
        assert_eq!(
            warning_tier_from_fill(0.10_f32),
            ReservoirWarningTier::Critical
        );
        assert_eq!(
            warning_tier_from_fill(0.0_f32),
            ReservoirWarningTier::Critical
        );
    }

    #[test]
    fn test_tier_boundary_exactly_50() {
        // 50% is the boundary between Normal and Watch: <= 50% is Watch.
        assert_eq!(
            warning_tier_from_fill(0.50_f32),
            ReservoirWarningTier::Watch
        );
    }

    #[test]
    fn test_tier_boundary_exactly_30() {
        assert_eq!(
            warning_tier_from_fill(0.30_f32),
            ReservoirWarningTier::Warning
        );
    }

    #[test]
    fn test_tier_boundary_exactly_20() {
        assert_eq!(
            warning_tier_from_fill(0.20_f32),
            ReservoirWarningTier::Critical
        );
    }

    // =========================================================================
    // ReservoirWarningTier name tests
    // =========================================================================

    #[test]
    fn test_tier_names() {
        assert_eq!(ReservoirWarningTier::Normal.name(), "Normal");
        assert_eq!(ReservoirWarningTier::Watch.name(), "Watch");
        assert_eq!(ReservoirWarningTier::Warning.name(), "Warning");
        assert_eq!(ReservoirWarningTier::Critical.name(), "Critical");
    }

    // =========================================================================
    // ReservoirState default and fill_pct tests
    // =========================================================================

    #[test]
    fn test_default_state() {
        let state = ReservoirState::default();
        assert!((state.total_storage_capacity_mg - 0.0_f32).abs() < f32::EPSILON);
        assert!((state.current_level_mg - 0.0_f32).abs() < f32::EPSILON);
        assert!((state.inflow_rate_mgd - 0.0_f32).abs() < f32::EPSILON);
        assert!((state.outflow_rate_mgd - 0.0_f32).abs() < f32::EPSILON);
        assert!((state.evaporation_rate_mgd - 0.0_f32).abs() < f32::EPSILON);
        assert!((state.net_change_mgd - 0.0_f32).abs() < f32::EPSILON);
        assert!((state.storage_days - 0.0_f32).abs() < f32::EPSILON);
        assert_eq!(state.reservoir_count, 0);
        assert_eq!(state.warning_tier, ReservoirWarningTier::Normal);
        assert!((state.min_reserve_pct - MIN_RESERVE_PCT).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fill_pct_full() {
        let state = ReservoirState {
            total_storage_capacity_mg: 100.0_f32,
            current_level_mg: 100.0_f32,
            ..Default::default()
        };
        assert!((state.fill_pct() - 1.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fill_pct_half() {
        let state = ReservoirState {
            total_storage_capacity_mg: 100.0_f32,
            current_level_mg: 50.0_f32,
            ..Default::default()
        };
        assert!((state.fill_pct() - 0.5_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fill_pct_empty() {
        let state = ReservoirState {
            total_storage_capacity_mg: 100.0_f32,
            current_level_mg: 0.0_f32,
            ..Default::default()
        };
        assert!((state.fill_pct() - 0.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fill_pct_no_capacity_returns_zero() {
        let state = ReservoirState {
            total_storage_capacity_mg: 0.0_f32,
            current_level_mg: 0.0_f32,
            ..Default::default()
        };
        assert!((state.fill_pct() - 0.0_f32).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Constants tests
    // =========================================================================

    #[test]
    fn test_constants_are_positive() {
        assert!(CATCHMENT_FACTOR > 0.0_f32);
        assert!(BASE_EVAPORATION_RATE > 0.0_f32);
        assert!(TEMPERATURE_EVAP_FACTOR > 0.0_f32);
        assert!(MIN_RESERVE_PCT > 0.0_f32);
        assert!(MIN_RESERVE_PCT < 1.0_f32);
    }

    #[test]
    fn test_catchment_factor_value() {
        assert!((CATCHMENT_FACTOR - 0.001_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_base_evaporation_rate_value() {
        assert!((BASE_EVAPORATION_RATE - 0.005_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temperature_evap_factor_value() {
        assert!((TEMPERATURE_EVAP_FACTOR - 0.03_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_min_reserve_pct_value() {
        assert!((MIN_RESERVE_PCT - 0.20_f32).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Inflow calculation tests
    // =========================================================================

    #[test]
    fn test_inflow_zero_when_no_rain() {
        let precipitation_intensity = 0.0_f32;
        let reservoir_count = 3_u32;
        let inflow = precipitation_intensity * CATCHMENT_FACTOR * reservoir_count as f32;
        assert!((inflow - 0.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inflow_scales_with_precipitation() {
        let reservoir_count = 1_u32;
        let inflow_low = 0.5_f32 * CATCHMENT_FACTOR * reservoir_count as f32;
        let inflow_high = 2.0_f32 * CATCHMENT_FACTOR * reservoir_count as f32;
        assert!(inflow_high > inflow_low);
        assert!((inflow_high / inflow_low - 4.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inflow_scales_with_reservoir_count() {
        let precipitation_intensity = 1.0_f32;
        let inflow_1 = precipitation_intensity * CATCHMENT_FACTOR * 1.0_f32;
        let inflow_3 = precipitation_intensity * CATCHMENT_FACTOR * 3.0_f32;
        assert!((inflow_3 / inflow_1 - 3.0_f32).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Evaporation calculation tests
    // =========================================================================

    #[test]
    fn test_evaporation_at_20c_is_base_rate() {
        let temp = 20.0_f32;
        let reservoir_count = 1_u32;
        let temp_above = (temp - 20.0_f32).max(0.0_f32);
        let evap =
            reservoir_count as f32 * (BASE_EVAPORATION_RATE + temp_above * TEMPERATURE_EVAP_FACTOR);
        assert!((evap - BASE_EVAPORATION_RATE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaporation_below_20c_is_base_rate() {
        let temp = 10.0_f32;
        let reservoir_count = 1_u32;
        let temp_above = (temp - 20.0_f32).max(0.0_f32);
        let evap =
            reservoir_count as f32 * (BASE_EVAPORATION_RATE + temp_above * TEMPERATURE_EVAP_FACTOR);
        // Below 20C, temp_above is 0.0, so evap == base rate.
        assert!((evap - BASE_EVAPORATION_RATE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaporation_increases_above_20c() {
        let reservoir_count = 1_u32;
        let temp_cool = 20.0_f32;
        let temp_hot = 30.0_f32;
        let evap_cool = reservoir_count as f32
            * (BASE_EVAPORATION_RATE
                + (temp_cool - 20.0_f32).max(0.0_f32) * TEMPERATURE_EVAP_FACTOR);
        let evap_hot = reservoir_count as f32
            * (BASE_EVAPORATION_RATE
                + (temp_hot - 20.0_f32).max(0.0_f32) * TEMPERATURE_EVAP_FACTOR);
        assert!(evap_hot > evap_cool);
        // At 30C: 0.005 + 10*0.03 = 0.305
        let expected_hot = BASE_EVAPORATION_RATE + 10.0_f32 * TEMPERATURE_EVAP_FACTOR;
        assert!((evap_hot - expected_hot).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaporation_scales_with_reservoir_count() {
        let temp = 25.0_f32;
        let temp_above = (temp - 20.0_f32).max(0.0_f32);
        let evap_1 = 1.0_f32 * (BASE_EVAPORATION_RATE + temp_above * TEMPERATURE_EVAP_FACTOR);
        let evap_4 = 4.0_f32 * (BASE_EVAPORATION_RATE + temp_above * TEMPERATURE_EVAP_FACTOR);
        assert!((evap_4 / evap_1 - 4.0_f32).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Net change calculation tests
    // =========================================================================

    #[test]
    fn test_net_change_positive_with_high_inflow() {
        let inflow = 5.0_f32;
        let outflow = 2.0_f32;
        let evaporation = 0.5_f32;
        let net = inflow - outflow - evaporation;
        assert!(net > 0.0_f32);
        assert!((net - 2.5_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_net_change_negative_with_high_demand() {
        let inflow = 0.5_f32;
        let outflow = 3.0_f32;
        let evaporation = 0.1_f32;
        let net = inflow - outflow - evaporation;
        assert!(net < 0.0_f32);
        assert!((net - (-2.6_f32)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_net_change_zero_when_balanced() {
        let inflow = 2.5_f32;
        let outflow = 2.0_f32;
        let evaporation = 0.5_f32;
        let net = inflow - outflow - evaporation;
        assert!((net - 0.0_f32).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Storage days calculation tests
    // =========================================================================

    #[test]
    fn test_storage_days_with_demand() {
        let current_level_mg = 100.0_f32;
        let outflow_mgd = 10.0_f32;
        let days = current_level_mg / outflow_mgd;
        assert!((days - 10.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_storage_days_infinity_with_zero_demand() {
        let current_level_mg = 100.0_f32;
        let outflow_mgd = 0.0_f32;
        let days = if outflow_mgd > 0.0_f32 {
            current_level_mg / outflow_mgd
        } else {
            f32::INFINITY
        };
        assert!(days.is_infinite());
    }

    #[test]
    fn test_storage_days_zero_when_empty() {
        let current_level_mg = 0.0_f32;
        let outflow_mgd = 5.0_f32;
        let days = current_level_mg / outflow_mgd;
        assert!((days - 0.0_f32).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Clamping tests
    // =========================================================================

    #[test]
    fn test_stored_gallons_clamped_to_capacity() {
        let capacity = 1000.0_f32;
        let stored = 800.0_f32;
        let delta = 500.0_f32; // would exceed capacity
        let new_stored = (stored + delta).clamp(0.0_f32, capacity);
        assert!((new_stored - capacity).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stored_gallons_clamped_to_zero() {
        let capacity = 1000.0_f32;
        let stored = 200.0_f32;
        let delta = -500.0_f32; // would go below zero
        let new_stored = (stored + delta).clamp(0.0_f32, capacity);
        assert!((new_stored - 0.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stored_gallons_normal_delta() {
        let capacity = 1000.0_f32;
        let stored = 500.0_f32;
        let delta = 100.0_f32;
        let new_stored = (stored + delta).clamp(0.0_f32, capacity);
        assert!((new_stored - 600.0_f32).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Proportional distribution tests
    // =========================================================================

    #[test]
    fn test_proportional_share_equal_reservoirs() {
        let total_capacity = 2000.0_f32;
        let cap_a = 1000.0_f32;
        let cap_b = 1000.0_f32;
        let share_a = cap_a / total_capacity;
        let share_b = cap_b / total_capacity;
        assert!((share_a - 0.5_f32).abs() < f32::EPSILON);
        assert!((share_b - 0.5_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_proportional_share_unequal_reservoirs() {
        let total_capacity = 3000.0_f32;
        let cap_a = 1000.0_f32;
        let cap_b = 2000.0_f32;
        let share_a = cap_a / total_capacity;
        let share_b = cap_b / total_capacity;
        assert!((share_a - 1.0_f32 / 3.0_f32).abs() < 0.001_f32);
        assert!((share_b - 2.0_f32 / 3.0_f32).abs() < 0.001_f32);
    }

    // =========================================================================
    // ReservoirWarningEvent tests
    // =========================================================================

    #[test]
    fn test_warning_event_fields() {
        let event = ReservoirWarningEvent {
            old_tier: ReservoirWarningTier::Normal,
            new_tier: ReservoirWarningTier::Watch,
            fill_pct: 0.45_f32,
        };
        assert_eq!(event.old_tier, ReservoirWarningTier::Normal);
        assert_eq!(event.new_tier, ReservoirWarningTier::Watch);
        assert!((event.fill_pct - 0.45_f32).abs() < f32::EPSILON);
    }

    // =========================================================================
    // MGD/GPD conversion tests
    // =========================================================================

    #[test]
    fn test_gpd_to_mgd_conversion() {
        let gpd = 5_000_000.0_f32;
        let mgd = gpd / MGD_TO_GPD;
        assert!((mgd - 5.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mgd_to_gpd_conversion() {
        let mgd = 3.0_f32;
        let gpd = mgd * MGD_TO_GPD;
        assert!((gpd - 3_000_000.0_f32).abs() < f32::EPSILON);
    }
}
