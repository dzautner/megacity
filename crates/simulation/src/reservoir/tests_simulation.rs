#[cfg(test)]
mod tests {
    use crate::reservoir::types::*;

    // =========================================================================
    // Full integration-style unit tests (logic only, no ECS app)
    // =========================================================================

    /// Simulate one tick of the reservoir update logic without a Bevy app.
    /// Returns the updated ReservoirState and whether a warning event would fire.
    fn simulate_tick(
        precipitation_intensity: f32,
        temperature: f32,
        total_demand_gpd: f32,
        reservoirs: &mut [(f32, f32)], // (stored_gallons, storage_capacity)
        state: &mut ReservoirState,
    ) -> Option<ReservoirWarningEvent> {
        let reservoir_count = reservoirs.len() as u32;
        state.reservoir_count = reservoir_count;

        let total_capacity_gallons: f32 = reservoirs.iter().map(|(_, cap)| *cap).sum();
        let total_stored_gallons: f32 = reservoirs.iter().map(|(stored, _)| *stored).sum();

        state.total_storage_capacity_mg = total_capacity_gallons / MGD_TO_GPD;
        state.current_level_mg = total_stored_gallons / MGD_TO_GPD;

        if reservoir_count == 0 {
            state.inflow_rate_mgd = 0.0_f32;
            state.outflow_rate_mgd = 0.0_f32;
            state.evaporation_rate_mgd = 0.0_f32;
            state.net_change_mgd = 0.0_f32;
            state.storage_days = 0.0_f32;
            let old_tier = state.warning_tier;
            state.warning_tier = ReservoirWarningTier::Normal;
            if old_tier != ReservoirWarningTier::Normal {
                return Some(ReservoirWarningEvent {
                    old_tier,
                    new_tier: ReservoirWarningTier::Normal,
                    fill_pct: 0.0_f32,
                });
            }
            return None;
        }

        let inflow_mgd = precipitation_intensity * CATCHMENT_FACTOR * reservoir_count as f32;
        let outflow_mgd = total_demand_gpd / MGD_TO_GPD;
        let temp_above_20 = (temperature - 20.0_f32).max(0.0_f32);
        let evaporation_mgd = reservoir_count as f32
            * (BASE_EVAPORATION_RATE + temp_above_20 * TEMPERATURE_EVAP_FACTOR);
        let net_change_mgd = inflow_mgd - outflow_mgd - evaporation_mgd;

        state.inflow_rate_mgd = inflow_mgd;
        state.outflow_rate_mgd = outflow_mgd;
        state.evaporation_rate_mgd = evaporation_mgd;
        state.net_change_mgd = net_change_mgd;

        let net_change_gallons = net_change_mgd * MGD_TO_GPD;
        let mut new_total_stored: f32 = 0.0_f32;

        for (stored, capacity) in reservoirs.iter_mut() {
            let share = if total_capacity_gallons > 0.0_f32 {
                *capacity / total_capacity_gallons
            } else {
                0.0_f32
            };
            let delta = net_change_gallons * share;
            *stored = (*stored + delta).clamp(0.0_f32, *capacity);
            new_total_stored += *stored;
        }

        state.current_level_mg = new_total_stored / MGD_TO_GPD;

        let fill_pct = state.fill_pct();
        let new_tier = warning_tier_from_fill(fill_pct);
        let old_tier = state.warning_tier;
        state.warning_tier = new_tier;

        state.storage_days = if outflow_mgd > 0.0_f32 {
            state.current_level_mg / outflow_mgd
        } else {
            f32::INFINITY
        };

        if old_tier != new_tier {
            Some(ReservoirWarningEvent {
                old_tier,
                new_tier,
                fill_pct,
            })
        } else {
            None
        }
    }

    #[test]
    fn test_simulate_tick_no_reservoirs() {
        let mut state = ReservoirState::default();
        let event = simulate_tick(1.0_f32, 25.0_f32, 100_000.0_f32, &mut [], &mut state);
        assert_eq!(state.reservoir_count, 0);
        assert!((state.inflow_rate_mgd).abs() < f32::EPSILON);
        assert!(event.is_none());
    }

