use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::weather::{ClimateZone, Weather};
use crate::SlowTickTimer;

// =============================================================================
// Cold Snap Detection Thresholds
// =============================================================================

/// Absolute temperature threshold in Celsius: 3+ days below this triggers a cold snap.
const COLD_SNAP_ABSOLUTE_THRESHOLD_C: f32 = -12.0;

/// Deviation below seasonal average that triggers a cold snap (Celsius).
const COLD_SNAP_SEASONAL_DEVIATION_C: f32 = 11.0;

/// Number of consecutive cold days required to activate a cold snap.
const COLD_SNAP_CONSECUTIVE_DAYS: u32 = 3;

// =============================================================================
// Pipe Burst Temperature Tiers
// =============================================================================

/// Baseline pipe burst probability per mile of water main per day (above freezing).
const PIPE_BURST_BASELINE: f32 = 0.0001;

/// Pipe burst probability at freezing (0C).
const PIPE_BURST_FREEZING: f32 = 0.001;

/// Pipe burst probability below -7C.
const PIPE_BURST_MINUS_7: f32 = 0.01;

/// Pipe burst probability below -18C.
const PIPE_BURST_MINUS_18: f32 = 0.05;

/// Pipe burst probability below -23C.
const PIPE_BURST_MINUS_23: f32 = 0.10;

// =============================================================================
// Effect Thresholds
// =============================================================================

/// Temperature below which construction is halted (Celsius).
const CONSTRUCTION_HALT_THRESHOLD_C: f32 = -9.0;

/// Temperature below which schools close (Celsius).
const SCHOOL_CLOSURE_THRESHOLD_C: f32 = -29.0;

/// Temperature below which homeless mortality grows exponentially (Celsius).
const HOMELESS_MORTALITY_THRESHOLD_C: f32 = -18.0;

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

// =============================================================================
// Deterministic pseudo-random (splitmix64, matching wind_damage.rs pattern)
// =============================================================================

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Returns a deterministic pseudo-random f32 in [0.0, 1.0) based on seed.
fn rand_f32(seed: u64) -> f32 {
    let hash = splitmix64(seed);
    (hash % 1_000_000) as f32 / 1_000_000.0
}

// =============================================================================
// Pure functions
// =============================================================================

/// Return the pipe burst probability per mile of water main per day for a given
/// temperature in Celsius.
///
/// Tiers from the specification:
/// - Above 0C:   0.0001 (baseline)
/// - 0C to -7C:  0.001  (freezing)
/// - -7C to -18C: 0.01
/// - -18C to -23C: 0.05
/// - Below -23C:  0.10
pub fn pipe_burst_probability(temp_c: f32) -> f32 {
    if temp_c > 0.0 {
        PIPE_BURST_BASELINE
    } else if temp_c > -7.0 {
        PIPE_BURST_FREEZING
    } else if temp_c > -18.0 {
        PIPE_BURST_MINUS_7
    } else if temp_c > -23.0 {
        PIPE_BURST_MINUS_18
    } else {
        PIPE_BURST_MINUS_23
    }
}

/// Determine whether the current temperature qualifies as "cold" relative to
/// the absolute threshold or the seasonal average.
///
/// A day is cold if either:
/// - Temperature is below -12C (absolute threshold), OR
/// - Temperature is more than 11C below the seasonal average
pub fn is_cold_day(temp_c: f32, seasonal_avg: f32) -> bool {
    temp_c < COLD_SNAP_ABSOLUTE_THRESHOLD_C
        || temp_c < (seasonal_avg - COLD_SNAP_SEASONAL_DEVIATION_C)
}

/// Compute the seasonal average temperature for the current season and climate zone.
///
/// Uses the midpoint of the season's min/max temperature range.
pub fn seasonal_average_temp(season: crate::weather::Season, zone: ClimateZone) -> f32 {
    let (t_min, t_max) = season.temperature_range_for_zone(zone);
    (t_min + t_max) / 2.0
}

