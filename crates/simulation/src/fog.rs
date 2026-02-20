use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::{CellType, WorldGrid};
use crate::time_of_day::GameClock;
use crate::weather::Weather;

/// Fog density tiers for gameplay logic and UI display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FogDensity {
    /// No fog, full visibility.
    None,
    /// Light mist, visibility > 1 km. Slight atmospheric haze.
    Mist,
    /// Moderate fog, visibility 200m - 1 km. Traffic caution advised.
    Moderate,
    /// Dense fog, visibility < 200m. Traffic severely impacted, flights suspended.
    Dense,
}

impl FogDensity {
    /// Human-readable name for UI display.
    pub fn name(self) -> &'static str {
        match self {
            FogDensity::None => "Clear",
            FogDensity::Mist => "Mist",
            FogDensity::Moderate => "Fog",
            FogDensity::Dense => "Dense Fog",
        }
    }
}

/// Persistent fog state resource tracking fog conditions and duration.
///
/// Fog forms when humidity is high (>90%) and temperature drops near the dew point
/// (within 2C). It is more likely near water cells and during early morning hours (4-8).
/// Fog typically lasts 2-4 game-hours and burns off by midday as temperature rises.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct FogState {
    /// Whether fog is currently active.
    pub active: bool,
    /// Current fog density tier.
    pub density: FogDensity,
    /// Visibility in meters (0.0 = total whiteout, 10000.0 = perfectly clear).
    pub visibility_m: f32,
    /// Number of game-hours the current fog event has been active.
    pub hours_active: u32,
    /// Maximum duration for the current fog event (2-4 hours).
    pub max_duration_hours: u32,
    /// Fraction of the grid that is water cells (cached, recomputed periodically).
    pub water_fraction: f32,
    /// Last day the water fraction was computed (to avoid recomputing every tick).
    pub water_fraction_last_day: u32,
    /// Traffic speed modifier applied during fog (1.0 = no effect, 0.8 = -20%).
    pub traffic_speed_modifier: f32,
    /// Whether airports should suspend flight operations due to dense fog.
    pub flights_suspended: bool,
    /// Last hour that was processed (to detect hour boundaries).
    pub last_update_hour: u32,
}

impl Default for FogState {
    fn default() -> Self {
        Self {
            active: false,
            density: FogDensity::None,
            visibility_m: 10000.0,
            hours_active: 0,
            max_duration_hours: 0,
            water_fraction: 0.0,
            water_fraction_last_day: 0,
            traffic_speed_modifier: 1.0,
            flights_suspended: false,
            last_update_hour: 0,
        }
    }
}

impl FogState {
    /// Classify current visibility into a fog density tier.
    pub fn fog_density(&self) -> FogDensity {
        if self.visibility_m > 1000.0 && !self.active {
            FogDensity::None
        } else if self.visibility_m > 1000.0 {
            FogDensity::Mist
        } else if self.visibility_m > 200.0 {
            FogDensity::Moderate
        } else {
            FogDensity::Dense
        }
    }
}

/// Calculate the dew point temperature (Magnus formula approximation).
///
/// Given ambient temperature in Celsius and relative humidity (0.0 - 1.0),
/// returns the dew point in Celsius.
pub fn dew_point(temperature_c: f32, humidity: f32) -> f32 {
    // Magnus formula constants (for -45C to 60C range)
    let a = 17.27;
    let b = 237.7;
    let h = humidity.clamp(0.01, 1.0); // avoid log(0)

    let gamma = (a * temperature_c) / (b + temperature_c) + h.ln();
    (b * gamma) / (a - gamma)
}

