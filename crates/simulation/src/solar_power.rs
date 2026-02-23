//! Solar farm power plant simulation (POWER-005).
//!
//! Tracks solar farm output based on time-of-day, season, and weather conditions.
//! Solar farms have a 50 MW nameplate capacity with variable output depending on:
//! - Season (capacity factor): Spring=0.22, Summer=0.28, Autumn=0.18, Winter=0.12
//! - Time of day: zero at night (hours 0-6, 18-24), peak at noon
//! - Weather: Overcast=-50%, Rain=-70%, Storm=-90%
//! - Fuel cost: $0/MWh, air pollution: Q=0.0

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::time_of_day::GameClock;
use crate::utilities::{UtilitySource, UtilityType};
use crate::weather::{Season, Weather, WeatherCondition};
use crate::{decode_or_warn, Saveable, SlowTickTimer, TickCounter};

// =============================================================================
// Constants
// =============================================================================

/// Nameplate capacity of a single solar farm in MW.
pub const SOLAR_NAMEPLATE_MW: f32 = 50.0;

/// Fuel cost per MWh for solar (zero — sunshine is free).
pub const SOLAR_FUEL_COST_PER_MWH: f32 = 0.0;

/// Air pollution emission factor (zero for solar).
pub const SOLAR_AIR_POLLUTION_Q: f32 = 0.0;

/// Grid footprint of a solar farm (4x4 cells).
pub const SOLAR_FARM_FOOTPRINT: (usize, usize) = (4, 4);

// =============================================================================
// Capacity factor by season
// =============================================================================

/// Returns the seasonal capacity factor for solar farms.
pub fn seasonal_capacity_factor(season: Season) -> f32 {
    match season {
        Season::Spring => 0.22,
        Season::Summer => 0.28,
        Season::Autumn => 0.18,
        Season::Winter => 0.12,
    }
}

// =============================================================================
// Time-of-day output curve
// =============================================================================

/// Returns a multiplier in [0.0, 1.0] representing solar irradiance at the given hour.
///
/// - Hours 0-6 and 18-24: zero output (night).
/// - Peak at noon (hour 12): 1.0.
/// - Smooth sinusoidal curve between sunrise (6) and sunset (18).
pub fn time_of_day_curve(hour: f32) -> f32 {
    if !(6.0..18.0).contains(&hour) {
        return 0.0;
    }
    // Map [6, 18] -> [0, PI] for a sine curve peaking at noon.
    let t = (hour - 6.0) / 12.0 * std::f32::consts::PI;
    t.sin()
}

// =============================================================================
// Weather modifier
// =============================================================================

/// Returns a multiplier in [0.0, 1.0] reducing solar output based on weather.
///
/// - Sunny / PartlyCloudy: 1.0 (full output)
/// - Overcast: 0.5 (-50%)
/// - Rain / HeavyRain / Snow: 0.3 (-70%)
/// - Storm: 0.1 (-90%)
pub fn weather_modifier(condition: WeatherCondition) -> f32 {
    match condition {
        WeatherCondition::Sunny | WeatherCondition::PartlyCloudy => 1.0,
        WeatherCondition::Overcast => 0.5,
        WeatherCondition::Rain | WeatherCondition::HeavyRain | WeatherCondition::Snow => 0.3,
        WeatherCondition::Storm => 0.1,
    }
}

// =============================================================================
// Resource
// =============================================================================

/// City-wide solar power generation state.
#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct SolarPowerState {
    /// Number of solar farms in the city.
    pub farm_count: u32,
    /// Current output per farm in MW (after all modifiers).
    pub output_per_farm_mw: f32,
    /// Total solar output across all farms in MW.
    pub total_output_mw: f32,
    /// Current seasonal capacity factor (for UI display).
    pub current_capacity_factor: f32,
    /// Current time-of-day multiplier (for UI display).
    pub current_time_curve: f32,
    /// Current weather multiplier (for UI display).
    pub current_weather_modifier: f32,
}

