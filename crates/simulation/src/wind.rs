use bevy::prelude::*;

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
}

impl Default for WindState {
    fn default() -> Self {
        Self {
            // Default: gentle westerly wind (blowing from the west toward the east)
            direction: 0.0, // 0 radians = east-ward
            speed: 0.3,
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

// =============================================================================
// Systems
// =============================================================================

/// Wind update interval in ticks (matches SlowTickTimer::INTERVAL).
const WIND_UPDATE_INTERVAL: u64 = 100;

/// Maximum angular change per update (radians). ~11 degrees.
const MAX_DIRECTION_SHIFT: f32 = 0.2;

/// Maximum speed change per update.
const MAX_SPEED_SHIFT: f32 = 0.08;

/// Updates wind direction and speed with a gentle random walk every WIND_UPDATE_INTERVAL ticks.
pub fn update_wind(tick: Res<TickCounter>, mut wind: ResMut<WindState>) {
    if tick.0 == 0 || !tick.0.is_multiple_of(WIND_UPDATE_INTERVAL) {
        return;
    }

    // Use tick as seed for deterministic pseudo-random
    let dir_seed = tick.0.wrapping_mul(0xa1b2c3d4e5f60718);
    let spd_seed = tick.0.wrapping_mul(0x1234567890abcdef);

    // Random walk on direction
    let dir_delta = rand_signed_f32(dir_seed) * MAX_DIRECTION_SHIFT;
    wind.direction = (wind.direction + dir_delta).rem_euclid(std::f32::consts::TAU);

    // Random walk on speed, clamped to [0, 1]
    let spd_delta = rand_signed_f32(spd_seed) * MAX_SPEED_SHIFT;
    wind.speed = (wind.speed + spd_delta).clamp(0.0, 1.0);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wind_default() {
        let w = WindState::default();
        assert!((w.direction - 0.0).abs() < f32::EPSILON);
        assert!((w.speed - 0.3).abs() < f32::EPSILON);
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
}
