use bevy::prelude::*;

use crate::time_of_day::GameClock;
use crate::weather::{ClimateZone, Weather, WeatherCondition};
use crate::TickCounter;

// =============================================================================
// Wind State
// =============================================================================

/// Global wind resource tracking direction and speed.
/// Direction is in radians [0, 2*PI): 0 = East, PI/2 = North, PI = West, 3PI/2 = South.
/// Speed is in [0, 1]: 0 = calm, 1 = strong.
#[derive(Resource, Debug, Clone)]
pub struct WindState {
    /// Wind direction in radians [0, 2*PI).
    pub direction: f32,
    /// Wind speed in [0, 1].
    pub speed: f32,
    /// Prevailing wind direction in radians. Defaults to westerly (PI, blowing eastward).
    /// The issue specifies 270 degrees = 3*PI/2 in compass terms, but since our coordinate
    /// system uses 0 = East, PI = West, a westerly wind blowing east is direction 0.
    /// We store the prevailing direction as the direction the wind blows *toward*.
    /// 270 degrees compass = westerly origin = wind blows toward east = 0 radians in our system.
    /// However, the issue says "prevailing wind direction: 270 degrees" which is the *source*
    /// direction. In our system direction means where wind blows toward, so prevailing = 0 rad
    /// would be an east-ward wind from the west. To match the issue's "270 degrees" literally
    /// (as a compass heading where the wind comes FROM), we convert: 270 deg = 3*PI/2 rad.
    /// But in our radians system 3*PI/2 = South direction. The issue likely means a standard
    /// westerly (wind FROM the west), so prevailing_direction = 0.0 (east-ward) is correct.
    ///
    /// We'll use radians matching our direction convention: 0 = blowing east (westerly wind).
    pub prevailing_direction: f32,
    /// Remaining ticks for a wind gust event. During gusts, speed is temporarily spiked.
    pub gust_remaining: u32,
    /// The previous weather condition, used to detect transitions that trigger gusts.
    pub prev_condition: WeatherCondition,
}

impl Default for WindState {
    fn default() -> Self {
        Self {
            // Default: gentle westerly wind (blowing from the west toward the east)
            direction: 0.0, // 0 radians = east-ward (westerly)
            speed: 0.3,
            prevailing_direction: 0.0, // westerly default (blowing east)
            gust_remaining: 0,
            prev_condition: WeatherCondition::Sunny,
        }
    }
}

impl WindState {
    /// Returns the compass direction as a string (N, NE, E, SE, S, SW, W, NW).
    /// The direction indicates where the wind is blowing **toward**.
    pub fn compass_direction(&self) -> &'static str {
        // Normalize to [0, 2*PI)
        let angle = self.direction.rem_euclid(std::f32::consts::TAU);
        // Divide the circle into 8 sectors of PI/4 each, offset by PI/8
        let sector =
            ((angle + std::f32::consts::FRAC_PI_8) / std::f32::consts::FRAC_PI_4) as u32 % 8;
        match sector {
            0 => "E",
            1 => "NE",
            2 => "N",
            3 => "NW",
            4 => "W",
            5 => "SW",
            6 => "S",
            7 => "SE",
            _ => "E",
        }
    }

    /// Returns a human-readable speed label.
    pub fn speed_label(&self) -> &'static str {
        if self.speed < 0.15 {
            "Calm"
        } else if self.speed < 0.4 {
            "Light"
        } else if self.speed < 0.7 {
            "Moderate"
        } else {
            "Strong"
        }
    }

    /// Returns the (dx, dy) unit vector of wind direction.
    /// dx = cos(direction), dy = sin(direction).
    pub fn direction_vector(&self) -> (f32, f32) {
        (self.direction.cos(), self.direction.sin())
    }
}

