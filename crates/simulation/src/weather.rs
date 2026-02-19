use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::time_of_day::GameClock;

/// Lightweight event fired whenever weather conditions change.
///
/// Consumers can listen for this with `EventReader<WeatherChangeEvent>` instead of
/// polling the `Weather` resource every tick.
#[derive(Event, Debug, Clone)]
pub struct WeatherChangeEvent {
    /// The weather condition before the change.
    pub old_condition: WeatherCondition,
    /// The weather condition after the change.
    pub new_condition: WeatherCondition,
    /// The season before the change (differs from `new_season` on season transitions).
    pub old_season: Season,
    /// The season after the change.
    pub new_season: Season,
    /// `true` when the new condition is Storm, or temperature crosses extreme
    /// thresholds (heat-wave >35 C, cold-snap < -5 C).
    pub is_extreme: bool,
}

/// Temperature thresholds for extreme weather classification.
const EXTREME_HEAT_THRESHOLD: f32 = 35.0;
const EXTREME_COLD_THRESHOLD: f32 = -5.0;

/// Returns `true` if the weather condition or temperature qualifies as extreme.
fn is_extreme_weather(condition: WeatherCondition, temperature: f32) -> bool {
    matches!(condition, WeatherCondition::Storm)
        || !(EXTREME_COLD_THRESHOLD..=EXTREME_HEAT_THRESHOLD).contains(&temperature)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn from_day(day: u32) -> Season {
        // 360-day year: 90 days per season (30 days/month, 3 months/season)
        let day_of_year = ((day.saturating_sub(1)) % 360) + 1;
        match day_of_year {
            1..=90 => Season::Spring,
            91..=180 => Season::Summer,
            181..=270 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }

    /// Seasonal happiness modifier: Summer +2, Spring +1, Autumn 0, Winter -2.
    pub fn happiness_modifier(self) -> f32 {
        match self {
            Season::Spring => 1.0,
            Season::Summer => 2.0,
            Season::Autumn => 0.0,
            Season::Winter => -2.0,
        }
    }

    /// Base grass color tint for terrain rendering, varying by season.
    pub fn grass_color(self) -> [f32; 3] {
        match self {
            Season::Spring => [0.35, 0.65, 0.15], // Bright green with slight yellow tint
            Season::Summer => [0.25, 0.55, 0.12], // Lush deep green
            Season::Autumn => [0.55, 0.40, 0.15], // Orange/brown
            Season::Winter => [0.75, 0.78, 0.82], // Grey/white with slight blue tint
        }
    }

    /// Seasonal min/max temperature range: (T_min, T_max).
    fn temperature_range(self) -> (f32, f32) {
        match self {
            Season::Spring => (8.0, 22.0),
            Season::Summer => (20.0, 36.0),
            Season::Autumn => (5.0, 19.0),
            Season::Winter => (-8.0, 6.0),
        }
    }
}

/// Weather conditions derived from atmospheric state (cloud cover, precipitation, temperature).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    Sunny,
    PartlyCloudy,
    Overcast,
    Rain,
    HeavyRain,
    Snow,
    Storm,
}

/// Legacy alias kept for backward compatibility with save system and downstream consumers.
pub type WeatherEvent = WeatherCondition;

impl WeatherCondition {
    /// Derive condition from atmospheric state.
    pub fn from_atmosphere(
        cloud_cover: f32,
        precipitation_intensity: f32,
        temperature: f32,
    ) -> Self {
        if precipitation_intensity > 0.7 && cloud_cover > 0.8 {
            if temperature < 0.0 {
                WeatherCondition::Snow
            } else {
                WeatherCondition::Storm
            }
        } else if precipitation_intensity > 0.4 {
            if temperature < 0.0 {
                WeatherCondition::Snow
            } else {
                WeatherCondition::HeavyRain
            }
        } else if precipitation_intensity > 0.1 {
            if temperature < 0.0 {
                WeatherCondition::Snow
            } else {
                WeatherCondition::Rain
            }
        } else if cloud_cover > 0.7 {
            WeatherCondition::Overcast
        } else if cloud_cover > 0.3 {
            WeatherCondition::PartlyCloudy
        } else {
            WeatherCondition::Sunny
        }
    }