/// Hourly fog update system. Checks atmospheric conditions each hour to determine
/// whether fog should form, persist, or dissipate.
///
/// Fog formation conditions:
/// - Humidity > 90%
/// - Temperature within 2C of dew point
/// - More likely near water cells (water_fraction > 0.05 adds bonus)
/// - More likely during early morning hours (4:00 - 8:00)
///
/// Fog dissipation:
/// - Burns off when temperature rises more than 4C above dew point
/// - Maximum duration of 2-4 game-hours
/// - Typically clears by midday (hour 12)
pub fn update_fog(
    clock: Res<GameClock>,
    weather: Res<Weather>,
    grid: Res<WorldGrid>,
    mut fog: ResMut<FogState>,
) {
    let current_hour = clock.hour_of_day();

    // Only update on hour boundaries
    if current_hour == fog.last_update_hour && !fog.active {
        return;
    }

    // Detect hour boundary crossing
    let hour_changed = current_hour != fog.last_update_hour;
    if hour_changed {
        fog.last_update_hour = current_hour;
    }

    // Recompute water fraction periodically (once per day)
    if clock.day != fog.water_fraction_last_day {
        let total_cells = (grid.width * grid.height) as f32;
        if total_cells > 0.0 {
            let water_count = grid
                .cells
                .iter()
                .filter(|c| c.cell_type == CellType::Water)
                .count() as f32;
            fog.water_fraction = water_count / total_cells;
        }
        fog.water_fraction_last_day = clock.day;
    }

    // Calculate dew point
    let dp = dew_point(weather.temperature, weather.humidity);
    let temp_above_dew = weather.temperature - dp;

    if fog.active {
        // --- Fog is currently active: check for dissipation ---
        if hour_changed {
            fog.hours_active += 1;
        }

        // Fog burns off when:
        // 1. Temperature rises > 4C above dew point, OR
        // 2. Duration exceeds max_duration_hours, OR
        // 3. It's past midday (hour >= 12) and conditions aren't perfect
        let should_dissipate = fog.hours_active >= fog.max_duration_hours
            || temp_above_dew > 4.0
            || (current_hour >= 12 && temp_above_dew > 2.0)
            || weather.humidity < 0.70;

        if should_dissipate {
            // Fog clears
            fog.active = false;
            fog.density = FogDensity::None;
            fog.visibility_m = 10000.0;
            fog.hours_active = 0;
            fog.max_duration_hours = 0;
            fog.traffic_speed_modifier = 1.0;
            fog.flights_suspended = false;
        } else {
            // Fog persists - update density based on current conditions
            update_fog_density(&weather, temp_above_dew, &mut fog);
        }
    } else if hour_changed {
        // --- No fog: check if fog should form ---
        let humidity_ok = weather.humidity > 0.90;
        let dew_point_ok = (0.0..2.0).contains(&temp_above_dew);

        if humidity_ok && dew_point_ok {
            // Base fog formation chance
            let mut fog_chance: f32 = 0.3;

            // Early morning bonus (hours 4-8)
            if (4..=8).contains(&current_hour) {
                fog_chance += 0.3;
            }

            // Water proximity bonus
            if fog.water_fraction > 0.05 {
                fog_chance += fog.water_fraction.min(0.3);
            }

            // Precipitation suppresses fog (rain washes it out, but drizzle doesn't)
            if weather.precipitation_intensity > 0.25 {
                fog_chance *= 0.2;
            }

            // Wind suppresses fog (high cloud cover often means wind)
            if weather.cloud_cover > 0.85 {
                fog_chance *= 0.5;
            }

            // Deterministic hash for fog decision
            let hash = (clock
                .day
                .wrapping_mul(7723)
                .wrapping_add(current_hour.wrapping_mul(4591)))
                % 100;
            let threshold = (fog_chance * 100.0).min(99.0) as u32;

            if hash < threshold {
                // Fog forms!
                fog.active = true;
                fog.hours_active = 0;

                // Duration: 2-4 hours, deterministic from day
                fog.max_duration_hours = 2 + (hash % 3); // 2, 3, or 4 hours

                // Set initial density
                update_fog_density(&weather, temp_above_dew, &mut fog);
            }
        }
    }
}

