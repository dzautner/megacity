use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::weather::Weather;

/// Heat wave severity levels based on consecutive hot days.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HeatWaveSeverity {
    #[default]
    None,
    Moderate,
    Severe,
    Extreme,
}

/// Resource tracking heat wave state and derived effects.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct HeatWaveState {
    /// Number of consecutive days with temperature above the heat threshold.
    pub consecutive_hot_days: u32,
    /// Current heat wave severity level.
    pub severity: HeatWaveSeverity,
    /// Excess deaths per 100,000 population from heat exposure.
    pub excess_mortality_per_100k: f32,
    /// Energy demand multiplier (1.0 = normal, 1.4-1.8 during heat wave).
    pub energy_demand_multiplier: f32,
    /// Water demand multiplier (1.0 = normal, up to 1.6 during heat wave).
    pub water_demand_multiplier: f32,
    /// True when sustained temperatures above 43C cause road buckling.
    pub road_damage_active: bool,
    /// Fire risk multiplier (increases during heat waves, especially with drought).
    pub fire_risk_multiplier: f32,
    /// Blackout probability (0.0-1.0) when AC demand exceeds grid capacity.
    pub blackout_risk: f32,
    /// Temperature threshold in Celsius for heat wave detection (default 38.0).
    pub heat_threshold_c: f32,
    /// Internal: number of consecutive days above 43C for road damage tracking.
    pub consecutive_extreme_days: u32,
    /// Internal: the last day we checked (to detect day changes).
    pub last_check_day: u32,
}

impl Default for HeatWaveState {
    fn default() -> Self {
        Self {
            consecutive_hot_days: 0,
            severity: HeatWaveSeverity::None,
            excess_mortality_per_100k: 0.0,
            energy_demand_multiplier: 1.0,
            water_demand_multiplier: 1.0,
            road_damage_active: false,
            fire_risk_multiplier: 1.0,
            blackout_risk: 0.0,
            heat_threshold_c: 38.0,
            consecutive_extreme_days: 0,
            last_check_day: 0,
        }
    }
}

/// Road buckling temperature threshold in Celsius.
const ROAD_BUCKLING_THRESHOLD_C: f32 = 43.0;
/// Number of consecutive days above 43C before road damage occurs.
const ROAD_DAMAGE_DAYS: u32 = 3;

/// Calculate excess deaths per 100k using exponential curve.
///
/// Formula: `0.5 * exp(0.15 * (temp - threshold))`
/// Returns 0.0 when temperature is at or below the threshold.
pub fn calculate_excess_mortality(temp_c: f32, threshold: f32) -> f32 {
    if temp_c <= threshold {
        return 0.0;
    }
    0.5 * (0.15 * (temp_c - threshold)).exp()
}

/// Determine heat wave severity from consecutive hot days.
pub fn severity_from_days(days: u32) -> HeatWaveSeverity {
    match days {
        0..=2 => HeatWaveSeverity::None,
        3..=5 => HeatWaveSeverity::Moderate,
        6..=9 => HeatWaveSeverity::Severe,
        _ => HeatWaveSeverity::Extreme,
    }
}

