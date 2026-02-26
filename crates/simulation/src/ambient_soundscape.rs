//! PLAY-009: Ambient City Soundscape.
//!
//! Maintains a simulation-side resource describing which ambient sound layers
//! should be active and at what intensity. The actual audio playback is handled
//! downstream by the rendering/audio layer; this module owns the data.
//!
//! Layers respond to population, traffic, green space, weather, and time of day.

use bevy::prelude::*;

use crate::citizen::{Citizen, CitizenStateComp};
use crate::parks_system::ParksState;
use crate::stats::CityStats;
use crate::time_of_day::GameClock;
use crate::trees::TreeGrid;
use crate::weather::Weather;

// =============================================================================
// Types
// =============================================================================

/// Categories of ambient sound that can play simultaneously.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmbientLayer {
    /// Base city noise — HVAC hum, distant sirens, general urban drone.
    /// Intensity scales logarithmically with population.
    CityHum,
    /// Road and traffic noise from vehicles on the move.
    /// Scales with the fraction of citizens currently commuting.
    Traffic,
    /// Nature sounds — birdsong, rustling leaves, wind through trees.
    /// Stronger with more green space and lower population density.
    Nature,
    /// Weather sounds — rain, thunder, howling wind.
    /// Derived from current precipitation and weather condition.
    Weather,
    /// Night-time ambience — crickets, distant dogs, quiet stillness.
    /// Active during late night / early morning hours (22:00 - 05:00).
    NightAmbience,
}

/// A single active ambient layer with its current intensity.
#[derive(Debug, Clone)]
pub struct ActiveAmbientLayer {
    /// Which layer this represents.
    pub layer: AmbientLayer,
    /// Intensity from 0.0 (silent) to 1.0 (full volume).
    pub intensity: f32,
}

// =============================================================================
// Resource
// =============================================================================

/// Tracks which ambient sound layers should be playing and at what intensity.
///
/// Updated periodically by the `update_ambient_soundscape` system. Downstream
/// audio systems read this resource to control actual playback.
#[derive(Resource, Debug, Clone)]
pub struct AmbientSoundState {
    /// Currently active ambient layers with their intensities.
    pub layers: Vec<ActiveAmbientLayer>,
}

impl Default for AmbientSoundState {
    fn default() -> Self {
        Self {
            layers: Vec::with_capacity(5),
        }
    }
}

impl AmbientSoundState {
    /// Look up the intensity of a specific layer. Returns 0.0 if the layer is
    /// not present (i.e. silent).
    pub fn intensity_of(&self, layer: AmbientLayer) -> f32 {
        self.layers
            .iter()
            .find(|l| l.layer == layer)
            .map_or(0.0, |l| l.intensity)
    }
}

// =============================================================================
// Update timer
// =============================================================================

/// Controls how often the ambient soundscape recalculates (every 2 seconds).
#[derive(Resource)]
struct AmbientUpdateTimer(Timer);

impl Default for AmbientUpdateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

// =============================================================================
// Constants
// =============================================================================

/// Population threshold below which CityHum is silent.
const CITY_HUM_MIN_POP: f32 = 100.0;
/// Population at which CityHum reaches full intensity.
const CITY_HUM_MAX_POP: f32 = 50_000.0;

/// Night ambience is active between these hours (22:00 to 05:00).
const NIGHT_START_HOUR: f32 = 22.0;
const NIGHT_END_HOUR: f32 = 5.0;

// =============================================================================
// System
// =============================================================================

