use bevy::prelude::*;

use crate::weather::{ClimateZone, WeatherCondition};

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
