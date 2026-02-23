use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Cold Snap Detection Thresholds
// =============================================================================

/// Absolute temperature threshold in Celsius: 3+ days below this triggers a cold snap.
pub(crate) const COLD_SNAP_ABSOLUTE_THRESHOLD_C: f32 = -12.0;

/// Deviation below seasonal average that triggers a cold snap (Celsius).
pub(crate) const COLD_SNAP_SEASONAL_DEVIATION_C: f32 = 11.0;

/// Number of consecutive cold days required to activate a cold snap.
pub(crate) const COLD_SNAP_CONSECUTIVE_DAYS: u32 = 3;

// =============================================================================
// Effect Thresholds
// =============================================================================

/// Temperature below which construction is halted (Celsius).
pub(crate) const CONSTRUCTION_HALT_THRESHOLD_C: f32 = -9.0;

/// Temperature below which schools close (Celsius).
pub(crate) const SCHOOL_CLOSURE_THRESHOLD_C: f32 = -29.0;

/// Temperature below which homeless mortality grows exponentially (Celsius).
pub(crate) const HOMELESS_MORTALITY_THRESHOLD_C: f32 = -18.0;

// =============================================================================
// Cold Snap Tier
// =============================================================================

/// Cold snap severity classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColdSnapTier {
    /// No cold snap active.
    #[default]
    Normal,
    /// Approaching thresholds: 1-2 consecutive cold days.
    Watch,
    /// Cold snap active: 3+ consecutive cold days.
    Warning,
    /// Extreme cold: active cold snap with temperature below -23C.
    Emergency,
}

impl ColdSnapTier {
    /// Human-readable label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            ColdSnapTier::Normal => "Normal",
            ColdSnapTier::Watch => "Cold Watch",
            ColdSnapTier::Warning => "Cold Warning",
            ColdSnapTier::Emergency => "Cold Emergency",
        }
    }
}

// =============================================================================
// Cold Snap State (resource)
// =============================================================================

/// Resource tracking cold snap conditions and derived effects.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ColdSnapState {
    /// Number of consecutive days below cold snap thresholds.
    pub consecutive_cold_days: u32,
    /// Number of pipe bursts currently affecting the water system.
    pub pipe_burst_count: u32,
    /// Whether a cold snap is currently active (3+ consecutive cold days).
    pub is_active: bool,
    /// Current cold snap tier classification.
    pub current_tier: ColdSnapTier,
    /// Heating demand modifier (1.0 = normal, 1.8-2.5 during cold snap).
    pub heating_demand_modifier: f32,
    /// Traffic capacity modifier (1.0 = normal, 0.8 during cold snap for vehicle failures).
    pub traffic_capacity_modifier: f32,
    /// Whether schools are closed due to extreme cold (below -29C).
    pub schools_closed: bool,
    /// Whether construction is halted due to cold (below -9C).
    pub construction_halted: bool,
    /// Homeless mortality rate per 100k per day (exponential below -18C without shelter).
    pub homeless_mortality_rate: f32,
    /// Water service reduction factor (1.0 = full service, lower = reduced).
    pub water_service_modifier: f32,
    /// Internal: the last day we checked (to detect day changes).
    pub last_check_day: u32,
}

impl Default for ColdSnapState {
    fn default() -> Self {
        Self {
            consecutive_cold_days: 0,
            pipe_burst_count: 0,
            is_active: false,
            current_tier: ColdSnapTier::Normal,
            heating_demand_modifier: 1.0,
            traffic_capacity_modifier: 1.0,
            schools_closed: false,
            construction_halted: false,
            homeless_mortality_rate: 0.0,
            water_service_modifier: 1.0,
            last_check_day: 0,
        }
    }
}

// =============================================================================
// Cold Snap Event
// =============================================================================

/// Event fired when cold snap conditions change, for notification to other systems.
#[derive(Event, Debug, Clone)]
pub struct ColdSnapEvent {
    /// The cold snap tier that triggered this event.
    pub tier: ColdSnapTier,
    /// Number of new pipe bursts this tick.
    pub new_pipe_bursts: u32,
    /// Whether schools were just closed.
    pub schools_closed: bool,
    /// Whether construction was just halted.
    pub construction_halted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_labels() {
        assert_eq!(ColdSnapTier::Normal.label(), "Normal");
        assert_eq!(ColdSnapTier::Watch.label(), "Cold Watch");
        assert_eq!(ColdSnapTier::Warning.label(), "Cold Warning");
        assert_eq!(ColdSnapTier::Emergency.label(), "Cold Emergency");
    }

    #[test]
    fn test_default_state() {
        let state = ColdSnapState::default();
        assert_eq!(state.consecutive_cold_days, 0);
        assert_eq!(state.pipe_burst_count, 0);
        assert!(!state.is_active);
        assert_eq!(state.current_tier, ColdSnapTier::Normal);
        assert!((state.heating_demand_modifier - 1.0).abs() < f32::EPSILON);
        assert!((state.traffic_capacity_modifier - 1.0).abs() < f32::EPSILON);
        assert!(!state.schools_closed);
        assert!(!state.construction_halted);
        assert!(state.homeless_mortality_rate.abs() < f32::EPSILON);
        assert!((state.water_service_modifier - 1.0).abs() < f32::EPSILON);
        assert_eq!(state.last_check_day, 0);
    }

    #[test]
    fn test_school_closure_threshold() {
        assert!(-30.0 < SCHOOL_CLOSURE_THRESHOLD_C);
        assert!(!(-28.0 < SCHOOL_CLOSURE_THRESHOLD_C));
    }

    #[test]
    fn test_construction_halt_threshold() {
        assert!(-10.0 < CONSTRUCTION_HALT_THRESHOLD_C);
        assert!(!(-8.0 < CONSTRUCTION_HALT_THRESHOLD_C));
    }
}
