use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::weather::Weather;
use crate::SlowTickTimer;

/// Rolling history window size: 30 in-game days of rainfall data.
const RAINFALL_HISTORY_SIZE: usize = 30;

/// Drought severity tiers based on the drought index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DroughtTier {
    /// Index > 0.8: no effects.
    #[default]
    Normal,
    /// Index 0.5 to 0.8: lawn watering banned, moderate agriculture impact.
    Moderate,
    /// Index 0.25 to 0.5: mandatory rationing, severe agriculture impact.
    Severe,
    /// Index < 0.25: emergency water imports, agriculture failure.
    Extreme,
}

/// Resource tracking drought conditions derived from rolling rainfall history.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct DroughtState {
    /// Rolling window of daily precipitation values (last 30 days).
    pub rainfall_history: Vec<f32>,
    /// Current drought index: `avg(rainfall_history) / expected_daily_rainfall`.
    pub current_index: f32,
    /// Current drought severity tier.
    pub current_tier: DroughtTier,
    /// Expected daily rainfall baseline (mm/day).
    pub expected_daily_rainfall: f32,
    /// Water demand modifier (1.0 = normal, lower = reduced demand).
    pub water_demand_modifier: f32,
    /// Agriculture output modifier (1.0 = normal, lower = reduced yield).
    pub agriculture_modifier: f32,
    /// Fire risk multiplier (1.0 = normal, higher = increased risk).
    pub fire_risk_multiplier: f32,
    /// Happiness modifier from drought (0.0 = no effect, negative = penalty).
    pub happiness_modifier: f32,
    /// Last game day that recorded rainfall into history.
    pub last_record_day: u32,
}

impl Default for DroughtState {
    fn default() -> Self {
        Self {
            rainfall_history: Vec::new(),
            current_index: 1.0,
            current_tier: DroughtTier::Normal,
            expected_daily_rainfall: 2.5,
            water_demand_modifier: 1.0,
            agriculture_modifier: 1.0,
            fire_risk_multiplier: 1.0,
            happiness_modifier: 0.0,
            last_record_day: 0,
        }
    }
}

/// Classify a drought index value into a severity tier.
pub fn drought_tier_from_index(index: f32) -> DroughtTier {
    if index > 0.8 {
        DroughtTier::Normal
    } else if index > 0.5 {
        DroughtTier::Moderate
    } else if index > 0.25 {
        DroughtTier::Severe
    } else {
        DroughtTier::Extreme
    }
}

/// Return effect modifiers for a given drought tier.
///
/// Returns `(water_demand_modifier, agriculture_modifier, fire_risk_multiplier, happiness_modifier)`.
pub fn drought_effects(tier: DroughtTier) -> (f32, f32, f32, f32) {
    match tier {
        DroughtTier::Normal => (1.0, 1.0, 1.0, 0.0),
        DroughtTier::Moderate => (0.8, 0.7, 2.0, 0.0),
        DroughtTier::Severe => (0.6, 0.4, 4.0, -20.0),
        DroughtTier::Extreme => (0.4, 0.0, 6.0, -40.0),
    }
}