    #[test]
    fn test_simulate_tick_single_reservoir_no_rain() {
        let mut state = ReservoirState::default();
        // Reservoir starts full at 1,000,000 gallons capacity.
        let mut reservoirs = vec![(1_000_000.0_f32, 1_000_000.0_f32)];
        let event = simulate_tick(
            0.0_f32,  // no rain
            20.0_f32, // 20C (base evap only)
            0.0_f32,  // no demand
            &mut reservoirs,
            &mut state,
        );
        // Evaporation should have reduced stored level.
        // Evap = 1 * (0.005 + 0) = 0.005 MGD = 5000 gallons.
        let expected_stored = 1_000_000.0_f32 - 5000.0_f32;
        assert!((reservoirs[0].0 - expected_stored).abs() < 1.0_f32);
        // Still nearly full, so Normal tier.
        assert_eq!(state.warning_tier, ReservoirWarningTier::Normal);
        assert!(event.is_none());
    }

    #[test]
    fn test_simulate_tick_high_demand_drains_reservoir() {
        let mut state = ReservoirState::default();
        // Start half full.
        let mut reservoirs = vec![(500_000.0_f32, 1_000_000.0_f32)];
        // Heavy demand: 400,000 GPD = 0.4 MGD.
        let event = simulate_tick(
            0.0_f32,       // no rain
            20.0_f32,      // 20C
            400_000.0_f32, // demand GPD
            &mut reservoirs,
            &mut state,
        );
        // Outflow = 0.4 MGD = 400,000 gallons. Evap = 5,000. Net = -405,000.
        let expected = 500_000.0_f32 - 400_000.0_f32 - 5_000.0_f32;
        assert!((reservoirs[0].0 - expected).abs() < 1.0_f32);
        // 95,000 / 1,000,000 = 9.5% -> Critical tier.
        assert_eq!(state.warning_tier, ReservoirWarningTier::Critical);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert_eq!(ev.old_tier, ReservoirWarningTier::Normal);
        assert_eq!(ev.new_tier, ReservoirWarningTier::Critical);
    }

    #[test]
    fn test_simulate_tick_rainfall_replenishes() {
        let mut state = ReservoirState::default();
        let mut reservoirs = vec![(500_000.0_f32, 1_000_000.0_f32)];
        // Heavy rain at 2.0 in/hr, no demand, cool temp.
        let event = simulate_tick(
            2.0_f32,  // heavy rain
            15.0_f32, // below 20C, base evap only
            0.0_f32,  // no demand
            &mut reservoirs,
            &mut state,
        );
        // Inflow = 2.0 * 0.001 * 1 = 0.002 MGD = 2000 gallons.
        // Evap = 0.005 MGD = 5000 gallons.
        // Net = 2000 - 0 - 5000 = -3000 gallons.
        let expected = 500_000.0_f32 - 3_000.0_f32;
        assert!((reservoirs[0].0 - expected).abs() < 1.0_f32);
        // 49.7% fill -> Watch tier (threshold is >50% for Normal).
        assert_eq!(state.warning_tier, ReservoirWarningTier::Watch);
        // Tier changed from Normal (default) to Watch, so event is fired.
        assert!(event.is_some());
    }

    #[test]
    fn test_simulate_tick_hot_temperature_increases_evaporation() {
        let mut state = ReservoirState::default();
        let mut reservoirs = vec![(1_000_000.0_f32, 1_000_000.0_f32)];
        // No rain, no demand, but 40C.
        simulate_tick(0.0_f32, 40.0_f32, 0.0_f32, &mut reservoirs, &mut state);
        // Evap = 1 * (0.005 + 20*0.03) = 1 * 0.605 = 0.605 MGD = 605,000 gallons.
        let expected = 1_000_000.0_f32 - 605_000.0_f32;
        assert!((reservoirs[0].0 - expected).abs() < 1.0_f32);
    }

