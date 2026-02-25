//! PROG-009: Hope and Discontent Dual Meters
//!
//! Tracks city-wide Hope (citizen optimism, 0.0–1.0) and Discontent (frustration,
//! 0.0–1.0). Every city condition/event nudges one or both meters. When hope drops
//! below 0.1 or discontent rises above 0.9 the city enters a political crisis.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::blackout::BlackoutState;
use crate::budget::ExtendedBudget;
use crate::coverage_metrics::CoverageMetrics;
use crate::crime::CrimeGrid;
use crate::economy::CityBudget;
use crate::homelessness::HomelessnessStats;
use crate::pollution::PollutionGrid;
use crate::{decode_or_warn, Saveable, SimulationSet, SlowTickTimer};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default hope level for a new city.
const DEFAULT_HOPE: f32 = 0.5;

/// Default discontent level for a new city.
const DEFAULT_DISCONTENT: f32 = 0.2;

/// Per-tick adjustment rate (slow-tick cadence). Small so meters drift gradually.
const ADJUSTMENT_RATE: f32 = 0.002;

/// Hope threshold below which a crisis is triggered.
const HOPE_CRISIS_THRESHOLD: f32 = 0.1;

/// Discontent threshold above which a crisis is triggered.
const DISCONTENT_CRISIS_THRESHOLD: f32 = 0.9;

/// Hope threshold for the warning state.
const HOPE_WARNING_THRESHOLD: f32 = 0.25;

/// Discontent threshold for the warning state.
const DISCONTENT_WARNING_THRESHOLD: f32 = 0.75;

/// Average pollution level (0–255) above which pollution causes discontent.
const POLLUTION_HIGH_THRESHOLD: f32 = 80.0;

/// Average crime level (0–255) below which crime is considered low.
const CRIME_LOW_THRESHOLD: f32 = 15.0;

/// Minimum service coverage average to provide a hope boost.
const SERVICE_COVERAGE_THRESHOLD: f32 = 0.5;

// ---------------------------------------------------------------------------
// CrisisState enum
// ---------------------------------------------------------------------------

/// The political stability state of the city.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Default)]
pub enum CrisisState {
    /// City is operating normally.
    #[default]
    Normal,
    /// Warning: hope is low or discontent is elevated.
    Warning,
    /// Full political crisis: hope critically low or discontent critically high.
    Crisis,
}

// ---------------------------------------------------------------------------
// HopeDiscontent resource
// ---------------------------------------------------------------------------

/// City-wide dual meters tracking citizen optimism and frustration.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct HopeDiscontent {
    /// Citizen optimism (0.0 = despair, 1.0 = utopia). Default 0.5.
    pub hope: f32,
    /// Citizen frustration (0.0 = content, 1.0 = revolt). Default 0.2.
    pub discontent: f32,
    /// Current political stability state, derived from hope/discontent.
    pub crisis_state: CrisisState,
}

impl Default for HopeDiscontent {
    fn default() -> Self {
        Self {
            hope: DEFAULT_HOPE,
            discontent: DEFAULT_DISCONTENT,
            crisis_state: CrisisState::Normal,
        }
    }
}

impl HopeDiscontent {
    /// Nudge hope by `delta` and clamp to [0.0, 1.0].
    pub fn adjust_hope(&mut self, delta: f32) {
        self.hope = (self.hope + delta).clamp(0.0, 1.0);
    }

    /// Nudge discontent by `delta` and clamp to [0.0, 1.0].
    pub fn adjust_discontent(&mut self, delta: f32) {
        self.discontent = (self.discontent + delta).clamp(0.0, 1.0);
    }

