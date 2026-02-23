//! POLL-003: Air Pollution Health Effects with AQI Tiers
//!
//! Applies health rate modifiers to citizens based on the pollution level at
//! their home cell, using EPA-style AQI tiers mapped to the u8 (0-255)
//! pollution range. Also provides a land-value multiplier and an immigration
//! penalty for high-pollution cities.

use bevy::prelude::*;

use crate::citizen::{CitizenDetails, HomeLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::immigration::CityAttractiveness;
use crate::land_value::LandValueGrid;
use crate::pollution::PollutionGrid;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// AQI tier definitions (mapped to u8 0-255 pollution values)
// ---------------------------------------------------------------------------

/// AQI tier classification for a pollution concentration value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AqiTier {
    Good,
    Moderate,
    UnhealthyForSensitive,
    Unhealthy,
    VeryUnhealthy,
    Hazardous,
}

impl AqiTier {
    /// Classify a u8 pollution concentration into an AQI tier.
    ///
    /// Standard EPA AQI uses 0-500, but our pollution grid uses u8 (0-255).
    /// We map proportionally:
    ///   Good:                     0-50   -> 0-50
    ///   Moderate:                51-100  -> 51-100
    ///   Unhealthy for Sensitive: 101-150 -> 101-150
    ///   Unhealthy:              151-200  -> 151-200
    ///   Very Unhealthy:         201-300  -> 201-250  (capped at 255)
    ///   Hazardous:              301+     -> 251-255
    ///
    /// Since our max is 255, "Very Unhealthy" covers 201-250 and
    /// "Hazardous" covers 251-255.
    pub fn from_concentration(concentration: u8) -> Self {
        match concentration {
            0..=50 => Self::Good,
            51..=100 => Self::Moderate,
            101..=150 => Self::UnhealthyForSensitive,
            151..=200 => Self::Unhealthy,
            201..=250 => Self::VeryUnhealthy,
            251..=255 => Self::Hazardous,
        }
    }
}

// ---------------------------------------------------------------------------
// Health modifier function
// ---------------------------------------------------------------------------

/// Returns the per-slow-tick health rate modifier for a given pollution
/// concentration. Positive values heal, negative values damage.
///
/// Tiers:
/// - Good (0-50):                     +0.01  (slight health bonus)
/// - Moderate (51-100):                0.00  (neutral)
/// - Unhealthy for Sensitive (101-150):-0.02  (mild damage)
/// - Unhealthy (151-200):             -0.05  (moderate damage)
/// - Very Unhealthy (201-250):        -0.10  (severe damage)
/// - Hazardous (251-255):             -0.20  (critical damage)
pub fn air_pollution_health_modifier(concentration: u8) -> f32 {
    match AqiTier::from_concentration(concentration) {
        AqiTier::Good => 0.01,
        AqiTier::Moderate => 0.0,
        AqiTier::UnhealthyForSensitive => -0.02,
        AqiTier::Unhealthy => -0.05,
        AqiTier::VeryUnhealthy => -0.10,
        AqiTier::Hazardous => -0.20,
    }
}

/// Returns a land-value multiplier based on pollution concentration.
///
/// Clean air increases land value; polluted areas decrease it.
/// - Good:                     1.05 (+5%)
/// - Moderate:                 1.00 (neutral)
/// - Unhealthy for Sensitive:  0.95 (-5%)
/// - Unhealthy:                0.85 (-15%)
/// - Very Unhealthy:           0.70 (-30%)
/// - Hazardous:                0.50 (-50%)
pub fn pollution_land_value_multiplier(concentration: u8) -> f32 {
    match AqiTier::from_concentration(concentration) {
        AqiTier::Good => 1.05,
        AqiTier::Moderate => 1.00,
        AqiTier::UnhealthyForSensitive => 0.95,
        AqiTier::Unhealthy => 0.85,
        AqiTier::VeryUnhealthy => 0.70,
        AqiTier::Hazardous => 0.50,
    }
}

/// Returns the immigration attractiveness penalty for a given average city
/// pollution level.
///
/// - Good (0-50):                      0.0  (no penalty)
/// - Moderate (51-100):               -1.0
/// - Unhealthy for Sensitive (101-150):-3.0
/// - Unhealthy (151-200):             -6.0
/// - Very Unhealthy (201-250):       -10.0
/// - Hazardous (251-255):            -15.0
pub fn pollution_immigration_penalty(avg_pollution: u8) -> f32 {
    match AqiTier::from_concentration(avg_pollution) {
        AqiTier::Good => 0.0,
        AqiTier::Moderate => -1.0,
        AqiTier::UnhealthyForSensitive => -3.0,
        AqiTier::Unhealthy => -6.0,
        AqiTier::VeryUnhealthy => -10.0,
        AqiTier::Hazardous => -15.0,
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Applies the AQI-based health modifier to each citizen based on pollution
/// at their home cell.
pub fn apply_pollution_health_effects(
    slow_timer: Res<SlowTickTimer>,
    pollution: Res<PollutionGrid>,
    mut citizens: Query<(&HomeLocation, &mut CitizenDetails)>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for (home, mut details) in &mut citizens {
        let gx = home.grid_x.min(GRID_WIDTH - 1);
        let gy = home.grid_y.min(GRID_HEIGHT - 1);
        let concentration = pollution.get(gx, gy);
        let modifier = air_pollution_health_modifier(concentration);
        details.health = (details.health + modifier).clamp(0.0, 100.0);
    }
}

/// Adjusts land values based on AQI-tier pollution multipliers.
pub fn apply_pollution_land_value_effects(
    slow_timer: Res<SlowTickTimer>,
    pollution: Res<PollutionGrid>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let concentration = pollution.get(x, y);
            let multiplier = pollution_land_value_multiplier(concentration);
            // Only apply if multiplier differs from 1.0 (avoid unnecessary work)
            if (multiplier - 1.0).abs() > f32::EPSILON {
                let current = land_value.get(x, y) as f32;
                let adjusted = (current * multiplier).clamp(0.0, 255.0) as u8;
                land_value.set(x, y, adjusted);
            }
        }
    }
}

