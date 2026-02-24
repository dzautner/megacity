//! POLL-012: Noise Pollution Land Value and Health Effects
//!
//! Implements a 7-tier noise classification system that applies land value
//! modifiers and health effects (stress, hearing risk) based on the noise
//! pollution level at each cell. Residential cells receive a 50% worse noise
//! penalty during nighttime hours (22:00-06:00) to model sleep disruption.

use bevy::prelude::*;

use crate::citizen::{CitizenDetails, HomeLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{WorldGrid, ZoneType};
use crate::land_value::LandValueGrid;
use crate::noise::NoisePollutionGrid;
use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// 7-tier noise classification
// ---------------------------------------------------------------------------

/// Noise tier classification mapped to the 0-100 noise pollution range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseTier {
    /// 0-10: Very quiet area (parks, rural)
    Quiet,
    /// 11-25: Typical residential background noise
    Normal,
    /// 26-40: Noticeable noise (busy local road)
    Noticeable,
    /// 41-55: Loud (avenue traffic, light industrial)
    Loud,
    /// 56-70: Very loud (highway, heavy industrial)
    VeryLoud,
    /// 71-85: Painful levels (airport proximity)
    Painful,
    /// 86-100: Dangerous levels (runway, heavy industry cluster)
    Dangerous,
}

impl NoiseTier {
    /// Classify a noise pollution level (0-100) into a tier.
    pub fn from_level(level: u8) -> Self {
        match level {
            0..=10 => Self::Quiet,
            11..=25 => Self::Normal,
            26..=40 => Self::Noticeable,
            41..=55 => Self::Loud,
            56..=70 => Self::VeryLoud,
            71..=85 => Self::Painful,
            86..=100 => Self::Dangerous,
            // Noise grid is capped at 100, but handle overflow gracefully
            _ => Self::Dangerous,
        }
    }

    /// Land value multiplier for this noise tier.
    ///
    /// - Quiet:      1.00 (baseline, no penalty)
    /// - Normal:     1.00 (baseline)
    /// - Noticeable: 0.95 (-5%)
    /// - Loud:       0.85 (-15%)
    /// - VeryLoud:   0.70 (-30%)
    /// - Painful:    0.50 (-50%)
    /// - Dangerous:  0.20 (-80%)
    pub fn land_value_multiplier(self) -> f32 {
        match self {
            Self::Quiet => 1.00,
            Self::Normal => 1.00,
            Self::Noticeable => 0.95,
            Self::Loud => 0.85,
            Self::VeryLoud => 0.70,
            Self::Painful => 0.50,
            Self::Dangerous => 0.20,
        }
    }

    /// Per-slow-tick health modifier for citizens living at this noise level.
    ///
    /// Quiet/Normal/Noticeable: no health impact.
    /// Loud+: stress-induced health damage that increases with tier.
    pub fn health_modifier(self) -> f32 {
        match self {
            Self::Quiet => 0.0,
            Self::Normal => 0.0,
            Self::Noticeable => 0.0,
            Self::Loud => -0.02,      // mild stress
            Self::VeryLoud => -0.05,  // significant stress
            Self::Painful => -0.10,   // hearing risk + severe stress
            Self::Dangerous => -0.20, // hearing damage + extreme stress
        }
    }
}

// ---------------------------------------------------------------------------
// Nighttime noise multiplier
// ---------------------------------------------------------------------------

/// Returns true if the current game hour is during nighttime (22:00-06:00),
/// when noise effects on residential areas are 50% worse due to sleep
/// disruption.
pub fn is_nighttime(hour: f32) -> bool {
    !(6.0..22.0).contains(&hour)
}

/// Nighttime amplification factor for noise effects in residential cells.
/// During 22:00-06:00, noise effects are 50% worse (multiplier = 1.5).
/// During daytime, no amplification (multiplier = 1.0).
pub fn nighttime_multiplier(hour: f32) -> f32 {
    if is_nighttime(hour) {
        1.5
    } else {
        1.0
    }
}

// ---------------------------------------------------------------------------
// Aggregate statistics resource
// ---------------------------------------------------------------------------