    /// Recompute `crisis_state` from current meter values.
    pub fn update_crisis_state(&mut self) {
        if self.hope < HOPE_CRISIS_THRESHOLD || self.discontent > DISCONTENT_CRISIS_THRESHOLD {
            self.crisis_state = CrisisState::Crisis;
        } else if self.hope < HOPE_WARNING_THRESHOLD
            || self.discontent > DISCONTENT_WARNING_THRESHOLD
        {
            self.crisis_state = CrisisState::Warning;
        } else {
            self.crisis_state = CrisisState::Normal;
        }
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl Saveable for HopeDiscontent {
    const SAVE_KEY: &'static str = "hope_discontent";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// System: update_hope_discontent
// ---------------------------------------------------------------------------

/// Adjusts hope and discontent based on city-wide conditions.
///
/// Runs on the `SlowTickTimer` cadence (every 100 ticks) so meter changes
/// are gradual and don't cause per-tick noise.
#[allow(clippy::too_many_arguments)]
pub fn update_hope_discontent(
    timer: Res<SlowTickTimer>,
    mut meters: ResMut<HopeDiscontent>,
    budget: Res<CityBudget>,
    extended_budget: Res<ExtendedBudget>,
    homelessness: Res<HomelessnessStats>,
    pollution: Res<PollutionGrid>,
    crime: Res<CrimeGrid>,
    coverage: Res<CoverageMetrics>,
    blackout: Res<BlackoutState>,
) {
    if !timer.should_run() {
        return;
    }

    // --- Economy: budget surplus/deficit ---
    let net_income = budget.monthly_income - budget.monthly_expenses;
    let debt = extended_budget.total_debt();
    if net_income > 0.0 && debt < 1000.0 {
        // Good economy: surplus and low debt
        meters.adjust_hope(ADJUSTMENT_RATE);
        meters.adjust_discontent(-ADJUSTMENT_RATE * 0.5);
    } else if net_income < -500.0 || debt > 50_000.0 {
        // Bad economy: significant deficit or heavy debt
        meters.adjust_hope(-ADJUSTMENT_RATE);
        meters.adjust_discontent(ADJUSTMENT_RATE);
    }

    // --- Homelessness ---
    if homelessness.total_homeless > 0 {
        let severity = (homelessness.total_homeless as f32 / 50.0).min(1.0);
        meters.adjust_hope(-ADJUSTMENT_RATE * severity);
        meters.adjust_discontent(ADJUSTMENT_RATE * severity);
    }

    // --- Pollution (city-wide average) ---
    let avg_pollution = compute_average_level(&pollution.levels);
    if avg_pollution > POLLUTION_HIGH_THRESHOLD {
        let severity = ((avg_pollution - POLLUTION_HIGH_THRESHOLD) / 175.0).min(1.0);
        meters.adjust_hope(-ADJUSTMENT_RATE * severity);
        meters.adjust_discontent(ADJUSTMENT_RATE * severity);
    }

    // --- Crime (city-wide average) ---
    let avg_crime = compute_average_level(&crime.levels);
    if avg_crime < CRIME_LOW_THRESHOLD {
        // Low crime is good
        meters.adjust_hope(ADJUSTMENT_RATE * 0.5);
    } else if avg_crime > 40.0 {
        // High crime is bad
        let severity = ((avg_crime - 40.0) / 60.0).min(1.0);
        meters.adjust_discontent(ADJUSTMENT_RATE * severity);
    }

    // --- Service coverage ---
    let avg_coverage = (coverage.power
        + coverage.water
        + coverage.education
        + coverage.fire
        + coverage.police
        + coverage.health)
        / 6.0;
    if avg_coverage > SERVICE_COVERAGE_THRESHOLD {
        meters.adjust_hope(ADJUSTMENT_RATE * 0.5);
    } else if avg_coverage < 0.2 {
        meters.adjust_discontent(ADJUSTMENT_RATE);
    }

    // --- Blackouts ---
    if blackout.active {
        meters.adjust_hope(-ADJUSTMENT_RATE);
        meters.adjust_discontent(ADJUSTMENT_RATE * 1.5);
    }

    // --- Update crisis state ---
    meters.update_crisis_state();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute the average of a u8 grid, only counting non-zero cells to avoid
/// dilution from empty wilderness. Returns 0.0 if no non-zero cells exist.
fn compute_average_level(levels: &[u8]) -> f32 {
    let mut sum: u64 = 0;
    let mut count: u64 = 0;
    for &v in levels {
        if v > 0 {
            sum += v as u64;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        sum as f32 / count as f32
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct HopeDiscontentPlugin;

impl Plugin for HopeDiscontentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HopeDiscontent>();

        // Register for save/load.
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<HopeDiscontent>();

        app.add_systems(
            FixedUpdate,
            update_hope_discontent
                .after(crate::economy::collect_taxes)
                .in_set(SimulationSet::Simulation),
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
    fn test_default_values() {
        let hd = HopeDiscontent::default();
        assert!((hd.hope - 0.5).abs() < f32::EPSILON);
        assert!((hd.discontent - 0.2).abs() < f32::EPSILON);
        assert_eq!(hd.crisis_state, CrisisState::Normal);
    }

    #[test]
    fn test_adjust_hope_clamps() {
        let mut hd = HopeDiscontent::default();
        hd.adjust_hope(10.0);
        assert!((hd.hope - 1.0).abs() < f32::EPSILON);
        hd.adjust_hope(-20.0);
        assert!(hd.hope.abs() < f32::EPSILON);
    }

    #[test]
    fn test_adjust_discontent_clamps() {
        let mut hd = HopeDiscontent::default();
        hd.adjust_discontent(10.0);
        assert!((hd.discontent - 1.0).abs() < f32::EPSILON);
        hd.adjust_discontent(-20.0);
        assert!(hd.discontent.abs() < f32::EPSILON);
    }

    #[test]
    fn test_crisis_state_normal() {
        let mut hd = HopeDiscontent {
            hope: 0.5,
            discontent: 0.3,
            crisis_state: CrisisState::Crisis,
        };
        hd.update_crisis_state();
        assert_eq!(hd.crisis_state, CrisisState::Normal);
    }

    #[test]
    fn test_crisis_state_warning_low_hope() {
        let mut hd = HopeDiscontent {
            hope: 0.2,
            discontent: 0.3,
            crisis_state: CrisisState::Normal,
        };
        hd.update_crisis_state();
        assert_eq!(hd.crisis_state, CrisisState::Warning);
    }

    #[test]
    fn test_crisis_state_warning_high_discontent() {
        let mut hd = HopeDiscontent {
            hope: 0.5,
            discontent: 0.8,
            crisis_state: CrisisState::Normal,
        };
        hd.update_crisis_state();
        assert_eq!(hd.crisis_state, CrisisState::Warning);
    }

    #[test]
    fn test_crisis_state_crisis_low_hope() {
        let mut hd = HopeDiscontent {
            hope: 0.05,
            discontent: 0.3,
            crisis_state: CrisisState::Normal,
        };
        hd.update_crisis_state();
        assert_eq!(hd.crisis_state, CrisisState::Crisis);
    }

    #[test]
    fn test_crisis_state_crisis_high_discontent() {
        let mut hd = HopeDiscontent {
            hope: 0.5,
            discontent: 0.95,
            crisis_state: CrisisState::Normal,
        };
        hd.update_crisis_state();
        assert_eq!(hd.crisis_state, CrisisState::Crisis);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let hd = HopeDiscontent {
            hope: 0.75,
            discontent: 0.42,
            crisis_state: CrisisState::Warning,
        };
        let bytes = hd.save_to_bytes().unwrap();
        let restored = HopeDiscontent::load_from_bytes(&bytes);
        assert!((restored.hope - 0.75).abs() < f32::EPSILON);
        assert!((restored.discontent - 0.42).abs() < f32::EPSILON);
        assert_eq!(restored.crisis_state, CrisisState::Warning);
    }

    #[test]
    fn test_compute_average_level_empty() {
        let levels = vec![0u8; 100];
        assert!(compute_average_level(&levels).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_average_level_nonzero() {
        let mut levels = vec![0u8; 100];
        levels[0] = 50;
        levels[1] = 100;
        // Average of non-zero: (50+100)/2 = 75
        assert!((compute_average_level(&levels) - 75.0).abs() < f32::EPSILON);
    }
}