    #[test]
    fn test_simulate_tick_stored_does_not_go_below_zero() {
        let mut state = ReservoirState::default();
        // Very small reservoir nearly empty.
        let mut reservoirs = vec![(100.0_f32, 1_000_000.0_f32)];
        // Huge demand.
        simulate_tick(
            0.0_f32,
            20.0_f32,
            10_000_000.0_f32,
            &mut reservoirs,
            &mut state,
        );
        assert!((reservoirs[0].0 - 0.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_simulate_tick_stored_does_not_exceed_capacity() {
        let mut state = ReservoirState::default();
        // Nearly full.
        let mut reservoirs = vec![(999_999.0_f32, 1_000_000.0_f32)];
        // Huge rainfall, no demand, cool.
        simulate_tick(10000.0_f32, 10.0_f32, 0.0_f32, &mut reservoirs, &mut state);
        // Should be clamped to capacity.
        assert!(reservoirs[0].0 <= reservoirs[0].1);
    }

    #[test]
    fn test_simulate_tick_multiple_reservoirs_proportional() {
        let mut state = ReservoirState::default();
        // Two reservoirs: one 1M capacity, one 3M capacity, both start full.
        let mut reservoirs = vec![
            (1_000_000.0_f32, 1_000_000.0_f32),
            (3_000_000.0_f32, 3_000_000.0_f32),
        ];
        // Drain with demand, no rain, 20C.
        simulate_tick(
            0.0_f32,
            20.0_f32,
            200_000.0_f32,
            &mut reservoirs,
            &mut state,
        );
        // Total capacity = 4M. Reservoir A gets 25% of delta, B gets 75%.
        // Evap = 2 * 0.005 = 0.01 MGD = 10,000 gallons.
        // Outflow = 0.2 MGD = 200,000 gallons.
        // Net = -210,000 gallons.
        // A delta = -210,000 * 0.25 = -52,500. B delta = -210,000 * 0.75 = -157,500.
        let expected_a = 1_000_000.0_f32 - 52_500.0_f32;
        let expected_b = 3_000_000.0_f32 - 157_500.0_f32;
        assert!((reservoirs[0].0 - expected_a).abs() < 1.0_f32);
        assert!((reservoirs[1].0 - expected_b).abs() < 1.0_f32);
    }

    #[test]
    fn test_simulate_tick_storage_days_calculation() {
        let mut state = ReservoirState::default();
        // 10M gallons stored, 1M GPD demand = 10 days.
        let mut reservoirs = vec![(10_000_000.0_f32, 20_000_000.0_f32)];
        simulate_tick(
            0.0_f32,
            20.0_f32,
            1_000_000.0_f32,
            &mut reservoirs,
            &mut state,
        );
        // After draining: stored ~ 10M - 1M - 5000 = ~8.995M
        // storage_days = 8.995 / 1.0 ~ 8.995
        // outflow_mgd = 1.0
        assert!(state.storage_days > 0.0_f32);
        assert!(state.storage_days < 10.0_f32); // reduced from draining
    }

    #[test]
    fn test_simulate_tier_transition_normal_to_watch() {
        let mut state = ReservoirState::default();
        // Start at 51% fill (Normal).
        let mut reservoirs = vec![(510_000.0_f32, 1_000_000.0_f32)];
        // Drain enough to cross 50% boundary.
        // Need to drain ~10,001+ gallons. Outflow 100,000 GPD would do it.
        let event = simulate_tick(
            0.0_f32,
            20.0_f32,
            100_000.0_f32,
            &mut reservoirs,
            &mut state,
        );
        // After: 510,000 - 100,000 - 5,000 = 405,000 = 40.5% -> Watch.
        assert_eq!(state.warning_tier, ReservoirWarningTier::Watch);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert_eq!(ev.old_tier, ReservoirWarningTier::Normal);
        assert_eq!(ev.new_tier, ReservoirWarningTier::Watch);
    }

    #[test]
    fn test_simulate_no_event_when_tier_unchanged() {
        let mut state = ReservoirState::default();
        // Start at 90% (Normal). Small drain stays Normal.
        let mut reservoirs = vec![(900_000.0_f32, 1_000_000.0_f32)];
        let event = simulate_tick(0.0_f32, 20.0_f32, 1_000.0_f32, &mut reservoirs, &mut state);
        assert_eq!(state.warning_tier, ReservoirWarningTier::Normal);
        assert!(event.is_none());
    }
}