/// Classify the cold snap tier from consecutive cold days and current temperature.
pub fn cold_snap_tier(consecutive_days: u32, temp_c: f32) -> ColdSnapTier {
    if consecutive_days >= COLD_SNAP_CONSECUTIVE_DAYS {
        if temp_c < -23.0 {
            ColdSnapTier::Emergency
        } else {
            ColdSnapTier::Warning
        }
    } else if consecutive_days >= 1 {
        ColdSnapTier::Watch
    } else {
        ColdSnapTier::Normal
    }
}

/// Calculate heating demand modifier based on cold snap tier and temperature.
///
/// During a cold snap, heating demand surges +80-150% above normal:
/// - Watch: +0% (monitoring only)
/// - Warning: +80% (1.8x)
/// - Emergency: +150% (2.5x)
///
/// Additionally, for non-active cold snaps at sub-zero temps, a mild
/// increase is applied proportional to how far below 0C.
pub fn heating_demand_modifier(tier: ColdSnapTier, temp_c: f32) -> f32 {
    match tier {
        ColdSnapTier::Normal => {
            if temp_c < 0.0 {
                // Mild increase when below freezing but no cold snap
                (1.0 + (-temp_c) * 0.02).min(1.3)
            } else {
                1.0
            }
        }
        ColdSnapTier::Watch => {
            if temp_c < 0.0 {
                (1.0 + (-temp_c) * 0.03).min(1.5)
            } else {
                1.0
            }
        }
        ColdSnapTier::Warning => 1.8,
        ColdSnapTier::Emergency => 2.5,
    }
}

/// Calculate homeless mortality rate per 100k per day.
///
/// Exponential curve below -18C: `2.0 * exp(0.3 * (threshold - temp))`
/// Returns 0.0 when temperature is above the threshold.
pub fn homeless_mortality(temp_c: f32) -> f32 {
    if temp_c >= HOMELESS_MORTALITY_THRESHOLD_C {
        return 0.0;
    }
    let excess = HOMELESS_MORTALITY_THRESHOLD_C - temp_c;
    2.0 * (0.3 * excess).exp()
}

/// Estimate water main miles from road cell count.
///
/// Approximation: each road cell represents ~0.003 miles of water main
/// (256x256 grid ~ 65k cells, typical city has ~6000 miles of water mains,
/// and roads cover roughly 30% of the grid).
const WATER_MAIN_MILES_PER_ROAD_CELL: f32 = 0.003;

/// Estimate total water main miles from road network cell count.
pub fn estimate_water_main_miles(road_cell_count: u32) -> f32 {
    road_cell_count as f32 * WATER_MAIN_MILES_PER_ROAD_CELL
}

/// Calculate the number of new pipe bursts based on temperature and water main miles.
///
/// Uses deterministic pseudo-random sampling: divides the water main network into
/// discrete segments and rolls for each segment.
pub fn calculate_pipe_bursts(temp_c: f32, water_main_miles: f32, seed: u64) -> u32 {
    let prob = pipe_burst_probability(temp_c);
    // Each "mile" is a discrete segment that can burst independently.
    let segments = water_main_miles.ceil() as u32;
    let mut bursts = 0u32;
    for i in 0..segments {
        let roll_seed = seed.wrapping_mul(0x517cc1b727220a95).wrapping_add(i as u64);
        if rand_f32(roll_seed) < prob {
            bursts += 1;
        }
    }
    bursts
}

/// Water service reduction based on pipe burst count relative to total water main miles.
///
/// Each burst reduces service proportionally. Clamped to [0.2, 1.0] (never below 20%
/// service -- some redundancy always exists).
pub fn water_service_from_bursts(pipe_burst_count: u32, water_main_miles: f32) -> f32 {
    if water_main_miles <= 0.0 {
        return 1.0;
    }
    // Each burst takes out roughly 0.5 miles of service capacity
    let affected_miles = pipe_burst_count as f32 * 0.5;
    let reduction = affected_miles / water_main_miles;
    (1.0 - reduction).clamp(0.2, 1.0)
}

