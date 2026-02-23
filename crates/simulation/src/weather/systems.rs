use bevy::prelude::*;

use super::climate::ClimateZone;
use super::state::{ConstructionModifiers, Weather, ROLLING_RAINFALL_DAYS};
use super::types::{is_extreme_weather, Season, WeatherChangeEvent, WeatherCondition};
use crate::time_of_day::GameClock;

/// Diurnal temperature factor: models realistic day/night temperature cycle.
///
/// Returns a value in `[0.0, 1.0]` where 0.0 is the daily minimum (around 06:00)
/// and 1.0 is the daily maximum (around 15:00).
///
/// Uses a cosine curve shifted so that:
/// - Minimum temperature occurs at hour 6 (just after sunrise)
/// - Maximum temperature occurs at hour 15 (mid-afternoon solar lag)
pub fn diurnal_factor(hour: u32) -> f32 {
    let h = (hour % 24) as f32;

    if (6.0..=15.0).contains(&h) {
        // Warming phase: 06:00 to 15:00 (9 hours)
        let t = (h - 6.0) / 9.0; // 0..1
        0.5 - 0.5 * (t * std::f32::consts::PI).cos()
    } else {
        // Cooling phase: 15:00 to 06:00 next day (15 hours)
        let hours_since_15 = if h >= 15.0 { h - 15.0 } else { h + 9.0 };
        let t = hours_since_15 / 15.0; // 0..1
        0.5 + 0.5 * (t * std::f32::consts::PI).cos()
    }
}

/// Updates `ConstructionModifiers` each tick based on current weather and season.
pub fn update_construction_modifiers(
    weather: Res<Weather>,
    mut modifiers: ResMut<ConstructionModifiers>,
) {
    let season_speed = ConstructionModifiers::season_speed_factor(weather.season);
    let weather_speed =
        ConstructionModifiers::weather_speed_factor(weather.current_event, weather.temperature);
    modifiers.speed_factor = season_speed * weather_speed;
    modifiers.cost_factor = ConstructionModifiers::season_cost_factor(weather.season);
}