/// System that updates the drought index based on rolling 30-day rainfall average.
///
/// Runs on the slow tick timer (every ~100 ticks). Reads current precipitation
/// from the `Weather` resource, records daily rainfall, and recomputes the
/// drought index and associated modifiers.
pub fn update_drought_index(
    weather: Res<Weather>,
    mut drought: ResMut<DroughtState>,
    timer: Res<SlowTickTimer>,
) {
    if !timer.should_run() {
        return;
    }

    // Record daily rainfall if the day changed since last record.
    let current_day = weather.last_update_day;
    if current_day > drought.last_record_day {
        // precipitation_intensity is 0.0..1.0; scale to approximate mm/day.
        // Expected baseline is 2.5mm, so a fully saturated day produces ~5mm.
        let daily_rainfall = weather.precipitation_intensity * 5.0;
        drought.rainfall_history.push(daily_rainfall);

        // Trim to rolling 30-day window.
        while drought.rainfall_history.len() > RAINFALL_HISTORY_SIZE {
            drought.rainfall_history.remove(0);
        }

        drought.last_record_day = current_day;
    }

    // Calculate drought index.
    if drought.rainfall_history.is_empty() || drought.expected_daily_rainfall <= 0.0 {
        drought.current_index = 1.0;
    } else {
        let sum: f32 = drought.rainfall_history.iter().sum();
        let avg = sum / drought.rainfall_history.len() as f32;
        drought.current_index = (avg / drought.expected_daily_rainfall).min(2.0);
    }

    // Determine tier and apply effects.
    drought.current_tier = drought_tier_from_index(drought.current_index);
    let (water_mod, agri_mod, fire_mult, happy_mod) = drought_effects(drought.current_tier);
    drought.water_demand_modifier = water_mod;
    drought.agriculture_modifier = agri_mod;
    drought.fire_risk_multiplier = fire_mult;
    drought.happiness_modifier = happy_mod;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_normal() {
        assert_eq!(drought_tier_from_index(1.0), DroughtTier::Normal);
        assert_eq!(drought_tier_from_index(0.81), DroughtTier::Normal);
    }

    #[test]
    fn test_tier_moderate() {
        assert_eq!(drought_tier_from_index(0.8), DroughtTier::Moderate);
        assert_eq!(drought_tier_from_index(0.51), DroughtTier::Moderate);
    }

    #[test]
    fn test_tier_severe() {
        assert_eq!(drought_tier_from_index(0.5), DroughtTier::Severe);
        assert_eq!(drought_tier_from_index(0.26), DroughtTier::Severe);
    }

    #[test]
    fn test_tier_extreme() {
        assert_eq!(drought_tier_from_index(0.25), DroughtTier::Extreme);
        assert_eq!(drought_tier_from_index(0.0), DroughtTier::Extreme);
    }

    #[test]
    fn test_effects_normal() {
        let (water, agri, fire, happy) = drought_effects(DroughtTier::Normal);
        assert!((water - 1.0).abs() < f32::EPSILON);
        assert!((agri - 1.0).abs() < f32::EPSILON);
        assert!((fire - 1.0).abs() < f32::EPSILON);
        assert!(happy.abs() < f32::EPSILON);
    }

    #[test]
    fn test_effects_moderate() {
        let (water, agri, fire, happy) = drought_effects(DroughtTier::Moderate);
        assert!((water - 0.8).abs() < f32::EPSILON);
        assert!((agri - 0.7).abs() < f32::EPSILON);
        assert!((fire - 2.0).abs() < f32::EPSILON);
        assert!(happy.abs() < f32::EPSILON);
    }

    #[test]
    fn test_effects_severe() {
        let (water, agri, fire, happy) = drought_effects(DroughtTier::Severe);
        assert!((water - 0.6).abs() < f32::EPSILON);
        assert!((agri - 0.4).abs() < f32::EPSILON);
        assert!((fire - 4.0).abs() < f32::EPSILON);
        assert!((happy - (-20.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effects_extreme() {
        let (water, agri, fire, happy) = drought_effects(DroughtTier::Extreme);
        assert!((water - 0.4).abs() < f32::EPSILON);
        assert!((agri - 0.0).abs() < f32::EPSILON);
        assert!((fire - 6.0).abs() < f32::EPSILON);
        assert!((happy - (-40.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rolling_window_trimmed_to_30() {
        let mut state = DroughtState::default();
        // Push 35 entries
        for i in 0..35 {
            state.rainfall_history.push(i as f32);
        }
        // Trim manually as the system would
        while state.rainfall_history.len() > 30 {
            state.rainfall_history.remove(0);
        }
        assert_eq!(state.rainfall_history.len(), 30);
        // First entry should be 5 (entries 0..4 were trimmed)
        assert!((state.rainfall_history[0] - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_index_calculation() {
        let mut state = DroughtState::default();
        state.expected_daily_rainfall = 2.5;
        // Push 30 days with 1.25mm rainfall each
        for _ in 0..30 {
            state.rainfall_history.push(1.25);
        }
        let sum: f32 = state.rainfall_history.iter().sum();
        let avg = sum / state.rainfall_history.len() as f32;
        let index = avg / state.expected_daily_rainfall;
        // 1.25 / 2.5 = 0.5 -> Severe drought
        assert!((index - 0.5).abs() < f32::EPSILON);
        assert_eq!(drought_tier_from_index(index), DroughtTier::Severe);
    }

    #[test]
    fn test_index_with_no_rainfall() {
        let mut state = DroughtState::default();
        state.expected_daily_rainfall = 2.5;
        for _ in 0..30 {
            state.rainfall_history.push(0.0);
        }
        let sum: f32 = state.rainfall_history.iter().sum();
        let avg = sum / state.rainfall_history.len() as f32;
        let index = avg / state.expected_daily_rainfall;
        assert!(index.abs() < f32::EPSILON);
        assert_eq!(drought_tier_from_index(index), DroughtTier::Extreme);
    }

    #[test]
    fn test_index_with_abundant_rainfall() {
        let mut state = DroughtState::default();
        state.expected_daily_rainfall = 2.5;
        for _ in 0..30 {
            state.rainfall_history.push(5.0);
        }
        let sum: f32 = state.rainfall_history.iter().sum();
        let avg = sum / state.rainfall_history.len() as f32;
        let index = avg / state.expected_daily_rainfall;
        // 5.0 / 2.5 = 2.0 -> Normal
        assert!((index - 2.0).abs() < f32::EPSILON);
        assert_eq!(drought_tier_from_index(index), DroughtTier::Normal);
    }

    #[test]
    fn test_default_state() {
        let state = DroughtState::default();
        assert!(state.rainfall_history.is_empty());
        assert!((state.current_index - 1.0).abs() < f32::EPSILON);
        assert_eq!(state.current_tier, DroughtTier::Normal);
        assert!((state.expected_daily_rainfall - 2.5).abs() < f32::EPSILON);
        assert!((state.water_demand_modifier - 1.0).abs() < f32::EPSILON);
        assert!((state.agriculture_modifier - 1.0).abs() < f32::EPSILON);
        assert!((state.fire_risk_multiplier - 1.0).abs() < f32::EPSILON);
        assert!(state.happiness_modifier.abs() < f32::EPSILON);
    }

    #[test]
    fn test_empty_history_gives_normal_index() {
        let state = DroughtState::default();
        // With empty history, index should be 1.0 (no drought)
        assert!((state.current_index - 1.0).abs() < f32::EPSILON);
        assert_eq!(
            drought_tier_from_index(state.current_index),
            DroughtTier::Normal
        );
    }
}

pub struct DroughtPlugin;

impl Plugin for DroughtPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DroughtState>().add_systems(
            FixedUpdate,
            update_drought_index
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
