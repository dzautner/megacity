//! Tests for the water pressure system.

#[cfg(test)]
mod tests {
    use crate::water_pressure::*;

    // -------------------------------------------------------------------------
    // Constants tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_base_pressure_elevation_value() {
        assert!((BASE_PRESSURE_ELEVATION - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_booster_elevation_gain_value() {
        assert!((BOOSTER_ELEVATION_GAIN - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_booster_pump_cost_value() {
        assert!((BOOSTER_PUMP_COST - 200_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pressure_falloff_range_value() {
        assert!((PRESSURE_FALLOFF_RANGE - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_constants_are_positive() {
        assert!(BASE_PRESSURE_ELEVATION > 0.0);
        assert!(BOOSTER_ELEVATION_GAIN > 0.0);
        assert!(BOOSTER_PUMP_COST > 0.0);
        assert!(PRESSURE_FALLOFF_RANGE > 0.0);
    }

    // -------------------------------------------------------------------------
    // Default state tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state_no_boosters() {
        let state = WaterPressureState::default();
        assert_eq!(state.booster_count, 0);
    }

    #[test]
    fn test_default_state_base_elevation() {
        let state = WaterPressureState::default();
        assert!((state.effective_elevation - BASE_PRESSURE_ELEVATION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_default_state_no_buildings() {
        let state = WaterPressureState::default();
        assert_eq!(state.buildings_full_pressure, 0);
        assert_eq!(state.buildings_reduced_pressure, 0);
        assert_eq!(state.buildings_no_pressure, 0);
    }

    #[test]
    fn test_default_state_full_average_pressure() {
        let state = WaterPressureState::default();
        assert!((state.average_pressure_factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_default_state_zero_cost() {
        let state = WaterPressureState::default();
        assert!((state.total_booster_cost - 0.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Effective pressure elevation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_effective_elevation_no_boosters() {
        let elev = effective_pressure_elevation(0);
        assert!((elev - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_elevation_one_booster() {
        let elev = effective_pressure_elevation(1);
        assert!((elev - 80.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_elevation_two_boosters() {
        let elev = effective_pressure_elevation(2);
        assert!((elev - 110.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_elevation_three_boosters() {
        let elev = effective_pressure_elevation(3);
        assert!((elev - 140.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_elevation_scales_linearly() {
        let elev_1 = effective_pressure_elevation(1);
        let elev_2 = effective_pressure_elevation(2);
        let diff = elev_2 - elev_1;
        assert!((diff - BOOSTER_ELEVATION_GAIN).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_elevation_large_count() {
        let count = 10;
        let elev = effective_pressure_elevation(count);
        let expected = BASE_PRESSURE_ELEVATION + count as f32 * BOOSTER_ELEVATION_GAIN;
        assert!((elev - expected).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Pressure factor tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_pressure_factor_at_base_level() {
        let factor = pressure_factor(0.0, 50.0);
        assert!((factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_at_effective_elevation() {
        let factor = pressure_factor(50.0, 50.0);
        assert!((factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_below_effective_elevation() {
        let factor = pressure_factor(30.0, 50.0);
        assert!((factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_slightly_above() {
        let factor = pressure_factor(55.0, 50.0);
        // 5 above effective, falloff range is 10 => 1.0 - 5/10 = 0.5
        assert!((factor - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_at_falloff_boundary() {
        let factor = pressure_factor(60.0, 50.0);
        // 10 above effective, exactly at falloff limit => 0.0
        assert!((factor - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_well_above() {
        let factor = pressure_factor(100.0, 50.0);
        assert!((factor - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_one_quarter_falloff() {
        let factor = pressure_factor(52.5, 50.0);
        // 2.5 above effective, falloff range is 10 => 1.0 - 2.5/10 = 0.75
        assert!((factor - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_three_quarter_falloff() {
        let factor = pressure_factor(57.5, 50.0);
        // 7.5 above effective, falloff range is 10 => 1.0 - 7.5/10 = 0.25
        assert!((factor - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_with_booster() {
        // One booster: effective = 80
        let effective = effective_pressure_elevation(1);
        let factor = pressure_factor(70.0, effective);
        // 70 < 80 => full pressure
        assert!((factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_negative_elevation() {
        let factor = pressure_factor(-10.0, 50.0);
        assert!((factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_factor_zero_effective() {
        let factor = pressure_factor(5.0, 0.0);
        // 5 above 0, falloff range is 10 => 0.5
        assert!((factor - 0.5).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Pressure classification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_classify_full_pressure() {
        assert_eq!(classify_pressure(1.0), PressureCategory::Full);
    }

    #[test]
    fn test_classify_no_pressure() {
        assert_eq!(classify_pressure(0.0), PressureCategory::None);
    }

    #[test]
    fn test_classify_reduced_pressure_mid() {
        assert_eq!(classify_pressure(0.5), PressureCategory::Reduced);
    }

    #[test]
    fn test_classify_reduced_pressure_low() {
        assert_eq!(classify_pressure(0.01), PressureCategory::Reduced);
    }

    #[test]
    fn test_classify_reduced_pressure_high() {
        assert_eq!(classify_pressure(0.99), PressureCategory::Reduced);
    }

    // -------------------------------------------------------------------------
    // Booster pump station component tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_booster_pump_station_creation() {
        let station = BoosterPumpStation {
            grid_x: 10,
            grid_y: 20,
        };
        assert_eq!(station.grid_x, 10);
        assert_eq!(station.grid_y, 20);
    }

    #[test]
    fn test_booster_pump_station_clone() {
        let station = BoosterPumpStation {
            grid_x: 5,
            grid_y: 15,
        };
        let cloned = station.clone();
        assert_eq!(cloned.grid_x, 5);
        assert_eq!(cloned.grid_y, 15);
    }

    // -------------------------------------------------------------------------
    // Cost calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_total_cost_zero_boosters() {
        let cost = 0_u32 as f64 * BOOSTER_PUMP_COST;
        assert!((cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_total_cost_one_booster() {
        let cost = 1_u32 as f64 * BOOSTER_PUMP_COST;
        assert!((cost - 200_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_total_cost_five_boosters() {
        let cost = 5_u32 as f64 * BOOSTER_PUMP_COST;
        assert!((cost - 1_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_total_cost_scales_linearly() {
        let cost_1 = 1_u32 as f64 * BOOSTER_PUMP_COST;
        let cost_3 = 3_u32 as f64 * BOOSTER_PUMP_COST;
        assert!((cost_3 - cost_1 * 3.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skip_default() {
        use crate::Saveable;
        let state = WaterPressureState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_roundtrip_with_boosters() {
        use crate::Saveable;
        let state = WaterPressureState {
            booster_count: 3,
            effective_elevation: 140.0,
            buildings_full_pressure: 100,
            buildings_reduced_pressure: 10,
            buildings_no_pressure: 5,
            average_pressure_factor: 0.85,
            total_booster_cost: 600_000.0,
        };
        let bytes = state.save_to_bytes().expect("should save non-default");
        let restored = WaterPressureState::load_from_bytes(&bytes);
        assert_eq!(restored.booster_count, 3);
        assert!((restored.effective_elevation - 140.0).abs() < f32::EPSILON);
        assert_eq!(restored.buildings_full_pressure, 100);
        assert_eq!(restored.buildings_reduced_pressure, 10);
        assert_eq!(restored.buildings_no_pressure, 5);
        assert!((restored.average_pressure_factor - 0.85).abs() < 0.01);
        assert!((restored.total_booster_cost - 600_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_saveable_load_invalid_bytes_returns_default() {
        use crate::Saveable;
        let restored = WaterPressureState::load_from_bytes(&[0xFF, 0x00, 0x01]);
        // Invalid bytes should return default.
        assert_eq!(restored.booster_count, 0);
        assert!((restored.effective_elevation - BASE_PRESSURE_ELEVATION).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_building_below_base_gets_full_pressure() {
        let effective = effective_pressure_elevation(0);
        let factor = pressure_factor(25.0, effective);
        assert!((factor - 1.0).abs() < f32::EPSILON);
        assert_eq!(classify_pressure(factor), PressureCategory::Full);
    }

    #[test]
    fn test_building_at_base_gets_full_pressure() {
        let effective = effective_pressure_elevation(0);
        let factor = pressure_factor(50.0, effective);
        assert!((factor - 1.0).abs() < f32::EPSILON);
        assert_eq!(classify_pressure(factor), PressureCategory::Full);
    }

    #[test]
    fn test_building_above_base_without_booster_gets_reduced() {
        let effective = effective_pressure_elevation(0);
        let factor = pressure_factor(55.0, effective);
        assert!(factor > 0.0 && factor < 1.0);
        assert_eq!(classify_pressure(factor), PressureCategory::Reduced);
    }

    #[test]
    fn test_building_far_above_base_without_booster_gets_none() {
        let effective = effective_pressure_elevation(0);
        let factor = pressure_factor(65.0, effective);
        assert!((factor - 0.0).abs() < f32::EPSILON);
        assert_eq!(classify_pressure(factor), PressureCategory::None);
    }

    #[test]
    fn test_booster_enables_higher_building() {
        // Without booster: elevation 70 => no pressure (70 > 50 + 10)
        let effective_0 = effective_pressure_elevation(0);
        let factor_0 = pressure_factor(70.0, effective_0);
        assert_eq!(classify_pressure(factor_0), PressureCategory::None);

        // With 1 booster: elevation 70 => full pressure (70 < 80)
        let effective_1 = effective_pressure_elevation(1);
        let factor_1 = pressure_factor(70.0, effective_1);
        assert_eq!(classify_pressure(factor_1), PressureCategory::Full);
    }

    #[test]
    fn test_chained_boosters() {
        // 3 boosters: effective = 50 + 90 = 140
        let effective = effective_pressure_elevation(3);
        assert!((effective - 140.0).abs() < f32::EPSILON);

        // Building at 130 => full pressure
        let factor = pressure_factor(130.0, effective);
        assert_eq!(classify_pressure(factor), PressureCategory::Full);

        // Building at 145 => reduced pressure (5 above 140, falloff 10 => 0.5)
        let factor = pressure_factor(145.0, effective);
        assert_eq!(classify_pressure(factor), PressureCategory::Reduced);
        assert!((factor - 0.5).abs() < f32::EPSILON);

        // Building at 155 => no pressure (15 above 140, exceeds falloff 10)
        let factor = pressure_factor(155.0, effective);
        assert_eq!(classify_pressure(factor), PressureCategory::None);
    }

    #[test]
    fn test_average_pressure_no_buildings() {
        // When there are no buildings, average defaults to 1.0.
        let building_count = 0u32;
        let pressure_sum = 0.0_f32;
        let avg = if building_count > 0 {
            pressure_sum / building_count as f32
        } else {
            1.0
        };
        assert!((avg - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_average_pressure_mixed_buildings() {
        // Simulate 3 buildings: full (1.0), reduced (0.5), none (0.0)
        let factors = [1.0_f32, 0.5, 0.0];
        let sum: f32 = factors.iter().sum();
        let avg = sum / factors.len() as f32;
        assert!((avg - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_state_serde_roundtrip() {
        let state = WaterPressureState {
            booster_count: 2,
            effective_elevation: 110.0,
            buildings_full_pressure: 50,
            buildings_reduced_pressure: 5,
            buildings_no_pressure: 2,
            average_pressure_factor: 0.92,
            total_booster_cost: 400_000.0,
        };
        let json = serde_json::to_string(&state).expect("serialize");
        let restored: WaterPressureState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.booster_count, 2);
        assert!((restored.effective_elevation - 110.0).abs() < f32::EPSILON);
        assert_eq!(restored.buildings_full_pressure, 50);
        assert_eq!(restored.buildings_reduced_pressure, 5);
        assert_eq!(restored.buildings_no_pressure, 2);
        assert!((restored.average_pressure_factor - 0.92).abs() < 0.01);
        assert!((restored.total_booster_cost - 400_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_booster_pump_serde_roundtrip() {
        let station = BoosterPumpStation {
            grid_x: 42,
            grid_y: 99,
        };
        let json = serde_json::to_string(&station).expect("serialize");
        let restored: BoosterPumpStation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.grid_x, 42);
        assert_eq!(restored.grid_y, 99);
    }
}