/// Returns the prevailing wind direction for a climate zone (in radians).
///
/// - Temperate / Oceanic / Continental: westerly (0.0 rad, blowing east)
/// - Tropical: easterly trade winds (PI rad, blowing west)
/// - Arid: variable, slight northerly (PI/2 + 0.3 rad)
/// - Mediterranean: westerly with slight north (0.3 rad)
/// - Subarctic: polar easterly (PI rad, blowing west)
pub fn prevailing_direction_for_zone(zone: ClimateZone) -> f32 {
    match zone {
        ClimateZone::Temperate => 0.0,                          // westerly
        ClimateZone::Tropical => std::f32::consts::PI,          // trade winds (easterly)
        ClimateZone::Arid => std::f32::consts::FRAC_PI_2 + 0.3, // variable northerly
        ClimateZone::Mediterranean => 0.3,                      // westerly with slight north
        ClimateZone::Continental => 0.0,                        // westerly
        ClimateZone::Subarctic => std::f32::consts::PI,         // polar easterly
        ClimateZone::Oceanic => 0.0,                            // westerly
    }
}

// =============================================================================
// Deterministic pseudo-random (splitmix64, matching disasters.rs)
// =============================================================================

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Returns a deterministic pseudo-random f32 in [-1.0, 1.0) based on seed.
fn rand_signed_f32(seed: u64) -> f32 {
    let hash = splitmix64(seed);
    // Map to [-1.0, 1.0)
    (hash % 2_000_000) as f32 / 1_000_000.0 - 1.0
}

/// Returns a deterministic pseudo-random f32 in [0.0, 1.0) based on seed.
fn rand_unsigned_f32(seed: u64) -> f32 {
    let hash = splitmix64(seed);
    (hash % 1_000_000) as f32 / 1_000_000.0
}

// =============================================================================
// Systems
// =============================================================================

/// Wind update interval in ticks (matches SlowTickTimer::INTERVAL).
const WIND_UPDATE_INTERVAL: u64 = 100;

/// Maximum angular perturbation per update (radians). ~5 degrees (calmer than old +-10).
const MAX_DIRECTION_SHIFT: f32 = 0.087; // ~5 degrees in radians

/// Reversion factor: how strongly direction reverts toward prevailing each update.
const PREVAILING_REVERSION_FACTOR: f32 = 0.1;

/// Diurnal wind boost for afternoon hours (12-18): 20% stronger.
const DIURNAL_AFTERNOON_BOOST: f32 = 1.2;

/// Computes the shortest signed angular difference from `from` to `to` (in radians).
/// Result is in [-PI, PI].
fn angle_diff(from: f32, to: f32) -> f32 {
    let diff = (to - from).rem_euclid(std::f32::consts::TAU);
    if diff > std::f32::consts::PI {
        diff - std::f32::consts::TAU
    } else {
        diff
    }
}