/// Applies an immigration penalty based on the city's average pollution level.
pub fn apply_pollution_immigration_penalty(
    slow_timer: Res<SlowTickTimer>,
    pollution: Res<PollutionGrid>,
    mut attractiveness: ResMut<CityAttractiveness>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Compute average pollution across the grid
    let total: u64 = pollution.levels.iter().map(|&v| v as u64).sum();
    let avg = (total / pollution.levels.len() as u64) as u8;
    let penalty = pollution_immigration_penalty(avg);
    attractiveness.overall_score = (attractiveness.overall_score + penalty).clamp(0.0, 100.0);
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PollutionHealthPlugin;

impl Plugin for PollutionHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                apply_pollution_health_effects,
                apply_pollution_land_value_effects,
                apply_pollution_immigration_penalty,
            )
                .after(crate::pollution::update_pollution)
                .after(crate::land_value::update_land_value)
                .after(crate::immigration::compute_attractiveness)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aqi_tier_boundaries() {
        assert_eq!(AqiTier::from_concentration(0), AqiTier::Good);
        assert_eq!(AqiTier::from_concentration(50), AqiTier::Good);
        assert_eq!(AqiTier::from_concentration(51), AqiTier::Moderate);
        assert_eq!(AqiTier::from_concentration(100), AqiTier::Moderate);
        assert_eq!(
            AqiTier::from_concentration(101),
            AqiTier::UnhealthyForSensitive
        );
        assert_eq!(
            AqiTier::from_concentration(150),
            AqiTier::UnhealthyForSensitive
        );
        assert_eq!(AqiTier::from_concentration(151), AqiTier::Unhealthy);
        assert_eq!(AqiTier::from_concentration(200), AqiTier::Unhealthy);
        assert_eq!(AqiTier::from_concentration(201), AqiTier::VeryUnhealthy);
        assert_eq!(AqiTier::from_concentration(250), AqiTier::VeryUnhealthy);
        assert_eq!(AqiTier::from_concentration(251), AqiTier::Hazardous);
        assert_eq!(AqiTier::from_concentration(255), AqiTier::Hazardous);
    }

    #[test]
    fn test_health_modifier_values() {
        assert!((air_pollution_health_modifier(0) - 0.01).abs() < f32::EPSILON);
        assert!((air_pollution_health_modifier(50) - 0.01).abs() < f32::EPSILON);
        assert!((air_pollution_health_modifier(75) - 0.0).abs() < f32::EPSILON);
        assert!((air_pollution_health_modifier(125) - (-0.02)).abs() < f32::EPSILON);
        assert!((air_pollution_health_modifier(175) - (-0.05)).abs() < f32::EPSILON);
        assert!((air_pollution_health_modifier(225) - (-0.10)).abs() < f32::EPSILON);
        assert!((air_pollution_health_modifier(255) - (-0.20)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_land_value_multiplier_values() {
        assert!((pollution_land_value_multiplier(25) - 1.05).abs() < f32::EPSILON);
        assert!((pollution_land_value_multiplier(75) - 1.00).abs() < f32::EPSILON);
        assert!((pollution_land_value_multiplier(125) - 0.95).abs() < f32::EPSILON);
        assert!((pollution_land_value_multiplier(175) - 0.85).abs() < f32::EPSILON);
        assert!((pollution_land_value_multiplier(225) - 0.70).abs() < f32::EPSILON);
        assert!((pollution_land_value_multiplier(255) - 0.50).abs() < f32::EPSILON);
    }

    #[test]
    fn test_immigration_penalty_values() {
        assert!((pollution_immigration_penalty(25) - 0.0).abs() < f32::EPSILON);
        assert!((pollution_immigration_penalty(75) - (-1.0)).abs() < f32::EPSILON);
        assert!((pollution_immigration_penalty(125) - (-3.0)).abs() < f32::EPSILON);
        assert!((pollution_immigration_penalty(175) - (-6.0)).abs() < f32::EPSILON);
        assert!((pollution_immigration_penalty(225) - (-10.0)).abs() < f32::EPSILON);
        assert!((pollution_immigration_penalty(255) - (-15.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_health_modifier_monotonically_decreasing() {
        let tiers = [0u8, 51, 101, 151, 201, 251];
        for window in tiers.windows(2) {
            let higher = air_pollution_health_modifier(window[0]);
            let lower = air_pollution_health_modifier(window[1]);
            assert!(
                higher >= lower,
                "health modifier should decrease with pollution: {} (at {}) >= {} (at {})",
                higher,
                window[0],
                lower,
                window[1]
            );
        }
    }

    #[test]
    fn test_land_value_multiplier_monotonically_decreasing() {
        let tiers = [0u8, 51, 101, 151, 201, 251];
        for window in tiers.windows(2) {
            let higher = pollution_land_value_multiplier(window[0]);
            let lower = pollution_land_value_multiplier(window[1]);
            assert!(
                higher >= lower,
                "land value multiplier should decrease with pollution: {} (at {}) >= {} (at {})",
                higher,
                window[0],
                lower,
                window[1]
            );
        }
    }
}