/// Hourly weather update system. Runs every time the game clock crosses an hour boundary.
///
/// Implements:
/// - Diurnal temperature curve: `T(hour) = T_min + (T_max - T_min) * diurnal_factor(hour)`
/// - Smooth transitions: `temperature += (target - temperature) * 0.3`
/// - Daily variation via deterministic hash on day
/// - Atmospheric state updates (cloud_cover, humidity, precipitation)
/// - Weather condition derived from atmospheric state
/// - All parameters driven by the active `ClimateZone`.
pub fn update_weather(
    clock: Res<GameClock>,
    mut weather: ResMut<Weather>,
    mut change_events: EventWriter<WeatherChangeEvent>,
    climate: Res<ClimateZone>,
) {
    let current_hour = clock.hour_of_day();

    // Only update on hour boundaries (when the integer hour changes)
    if current_hour == weather.last_update_hour && clock.day == weather.last_update_day {
        return;
    }

    // Snapshot pre-update state for change detection
    let old_condition = weather.current_event;
    let old_season = weather.season;
    let old_was_extreme = weather.prev_extreme;

    let day_changed = clock.day != weather.last_update_day;
    weather.last_update_hour = current_hour;
    weather.last_update_day = clock.day;

    // Roll daily rainfall into history on day boundary (before resetting for new day)
    if day_changed {
        roll_daily_rainfall(&mut weather);
    }

    // Update season
    weather.season = Season::from_day(clock.day);

    // Get climate parameters for the current season and zone
    let zone = *climate;
    let climate_params = zone.season_params(weather.season);

    // --- Diurnal temperature ---
    let (t_min, t_max) = (climate_params.t_min, climate_params.t_max);
    // Add daily variation (deterministic based on day) of +/- 3 degrees
    let day_variation = ((clock.day as f32 * 0.1).sin()) * 3.0;
    let effective_min = t_min + day_variation;
    let effective_max = t_max + day_variation;

    let factor = diurnal_factor(current_hour);
    let target_temp = effective_min + (effective_max - effective_min) * factor;

    // Smooth transition toward target
    weather.temperature += (target_temp - weather.temperature) * 0.3;

    // --- Atmospheric state updates (daily events + hourly cloud drift) ---
    if day_changed {
        // Count down event duration
        if weather.event_days_remaining > 0 {
            weather.event_days_remaining -= 1;
            if weather.event_days_remaining == 0 {
                // Event ended: reset atmospheric state toward clear
                weather.cloud_cover *= 0.5;
                weather.atmo_precipitation = 0.0;
                weather.humidity *= 0.7;
            }
        }

        // Random weather events (deterministic based on day hash)
        if weather.event_days_remaining == 0 {
            let hash = (clock.day.wrapping_mul(2654435761)) % 100;

            // Compute the precipitation threshold for the current season/zone.
            // The base precipitation_chance (0.0..1.0) is scaled to a 0..99 hash range.
            let precip_threshold = (climate_params.precipitation_chance * 100.0) as u32;

            // Check if a precipitation event should occur
            let is_precip_day = hash < precip_threshold;

            // Check for extreme weather events (heat wave in summer, cold snap in winter)
            let is_extreme_day = hash < 4; // ~4% chance for extreme events

            match (weather.season, is_extreme_day, is_precip_day) {
                // Summer heat wave (only if extreme day, any climate)
                (Season::Summer, true, _) => {
                    weather.cloud_cover = 0.05;
                    weather.atmo_precipitation = 0.0;
                    weather.humidity = 0.3;
                    weather.event_days_remaining = 3 + (hash % 4);
                    weather.temperature = t_max + 8.0;
                }
                // Winter cold snap (only if extreme day and snow is enabled)
                (Season::Winter, true, _) if climate_params.snow_enabled => {
                    weather.cloud_cover = 0.2;
                    weather.atmo_precipitation = 0.0;
                    weather.humidity = 0.4;
                    weather.event_days_remaining = 3 + (hash % 5);
                    weather.temperature = t_min - 10.0;
                }
                // Precipitation event
                (_, _, true) => {
                    let is_storm = hash < (precip_threshold / 3).max(1);
                    if is_storm {
                        // Storm / heavy precipitation
                        weather.cloud_cover = 0.9;
                        weather.atmo_precipitation = 0.7 + (hash % 20) as f32 * 0.01;
                        weather.humidity = 0.9 + (hash % 10) as f32 * 0.005;
                        weather.event_days_remaining = 1 + (hash % 3);
                    } else {
                        // Normal rain/snow
                        weather.cloud_cover = 0.7 + (hash % 20) as f32 * 0.01;
                        weather.atmo_precipitation = 0.2 + (hash % 15) as f32 * 0.02;
                        weather.humidity = 0.8;
                        weather.event_days_remaining = 2 + (hash % 4);
                    }
                }
                // No event: drift toward seasonal baseline
                _ => {
                    let seasonal_baseline_cloud = zone.baseline_cloud_cover(weather.season);
                    weather.cloud_cover += (seasonal_baseline_cloud - weather.cloud_cover) * 0.2;
                    weather.atmo_precipitation *= 0.5; // decay precipitation
                    let seasonal_humidity = zone.baseline_humidity(weather.season);
                    weather.humidity += (seasonal_humidity - weather.humidity) * 0.2;
                }
            }
        }
    }

    // Hourly cloud drift: small random-ish perturbation based on hour + day
    let hour_hash =
        ((clock.day.wrapping_mul(7919)).wrapping_add(current_hour.wrapping_mul(6271))) % 1000;
    let drift = (hour_hash as f32 / 1000.0 - 0.5) * 0.06; // +/- 0.03
    weather.cloud_cover = (weather.cloud_cover + drift).clamp(0.0, 1.0);

    // Clamp all atmospheric values
    weather.humidity = weather.humidity.clamp(0.0, 1.0);
    let atmo_precip = weather.atmo_precipitation.clamp(0.0, 1.0);
    weather.atmo_precipitation = atmo_precip;

    // If snow is disabled for this zone/season, convert snow to rain
    let snow_enabled = zone.season_params(weather.season).snow_enabled;
    let effective_temp = if !snow_enabled && weather.temperature < 0.0 {
        // Force positive temperature so WeatherCondition::from_atmosphere won't produce Snow
        0.1
    } else {
        weather.temperature
    };

    // Derive weather condition from atmospheric state (using 0-1 atmospheric signal)
    weather.current_event =
        WeatherCondition::from_atmosphere(weather.cloud_cover, atmo_precip, effective_temp);

    // --- Set physical precipitation intensity (inches per hour) ---
    let day_hash = (clock.day.wrapping_mul(2654435761)) % 100;
    let physical_intensity =
        precipitation_intensity_for_event(weather.current_event, weather.season, day_hash);
    weather.precipitation_intensity = physical_intensity;

    // Accumulate hourly rainfall (each hour boundary adds one hour of rainfall)
    weather.daily_rainfall += physical_intensity;

    // Recompute rolling 30-day total
    weather.rolling_30day_rainfall =
        weather.rainfall_history.iter().sum::<f32>() + weather.daily_rainfall;

    // --- Fire WeatherChangeEvent if anything meaningful changed ---
    let new_condition = weather.current_event;
    let new_season = weather.season;
    let new_is_extreme = is_extreme_weather(new_condition, weather.temperature);

    let condition_changed = old_condition != new_condition;
    let season_changed = old_season != new_season;
    let extreme_crossed = old_was_extreme != new_is_extreme;

    // Store current extreme state for next tick's comparison
    weather.prev_extreme = new_is_extreme;

    if condition_changed || season_changed || extreme_crossed {
        change_events.send(WeatherChangeEvent {
            old_condition,
            new_condition,
            old_season,
            new_season,
            is_extreme: new_is_extreme,
        });
    }
}