// =============================================================================
// System
// =============================================================================

/// System that updates the `ColdSnapState` resource based on current weather.
///
/// Runs on the slow tick timer (every ~100 ticks). Reads the `Weather` resource
/// for temperature and tracks consecutive cold days, pipe bursts, and derived
/// effects (heating demand, traffic capacity, school closures, construction halt,
/// homeless mortality).
pub fn update_cold_snap(
    weather: Res<Weather>,
    climate: Res<ClimateZone>,
    mut state: ResMut<ColdSnapState>,
    timer: Res<SlowTickTimer>,
    mut events: EventWriter<ColdSnapEvent>,
) {
    if !timer.should_run() {
        return;
    }

    let temp = weather.temperature;
    let current_day = weather.last_update_day;
    let seasonal_avg = seasonal_average_temp(weather.season, *climate);

    // --- Day change: update consecutive cold day counter ---
    if current_day != state.last_check_day && current_day > 0 {
        state.last_check_day = current_day;

        if is_cold_day(temp, seasonal_avg) {
            state.consecutive_cold_days += 1;
        } else {
            // Reset streak on a non-cold day; also repair pipes over time
            state.consecutive_cold_days = 0;
            // Pipes get repaired: reduce burst count by 20% per warm day (min 0)
            state.pipe_burst_count = (state.pipe_burst_count as f32 * 0.8) as u32;
        }
    }

    // --- Determine tier ---
    let prev_tier = state.current_tier;
    state.current_tier = cold_snap_tier(state.consecutive_cold_days, temp);
    state.is_active = matches!(
        state.current_tier,
        ColdSnapTier::Warning | ColdSnapTier::Emergency
    );

    // --- Pipe bursts (only when below freezing) ---
    let mut new_bursts = 0u32;
    if temp <= 0.0 {
        // Approximate water main miles from a rough road cell estimate.
        // In a real integration, this would read from the road network resource.
        // For now, use a conservative estimate of 5000 road cells (~15 miles).
        let estimated_road_cells: u32 = 5000;
        let water_main_miles = estimate_water_main_miles(estimated_road_cells);
        let seed = current_day as u64 * 0xdeadbeef_cafebabe;
        new_bursts = calculate_pipe_bursts(temp, water_main_miles, seed);
        state.pipe_burst_count = state.pipe_burst_count.saturating_add(new_bursts);

        // Update water service modifier
        state.water_service_modifier =
            water_service_from_bursts(state.pipe_burst_count, water_main_miles);
    } else {
        // Above freezing: water service recovers
        state.water_service_modifier = water_service_from_bursts(state.pipe_burst_count, 15.0);
    }

    // --- Heating demand surge ---
    state.heating_demand_modifier = heating_demand_modifier(state.current_tier, temp);

    // --- Traffic capacity: -20% during active cold snap (vehicle failures) ---
    state.traffic_capacity_modifier = if state.is_active { 0.8 } else { 1.0 };

    // --- School closures below -29C ---
    state.schools_closed = temp < SCHOOL_CLOSURE_THRESHOLD_C;

    // --- Construction halted below -9C ---
    state.construction_halted = temp < CONSTRUCTION_HALT_THRESHOLD_C;

    // --- Homeless mortality ---
    state.homeless_mortality_rate = homeless_mortality(temp);

    // --- Fire event on tier change or new pipe bursts ---
    let tier_changed = state.current_tier != prev_tier;
    if tier_changed || new_bursts > 0 {
        events.send(ColdSnapEvent {
            tier: state.current_tier,
            new_pipe_bursts: new_bursts,
            schools_closed: state.schools_closed,
            construction_halted: state.construction_halted,
        });
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Pipe burst probability tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_pipe_burst_above_freezing() {
        assert!(
            (pipe_burst_probability(10.0) - PIPE_BURST_BASELINE).abs() < f32::EPSILON,
            "Above freezing should return baseline"
        );
        assert!(
            (pipe_burst_probability(0.1) - PIPE_BURST_BASELINE).abs() < f32::EPSILON,
            "Just above freezing should return baseline"
        );
    }

    #[test]
    fn test_pipe_burst_at_freezing() {
        assert!(
            (pipe_burst_probability(0.0) - PIPE_BURST_FREEZING).abs() < f32::EPSILON,
            "At freezing should return freezing tier"
        );
        assert!(
            (pipe_burst_probability(-3.0) - PIPE_BURST_FREEZING).abs() < f32::EPSILON,
            "Between 0C and -7C should return freezing tier"
        );
        assert!(
            (pipe_burst_probability(-6.9) - PIPE_BURST_FREEZING).abs() < f32::EPSILON,
            "Just above -7C should return freezing tier"
        );
    }

    #[test]
    fn test_pipe_burst_below_minus_7() {
        assert!(
            (pipe_burst_probability(-7.0) - PIPE_BURST_MINUS_7).abs() < f32::EPSILON,
            "At -7C should return minus-7 tier"
        );
        assert!(
            (pipe_burst_probability(-12.0) - PIPE_BURST_MINUS_7).abs() < f32::EPSILON,
            "Between -7C and -18C should return minus-7 tier"
        );
        assert!(
            (pipe_burst_probability(-17.9) - PIPE_BURST_MINUS_7).abs() < f32::EPSILON,
            "Just above -18C should return minus-7 tier"
        );
    }

    #[test]
    fn test_pipe_burst_below_minus_18() {
        assert!(
            (pipe_burst_probability(-18.0) - PIPE_BURST_MINUS_18).abs() < f32::EPSILON,
            "At -18C should return minus-18 tier"
        );
        assert!(
            (pipe_burst_probability(-20.0) - PIPE_BURST_MINUS_18).abs() < f32::EPSILON,
            "Between -18C and -23C should return minus-18 tier"
        );
        assert!(
            (pipe_burst_probability(-22.9) - PIPE_BURST_MINUS_18).abs() < f32::EPSILON,
            "Just above -23C should return minus-18 tier"
        );
    }

    #[test]
    fn test_pipe_burst_below_minus_23() {
        assert!(
            (pipe_burst_probability(-23.0) - PIPE_BURST_MINUS_23).abs() < f32::EPSILON,
            "At -23C should return minus-23 tier"
        );
        assert!(
            (pipe_burst_probability(-30.0) - PIPE_BURST_MINUS_23).abs() < f32::EPSILON,
            "Well below -23C should return minus-23 tier"
        );
    }

    #[test]
    fn test_pipe_burst_monotonically_increasing() {
        let temps = [10.0, 0.0, -7.0, -18.0, -23.0, -30.0];
        let mut prev = 0.0f32;
        for &temp in &temps {
            let prob = pipe_burst_probability(temp);
            assert!(
                prob >= prev,
                "Probability should increase as temp drops: at {}C got {} < {}",
                temp,
                prob,
                prev
            );
            prev = prob;
        }
    }

    // -----------------------------------------------------------------------
    // Cold day detection tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_cold_day_absolute() {
        // Below -12C is always cold regardless of seasonal average
        assert!(is_cold_day(-13.0, 0.0));
        assert!(is_cold_day(-20.0, -5.0));
        // At or above -12C, depends on seasonal average
        assert!(!is_cold_day(-12.0, 0.0)); // -12 is not < -12
    }

    #[test]
    fn test_is_cold_day_seasonal_deviation() {
        // 11C below seasonal average of 5C = -6C threshold
        // -7C is below -6C, so it's cold
        assert!(is_cold_day(-7.0, 5.0));
        // -5C is above -6C, so not cold (and above -12C absolute)
        assert!(!is_cold_day(-5.0, 5.0));
    }

    #[test]
    fn test_is_cold_day_warm_season_deviation() {
        // In summer with avg 25C: deviation threshold = 25 - 11 = 14C
        // 13C is below 14C, so it's a cold day (unusual cold spell in summer)
        assert!(is_cold_day(13.0, 25.0));
        // 15C is above 14C, not cold
        assert!(!is_cold_day(15.0, 25.0));
    }

    // -----------------------------------------------------------------------
    // Seasonal average temperature tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_seasonal_average_temperate_winter() {
        let avg = seasonal_average_temp(crate::weather::Season::Winter, ClimateZone::Temperate);
        // Temperate winter: t_min=-8, t_max=6, avg=-1.0
        assert!(
            (avg - (-1.0)).abs() < 0.01,
            "Temperate winter avg should be ~-1.0, got {}",
            avg
        );
    }

    #[test]
    fn test_seasonal_average_subarctic_winter() {
        let avg = seasonal_average_temp(crate::weather::Season::Winter, ClimateZone::Subarctic);
        // Subarctic winter: t_min=-34, t_max=-12, avg=-23.0
        assert!(
            (avg - (-23.0)).abs() < 0.01,
            "Subarctic winter avg should be ~-23.0, got {}",
            avg
        );
    }

    // -----------------------------------------------------------------------
    // Cold snap tier tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_tier_normal() {
        assert_eq!(cold_snap_tier(0, 5.0), ColdSnapTier::Normal);
        assert_eq!(cold_snap_tier(0, -10.0), ColdSnapTier::Normal);
    }

    #[test]
    fn test_tier_watch() {
        assert_eq!(cold_snap_tier(1, -15.0), ColdSnapTier::Watch);
        assert_eq!(cold_snap_tier(2, -20.0), ColdSnapTier::Watch);
    }

    #[test]
    fn test_tier_warning() {
        assert_eq!(cold_snap_tier(3, -15.0), ColdSnapTier::Warning);
        assert_eq!(cold_snap_tier(5, -20.0), ColdSnapTier::Warning);
        assert_eq!(cold_snap_tier(10, -10.0), ColdSnapTier::Warning);
    }

    #[test]
    fn test_tier_emergency() {
        assert_eq!(cold_snap_tier(3, -24.0), ColdSnapTier::Emergency);
        assert_eq!(cold_snap_tier(5, -30.0), ColdSnapTier::Emergency);
        // At exactly -23C, still Warning (threshold is < -23)
        assert_eq!(cold_snap_tier(3, -23.0), ColdSnapTier::Warning);
    }

    #[test]
    fn test_tier_labels() {
        assert_eq!(ColdSnapTier::Normal.label(), "Normal");
        assert_eq!(ColdSnapTier::Watch.label(), "Cold Watch");
        assert_eq!(ColdSnapTier::Warning.label(), "Cold Warning");
        assert_eq!(ColdSnapTier::Emergency.label(), "Cold Emergency");
    }

    // -----------------------------------------------------------------------
    // Heating demand modifier tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_heating_normal_above_zero() {
        let modifier = heating_demand_modifier(ColdSnapTier::Normal, 10.0);
        assert!(
            (modifier - 1.0).abs() < f32::EPSILON,
            "Above zero, normal tier should be 1.0, got {}",
            modifier
        );
    }

    #[test]
    fn test_heating_normal_below_zero() {
        let modifier = heating_demand_modifier(ColdSnapTier::Normal, -5.0);
        // 1.0 + 5 * 0.02 = 1.10
        assert!(
            (modifier - 1.1).abs() < 0.01,
            "Normal tier at -5C should be ~1.1, got {}",
            modifier
        );
    }

    #[test]
    fn test_heating_normal_capped() {
        let modifier = heating_demand_modifier(ColdSnapTier::Normal, -30.0);
        // 1.0 + 30 * 0.02 = 1.6, but capped at 1.3
        assert!(
            (modifier - 1.3).abs() < f32::EPSILON,
            "Normal tier heating cap should be 1.3, got {}",
            modifier
        );
    }

    #[test]
    fn test_heating_warning() {
        let modifier = heating_demand_modifier(ColdSnapTier::Warning, -15.0);
        assert!(
            (modifier - 1.8).abs() < f32::EPSILON,
            "Warning tier should be 1.8, got {}",
            modifier
        );
    }

    #[test]
    fn test_heating_emergency() {
        let modifier = heating_demand_modifier(ColdSnapTier::Emergency, -25.0);
        assert!(
            (modifier - 2.5).abs() < f32::EPSILON,
            "Emergency tier should be 2.5, got {}",
            modifier
        );
    }

    // -----------------------------------------------------------------------
    // Homeless mortality tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_homeless_mortality_above_threshold() {
        assert!(
            homeless_mortality(-17.0).abs() < f32::EPSILON,
            "Above -18C should have zero mortality"
        );
        assert!(
            homeless_mortality(0.0).abs() < f32::EPSILON,
            "Above freezing should have zero mortality"
        );
    }

    #[test]
    fn test_homeless_mortality_at_threshold() {
        assert!(
            homeless_mortality(-18.0).abs() < f32::EPSILON,
            "At exactly -18C should have zero mortality"
        );
    }

    #[test]
    fn test_homeless_mortality_below_threshold() {
        let rate = homeless_mortality(-20.0);
        // 2.0 * exp(0.3 * 2) = 2.0 * exp(0.6) ~ 3.644
        let expected = 2.0 * (0.3_f32 * 2.0).exp();
        assert!(
            (rate - expected).abs() < 0.01,
            "At -20C expected ~{}, got {}",
            expected,
            rate
        );
    }

    #[test]
    fn test_homeless_mortality_exponential_growth() {
        let m20 = homeless_mortality(-20.0);
        let m25 = homeless_mortality(-25.0);
        let m30 = homeless_mortality(-30.0);
        assert!(
            m25 > m20 * 2.0,
            "Mortality should grow fast: -25C={} vs -20C={}",
            m25,
            m20
        );
        assert!(
            m30 > m25 * 2.0,
            "Mortality should grow fast: -30C={} vs -25C={}",
            m30,
            m25
        );
    }

    // -----------------------------------------------------------------------
    // Water main estimation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_estimate_water_main_miles() {
        let miles = estimate_water_main_miles(5000);
        assert!(
            (miles - 15.0).abs() < 0.01,
            "5000 road cells should be ~15 miles, got {}",
            miles
        );
    }

    #[test]
    fn test_estimate_water_main_miles_zero() {
        assert!(estimate_water_main_miles(0).abs() < f32::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Water service modifier tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_water_service_no_bursts() {
        let service = water_service_from_bursts(0, 15.0);
        assert!(
            (service - 1.0).abs() < f32::EPSILON,
            "No bursts should give full service, got {}",
            service
        );
    }

    #[test]
    fn test_water_service_some_bursts() {
        // 10 bursts * 0.5 miles each = 5 miles affected out of 15 total
        // 1.0 - 5/15 = 1.0 - 0.333 = 0.667
        let service = water_service_from_bursts(10, 15.0);
        assert!(
            (service - 0.667).abs() < 0.01,
            "10 bursts on 15 miles should be ~0.667, got {}",
            service
        );
    }

    #[test]
    fn test_water_service_clamped_minimum() {
        // Many bursts should still not go below 0.2
        let service = water_service_from_bursts(1000, 15.0);
        assert!(
            (service - 0.2).abs() < f32::EPSILON,
            "Water service should not go below 0.2, got {}",
            service
        );
    }

    #[test]
    fn test_water_service_zero_miles() {
        let service = water_service_from_bursts(10, 0.0);
        assert!(
            (service - 1.0).abs() < f32::EPSILON,
            "Zero water main miles should give full service, got {}",
            service
        );
    }

    // -----------------------------------------------------------------------
    // Pipe burst calculation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_calculate_pipe_bursts_deterministic() {
        let a = calculate_pipe_bursts(-20.0, 15.0, 42);
        let b = calculate_pipe_bursts(-20.0, 15.0, 42);
        assert_eq!(a, b, "Same seed should produce same result");
    }

    #[test]
    fn test_calculate_pipe_bursts_different_seeds() {
        // With different seeds, results may differ (not guaranteed, but likely for many calls)
        let mut results = std::collections::HashSet::new();
        for seed in 0..100u64 {
            results.insert(calculate_pipe_bursts(-20.0, 15.0, seed));
        }
        // With 100 different seeds at high probability (0.05), we should see variation
        assert!(
            results.len() > 1,
            "Different seeds should produce varying results"
        );
    }

    #[test]
    fn test_calculate_pipe_bursts_above_freezing() {
        // At 10C with baseline probability 0.0001, 15 miles is very unlikely to burst
        let mut total_bursts = 0u32;
        for seed in 0..100u64 {
            total_bursts += calculate_pipe_bursts(10.0, 15.0, seed);
        }
        // 100 runs * 15 segments * 0.0001 probability = ~0.15 expected total
        // Allow up to 5 for statistical variation
        assert!(
            total_bursts < 5,
            "Above freezing should have very few bursts, got {}",
            total_bursts
        );
    }

    // -----------------------------------------------------------------------
    // Effect threshold tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_school_closure_threshold() {
        // Schools close below -29C
        assert!(-30.0 < SCHOOL_CLOSURE_THRESHOLD_C);
        assert!(!(-28.0 < SCHOOL_CLOSURE_THRESHOLD_C));
    }

    #[test]
    fn test_construction_halt_threshold() {
        // Construction halted below -9C
        assert!(-10.0 < CONSTRUCTION_HALT_THRESHOLD_C);
        assert!(!(-8.0 < CONSTRUCTION_HALT_THRESHOLD_C));
    }

    // -----------------------------------------------------------------------
    // ColdSnapState default tests
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Integration tests using Bevy App
    // -----------------------------------------------------------------------

    /// Helper: build a minimal Bevy App with cold snap system.
    fn cold_snap_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<SlowTickTimer>()
            .init_resource::<Weather>()
            .init_resource::<ClimateZone>()
            .init_resource::<ColdSnapState>()
            .add_event::<ColdSnapEvent>()
            .add_systems(Update, update_cold_snap);
        app
    }

    fn advance_with_day(app: &mut App, day: u32) {
        {
            let mut timer = app.world_mut().resource_mut::<SlowTickTimer>();
            // Set counter to a multiple of INTERVAL so should_run() returns true
            timer.counter = SlowTickTimer::INTERVAL;
        }
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.last_update_day = day;
        }
        app.update();
    }

    #[test]
    fn test_system_no_cold_snap_above_threshold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = 5.0;
            weather.season = crate::weather::Season::Winter;
        }
        advance_with_day(&mut app, 1);

        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.current_tier, ColdSnapTier::Normal);
        assert!(!state.is_active);
        assert_eq!(state.consecutive_cold_days, 0);
    }

    #[test]
    fn test_system_cold_snap_activates_after_3_days() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -15.0; // Below -12C absolute threshold
            weather.season = crate::weather::Season::Winter;
        }

        // Day 1: Watch
        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.consecutive_cold_days, 1);
        assert_eq!(state.current_tier, ColdSnapTier::Watch);
        assert!(!state.is_active);

        // Day 2: Still Watch
        advance_with_day(&mut app, 2);
        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.consecutive_cold_days, 2);
        assert_eq!(state.current_tier, ColdSnapTier::Watch);

        // Day 3: Warning (cold snap active)
        advance_with_day(&mut app, 3);
        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.consecutive_cold_days, 3);
        assert_eq!(state.current_tier, ColdSnapTier::Warning);
        assert!(state.is_active);
    }

    #[test]
    fn test_system_emergency_at_extreme_cold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -25.0; // Below -23C for Emergency
            weather.season = crate::weather::Season::Winter;
        }

        for day in 1..=3 {
            advance_with_day(&mut app, day);
        }

        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.current_tier, ColdSnapTier::Emergency);
        assert!(state.is_active);
        assert!((state.heating_demand_modifier - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_system_resets_on_warm_day() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -15.0;
            weather.season = crate::weather::Season::Winter;
        }

        // Build up 3 cold days
        for day in 1..=3 {
            advance_with_day(&mut app, day);
        }
        let state = app.world().resource::<ColdSnapState>();
        assert!(state.is_active);

        // Warm day resets
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = 5.0;
        }
        advance_with_day(&mut app, 4);

        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(state.consecutive_cold_days, 0);
        assert_eq!(state.current_tier, ColdSnapTier::Normal);
        assert!(!state.is_active);
    }

    #[test]
    fn test_system_traffic_capacity_during_cold_snap() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -15.0;
            weather.season = crate::weather::Season::Winter;
        }

        // Build up to active cold snap
        for day in 1..=3 {
            advance_with_day(&mut app, day);
        }

        let state = app.world().resource::<ColdSnapState>();
        assert!(
            (state.traffic_capacity_modifier - 0.8).abs() < f32::EPSILON,
            "Traffic capacity should be 0.8 during cold snap, got {}",
            state.traffic_capacity_modifier
        );
    }

    #[test]
    fn test_system_school_closure() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -30.0; // Below -29C
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(state.schools_closed, "Schools should close below -29C");
    }

    #[test]
    fn test_system_construction_halted() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -10.0; // Below -9C
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(
            state.construction_halted,
            "Construction should halt below -9C"
        );
    }

    #[test]
    fn test_system_construction_not_halted_above_threshold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -8.0; // Above -9C
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(
            !state.construction_halted,
            "Construction should not halt above -9C"
        );
    }

    #[test]
    fn test_system_homeless_mortality_at_extreme_cold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -25.0;
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(
            state.homeless_mortality_rate > 0.0,
            "Homeless mortality should be positive below -18C"
        );
    }

    #[test]
    fn test_system_no_mortality_above_threshold() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -10.0; // Above -18C
            weather.season = crate::weather::Season::Winter;
        }

        advance_with_day(&mut app, 1);
        let state = app.world().resource::<ColdSnapState>();
        assert!(
            state.homeless_mortality_rate.abs() < f32::EPSILON,
            "Homeless mortality should be zero above -18C"
        );
    }

    #[test]
    fn test_system_event_fired_on_tier_change() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -15.0;
            weather.season = crate::weather::Season::Winter;
        }

        // Day 1: Normal -> Watch fires event
        advance_with_day(&mut app, 1);

        let events = app.world().resource::<Events<ColdSnapEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();
        assert!(
            !fired.is_empty(),
            "ColdSnapEvent should fire on tier change"
        );
        assert_eq!(fired[0].tier, ColdSnapTier::Watch);
    }

    #[test]
    fn test_system_skips_when_timer_not_ready() {
        let mut app = cold_snap_test_app();
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.temperature = -20.0;
            weather.season = crate::weather::Season::Winter;
            weather.last_update_day = 1;
        }
        // Don't set timer to interval - it starts at 0 which is a multiple of 100
        // but SlowTickTimer::should_run checks counter.is_multiple_of(100)
        // 0.is_multiple_of(100) is true in Rust, so set to non-multiple
        {
            let mut timer = app.world_mut().resource_mut::<SlowTickTimer>();
            timer.counter = 1; // Not a multiple of 100
        }
        app.update();

        let state = app.world().resource::<ColdSnapState>();
        assert_eq!(
            state.consecutive_cold_days, 0,
            "Should not update when timer not ready"
        );
    }

    // -----------------------------------------------------------------------
    // Deterministic PRNG tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_splitmix64_deterministic() {
        let a = splitmix64(42);
        let b = splitmix64(42);
        assert_eq!(a, b);
        assert_ne!(splitmix64(42), splitmix64(43));
    }

    #[test]
    fn test_rand_f32_range() {
        for seed in 0..1000u64 {
            let val = rand_f32(seed);
            assert!(
                (0.0..1.0).contains(&val),
                "rand_f32({}) = {} out of range",
                seed,
                val
            );
        }
    }
}