/// Periodically recalculates ambient sound layer intensities based on city state.
#[allow(clippy::too_many_arguments)]
fn update_ambient_soundscape(
    time: Res<Time>,
    mut timer: ResMut<AmbientUpdateTimer>,
    stats: Res<CityStats>,
    weather: Res<Weather>,
    clock: Res<GameClock>,
    trees: Res<TreeGrid>,
    parks: Res<ParksState>,
    citizens: Query<&CitizenStateComp, With<Citizen>>,
    mut state: ResMut<AmbientSoundState>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    state.layers.clear();

    let population = stats.population as f32;

    // -- CityHum: logarithmic scaling with population -------------------------
    let city_hum = if population > CITY_HUM_MIN_POP {
        let log_pop = (population / CITY_HUM_MIN_POP).ln();
        let log_max = (CITY_HUM_MAX_POP / CITY_HUM_MIN_POP).ln();
        (log_pop / log_max).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if city_hum > 0.0 {
        state.layers.push(ActiveAmbientLayer {
            layer: AmbientLayer::CityHum,
            intensity: city_hum,
        });
    }

    // -- Traffic: fraction of citizens currently commuting --------------------
    let total_citizens = citizens.iter().count() as f32;
    let commuting = citizens
        .iter()
        .filter(|s| s.0.is_commuting())
        .count() as f32;
    let traffic_intensity = if total_citizens > 0.0 {
        (commuting / total_citizens).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if traffic_intensity > 0.0 {
        state.layers.push(ActiveAmbientLayer {
            layer: AmbientLayer::Traffic,
            intensity: traffic_intensity,
        });
    }

    // -- Nature: inversely proportional to density, boosted by green space ----
    let total_cells = (crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT) as f32;
    let tree_count = trees.cells.iter().filter(|&&v| v).count() as f32;
    let tree_fraction = tree_count / total_cells;
    let park_count = (parks.small_park_count + parks.large_park_count) as f32;
    // Park bonus: each park adds a bit (capped contribution)
    let park_bonus = (park_count * 0.02).min(0.3);
    // Density penalty: more population = less nature sounds
    let density_penalty = (population / CITY_HUM_MAX_POP).clamp(0.0, 0.8);
    let nature_intensity = ((tree_fraction * 3.0) + park_bonus - density_penalty).clamp(0.0, 1.0);
    if nature_intensity > 0.0 {
        state.layers.push(ActiveAmbientLayer {
            layer: AmbientLayer::Nature,
            intensity: nature_intensity,
        });
    }

    // -- Weather: precipitation and storm intensity ---------------------------
    let weather_intensity = compute_weather_intensity(&weather);
    if weather_intensity > 0.0 {
        state.layers.push(ActiveAmbientLayer {
            layer: AmbientLayer::Weather,
            intensity: weather_intensity,
        });
    }

    // -- Night ambience: active between 22:00 and 05:00 -----------------------
    let night_intensity = compute_night_intensity(clock.hour);
    if night_intensity > 0.0 {
        state.layers.push(ActiveAmbientLayer {
            layer: AmbientLayer::NightAmbience,
            intensity: night_intensity,
        });
    }
}

/// Compute weather layer intensity from precipitation and condition.
fn compute_weather_intensity(weather: &Weather) -> f32 {
    use crate::weather::WeatherCondition;

    let base: f32 = match weather.current_event {
        WeatherCondition::Storm => 1.0,
        WeatherCondition::HeavyRain => 0.8,
        WeatherCondition::Rain => 0.5,
        WeatherCondition::Snow => 0.4,
        WeatherCondition::Overcast => 0.1,
        WeatherCondition::PartlyCloudy | WeatherCondition::Sunny => 0.0,
    };

    // Also factor in precipitation intensity (inches/hr, typically 0-4+)
    let precip_factor = (weather.precipitation_intensity / 2.0).clamp(0.0, 1.0);

    // Take the max of condition-based and precipitation-based intensity
    base.max(precip_factor)
}

/// Compute night ambience intensity with smooth fade-in/fade-out at boundaries.
fn compute_night_intensity(hour: f32) -> f32 {
    // Core night: 23:00 - 04:00 => full intensity
    // Fade in:  22:00 - 23:00
    // Fade out: 04:00 - 05:00
    if hour >= NIGHT_START_HOUR + 1.0 || hour < NIGHT_END_HOUR - 1.0 {
        // Deep night
        1.0
    } else if hour >= NIGHT_START_HOUR {
        // Fade in from 22:00 to 23:00
        hour - NIGHT_START_HOUR
    } else if hour < NIGHT_END_HOUR {
        // Fade out from 04:00 to 05:00
        NIGHT_END_HOUR - hour
    } else {
        0.0
    }
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers the ambient soundscape resource and update system.
pub struct AmbientSoundscapePlugin;

impl Plugin for AmbientSoundscapePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AmbientSoundState>()
            .init_resource::<AmbientUpdateTimer>()
            .add_systems(
                Update,
                update_ambient_soundscape.in_set(crate::SimulationUpdateSet::Visual),
            );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ambient_state_default_is_empty() {
        let state = AmbientSoundState::default();
        assert!(state.layers.is_empty());
    }

    #[test]
    fn test_intensity_of_missing_layer_returns_zero() {
        let state = AmbientSoundState::default();
        assert_eq!(state.intensity_of(AmbientLayer::CityHum), 0.0);
    }

    #[test]
    fn test_intensity_of_present_layer() {
        let state = AmbientSoundState {
            layers: vec![ActiveAmbientLayer {
                layer: AmbientLayer::Traffic,
                intensity: 0.75,
            }],
        };
        assert!((state.intensity_of(AmbientLayer::Traffic) - 0.75).abs() < f32::EPSILON);
        assert_eq!(state.intensity_of(AmbientLayer::Nature), 0.0);
    }

    #[test]
    fn test_night_intensity_deep_night() {
        // At midnight: should be 1.0
        assert_eq!(compute_night_intensity(0.0), 1.0);
        // At 2 AM: should be 1.0
        assert_eq!(compute_night_intensity(2.0), 1.0);
        // At 23:30: should be 1.0
        assert_eq!(compute_night_intensity(23.5), 1.0);
    }

    #[test]
    fn test_night_intensity_fade_in() {
        // At 22:00: intensity = 0.0 (just starting fade)
        assert!((compute_night_intensity(22.0)).abs() < f32::EPSILON);
        // At 22:30: intensity = 0.5
        assert!((compute_night_intensity(22.5) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_night_intensity_fade_out() {
        // At 4:00: intensity = 1.0 (still in fade-out zone, 5.0 - 4.0 = 1.0)
        assert!((compute_night_intensity(4.0) - 1.0).abs() < f32::EPSILON);
        // At 4:50: intensity = 0.15-ish (5.0 - 4.833 ~= 0.167)
        let val = compute_night_intensity(4.5);
        assert!((val - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_night_intensity_daytime_is_zero() {
        assert_eq!(compute_night_intensity(6.0), 0.0);
        assert_eq!(compute_night_intensity(12.0), 0.0);
        assert_eq!(compute_night_intensity(18.0), 0.0);
    }

    #[test]
    fn test_weather_intensity_sunny_is_zero() {
        let weather = Weather {
            current_event: crate::weather::WeatherCondition::Sunny,
            precipitation_intensity: 0.0,
            ..Default::default()
        };
        assert_eq!(compute_weather_intensity(&weather), 0.0);
    }

    #[test]
    fn test_weather_intensity_storm_is_full() {
        let weather = Weather {
            current_event: crate::weather::WeatherCondition::Storm,
            precipitation_intensity: 3.0,
            ..Default::default()
        };
        assert_eq!(compute_weather_intensity(&weather), 1.0);
    }

    #[test]
    fn test_weather_intensity_rain() {
        let weather = Weather {
            current_event: crate::weather::WeatherCondition::Rain,
            precipitation_intensity: 0.3,
            ..Default::default()
        };
        let intensity = compute_weather_intensity(&weather);
        // Condition-based: 0.5, precip-based: 0.3/2.0 = 0.15 => max = 0.5
        assert!((intensity - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weather_intensity_heavy_precipitation_overrides() {
        let weather = Weather {
            current_event: crate::weather::WeatherCondition::Overcast,
            precipitation_intensity: 3.0,
            ..Default::default()
        };
        let intensity = compute_weather_intensity(&weather);
        // Condition-based: 0.1, precip-based: 3.0/2.0 = 1.5 clamped to 1.0 => max = 1.0
        assert!((intensity - 1.0).abs() < f32::EPSILON);
    }
}