/// Compute precipitation intensity (inches per hour) for a given weather condition
/// and season. Summer storms tend to be heavier; winter precipitation is lighter.
///
/// The returned value is in inches/hr and drives the `PrecipitationCategory` scale:
/// - Clear/Sunny/PartlyCloudy/Overcast: 0.0
/// - Rain: 0.1 - 1.0 (varies by season; summer storms heavier)
/// - HeavyRain: 1.0 - 2.5
/// - Storm: 2.0 - 4.0+
/// - Snow: 0.05 - 0.3 (water equivalent)
///
/// A deterministic `day_hash` (0..99) is used for variation within each range.
pub fn precipitation_intensity_for_event(
    condition: WeatherCondition,
    season: Season,
    day_hash: u32,
) -> f32 {
    // day_hash is expected to be in 0..99; clamp just in case
    let h = (day_hash % 100) as f32 / 100.0; // 0.0 .. 0.99

    match condition {
        WeatherCondition::Sunny | WeatherCondition::PartlyCloudy | WeatherCondition::Overcast => {
            0.0
        }

        WeatherCondition::Rain => {
            // Base: 0.1 - 0.7, with seasonal modifier
            let base = 0.1 + h * 0.6; // 0.1 .. 0.7
            let seasonal = match season {
                Season::Summer => 1.3, // summer storms heavier
                Season::Spring => 1.0,
                Season::Autumn => 1.1,
                Season::Winter => 0.7, // lighter winter rain
            };
            (base * seasonal).clamp(0.1, 1.0)
        }

        WeatherCondition::HeavyRain => {
            // Base: 1.0 - 2.0, with seasonal modifier
            let base = 1.0 + h * 1.0; // 1.0 .. 2.0
            let seasonal = match season {
                Season::Summer => 1.25,
                Season::Spring => 1.0,
                Season::Autumn => 1.1,
                Season::Winter => 0.85,
            };
            (base * seasonal).clamp(1.0, 2.5)
        }

        WeatherCondition::Storm => {
            // Base: 2.0 - 3.5, with seasonal modifier
            let base = 2.0 + h * 1.5; // 2.0 .. 3.5
            let seasonal = match season {
                Season::Summer => 1.3, // tropical-style downpours
                Season::Spring => 1.0,
                Season::Autumn => 1.1,
                Season::Winter => 0.9,
            };
            (base * seasonal).max(2.0) // no upper clamp; Extreme (4.0+) is valid
        }

        WeatherCondition::Snow => {
            // Water equivalent: 0.05 - 0.3
            // 0.05 .. 0.30
            0.05 + h * 0.25
        }
    }
}

/// Precipitation tracking ordering anchor.
///
/// All precipitation intensity setting, accumulation, and rolling-window
/// computation is performed inline in `update_weather`. This system exists
/// as a public ordering label so downstream systems (stormwater, fire, solar)
/// can schedule `.after(update_precipitation)` to ensure weather data is fresh.
pub fn update_precipitation(weather: Res<Weather>) {
    // All work is done in update_weather. This system reads weather to
    // establish a scheduling dependency.
    let _ = weather.precipitation_intensity;
}

/// Roll the daily rainfall into the 30-day history buffer.
/// Called from `update_weather` when a new day starts.
pub(super) fn roll_daily_rainfall(weather: &mut Weather) {
    // Ensure history buffer is the right size
    if weather.rainfall_history.len() != ROLLING_RAINFALL_DAYS {
        weather.rainfall_history = vec![0.0; ROLLING_RAINFALL_DAYS];
    }

    // Shift history: drop oldest day, append yesterday's total
    weather.rainfall_history.remove(0);
    weather.rainfall_history.push(weather.daily_rainfall);

    // Reset daily accumulator for the new day
    weather.daily_rainfall = 0.0;
}