/// Updates wind direction and speed with weather-aware variation every WIND_UPDATE_INTERVAL ticks.
///
/// Features:
/// - Direction reverts toward prevailing wind (set per climate zone)
/// - Random perturbation of +-5 degrees per update (calmer than old +-10)
/// - Storm events boost wind speed to 0.6-0.9
/// - Calm clear weather reduces wind speed to 0.0-0.1
/// - Diurnal variation: afternoon winds (12-18) are 20% stronger
/// - Wind gust events: temporary speed spikes during weather transitions
pub fn update_wind(
    tick: Res<TickCounter>,
    mut wind: ResMut<WindState>,
    weather: Res<Weather>,
    climate: Res<ClimateZone>,
    clock: Res<GameClock>,
) {
    if tick.0 == 0 || !tick.0.is_multiple_of(WIND_UPDATE_INTERVAL) {
        return;
    }

    // Update prevailing direction from climate zone
    wind.prevailing_direction = prevailing_direction_for_zone(*climate);

    // Use tick as seed for deterministic pseudo-random
    let dir_seed = tick.0.wrapping_mul(0xa1b2c3d4e5f60718);
    let spd_seed = tick.0.wrapping_mul(0x1234567890abcdef);
    let gust_seed = tick.0.wrapping_mul(0xfedcba9876543210);

    // ---- Direction update ----
    // 1. Revert toward prevailing direction
    let reversion =
        angle_diff(wind.direction, wind.prevailing_direction) * PREVAILING_REVERSION_FACTOR;
    wind.direction = (wind.direction + reversion).rem_euclid(std::f32::consts::TAU);

    // 2. Random perturbation (+-5 degrees)
    let dir_delta = rand_signed_f32(dir_seed) * MAX_DIRECTION_SHIFT;
    wind.direction = (wind.direction + dir_delta).rem_euclid(std::f32::consts::TAU);

    // ---- Speed update ----
    // Determine target speed based on weather conditions
    let condition = weather.current_event;
    let target_speed = match condition {
        // Storm: high wind speed 0.6-0.9
        WeatherCondition::Storm => 0.6 + rand_unsigned_f32(spd_seed) * 0.3,
        // Heavy rain: moderately elevated wind
        WeatherCondition::HeavyRain => 0.4 + rand_unsigned_f32(spd_seed) * 0.2,
        // Clear/Sunny with low cloud cover (proxy for high pressure): calm winds 0.0-0.1
        WeatherCondition::Sunny if weather.cloud_cover < 0.15 => rand_unsigned_f32(spd_seed) * 0.1,
        // Light/moderate conditions: gentle random walk around 0.2-0.4
        _ => 0.2 + rand_unsigned_f32(spd_seed) * 0.2,
    };

    // Smooth transition toward target speed
    let base_speed = wind.speed + (target_speed - wind.speed) * 0.3;

    // ---- Diurnal variation ----
    // Afternoon hours (12-18) get a 20% boost
    let hour = clock.hour_of_day();
    let diurnal_multiplier = if (12..=18).contains(&hour) {
        DIURNAL_AFTERNOON_BOOST
    } else {
        1.0
    };

    wind.speed = (base_speed * diurnal_multiplier).clamp(0.0, 1.0);

    // ---- Gust events ----
    // Detect weather transitions that trigger gusts
    if wind.prev_condition != condition {
        // Weather changed: trigger a gust lasting 1-2 ticks
        let gust_duration = 1 + (splitmix64(gust_seed) % 2) as u32;
        wind.gust_remaining = gust_duration;
    }
    wind.prev_condition = condition;

    // Apply gust: temporary speed spike
    if wind.gust_remaining > 0 {
        let gust_boost = 0.2 + rand_unsigned_f32(gust_seed.wrapping_add(tick.0)) * 0.15;
        wind.speed = (wind.speed + gust_boost).clamp(0.0, 1.0);
        wind.gust_remaining -= 1;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::Weather;

    #[test]
    fn test_wind_default() {
        let w = WindState::default();
        assert!((w.direction - 0.0).abs() < f32::EPSILON);
        assert!((w.speed - 0.3).abs() < f32::EPSILON);
        assert!((w.prevailing_direction - 0.0).abs() < f32::EPSILON);
        assert_eq!(w.gust_remaining, 0);
    }

    #[test]
    fn test_compass_direction() {
        let mut w = WindState::default();

        w.direction = 0.0;
        assert_eq!(w.compass_direction(), "E");

        w.direction = std::f32::consts::FRAC_PI_2;
        assert_eq!(w.compass_direction(), "N");

        w.direction = std::f32::consts::PI;
        assert_eq!(w.compass_direction(), "W");

        w.direction = 3.0 * std::f32::consts::FRAC_PI_2;
        assert_eq!(w.compass_direction(), "S");

        w.direction = std::f32::consts::FRAC_PI_4;
        assert_eq!(w.compass_direction(), "NE");
    }

    #[test]
    fn test_speed_label() {
        let mut w = WindState::default();

        w.speed = 0.0;
        assert_eq!(w.speed_label(), "Calm");

        w.speed = 0.2;
        assert_eq!(w.speed_label(), "Light");

        w.speed = 0.5;
        assert_eq!(w.speed_label(), "Moderate");

        w.speed = 0.9;
        assert_eq!(w.speed_label(), "Strong");
    }

    #[test]
    fn test_direction_vector() {
        let mut w = WindState::default();

        // East (0 rad): dx=1, dy=0
        w.direction = 0.0;
        let (dx, dy) = w.direction_vector();
        assert!((dx - 1.0).abs() < 0.01);
        assert!(dy.abs() < 0.01);

        // North (PI/2): dx=0, dy=1
        w.direction = std::f32::consts::FRAC_PI_2;
        let (dx, dy) = w.direction_vector();
        assert!(dx.abs() < 0.01);
        assert!((dy - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_splitmix64_deterministic() {
        let a = splitmix64(42);
        let b = splitmix64(42);
        assert_eq!(a, b);
        assert_ne!(splitmix64(42), splitmix64(43));
    }

    #[test]
    fn test_rand_signed_f32_range() {
        for seed in 0..1000u64 {
            let val = rand_signed_f32(seed);
            assert!(
                val >= -1.0 && val < 1.0,
                "rand_signed_f32({}) = {} out of range",
                seed,
                val
            );
        }
    }

    #[test]
    fn test_rand_unsigned_f32_range() {
        for seed in 0..1000u64 {
            let val = rand_unsigned_f32(seed);
            assert!(
                val >= 0.0 && val < 1.0,
                "rand_unsigned_f32({}) = {} out of range",
                seed,
                val
            );
        }
    }

    #[test]
    fn test_angle_diff() {
        // Same angle
        assert!((angle_diff(0.0, 0.0)).abs() < f32::EPSILON);

        // Quarter turn positive
        let diff = angle_diff(0.0, std::f32::consts::FRAC_PI_2);
        assert!((diff - std::f32::consts::FRAC_PI_2).abs() < 0.01);

        // Quarter turn negative (wrapping)
        let diff = angle_diff(std::f32::consts::FRAC_PI_2, 0.0);
        assert!((diff - (-std::f32::consts::FRAC_PI_2)).abs() < 0.01);

        // Shortest path across 0/2PI boundary
        let diff = angle_diff(0.1, std::f32::consts::TAU - 0.1);
        assert!(diff < 0.0, "Should take the short way around");
        assert!((diff - (-0.2)).abs() < 0.01);
    }

    #[test]
    fn test_prevailing_direction_for_zones() {
        // Temperate should be westerly (0.0)
        assert!((prevailing_direction_for_zone(ClimateZone::Temperate) - 0.0).abs() < f32::EPSILON);
        // Tropical should be trade winds (PI)
        assert!(
            (prevailing_direction_for_zone(ClimateZone::Tropical) - std::f32::consts::PI).abs()
                < f32::EPSILON
        );
        // Oceanic should be westerly (0.0)
        assert!((prevailing_direction_for_zone(ClimateZone::Oceanic) - 0.0).abs() < f32::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Integration tests using Bevy App
    // -----------------------------------------------------------------------

    /// Helper: build a minimal Bevy App with wind system and required resources.
    fn wind_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<TickCounter>()
            .init_resource::<WindState>()
            .init_resource::<Weather>()
            .init_resource::<ClimateZone>()
            .init_resource::<GameClock>()
            .add_systems(Update, update_wind);
        app
    }

    /// Advance the app by setting the tick counter and running an update.
    fn advance_wind(app: &mut App, tick_value: u64) {
        app.world_mut().resource_mut::<TickCounter>().0 = tick_value;
        app.update();
    }

    #[test]
    fn test_wind_direction_trends_toward_prevailing() {
        let mut app = wind_test_app();

        // Start wind pointing south (3*PI/2), prevailing is east (0.0 = westerly default)
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.direction = 3.0 * std::f32::consts::FRAC_PI_2; // South
            wind.speed = 0.3;
        }

        // Set weather to mild (no storm/calm override)
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::PartlyCloudy;
            weather.cloud_cover = 0.4;
        }

        let initial_direction = app.world().resource::<WindState>().direction;

        // Run many wind updates
        for i in 1..=50 {
            advance_wind(&mut app, i * WIND_UPDATE_INTERVAL);
        }

        let final_direction = app.world().resource::<WindState>().direction;
        let prevailing = app.world().resource::<WindState>().prevailing_direction;

        // The wind should have moved closer to prevailing (0.0) from initial (3*PI/2)
        let initial_dist = angle_diff(initial_direction, prevailing).abs();
        let final_dist = angle_diff(final_direction, prevailing).abs();

        assert!(
            final_dist < initial_dist,
            "Wind should trend toward prevailing direction. Initial distance: {}, Final distance: {}",
            initial_dist,
            final_dist
        );
    }

    #[test]
    fn test_storm_increases_wind_speed() {
        let mut app = wind_test_app();

        // Set storm weather
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Storm;
            weather.cloud_cover = 0.95;
            weather.precipitation_intensity = 0.85;
        }

        // Start with low speed
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.1;
            // Set prev_condition to Storm so no gust fires from transition
            wind.prev_condition = WeatherCondition::Storm;
        }

        // Run several updates to let speed converge
        for i in 1..=20 {
            advance_wind(&mut app, i * WIND_UPDATE_INTERVAL);
        }

        let final_speed = app.world().resource::<WindState>().speed;
        assert!(
            final_speed >= 0.6,
            "Storm wind speed should reach 0.6+, got {}",
            final_speed
        );
    }

    #[test]
    fn test_calm_clear_weather_low_wind() {
        let mut app = wind_test_app();

        // Set clear sunny weather with very low cloud cover (high pressure proxy)
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.05;
            weather.precipitation_intensity = 0.0;
        }

        // Start with moderate speed
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.5;
            // Set prev_condition to Sunny so no gust fires
            wind.prev_condition = WeatherCondition::Sunny;
        }

        // Run several updates to let speed converge downward
        for i in 1..=30 {
            advance_wind(&mut app, i * WIND_UPDATE_INTERVAL);
        }

        let final_speed = app.world().resource::<WindState>().speed;
        assert!(
            final_speed <= 0.15,
            "Calm clear weather should produce low wind speed (<= 0.15), got {}",
            final_speed
        );
    }

    #[test]
    fn test_diurnal_afternoon_boost() {
        // Test that afternoon hours produce higher wind speed than morning
        let mut app_afternoon = wind_test_app();
        let mut app_morning = wind_test_app();

        // Both start with identical state
        let setup = |app: &mut App| {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::PartlyCloudy;
            weather.cloud_cover = 0.4;

            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.3;
            wind.prev_condition = WeatherCondition::PartlyCloudy;
        };

        setup(&mut app_afternoon);
        setup(&mut app_morning);

        // Set afternoon clock (hour 15)
        app_afternoon.world_mut().resource_mut::<GameClock>().hour = 15.0;
        // Set morning clock (hour 8)
        app_morning.world_mut().resource_mut::<GameClock>().hour = 8.0;

        advance_wind(&mut app_afternoon, WIND_UPDATE_INTERVAL);
        advance_wind(&mut app_morning, WIND_UPDATE_INTERVAL);

        let afternoon_speed = app_afternoon.world().resource::<WindState>().speed;
        let morning_speed = app_morning.world().resource::<WindState>().speed;

        assert!(
            afternoon_speed > morning_speed,
            "Afternoon wind ({}) should be stronger than morning wind ({})",
            afternoon_speed,
            morning_speed
        );
    }

    #[test]
    fn test_gust_on_weather_transition() {
        let mut app = wind_test_app();

        // Start with sunny weather, record prev_condition as Sunny
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
        }
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.3;
            wind.prev_condition = WeatherCondition::Sunny;
        }

        // Now switch to storm -- this should trigger a gust
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Storm;
            weather.cloud_cover = 0.95;
            weather.precipitation_intensity = 0.85;
        }

        advance_wind(&mut app, WIND_UPDATE_INTERVAL);

        // After the update, prev_condition should now be Storm
        let wind = app.world().resource::<WindState>();
        assert_eq!(
            wind.prev_condition,
            WeatherCondition::Storm,
            "prev_condition should update to Storm"
        );
        // Speed should be elevated (storm target + gust boost)
        // At minimum the storm target alone is 0.6+, gust adds 0.2+
        assert!(
            wind.speed >= 0.4,
            "Wind speed during gust should be elevated, got {}",
            wind.speed
        );
    }

    #[test]
    fn test_climate_zone_changes_prevailing() {
        let mut app = wind_test_app();

        // Set tropical climate zone
        *app.world_mut().resource_mut::<ClimateZone>() = ClimateZone::Tropical;

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::PartlyCloudy;
            weather.cloud_cover = 0.4;
        }

        advance_wind(&mut app, WIND_UPDATE_INTERVAL);

        let wind = app.world().resource::<WindState>();
        assert!(
            (wind.prevailing_direction - std::f32::consts::PI).abs() < f32::EPSILON,
            "Tropical zone should have prevailing direction PI (trade winds), got {}",
            wind.prevailing_direction
        );
    }
}