    /// Whether this condition counts as precipitation for gameplay purposes.
    pub fn is_precipitation(self) -> bool {
        matches!(
            self,
            WeatherCondition::Rain
                | WeatherCondition::HeavyRain
                | WeatherCondition::Snow
                | WeatherCondition::Storm
        )
    }
}

/// Diurnal temperature factor: models realistic day/night temperature cycle.
///
/// Returns a value in `[0.0, 1.0]` where 0.0 is the daily minimum (around 06:00)
/// and 1.0 is the daily maximum (around 15:00).
///
/// Uses a cosine curve shifted so that:
/// - Minimum temperature occurs at hour 6 (just after sunrise)
/// - Maximum temperature occurs at hour 15 (mid-afternoon solar lag)
pub fn diurnal_factor(hour: u32) -> f32 {
    // Center of the cosine at hour 15 (peak), period 24 hours
    // cos((hour - 15) * 2*PI / 24) maps:
    //   hour=15 -> cos(0) = 1.0
    //   hour=3  -> cos(PI) = -1.0
    //   hour=6  -> cos(-9 * PI/12) = cos(-3*PI/4) ~ -0.707
    //
    // We want minimum at 6, maximum at 15. A shifted cosine:
    // factor = 0.5 + 0.5 * cos((hour - 15) * 2*PI / 24)
    // At hour 15: 0.5 + 0.5*1.0 = 1.0
    // At hour 3: 0.5 + 0.5*(-1.0) = 0.0
    // At hour 6: 0.5 + 0.5*cos(-3*PI/4) ~ 0.146
    //
    // To get exact 0.0 at hour 6, use a piecewise or adjusted formula.
    // Simple approach: remap so hour 6->0.0, hour 15->1.0 using cosine.
    // Phase: peak at 15, trough at 15-12=3. We want trough at 6.
    // Shift: use (hour - 10.5) to center between 6 and 15 (midpoint = 10.5)
    // cos((hour - 10.5) * PI / 9) gives:
    //   hour=10.5 -> cos(0) = 1.0 (wrong, we want peak at 15)
    //
    // Cleanest: use a sine with the right phase.
    // sin((hour - 6) * PI / 18) * sin(...)  -- no, keep it simple:
    //
    // factor = 0.5 - 0.5 * cos((hour - 6) * PI / 9)  for hour in [6..15] rising
    // For a full 24-hour smooth cycle, use:
    // factor = 0.5 + 0.5 * cos((hour - 15) * 2*PI / 24)
    // Then clamp and renormalize so min->0, max->1.

    let h = (hour % 24) as f32;

    // Piecewise smooth: nighttime cooling from 15:00 to 06:00 (15 hours),
    // daytime warming from 06:00 to 15:00 (9 hours).
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

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Weather {
    pub season: Season,
    pub temperature: f32, // -10 to 40 Celsius
    pub current_event: WeatherCondition,
    pub event_days_remaining: u32,
    pub last_update_day: u32,
    /// Whether natural disasters (tornado, earthquake, flood) can occur.
    pub disasters_enabled: bool,
    /// Relative humidity (0.0 to 1.0).
    #[serde(default = "default_humidity")]
    pub humidity: f32,
    /// Cloud cover fraction (0.0 = clear sky, 1.0 = fully overcast).
    #[serde(default)]
    pub cloud_cover: f32,
    /// Precipitation intensity (0.0 = none, 1.0 = torrential).
    #[serde(default)]
    pub precipitation_intensity: f32,
    /// Last hour that triggered a weather update (used for hourly boundary detection).
    #[serde(default)]
    pub last_update_hour: u32,
    /// Whether the previous tick ended in an extreme weather state (for change detection).
    #[serde(default)]
    pub prev_extreme: bool,
}

fn default_humidity() -> f32 {
    0.5
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            season: Season::Spring,
            temperature: 15.0,
            current_event: WeatherCondition::Sunny,
            event_days_remaining: 0,
            last_update_day: 0,
            disasters_enabled: true,
            humidity: 0.5,
            cloud_cover: 0.1,
            precipitation_intensity: 0.0,
            last_update_hour: 0,
            prev_extreme: false,
        }
    }
}

impl Weather {
    /// Seasonal base temperature range: (T_min, T_max)
    fn seasonal_range(season: Season) -> (f32, f32) {
        season.temperature_range()
    }

    /// Derive the current weather condition from atmospheric state.
    pub fn condition(&self) -> WeatherCondition {
        WeatherCondition::from_atmosphere(
            self.cloud_cover,
            self.precipitation_intensity,
            self.temperature,
        )
    }