/// Update fog density, visibility, traffic modifier, and flight status
/// based on current atmospheric conditions.
fn update_fog_density(weather: &Weather, temp_above_dew: f32, fog: &mut FogState) {
    // Denser fog when temperature is very close to dew point and humidity is very high
    let density_factor = if temp_above_dew < 0.5 && weather.humidity > 0.95 {
        // Dense fog: visibility < 200m
        0.99
    } else if temp_above_dew < 1.0 && weather.humidity > 0.92 {
        // Moderate fog: visibility 200-1000m
        0.95
    } else {
        // Light mist: visibility > 1000m
        0.3
    };

    // Visibility ranges: Dense < 200m, Moderate 200-1000m, Mist 1000-5000m
    fog.visibility_m = 10000.0 * (1.0 - density_factor);
    fog.visibility_m = fog.visibility_m.clamp(50.0, 10000.0);

    fog.density = fog.fog_density();

    // Traffic speed modifier: -20% in fog (as specified in issue)
    fog.traffic_speed_modifier = match fog.density {
        FogDensity::None => 1.0,
        FogDensity::Mist => 0.9,
        FogDensity::Moderate => 0.8,
        FogDensity::Dense => 0.7, // -30% for dense fog
    };

    // Flights suspended in dense fog (visibility < 200m)
    fog.flights_suspended = fog.density == FogDensity::Dense;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dew_point_calculation() {
        // At 100% humidity, dew point equals temperature
        let dp = dew_point(20.0, 1.0);
        assert!(
            (dp - 20.0).abs() < 0.5,
            "At 100% humidity, dew point should be ~20C, got {}",
            dp
        );

        // At lower humidity, dew point is below temperature
        let dp = dew_point(20.0, 0.5);
        assert!(dp < 20.0, "At 50% humidity, dew point should be below 20C");
        assert!(dp > 0.0, "Dew point should be positive for 20C/50%");
    }

    #[test]
    fn test_dew_point_high_humidity() {
        // At 95% humidity and 15C, dew point should be close to 15C
        let dp = dew_point(15.0, 0.95);
        let diff = 15.0 - dp;
        assert!(
            diff < 2.0,
            "At 95% humidity, temp-dewpoint should be < 2C, got {}",
            diff
        );
    }

    #[test]
    fn test_fog_density_classification() {
        let mut fog = FogState::default();

        fog.active = true;
        fog.visibility_m = 100.0;
        assert_eq!(fog.fog_density(), FogDensity::Dense);

        fog.visibility_m = 500.0;
        assert_eq!(fog.fog_density(), FogDensity::Moderate);

        fog.visibility_m = 3000.0;
        assert_eq!(fog.fog_density(), FogDensity::Mist);

        fog.active = false;
        fog.visibility_m = 10000.0;
        assert_eq!(fog.fog_density(), FogDensity::None);
    }

    #[test]
    fn test_fog_state_default() {
        let fog = FogState::default();
        assert!(!fog.active);
        assert_eq!(fog.density, FogDensity::None);
        assert!((fog.visibility_m - 10000.0).abs() < f32::EPSILON);
        assert!((fog.traffic_speed_modifier - 1.0).abs() < f32::EPSILON);
        assert!(!fog.flights_suspended);
    }

    #[test]
    fn test_fog_density_names() {
        assert_eq!(FogDensity::None.name(), "Clear");
        assert_eq!(FogDensity::Mist.name(), "Mist");
        assert_eq!(FogDensity::Moderate.name(), "Fog");
        assert_eq!(FogDensity::Dense.name(), "Dense Fog");
    }

    #[test]
    fn test_traffic_speed_modifier_values() {
        let mut fog = FogState::default();
        let weather = Weather::default();

        // Dense fog
        update_fog_density(&weather, 0.3, &mut fog);
        // With default weather humidity (0.5), this won't produce dense fog
        // but let's test with known conditions

        // Simulate high humidity weather
        let mut w = Weather::default();
        w.humidity = 0.96;

        update_fog_density(&w, 0.3, &mut fog);
        assert!(
            fog.traffic_speed_modifier <= 0.8,
            "Dense fog should have speed modifier <= 0.8, got {}",
            fog.traffic_speed_modifier
        );
    }

    #[test]
    fn test_dense_fog_suspends_flights() {
        let mut fog = FogState::default();
        let mut w = Weather::default();
        w.humidity = 0.96;

        update_fog_density(&w, 0.3, &mut fog);
        assert!(fog.flights_suspended, "Dense fog should suspend flights");
    }

    #[test]
    fn test_no_fog_no_flight_suspension() {
        let fog = FogState::default();
        assert!(!fog.flights_suspended, "No fog should not suspend flights");
    }
}