/// City-wide noise effect statistics for UI display and tracking.
#[derive(Resource, Debug, Clone, Default)]
pub struct NoiseEffectsStats {
    /// Number of cells at Loud or above
    pub loud_cells: u32,
    /// Number of cells at Painful or above
    pub painful_cells: u32,
    /// Number of cells at Dangerous level
    pub dangerous_cells: u32,
    /// Average noise tier across all non-zero cells (0.0=Quiet, 6.0=Dangerous)
    pub avg_noise_tier: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Applies land value modifiers based on noise tier at each cell.
pub fn apply_noise_land_value_effects(
    slow_timer: Res<SlowTickTimer>,
    noise: Res<NoisePollutionGrid>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let level = noise.get(x, y);
            let tier = NoiseTier::from_level(level);
            let multiplier = tier.land_value_multiplier();

            // Only apply if multiplier differs from 1.0
            if (multiplier - 1.0).abs() > f32::EPSILON {
                let current = land_value.get(x, y) as f32;
                let adjusted = (current * multiplier).clamp(0.0, 255.0) as u8;
                land_value.set(x, y, adjusted);
            }
        }
    }
}

/// Applies health effects (stress and hearing risk) to citizens based on noise
/// at their home cell. Residential cells get 50% worse effects at night.
pub fn apply_noise_health_effects(
    slow_timer: Res<SlowTickTimer>,
    noise: Res<NoisePollutionGrid>,
    grid: Res<WorldGrid>,
    clock: Res<GameClock>,
    mut citizens: Query<(&HomeLocation, &mut CitizenDetails)>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let night_mult = nighttime_multiplier(clock.hour);

    for (home, mut details) in &mut citizens {
        let gx = home.grid_x.min(GRID_WIDTH - 1);
        let gy = home.grid_y.min(GRID_HEIGHT - 1);
        let level = noise.get(gx, gy);
        let tier = NoiseTier::from_level(level);
        let base_modifier = tier.health_modifier();

        // Skip if no health effect
        if base_modifier >= 0.0 {
            continue;
        }

        // Apply nighttime amplification for residential cells
        let cell = grid.get(gx, gy);
        let is_residential = matches!(
            cell.zone,
            ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh
        );

        let modifier = if is_residential {
            base_modifier * night_mult
        } else {
            base_modifier
        };

        details.health = (details.health + modifier).clamp(0.0, 100.0);
    }
}

/// Updates aggregate noise statistics for the city.
pub fn update_noise_effects_stats(
    slow_timer: Res<SlowTickTimer>,
    noise: Res<NoisePollutionGrid>,
    mut stats: ResMut<NoiseEffectsStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut loud = 0u32;
    let mut painful = 0u32;
    let mut dangerous = 0u32;
    let mut tier_sum = 0u64;
    let mut nonzero = 0u64;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let level = noise.get(x, y);
            if level == 0 {
                continue;
            }
            nonzero += 1;
            let tier = NoiseTier::from_level(level);
            let tier_val = match tier {
                NoiseTier::Quiet => 0,
                NoiseTier::Normal => 1,
                NoiseTier::Noticeable => 2,
                NoiseTier::Loud => 3,
                NoiseTier::VeryLoud => 4,
                NoiseTier::Painful => 5,
                NoiseTier::Dangerous => 6,
            };
            tier_sum += tier_val;

            match tier {
                NoiseTier::Loud
                | NoiseTier::VeryLoud
                | NoiseTier::Painful
                | NoiseTier::Dangerous => loud += 1,
                _ => {}
            }
            match tier {
                NoiseTier::Painful | NoiseTier::Dangerous => painful += 1,
                _ => {}
            }
            if tier == NoiseTier::Dangerous {
                dangerous += 1;
            }
        }
    }

    stats.loud_cells = loud;
    stats.painful_cells = painful;
    stats.dangerous_cells = dangerous;
    stats.avg_noise_tier = if nonzero > 0 {
        tier_sum as f32 / nonzero as f32
    } else {
        0.0
    };
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct NoiseEffectsPlugin;