impl Saveable for SolarPowerState {
    const SAVE_KEY: &'static str = "solar_power";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.farm_count == 0 && self.total_output_mw == 0.0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// System
// =============================================================================

/// Recalculates solar farm output based on current time, season, and weather.
///
/// Actual output = nameplate * capacity_factor * time_curve * weather_modifier
pub fn update_solar_power(
    tick: Res<TickCounter>,
    timer: Res<SlowTickTimer>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    mut state: ResMut<SolarPowerState>,
    utilities: Query<&UtilitySource>,
) {
    // Run on the slow tick interval to avoid unnecessary per-tick computation.
    if tick.0 == 0 || !timer.should_run() {
        return;
    }

    // Count solar farms
    let farm_count = utilities
        .iter()
        .filter(|u| u.utility_type == UtilityType::SolarFarm)
        .count() as u32;

    let capacity_factor = seasonal_capacity_factor(weather.season);
    let time_curve = time_of_day_curve(clock.hour);
    let weather_mod = weather_modifier(weather.current_event);

    let output_per_farm = SOLAR_NAMEPLATE_MW * capacity_factor * time_curve * weather_mod;

    state.farm_count = farm_count;
    state.output_per_farm_mw = output_per_farm;
    state.total_output_mw = output_per_farm * farm_count as f32;
    state.current_capacity_factor = capacity_factor;
    state.current_time_curve = time_curve;
    state.current_weather_modifier = weather_mod;
}

// =============================================================================
// Plugin
// =============================================================================

pub struct SolarPowerPlugin;

impl Plugin for SolarPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SolarPowerState>().add_systems(
            FixedUpdate,
            update_solar_power
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<SolarPowerState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_curve_night_hours_zero() {
        assert_eq!(time_of_day_curve(0.0), 0.0);
        assert_eq!(time_of_day_curve(3.0), 0.0);
        assert_eq!(time_of_day_curve(5.9), 0.0);
        assert_eq!(time_of_day_curve(18.0), 0.0);
        assert_eq!(time_of_day_curve(21.0), 0.0);
        assert_eq!(time_of_day_curve(23.9), 0.0);
    }

    #[test]
    fn test_time_curve_peaks_at_noon() {
        let noon = time_of_day_curve(12.0);
        assert!(
            (noon - 1.0).abs() < 0.001,
            "noon output should be ~1.0, got {}",
            noon
        );
    }

    #[test]
    fn test_time_curve_symmetric() {
        let morning = time_of_day_curve(9.0);
        let afternoon = time_of_day_curve(15.0);
        assert!(
            (morning - afternoon).abs() < 0.001,
            "9 AM and 3 PM should have symmetric output: {} vs {}",
            morning,
            afternoon
        );
    }

    #[test]
    fn test_time_curve_monotonic_to_noon() {
        let mut prev = 0.0;
        for h in 6..=12 {
            let val = time_of_day_curve(h as f32);
            assert!(
                val >= prev,
                "curve should increase from sunrise to noon: hour {} val {} < prev {}",
                h,
                val,
                prev
            );
            prev = val;
        }
    }

    #[test]
    fn test_seasonal_factors() {
        assert_eq!(seasonal_capacity_factor(Season::Spring), 0.22);
        assert_eq!(seasonal_capacity_factor(Season::Summer), 0.28);
        assert_eq!(seasonal_capacity_factor(Season::Autumn), 0.18);
        assert_eq!(seasonal_capacity_factor(Season::Winter), 0.12);
    }

    #[test]
    fn test_summer_higher_than_winter() {
        assert!(
            seasonal_capacity_factor(Season::Summer) > seasonal_capacity_factor(Season::Winter)
        );
    }

    #[test]
    fn test_weather_modifiers() {
        assert_eq!(weather_modifier(WeatherCondition::Sunny), 1.0);
        assert_eq!(weather_modifier(WeatherCondition::PartlyCloudy), 1.0);
        assert_eq!(weather_modifier(WeatherCondition::Overcast), 0.5);
        assert_eq!(weather_modifier(WeatherCondition::Rain), 0.3);
        assert_eq!(weather_modifier(WeatherCondition::HeavyRain), 0.3);
        assert_eq!(weather_modifier(WeatherCondition::Snow), 0.3);
        assert_eq!(weather_modifier(WeatherCondition::Storm), 0.1);
    }

    #[test]
    fn test_storm_heavily_reduces_output() {
        let sunny = weather_modifier(WeatherCondition::Sunny);
        let storm = weather_modifier(WeatherCondition::Storm);
        assert!(storm < sunny * 0.15, "storm should reduce output by >= 85%");
    }

    #[test]
    fn test_full_output_formula() {
        // Summer, noon, sunny: maximum output
        let output = SOLAR_NAMEPLATE_MW
            * seasonal_capacity_factor(Season::Summer)
            * time_of_day_curve(12.0)
            * weather_modifier(WeatherCondition::Sunny);
        let expected = 50.0 * 0.28 * 1.0 * 1.0;
        assert!(
            (output - expected).abs() < 0.001,
            "peak summer output should be {} MW, got {}",
            expected,
            output
        );
    }

    #[test]
    fn test_night_output_zero() {
        let output = SOLAR_NAMEPLATE_MW
            * seasonal_capacity_factor(Season::Summer)
            * time_of_day_curve(2.0)
            * weather_modifier(WeatherCondition::Sunny);
        assert_eq!(output, 0.0, "night output should be zero");
    }

    #[test]
    fn test_saveable_roundtrip() {
        let state = SolarPowerState {
            farm_count: 3,
            output_per_farm_mw: 7.0,
            total_output_mw: 21.0,
            current_capacity_factor: 0.28,
            current_time_curve: 1.0,
            current_weather_modifier: 0.5,
        };
        let bytes = state.save_to_bytes().unwrap();
        let loaded = SolarPowerState::load_from_bytes(&bytes);
        assert_eq!(loaded.farm_count, 3);
        assert!((loaded.total_output_mw - 21.0).abs() < 0.001);
    }

    #[test]
    fn test_saveable_skip_when_empty() {
        let state = SolarPowerState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "empty state should skip save"
        );
    }
}