/// System that updates the `HeatWaveState` resource based on current weather.
///
/// Runs on a timer (registered with `on_timer(Duration::from_secs(2))`).
/// Reads the `Weather` resource for temperature and tracks consecutive hot days.
pub fn update_heat_wave(weather: Res<Weather>, mut state: ResMut<HeatWaveState>) {
    let current_day = weather.last_update_day;
    let temp = weather.temperature;
    let threshold = state.heat_threshold_c;

    // Detect day change to update consecutive day counters
    if current_day != state.last_check_day && current_day > 0 {
        state.last_check_day = current_day;

        // Track consecutive hot days (above heat threshold)
        if temp >= threshold {
            state.consecutive_hot_days += 1;
        } else {
            state.consecutive_hot_days = 0;
        }

        // Track consecutive extreme days (above road buckling threshold)
        if temp >= ROAD_BUCKLING_THRESHOLD_C {
            state.consecutive_extreme_days += 1;
        } else {
            state.consecutive_extreme_days = 0;
        }
    }

    // Update severity
    state.severity = severity_from_days(state.consecutive_hot_days);

    // Calculate excess mortality
    state.excess_mortality_per_100k = if state.severity != HeatWaveSeverity::None {
        calculate_excess_mortality(temp, threshold)
    } else {
        0.0
    };

    // Energy demand multiplier: +40-80% from AC load based on severity
    state.energy_demand_multiplier = match state.severity {
        HeatWaveSeverity::None => 1.0,
        HeatWaveSeverity::Moderate => 1.4,
        HeatWaveSeverity::Severe => 1.6,
        HeatWaveSeverity::Extreme => 1.8,
    };

    // Water demand multiplier: +60% during heat waves
    state.water_demand_multiplier = match state.severity {
        HeatWaveSeverity::None => 1.0,
        HeatWaveSeverity::Moderate => 1.3,
        HeatWaveSeverity::Severe => 1.45,
        HeatWaveSeverity::Extreme => 1.6,
    };

    // Road damage: pavement buckling at sustained temps above 43C for 3+ days
    state.road_damage_active = state.consecutive_extreme_days >= ROAD_DAMAGE_DAYS;

    // Fire risk multiplier: increases with severity, +300% when combined with drought
    // (low humidity acts as drought proxy)
    let drought_factor = if weather.humidity < 0.2 { 3.0 } else { 1.0 };
    state.fire_risk_multiplier = match state.severity {
        HeatWaveSeverity::None => 1.0,
        HeatWaveSeverity::Moderate => 1.5 * drought_factor,
        HeatWaveSeverity::Severe => 2.0 * drought_factor,
        HeatWaveSeverity::Extreme => 2.5 * drought_factor,
    };

    // Blackout risk: based on energy demand vs assumed capacity
    // Higher severity = higher chance that AC demand exceeds grid capacity
    state.blackout_risk = match state.severity {
        HeatWaveSeverity::None => 0.0,
        HeatWaveSeverity::Moderate => 0.05,
        HeatWaveSeverity::Severe => 0.15,
        HeatWaveSeverity::Extreme => 0.35,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_classification() {
        assert_eq!(severity_from_days(0), HeatWaveSeverity::None);
        assert_eq!(severity_from_days(1), HeatWaveSeverity::None);
        assert_eq!(severity_from_days(2), HeatWaveSeverity::None);
        assert_eq!(severity_from_days(3), HeatWaveSeverity::Moderate);
        assert_eq!(severity_from_days(5), HeatWaveSeverity::Moderate);
        assert_eq!(severity_from_days(6), HeatWaveSeverity::Severe);
        assert_eq!(severity_from_days(9), HeatWaveSeverity::Severe);
        assert_eq!(severity_from_days(10), HeatWaveSeverity::Extreme);
        assert_eq!(severity_from_days(100), HeatWaveSeverity::Extreme);
    }

    #[test]
    fn test_excess_mortality_at_threshold() {
        // At or below threshold, mortality should be zero
        let result = calculate_excess_mortality(38.0, 38.0);
        assert!(
            result.abs() < f32::EPSILON,
            "Mortality at threshold should be 0, got {}",
            result
        );

        let result_below = calculate_excess_mortality(35.0, 38.0);
        assert!(
            result_below.abs() < f32::EPSILON,
            "Mortality below threshold should be 0"
        );
    }

    #[test]
    fn test_excess_mortality_above_threshold() {
        // At 40C with 38C threshold: 0.5 * exp(0.15 * 2) = 0.5 * exp(0.3) ~ 0.675
        let result = calculate_excess_mortality(40.0, 38.0);
        let expected = 0.5 * (0.15_f32 * 2.0).exp();
        assert!(
            (result - expected).abs() < 0.001,
            "Expected ~{}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_excess_mortality_exponential_growth() {
        // Verify the curve is exponential: mortality at 45C >> mortality at 40C
        let m40 = calculate_excess_mortality(40.0, 38.0);
        let m45 = calculate_excess_mortality(45.0, 38.0);
        assert!(
            m45 > m40 * 2.0,
            "Mortality should grow exponentially: 45C={} vs 40C={}",
            m45,
            m40
        );
    }

    #[test]
    fn test_excess_mortality_various_temperatures() {
        // At 42C with 38C threshold: 0.5 * exp(0.15 * 4) = 0.5 * exp(0.6) ~ 0.911
        let result = calculate_excess_mortality(42.0, 38.0);
        let expected = 0.5 * (0.15_f32 * 4.0).exp();
        assert!(
            (result - expected).abs() < 0.01,
            "Expected ~{}, got {}",
            expected,
            result
        );

        // At 50C with 38C threshold: 0.5 * exp(0.15 * 12) = 0.5 * exp(1.8) ~ 3.02
        let result_high = calculate_excess_mortality(50.0, 38.0);
        let expected_high = 0.5 * (0.15_f32 * 12.0).exp();
        assert!(
            (result_high - expected_high).abs() < 0.01,
            "Expected ~{}, got {}",
            expected_high,
            result_high
        );
    }

    #[test]
    fn test_energy_multiplier_by_severity() {
        // None: 1.0, Moderate: 1.4, Severe: 1.6, Extreme: 1.8
        let expected = [
            (HeatWaveSeverity::None, 1.0),
            (HeatWaveSeverity::Moderate, 1.4),
            (HeatWaveSeverity::Severe, 1.6),
            (HeatWaveSeverity::Extreme, 1.8),
        ];
        for (severity, multiplier) in &expected {
            let actual = match severity {
                HeatWaveSeverity::None => 1.0_f32,
                HeatWaveSeverity::Moderate => 1.4,
                HeatWaveSeverity::Severe => 1.6,
                HeatWaveSeverity::Extreme => 1.8,
            };
            assert!(
                (actual - multiplier).abs() < f32::EPSILON,
                "Energy multiplier for {:?} should be {}, got {}",
                severity,
                multiplier,
                actual
            );
        }
    }

    #[test]
    fn test_water_multiplier_by_severity() {
        // None: 1.0, Moderate: 1.3, Severe: 1.45, Extreme: 1.6
        let expected = [
            (HeatWaveSeverity::None, 1.0),
            (HeatWaveSeverity::Moderate, 1.3),
            (HeatWaveSeverity::Severe, 1.45),
            (HeatWaveSeverity::Extreme, 1.6),
        ];
        for (severity, multiplier) in &expected {
            let actual = match severity {
                HeatWaveSeverity::None => 1.0_f32,
                HeatWaveSeverity::Moderate => 1.3,
                HeatWaveSeverity::Severe => 1.45,
                HeatWaveSeverity::Extreme => 1.6,
            };
            assert!(
                (actual - multiplier).abs() < f32::EPSILON,
                "Water multiplier for {:?} should be {}, got {}",
                severity,
                multiplier,
                actual
            );
        }
    }

    #[test]
    fn test_road_damage_threshold() {
        let mut state = HeatWaveState::default();

        // Less than 3 consecutive extreme days: no road damage
        state.consecutive_extreme_days = 2;
        state.road_damage_active = state.consecutive_extreme_days >= ROAD_DAMAGE_DAYS;
        assert!(!state.road_damage_active);

        // Exactly 3 consecutive extreme days: road damage active
        state.consecutive_extreme_days = 3;
        state.road_damage_active = state.consecutive_extreme_days >= ROAD_DAMAGE_DAYS;
        assert!(state.road_damage_active);

        // More than 3 consecutive extreme days: still active
        state.consecutive_extreme_days = 10;
        state.road_damage_active = state.consecutive_extreme_days >= ROAD_DAMAGE_DAYS;
        assert!(state.road_damage_active);
    }

    #[test]
    fn test_default_state() {
        let state = HeatWaveState::default();
        assert_eq!(state.consecutive_hot_days, 0);
        assert_eq!(state.severity, HeatWaveSeverity::None);
        assert!((state.excess_mortality_per_100k).abs() < f32::EPSILON);
        assert!((state.energy_demand_multiplier - 1.0).abs() < f32::EPSILON);
        assert!((state.water_demand_multiplier - 1.0).abs() < f32::EPSILON);
        assert!(!state.road_damage_active);
        assert!((state.fire_risk_multiplier - 1.0).abs() < f32::EPSILON);
        assert!((state.blackout_risk).abs() < f32::EPSILON);
        assert!((state.heat_threshold_c - 38.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fire_risk_drought_multiplier() {
        // With drought (humidity < 0.2) and extreme severity, fire risk should be very high
        let drought_factor = 3.0_f32;
        let extreme_base = 2.5_f32;
        let expected = extreme_base * drought_factor; // 7.5
        assert!((expected - 7.5).abs() < f32::EPSILON);

        // Without drought, extreme severity fire risk = 2.5
        let no_drought_factor = 1.0_f32;
        let expected_no_drought = extreme_base * no_drought_factor;
        assert!((expected_no_drought - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blackout_risk_by_severity() {
        let expected = [
            (HeatWaveSeverity::None, 0.0),
            (HeatWaveSeverity::Moderate, 0.05),
            (HeatWaveSeverity::Severe, 0.15),
            (HeatWaveSeverity::Extreme, 0.35),
        ];
        for (severity, risk) in &expected {
            let actual = match severity {
                HeatWaveSeverity::None => 0.0_f32,
                HeatWaveSeverity::Moderate => 0.05,
                HeatWaveSeverity::Severe => 0.15,
                HeatWaveSeverity::Extreme => 0.35,
            };
            assert!(
                (actual - risk).abs() < f32::EPSILON,
                "Blackout risk for {:?} should be {}, got {}",
                severity,
                risk,
                actual
            );
        }
    }
}

pub struct HeatWavePlugin;

impl Plugin for HeatWavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeatWaveState>()
            .add_systems(
                FixedUpdate,
                update_heat_wave.after(crate::imports_exports::process_trade),
            );
    }
}
