use bevy::prelude::*;

use crate::time_of_day::GameClock;
use crate::weather::{ClimateZone, Weather, WeatherCondition};
use crate::TickCounter;

use super::types::{prevailing_direction_for_zone, WindState};

// =============================================================================
// Deterministic pseudo-random (splitmix64, matching disasters.rs)
// =============================================================================

pub(crate) fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Returns a deterministic pseudo-random f32 in [-1.0, 1.0) based on seed.
pub(crate) fn rand_signed_f32(seed: u64) -> f32 {
    let hash = splitmix64(seed);
    // Map to [-1.0, 1.0)
    (hash % 2_000_000) as f32 / 1_000_000.0 - 1.0
}

/// Returns a deterministic pseudo-random f32 in [0.0, 1.0) based on seed.
pub(crate) fn rand_unsigned_f32(seed: u64) -> f32 {
    let hash = splitmix64(seed);
    (hash % 1_000_000) as f32 / 1_000_000.0
}

// =============================================================================
// Constants
// =============================================================================

/// Wind update interval in ticks (matches SlowTickTimer::INTERVAL).
pub(crate) const WIND_UPDATE_INTERVAL: u64 = 100;

/// Maximum angular perturbation per update (radians). ~5 degrees (calmer than old +-10).
const MAX_DIRECTION_SHIFT: f32 = 0.087; // ~5 degrees in radians

/// Reversion factor: how strongly direction reverts toward prevailing each update.
const PREVAILING_REVERSION_FACTOR: f32 = 0.1;

/// Diurnal wind boost for afternoon hours (12-18): 20% stronger.
const DIURNAL_AFTERNOON_BOOST: f32 = 1.2;

// =============================================================================
// Helper functions
// =============================================================================

/// Computes the shortest signed angular difference from `from` to `to` (in radians).
/// Result is in [-PI, PI].
pub(crate) fn angle_diff(from: f32, to: f32) -> f32 {
    let diff = (to - from).rem_euclid(std::f32::consts::TAU);
    if diff > std::f32::consts::PI {
        diff - std::f32::consts::TAU
    } else {
        diff
    }
}

// =============================================================================
// Systems
// =============================================================================

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