    /// Power consumption multiplier (heating in winter, cooling in summer)
    pub fn power_multiplier(&self) -> f32 {
        match self.season {
            Season::Winter => 1.4,
            Season::Summer => 1.2,
            _ => 1.0,
        }
    }

    /// Water consumption multiplier
    pub fn water_multiplier(&self) -> f32 {
        match self.season {
            Season::Summer => 1.3,
            Season::Winter => 0.9,
            _ => 1.0,
        }
    }

    /// Agricultural output multiplier (farms produce less in winter)
    pub fn agriculture_multiplier(&self) -> f32 {
        match self.season {
            Season::Spring => 1.2,
            Season::Summer => 1.0,
            Season::Autumn => 0.8,
            Season::Winter => 0.3,
        }
    }

    /// Park effectiveness multiplier (people visit parks more in good weather)
    pub fn park_multiplier(&self) -> f32 {
        let cond = self.current_event;
        match (self.season, cond) {
            (_, WeatherCondition::Rain)
            | (_, WeatherCondition::HeavyRain)
            | (_, WeatherCondition::Storm) => 0.3,
            (_, WeatherCondition::Snow) => 0.2,
            (_, WeatherCondition::Overcast) => 0.6,
            (Season::Summer, WeatherCondition::Sunny) => 1.5,
            (Season::Spring, _) => 1.3,
            (Season::Autumn, _) => 0.8,
            (Season::Winter, _) => 0.4,
            _ => 1.0,
        }
    }

    /// Happiness modifier from weather (events + seasonal baseline)
    pub fn happiness_modifier(&self) -> f32 {
        let mut modifier = self.season.happiness_modifier();
        // Extreme temperature penalties (replaces old HeatWave/ColdSnap event modifiers)
        if self.temperature > 35.0 {
            modifier -= 5.0; // equivalent to old HeatWave
        } else if self.temperature < -5.0 {
            modifier -= 8.0; // equivalent to old ColdSnap
        }
        match self.current_event {
            WeatherCondition::Storm => modifier -= 3.0,
            WeatherCondition::HeavyRain => modifier -= 2.0,
            WeatherCondition::Rain | WeatherCondition::Snow => modifier -= 1.0,
            WeatherCondition::Overcast => modifier -= 0.5,
            WeatherCondition::Sunny | WeatherCondition::PartlyCloudy => {
                if self.season == Season::Spring || self.season == Season::Summer {
                    modifier += 2.0;
                }
            }
        }
        modifier
    }

    /// Travel speed multiplier (snow/rain slows traffic)
    pub fn travel_speed_multiplier(&self) -> f32 {
        match self.current_event {
            WeatherCondition::Storm => 0.5,
            WeatherCondition::HeavyRain => 0.65,
            WeatherCondition::Snow => 0.6,
            WeatherCondition::Rain => 0.8,
            WeatherCondition::Overcast | WeatherCondition::PartlyCloudy => {
                if self.season == Season::Winter {
                    0.85
                } else {
                    1.0
                }
            }
            WeatherCondition::Sunny => {
                if self.season == Season::Winter {
                    0.85
                } else {
                    1.0
                }
            }
        }
    }
}