impl Plugin for NoiseEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoiseEffectsStats>().add_systems(
            FixedUpdate,
            (
                apply_noise_land_value_effects,
                apply_noise_health_effects,
                update_noise_effects_stats,
            )
                .after(crate::noise::update_noise_pollution)
                .after(crate::land_value::update_land_value)
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
    fn test_noise_tier_boundaries() {
        assert_eq!(NoiseTier::from_level(0), NoiseTier::Quiet);
        assert_eq!(NoiseTier::from_level(10), NoiseTier::Quiet);
        assert_eq!(NoiseTier::from_level(11), NoiseTier::Normal);
        assert_eq!(NoiseTier::from_level(25), NoiseTier::Normal);
        assert_eq!(NoiseTier::from_level(26), NoiseTier::Noticeable);
        assert_eq!(NoiseTier::from_level(40), NoiseTier::Noticeable);
        assert_eq!(NoiseTier::from_level(41), NoiseTier::Loud);
        assert_eq!(NoiseTier::from_level(55), NoiseTier::Loud);
        assert_eq!(NoiseTier::from_level(56), NoiseTier::VeryLoud);
        assert_eq!(NoiseTier::from_level(70), NoiseTier::VeryLoud);
        assert_eq!(NoiseTier::from_level(71), NoiseTier::Painful);
        assert_eq!(NoiseTier::from_level(85), NoiseTier::Painful);
        assert_eq!(NoiseTier::from_level(86), NoiseTier::Dangerous);
        assert_eq!(NoiseTier::from_level(100), NoiseTier::Dangerous);
    }

    #[test]
    fn test_overflow_level_maps_to_dangerous() {
        assert_eq!(NoiseTier::from_level(120), NoiseTier::Dangerous);
        assert_eq!(NoiseTier::from_level(255), NoiseTier::Dangerous);
    }

    #[test]
    fn test_land_value_multiplier_quiet_neutral() {
        let mult = NoiseTier::Quiet.land_value_multiplier();
        assert!(
            (mult - 1.00).abs() < f32::EPSILON,
            "quiet areas should have neutral land value, got {}",
            mult
        );
    }

    #[test]
    fn test_land_value_multiplier_dangerous_penalty() {
        let mult = NoiseTier::Dangerous.land_value_multiplier();
        assert!(
            (mult - 0.20).abs() < f32::EPSILON,
            "dangerous noise should get -80% land value, got {}",
            mult
        );
    }

    #[test]
    fn test_land_value_multiplier_monotonically_decreasing() {
        let tiers = [
            NoiseTier::Quiet,
            NoiseTier::Normal,
            NoiseTier::Noticeable,
            NoiseTier::Loud,
            NoiseTier::VeryLoud,
            NoiseTier::Painful,
            NoiseTier::Dangerous,
        ];
        for window in tiers.windows(2) {
            let higher = window[0].land_value_multiplier();
            let lower = window[1].land_value_multiplier();
            assert!(
                higher >= lower,
                "land value multiplier should decrease: {:?}={} >= {:?}={}",
                window[0],
                higher,
                window[1],
                lower
            );
        }
    }

    #[test]
    fn test_health_modifier_quiet_normal_noticeable_zero() {
        assert!((NoiseTier::Quiet.health_modifier()).abs() < f32::EPSILON);
        assert!((NoiseTier::Normal.health_modifier()).abs() < f32::EPSILON);
        assert!((NoiseTier::Noticeable.health_modifier()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_health_modifier_loud_and_above_negative() {
        assert!(NoiseTier::Loud.health_modifier() < 0.0);
        assert!(NoiseTier::VeryLoud.health_modifier() < 0.0);
        assert!(NoiseTier::Painful.health_modifier() < 0.0);
        assert!(NoiseTier::Dangerous.health_modifier() < 0.0);
    }

    #[test]
    fn test_health_modifier_monotonically_decreasing() {
        let tiers = [
            NoiseTier::Loud,
            NoiseTier::VeryLoud,
            NoiseTier::Painful,
            NoiseTier::Dangerous,
        ];
        for window in tiers.windows(2) {
            let higher = window[0].health_modifier();
            let lower = window[1].health_modifier();
            assert!(
                higher >= lower,
                "health modifier should decrease: {:?}={} >= {:?}={}",
                window[0],
                higher,
                window[1],
                lower
            );
        }
    }

    #[test]
    fn test_nighttime_detection() {
        assert!(is_nighttime(22.0));
        assert!(is_nighttime(23.5));
        assert!(is_nighttime(0.0));
        assert!(is_nighttime(3.0));
        assert!(is_nighttime(5.9));
        assert!(!is_nighttime(6.0));
        assert!(!is_nighttime(12.0));
        assert!(!is_nighttime(18.0));
        assert!(!is_nighttime(21.9));
    }

    #[test]
    fn test_nighttime_multiplier_values() {
        assert!(
            (nighttime_multiplier(3.0) - 1.5).abs() < f32::EPSILON,
            "nighttime multiplier at 3am should be 1.5"
        );
        assert!(
            (nighttime_multiplier(12.0) - 1.0).abs() < f32::EPSILON,
            "daytime multiplier at noon should be 1.0"
        );
    }
}