/// Hourly weather update system. Runs every time the game clock crosses an hour boundary.
///
/// Implements:
/// - Diurnal temperature curve: `T(hour) = T_min + (T_max - T_min) * diurnal_factor(hour)`
/// - Smooth transitions: `temperature += (target - temperature) * 0.3`
/// - Daily variation via deterministic hash on day
/// - Atmospheric state updates (cloud_cover, humidity, precipitation)
/// - Weather condition derived from atmospheric state
pub fn update_weather(
    clock: Res<GameClock>,
    mut weather: ResMut<Weather>,
    mut change_events: EventWriter<WeatherChangeEvent>,
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

    // Update season
    weather.season = Season::from_day(clock.day);

    // --- Diurnal temperature ---
    let (t_min, t_max) = Weather::seasonal_range(weather.season);
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
                weather.precipitation_intensity = 0.0;
                weather.humidity *= 0.7;
            }
        }

        // Random weather events (deterministic based on day hash)
        if weather.event_days_remaining == 0 {
            let hash = (clock.day.wrapping_mul(2654435761)) % 100;
            match (weather.season, hash) {
                (Season::Spring, 0..=8) => {
                    // Spring rain
                    weather.cloud_cover = 0.7 + (hash % 20) as f32 * 0.01;
                    weather.precipitation_intensity = 0.2 + (hash % 15) as f32 * 0.02;
                    weather.humidity = 0.8;
                    weather.event_days_remaining = 2 + (hash % 3);
                }
                (Season::Summer, 0..=3) => {
                    // Summer heat wave (high pressure, clear skies, extreme heat)
                    weather.cloud_cover = 0.05;
                    weather.precipitation_intensity = 0.0;
                    weather.humidity = 0.3;
                    weather.event_days_remaining = 3 + (hash % 4);
                    // Push temperature up beyond normal range
                    weather.temperature = t_max + 8.0;
                }
                (Season::Summer, 4..=7) => {
                    // Summer storm
                    weather.cloud_cover = 0.9;
                    weather.precipitation_intensity = 0.8;
                    weather.humidity = 0.95;
                    weather.event_days_remaining = 1 + (hash % 2);
                }
                (Season::Autumn, 0..=10) => {
                    // Autumn rain
                    weather.cloud_cover = 0.75 + (hash % 15) as f32 * 0.01;
                    weather.precipitation_intensity = 0.25 + (hash % 20) as f32 * 0.015;
                    weather.humidity = 0.85;
                    weather.event_days_remaining = 2 + (hash % 4);
                }
                (Season::Winter, 0..=5) => {
                    // Winter cold snap (clear but frigid)
                    weather.cloud_cover = 0.2;
                    weather.precipitation_intensity = 0.0;
                    weather.humidity = 0.4;
                    weather.event_days_remaining = 3 + (hash % 5);
                    // Push temperature down below normal range
                    weather.temperature = t_min - 10.0;
                }
                (Season::Winter, 6..=8) => {
                    // Winter storm / snow
                    weather.cloud_cover = 0.9;
                    weather.precipitation_intensity = 0.7;
                    weather.humidity = 0.9;
                    weather.event_days_remaining = 1 + (hash % 3);
                }
                _ => {
                    // No new event: drift cloud cover toward seasonal baseline
                    let seasonal_baseline_cloud = match weather.season {
                        Season::Spring => 0.3,
                        Season::Summer => 0.15,
                        Season::Autumn => 0.4,
                        Season::Winter => 0.5,
                    };
                    weather.cloud_cover += (seasonal_baseline_cloud - weather.cloud_cover) * 0.2;
                    weather.precipitation_intensity *= 0.5; // decay precipitation
                    let seasonal_humidity = match weather.season {
                        Season::Spring => 0.55,
                        Season::Summer => 0.4,
                        Season::Autumn => 0.6,
                        Season::Winter => 0.65,
                    };
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
    weather.precipitation_intensity = weather.precipitation_intensity.clamp(0.0, 1.0);

    // Derive weather condition from atmospheric state
    weather.current_event = WeatherCondition::from_atmosphere(
        weather.cloud_cover,
        weather.precipitation_intensity,
        weather.temperature,
    );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_season_from_day() {
        assert_eq!(Season::from_day(1), Season::Spring);
        assert_eq!(Season::from_day(90), Season::Spring);
        assert_eq!(Season::from_day(91), Season::Summer);
        assert_eq!(Season::from_day(180), Season::Summer);
        assert_eq!(Season::from_day(181), Season::Autumn);
        assert_eq!(Season::from_day(270), Season::Autumn);
        assert_eq!(Season::from_day(271), Season::Winter);
        assert_eq!(Season::from_day(360), Season::Winter);
        assert_eq!(Season::from_day(361), Season::Spring); // wraps
    }

    #[test]
    fn test_season_happiness_modifiers() {
        assert_eq!(Season::Spring.happiness_modifier(), 1.0);
        assert_eq!(Season::Summer.happiness_modifier(), 2.0);
        assert_eq!(Season::Autumn.happiness_modifier(), 0.0);
        assert_eq!(Season::Winter.happiness_modifier(), -2.0);
    }

    #[test]
    fn test_multipliers_in_range() {
        let weather = Weather::default();
        assert!((0.5..=2.0).contains(&weather.power_multiplier()));
        assert!((0.5..=2.0).contains(&weather.water_multiplier()));
        assert!((0.0..=2.0).contains(&weather.park_multiplier()));
        assert!((0.3..=1.5).contains(&weather.travel_speed_multiplier()));
    }

    #[test]
    fn test_weather_condition_modifiers() {
        let mut w = Weather::default();
        // Simulate heat wave: extreme temperature
        w.temperature = 38.0;
        w.current_event = WeatherCondition::Sunny;
        // HeatWave equivalent: seasonal(Spring=+1) + extreme_heat(-5) + sunny_spring(+2) = -2
        assert!(w.happiness_modifier() < 0.0);

        w.current_event = WeatherCondition::Sunny;
        w.temperature = 25.0;
        w.season = Season::Summer;
        // Clear+Summer: seasonal(+2) + sunny_bonus(+2) = +4
        assert!(w.happiness_modifier() > 0.0);

        w.season = Season::Winter;
        w.temperature = -10.0;
        w.current_event = WeatherCondition::Snow;
        // ColdSnap equivalent: seasonal(-2) + extreme_cold(-8) + snow(-1) = -11
        assert!(w.happiness_modifier() < -5.0);
    }

    #[test]
    fn test_diurnal_factor_peak_at_15() {
        let peak = diurnal_factor(15);
        assert!(
            (peak - 1.0_f32).abs() < 0.01,
            "Peak at 15:00 should be ~1.0, got {}",
            peak
        );
    }

    #[test]
    fn test_diurnal_factor_minimum_at_06() {
        let minimum = diurnal_factor(6);
        assert!(
            minimum.abs() < 0.01_f32,
            "Minimum at 06:00 should be ~0.0, got {}",
            minimum
        );
    }

    #[test]
    fn test_diurnal_factor_range() {
        for hour in 0..24 {
            let f = diurnal_factor(hour);
            assert!(
                f >= -0.01 && f <= 1.01,
                "diurnal_factor({}) = {} out of range",
                hour,
                f
            );
        }
    }

    #[test]
    fn test_diurnal_factor_monotonic_morning() {
        // Should be monotonically increasing from 6 to 15
        let mut prev = diurnal_factor(6);
        for hour in 7..=15 {
            let current = diurnal_factor(hour);
            assert!(
                current >= prev,
                "diurnal_factor should increase from {} to {}: {} < {}",
                hour - 1,
                hour,
                current,
                prev
            );
            prev = current;
        }
    }

    #[test]
    fn test_diurnal_factor_monotonic_evening() {
        // Should be monotonically decreasing from 15 to 6 (next day)
        let mut prev = diurnal_factor(15);
        for hour_offset in 1..=15 {
            let hour = (15 + hour_offset) % 24;
            let current = diurnal_factor(hour);
            assert!(
                current <= prev + 0.01, // small epsilon for floating point
                "diurnal_factor should decrease from {} to {}: {} > {}",
                (hour + 23) % 24,
                hour,
                current,
                prev
            );
            prev = current;
        }
    }

    #[test]
    fn test_condition_from_atmosphere_sunny() {
        let cond = WeatherCondition::from_atmosphere(0.1, 0.0, 20.0);
        assert_eq!(cond, WeatherCondition::Sunny);
    }

    #[test]
    fn test_condition_from_atmosphere_partly_cloudy() {
        let cond = WeatherCondition::from_atmosphere(0.5, 0.0, 20.0);
        assert_eq!(cond, WeatherCondition::PartlyCloudy);
    }

    #[test]
    fn test_condition_from_atmosphere_overcast() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.05, 20.0);
        assert_eq!(cond, WeatherCondition::Overcast);
    }

    #[test]
    fn test_condition_from_atmosphere_rain() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.2, 10.0);
        assert_eq!(cond, WeatherCondition::Rain);
    }

    #[test]
    fn test_condition_from_atmosphere_heavy_rain() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.5, 10.0);
        assert_eq!(cond, WeatherCondition::HeavyRain);
    }

    #[test]
    fn test_condition_from_atmosphere_snow() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.3, -5.0);
        assert_eq!(cond, WeatherCondition::Snow);
    }

    #[test]
    fn test_condition_from_atmosphere_storm() {
        let cond = WeatherCondition::from_atmosphere(0.9, 0.8, 15.0);
        assert_eq!(cond, WeatherCondition::Storm);
    }

    #[test]
    fn test_condition_is_precipitation() {
        assert!(!WeatherCondition::Sunny.is_precipitation());
        assert!(!WeatherCondition::PartlyCloudy.is_precipitation());
        assert!(!WeatherCondition::Overcast.is_precipitation());
        assert!(WeatherCondition::Rain.is_precipitation());
        assert!(WeatherCondition::HeavyRain.is_precipitation());
        assert!(WeatherCondition::Snow.is_precipitation());
        assert!(WeatherCondition::Storm.is_precipitation());
    }

    #[test]
    fn test_smooth_temperature_transition() {
        // Verify that the smooth transition formula converges
        let target: f32 = 25.0;
        let mut temp: f32 = 10.0;
        for _ in 0..20 {
            temp += (target - temp) * 0.3;
        }
        assert!(
            (temp - target).abs() < 0.1,
            "Temperature should converge to target, got {}",
            temp
        );
    }

    #[test]
    fn test_hourly_temperature_varies() {
        // Check that temperature at 6am differs from temperature at 3pm for summer
        let (t_min, t_max) = Season::Summer.temperature_range();
        let factor_6 = diurnal_factor(6);
        let factor_15 = diurnal_factor(15);
        let temp_6 = t_min + (t_max - t_min) * factor_6;
        let temp_15 = t_min + (t_max - t_min) * factor_15;
        assert!(
            temp_15 > temp_6 + 5.0,
            "Afternoon should be significantly warmer: {}C vs {}C",
            temp_15,
            temp_6
        );
    }

    #[test]
    fn test_default_weather_has_new_fields() {
        let w = Weather::default();
        assert!((w.humidity - 0.5_f32).abs() < 0.01);
        assert!(w.cloud_cover < 0.2_f32);
        assert!(w.precipitation_intensity < 0.01_f32);
        assert_eq!(w.last_update_hour, 0);
    }

    #[test]
    fn test_weather_condition_method() {
        let mut w = Weather::default();
        w.cloud_cover = 0.1;
        w.precipitation_intensity = 0.0;
        w.temperature = 20.0;
        assert_eq!(w.condition(), WeatherCondition::Sunny);

        w.cloud_cover = 0.9;
        w.precipitation_intensity = 0.8;
        w.temperature = 20.0;
        assert_eq!(w.condition(), WeatherCondition::Storm);
    }

    #[test]
    fn test_travel_speed_new_conditions() {
        let mut w = Weather::default();
        w.current_event = WeatherCondition::HeavyRain;
        assert!(w.travel_speed_multiplier() < 0.7);

        w.current_event = WeatherCondition::Snow;
        assert!(w.travel_speed_multiplier() < 0.7);
    }

    #[test]
    fn test_park_multiplier_new_conditions() {
        let mut w = Weather::default();
        w.current_event = WeatherCondition::HeavyRain;
        assert!(w.park_multiplier() < 0.5);

        w.current_event = WeatherCondition::Overcast;
        assert!(w.park_multiplier() < 0.8);

        w.current_event = WeatherCondition::Snow;
        assert!(w.park_multiplier() < 0.3);
    }

    // -----------------------------------------------------------------------
    // WeatherChangeEvent tests
    // -----------------------------------------------------------------------

    /// Helper: build a minimal Bevy App with weather system and resources.
    fn weather_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<GameClock>()
            .init_resource::<Weather>()
            .add_event::<WeatherChangeEvent>()
            .add_systems(Update, update_weather);
        app
    }

    #[test]
    fn test_event_fired_on_clear_to_rain_transition() {
        let mut app = weather_test_app();

        // Start: Sunny, day 1, hour 0
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.precipitation_intensity = 0.0;
            weather.temperature = 20.0;
            weather.last_update_day = 1;
            weather.last_update_hour = 5;
            weather.season = Season::Spring;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 1;
            clock.hour = 6.0; // different hour to trigger update
        }

        // Force rainy atmospheric state by setting cloud_cover and precipitation
        // before the system runs. The system will derive Rain from atmosphere.
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.cloud_cover = 0.8;
            weather.precipitation_intensity = 0.3;
        }

        app.update();

        // Read events
        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(
            !fired.is_empty(),
            "WeatherChangeEvent should fire when condition changes"
        );
        let evt = &fired[0];
        assert_eq!(evt.old_condition, WeatherCondition::Sunny);
        assert_eq!(evt.new_condition, WeatherCondition::Rain);
        assert!(!evt.is_extreme, "Rain is not extreme weather");
    }

    #[test]
    fn test_event_is_extreme_for_storm() {
        let mut app = weather_test_app();

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.precipitation_intensity = 0.0;
            weather.temperature = 20.0;
            weather.last_update_day = 1;
            weather.last_update_hour = 5;
            weather.season = Season::Summer;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 1;
            clock.hour = 6.0;
        }

        // Set storm-level atmospheric state
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.cloud_cover = 0.95;
            weather.precipitation_intensity = 0.85;
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(!fired.is_empty(), "Event should fire for Storm");
        let evt = &fired[0];
        assert_eq!(evt.new_condition, WeatherCondition::Storm);
        assert!(evt.is_extreme, "Storm should be flagged as extreme");
    }

    #[test]
    fn test_event_is_extreme_for_heat_wave() {
        let mut app = weather_test_app();

        // Set up non-extreme state, then push temp to extreme
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.05;
            weather.precipitation_intensity = 0.0;
            weather.temperature = 50.0; // will smooth but stay > 35C
            weather.last_update_day = 120;
            weather.last_update_hour = 14;
            weather.season = Season::Summer;
            weather.event_days_remaining = 5;
            weather.prev_extreme = false; // previous tick was NOT extreme
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 120; // Summer day
            clock.hour = 15.0; // peak heat hour
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(
            !fired.is_empty(),
            "Event should fire when crossing extreme heat threshold"
        );
        let evt = &fired[fired.len() - 1];
        assert!(evt.is_extreme, "Temperature > 35C should be extreme");
    }

    #[test]
    fn test_event_is_extreme_for_cold_snap() {
        let mut app = weather_test_app();

        // Set up non-extreme state, then push temp to extreme cold
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.2;
            weather.precipitation_intensity = 0.0;
            weather.temperature = -25.0; // will smooth but stay < -5C
            weather.last_update_day = 300;
            weather.last_update_hour = 5;
            weather.season = Season::Winter;
            weather.event_days_remaining = 5;
            weather.prev_extreme = false; // previous tick was NOT extreme
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 300; // Winter day
            clock.hour = 6.0; // trough temp hour
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(
            !fired.is_empty(),
            "Event should fire when crossing extreme cold threshold"
        );
        let evt = &fired[fired.len() - 1];
        assert!(evt.is_extreme, "Temperature < -5C should be extreme");
    }

    #[test]
    fn test_event_fired_on_season_change() {
        let mut app = weather_test_app();

        // Set up: end of Spring (day 90)
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.precipitation_intensity = 0.0;
            weather.temperature = 20.0;
            weather.last_update_day = 90;
            weather.last_update_hour = 11;
            weather.season = Season::Spring;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 91; // Summer starts
            clock.hour = 12.0;
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(!fired.is_empty(), "Event should fire on season transition");
        let evt = &fired[0];
        assert_eq!(evt.old_season, Season::Spring);
        assert_eq!(evt.new_season, Season::Summer);
    }

    #[test]
    fn test_no_event_when_nothing_changes() {
        let mut app = weather_test_app();

        // Set up: Sunny, clear, mild temperature, Spring
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.precipitation_intensity = 0.0;
            weather.temperature = 15.0;
            weather.last_update_day = 1;
            weather.last_update_hour = 5;
            weather.season = Season::Spring;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 1;
            clock.hour = 6.0;
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        // The condition should remain Sunny (low cloud cover, no precipitation),
        // season stays Spring (day 1), and temperature is mild.
        // No event should fire.
        assert!(
            fired.is_empty(),
            "No event should fire when weather does not change; got {} events",
            fired.len()
        );
    }

    #[test]
    fn test_is_extreme_weather_helper() {
        // Storm is always extreme
        assert!(is_extreme_weather(WeatherCondition::Storm, 20.0));
        // Heat wave
        assert!(is_extreme_weather(WeatherCondition::Sunny, 36.0));
        // Cold snap
        assert!(is_extreme_weather(WeatherCondition::Sunny, -6.0));
        // Normal conditions
        assert!(!is_extreme_weather(WeatherCondition::Sunny, 20.0));
        assert!(!is_extreme_weather(WeatherCondition::Rain, 10.0));
        assert!(!is_extreme_weather(WeatherCondition::Snow, -3.0));
    }
}
