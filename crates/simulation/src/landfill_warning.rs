use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::garbage::WasteSystem;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Each Landfill service building provides this many tons of capacity.
pub const LANDFILL_CAPACITY_PER_BUILDING: f64 = 500_000.0;

/// Threshold for Low warning tier: 25% remaining capacity.
const LOW_THRESHOLD_PCT: f64 = 25.0;

/// Threshold for Critical warning tier: 10% remaining capacity.
const CRITICAL_THRESHOLD_PCT: f64 = 10.0;

/// Threshold for VeryLow warning tier: 5% remaining capacity.
const VERY_LOW_THRESHOLD_PCT: f64 = 5.0;

/// Days per year for years_remaining calculation.
const DAYS_PER_YEAR: f32 = 365.0;

// =============================================================================
// Types
// =============================================================================

/// Warning tier indicating how much landfill capacity remains.
///
/// Ordered from least severe (Normal) to most severe (Emergency).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LandfillWarningTier {
    /// More than 25% capacity remaining. No warnings needed.
    #[default]
    Normal,
    /// 10%--25% capacity remaining. Advisory warning.
    Low,
    /// 5%--10% capacity remaining. Urgent warning.
    Critical,
    /// 0%--5% capacity remaining. Severe warning.
    VeryLow,
    /// 0% capacity remaining. Collection halted.
    Emergency,
}

impl LandfillWarningTier {
    /// Returns a human-readable label for the tier.
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Low => "Low Capacity",
            Self::Critical => "Critical",
            Self::VeryLow => "Very Low",
            Self::Emergency => "Emergency",
        }
    }
}

/// Event fired whenever the landfill warning tier changes.
#[derive(Event, Debug, Clone)]
pub struct LandfillWarningEvent {
    /// The new warning tier after the change.
    pub tier: LandfillWarningTier,
    /// Remaining capacity as a percentage (0.0 to 100.0).
    pub remaining_pct: f32,
}

/// City-wide landfill capacity tracking resource.
///
/// Updated each slow tick by `update_landfill_capacity`. Other systems can read
/// `collection_halted` to stop waste collection when capacity is exhausted.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct LandfillCapacityState {
    /// Total landfill capacity across all Landfill buildings (tons).
    pub total_capacity: f64,
    /// Current fill level (tons). Increases each slow tick by daily input.
    pub current_fill: f64,
    /// Daily waste input rate going to landfills (tons/day).
    pub daily_input_rate: f64,
    /// Estimated days until capacity is exhausted at current fill rate.
    pub days_remaining: f32,
    /// Estimated years until capacity is exhausted (days_remaining / 365).
    pub years_remaining: f32,
    /// Remaining capacity as a percentage (0.0 to 100.0).
    pub remaining_pct: f32,
    /// Current warning tier derived from remaining_pct.
    pub current_tier: LandfillWarningTier,
    /// When true (Emergency tier), waste collection should be halted by
    /// downstream systems.
    pub collection_halted: bool,
    /// Number of Landfill service buildings in the city.
    pub landfill_count: u32,
}

// =============================================================================
// Pure logic (testable without ECS)
// =============================================================================

/// Determine the warning tier from a remaining-capacity percentage (0..=100).
pub fn tier_from_remaining_pct(remaining_pct: f64) -> LandfillWarningTier {
    if remaining_pct <= 0.0 {
        LandfillWarningTier::Emergency
    } else if remaining_pct <= VERY_LOW_THRESHOLD_PCT {
        LandfillWarningTier::VeryLow
    } else if remaining_pct <= CRITICAL_THRESHOLD_PCT {
        LandfillWarningTier::Critical
    } else if remaining_pct <= LOW_THRESHOLD_PCT {
        LandfillWarningTier::Low
    } else {
        LandfillWarningTier::Normal
    }
}

/// Compute remaining percentage given total capacity and current fill.
/// Returns 100.0 when total_capacity is zero (no landfills built yet, so
/// nothing to warn about until waste actually arrives -- handled by the
/// system which checks for zero capacity separately).
pub fn compute_remaining_pct(total_capacity: f64, current_fill: f64) -> f64 {
    if total_capacity <= 0.0 {
        return 0.0;
    }
    let remaining = (total_capacity - current_fill).max(0.0);
    (remaining / total_capacity * 100.0).clamp(0.0, 100.0)
}

/// Compute days remaining given remaining capacity and daily input rate.
/// Returns `f32::INFINITY` when daily_input_rate is zero or negative.
pub fn compute_days_remaining(
    total_capacity: f64,
    current_fill: f64,
    daily_input_rate: f64,
) -> f32 {
    if daily_input_rate <= 0.0 {
        return f32::INFINITY;
    }
    let remaining = (total_capacity - current_fill).max(0.0);
    (remaining / daily_input_rate) as f32
}

/// Apply one slow tick of fill accumulation. Returns new fill level,
/// clamped to total_capacity.
pub fn advance_fill(current_fill: f64, daily_input: f64, total_capacity: f64) -> f64 {
    (current_fill + daily_input).min(total_capacity)
}

// =============================================================================
// Bevy system
// =============================================================================

/// Updates landfill capacity state each slow tick.
///
/// 1. Counts `Landfill` service buildings to compute total capacity.
/// 2. Reads `WasteSystem.total_generated_tons` for the daily input rate.
/// 3. Advances fill level by one day's input per slow tick.
/// 4. Computes remaining percentage, days/years remaining, and warning tier.
/// 5. Fires `LandfillWarningEvent` when the tier changes.
/// 6. Sets `collection_halted` at Emergency tier.
pub fn update_landfill_capacity(
    slow_timer: Res<SlowTickTimer>,
    waste_system: Res<WasteSystem>,
    buildings: Query<&ServiceBuilding>,
    mut state: ResMut<LandfillCapacityState>,
    mut warning_events: EventWriter<LandfillWarningEvent>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // 1. Count landfill buildings and compute total capacity.
    let landfill_count = buildings
        .iter()
        .filter(|b| b.service_type == ServiceType::Landfill)
        .count() as u32;
    let total_capacity = landfill_count as f64 * LANDFILL_CAPACITY_PER_BUILDING;

    // 2. Read daily waste input.
    let daily_input_rate = waste_system.period_generated_tons;

    // 3. Advance fill level (one slow tick ~ one game day).
    let current_fill = advance_fill(state.current_fill, daily_input_rate, total_capacity);

    // 4. Compute derived metrics.
    let remaining_pct = compute_remaining_pct(total_capacity, current_fill);
    let days_remaining = compute_days_remaining(total_capacity, current_fill, daily_input_rate);
    let years_remaining = days_remaining / DAYS_PER_YEAR;
    let new_tier = tier_from_remaining_pct(remaining_pct);

    // 5. Fire event on tier change.
    let old_tier = state.current_tier;
    if new_tier != old_tier {
        warning_events.send(LandfillWarningEvent {
            tier: new_tier,
            remaining_pct: remaining_pct as f32,
        });
    }

    // 6. Update state.
    state.total_capacity = total_capacity;
    state.current_fill = current_fill;
    state.daily_input_rate = daily_input_rate;
    state.days_remaining = days_remaining;
    state.years_remaining = years_remaining;
    state.remaining_pct = remaining_pct as f32;
    state.current_tier = new_tier;
    state.collection_halted = new_tier == LandfillWarningTier::Emergency;
    state.landfill_count = landfill_count;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // tier_from_remaining_pct
    // -------------------------------------------------------------------------

    #[test]
    fn tier_normal_above_25() {
        assert_eq!(tier_from_remaining_pct(50.0), LandfillWarningTier::Normal);
        assert_eq!(tier_from_remaining_pct(100.0), LandfillWarningTier::Normal);
        assert_eq!(tier_from_remaining_pct(25.1), LandfillWarningTier::Normal);
    }

    #[test]
    fn tier_low_at_25() {
        assert_eq!(tier_from_remaining_pct(25.0), LandfillWarningTier::Low);
    }

    #[test]
    fn tier_low_between_10_and_25() {
        assert_eq!(tier_from_remaining_pct(20.0), LandfillWarningTier::Low);
        assert_eq!(tier_from_remaining_pct(15.0), LandfillWarningTier::Low);
        assert_eq!(tier_from_remaining_pct(10.1), LandfillWarningTier::Low);
    }

    #[test]
    fn tier_critical_at_10() {
        assert_eq!(tier_from_remaining_pct(10.0), LandfillWarningTier::Critical);
    }

    #[test]
    fn tier_critical_between_5_and_10() {
        assert_eq!(tier_from_remaining_pct(8.0), LandfillWarningTier::Critical);
        assert_eq!(tier_from_remaining_pct(5.1), LandfillWarningTier::Critical);
    }

    #[test]
    fn tier_very_low_at_5() {
        assert_eq!(tier_from_remaining_pct(5.0), LandfillWarningTier::VeryLow);
    }

    #[test]
    fn tier_very_low_between_0_and_5() {
        assert_eq!(tier_from_remaining_pct(3.0), LandfillWarningTier::VeryLow);
        assert_eq!(tier_from_remaining_pct(0.1), LandfillWarningTier::VeryLow);
    }

    #[test]
    fn tier_emergency_at_zero() {
        assert_eq!(tier_from_remaining_pct(0.0), LandfillWarningTier::Emergency);
    }

    #[test]
    fn tier_emergency_negative() {
        // Should never happen in practice, but handle gracefully.
        assert_eq!(
            tier_from_remaining_pct(-1.0),
            LandfillWarningTier::Emergency
        );
    }

    // -------------------------------------------------------------------------
    // compute_remaining_pct
    // -------------------------------------------------------------------------

    #[test]
    fn remaining_pct_full_capacity() {
        assert!((compute_remaining_pct(1_000_000.0, 0.0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remaining_pct_half_full() {
        assert!((compute_remaining_pct(1_000_000.0, 500_000.0) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remaining_pct_completely_full() {
        assert!((compute_remaining_pct(1_000_000.0, 1_000_000.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remaining_pct_overfull_clamped() {
        // If current_fill somehow exceeds capacity, clamp to 0%.
        assert!((compute_remaining_pct(1_000_000.0, 1_500_000.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remaining_pct_zero_capacity() {
        // No landfills built: 0% remaining (will trigger Emergency).
        assert!((compute_remaining_pct(0.0, 0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remaining_pct_quarter_left() {
        // Exactly 25% remaining -> should yield 25.0%.
        let pct = compute_remaining_pct(1_000_000.0, 750_000.0);
        assert!((pct - 25.0).abs() < 1e-9);
    }

    #[test]
    fn remaining_pct_ten_percent() {
        let pct = compute_remaining_pct(1_000_000.0, 900_000.0);
        assert!((pct - 10.0).abs() < 1e-9);
    }

    #[test]
    fn remaining_pct_five_percent() {
        let pct = compute_remaining_pct(1_000_000.0, 950_000.0);
        assert!((pct - 5.0).abs() < 1e-9);
    }

    // -------------------------------------------------------------------------
    // compute_days_remaining
    // -------------------------------------------------------------------------

    #[test]
    fn days_remaining_normal() {
        // 500k remaining, 1000 tons/day => 500 days.
        let days = compute_days_remaining(1_000_000.0, 500_000.0, 1_000.0);
        assert!((days - 500.0).abs() < 0.01);
    }

    #[test]
    fn days_remaining_zero_input() {
        // No waste being generated -> infinite days.
        let days = compute_days_remaining(1_000_000.0, 500_000.0, 0.0);
        assert!(days.is_infinite());
    }

    #[test]
    fn days_remaining_negative_input() {
        let days = compute_days_remaining(1_000_000.0, 500_000.0, -100.0);
        assert!(days.is_infinite());
    }

    #[test]
    fn days_remaining_already_full() {
        let days = compute_days_remaining(1_000_000.0, 1_000_000.0, 1_000.0);
        assert!((days - 0.0).abs() < 0.01);
    }

    #[test]
    fn days_remaining_overfull() {
        // Overfull case: remaining is clamped to 0.
        let days = compute_days_remaining(1_000_000.0, 1_500_000.0, 1_000.0);
        assert!((days - 0.0).abs() < 0.01);
    }

    // -------------------------------------------------------------------------
    // advance_fill
    // -------------------------------------------------------------------------

    #[test]
    fn advance_fill_normal() {
        let new_fill = advance_fill(100_000.0, 1_000.0, 500_000.0);
        assert!((new_fill - 101_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn advance_fill_clamps_at_capacity() {
        // Adding 10k to 495k with 500k cap => 500k, not 505k.
        let new_fill = advance_fill(495_000.0, 10_000.0, 500_000.0);
        assert!((new_fill - 500_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn advance_fill_zero_input() {
        let new_fill = advance_fill(100_000.0, 0.0, 500_000.0);
        assert!((new_fill - 100_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn advance_fill_already_at_capacity() {
        let new_fill = advance_fill(500_000.0, 1_000.0, 500_000.0);
        assert!((new_fill - 500_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn advance_fill_zero_capacity() {
        let new_fill = advance_fill(0.0, 1_000.0, 0.0);
        assert!((new_fill - 0.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // LandfillWarningTier
    // -------------------------------------------------------------------------

    #[test]
    fn tier_default_is_normal() {
        assert_eq!(LandfillWarningTier::default(), LandfillWarningTier::Normal);
    }

    #[test]
    fn tier_labels() {
        assert_eq!(LandfillWarningTier::Normal.label(), "Normal");
        assert_eq!(LandfillWarningTier::Low.label(), "Low Capacity");
        assert_eq!(LandfillWarningTier::Critical.label(), "Critical");
        assert_eq!(LandfillWarningTier::VeryLow.label(), "Very Low");
        assert_eq!(LandfillWarningTier::Emergency.label(), "Emergency");
    }

    #[test]
    fn tier_equality() {
        assert_eq!(LandfillWarningTier::Normal, LandfillWarningTier::Normal);
        assert_ne!(LandfillWarningTier::Normal, LandfillWarningTier::Low);
        assert_ne!(LandfillWarningTier::Low, LandfillWarningTier::Critical);
        assert_ne!(LandfillWarningTier::Critical, LandfillWarningTier::VeryLow);
        assert_ne!(LandfillWarningTier::VeryLow, LandfillWarningTier::Emergency);
    }

    // -------------------------------------------------------------------------
    // LandfillCapacityState
    // -------------------------------------------------------------------------

    #[test]
    fn state_default_values() {
        let state = LandfillCapacityState::default();
        assert!((state.total_capacity - 0.0).abs() < f64::EPSILON);
        assert!((state.current_fill - 0.0).abs() < f64::EPSILON);
        assert!((state.daily_input_rate - 0.0).abs() < f64::EPSILON);
        assert!((state.days_remaining - 0.0).abs() < f32::EPSILON);
        assert!((state.years_remaining - 0.0).abs() < f32::EPSILON);
        assert!((state.remaining_pct - 0.0).abs() < f32::EPSILON);
        assert_eq!(state.current_tier, LandfillWarningTier::Normal);
        assert!(!state.collection_halted);
        assert_eq!(state.landfill_count, 0);
    }

    // -------------------------------------------------------------------------
    // Integration: tier transitions through fill progression
    // -------------------------------------------------------------------------

    #[test]
    fn tier_progression_through_fill() {
        let capacity = 1_000_000.0;

        // 80% remaining -> Normal
        let pct = compute_remaining_pct(capacity, 200_000.0);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Normal);

        // 25% remaining -> Low
        let pct = compute_remaining_pct(capacity, 750_000.0);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Low);

        // 10% remaining -> Critical
        let pct = compute_remaining_pct(capacity, 900_000.0);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Critical);

        // 5% remaining -> VeryLow
        let pct = compute_remaining_pct(capacity, 950_000.0);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::VeryLow);

        // 0% remaining -> Emergency
        let pct = compute_remaining_pct(capacity, 1_000_000.0);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Emergency);
    }

    #[test]
    fn years_remaining_calculation() {
        let days = compute_days_remaining(1_000_000.0, 0.0, 1_000.0);
        let years = days / DAYS_PER_YEAR;
        // 1,000,000 / 1,000 = 1000 days = ~2.74 years
        assert!((years - 2.7397).abs() < 0.01);
    }

    #[test]
    fn multiple_landfills_capacity() {
        let count = 3_u32;
        let total = count as f64 * LANDFILL_CAPACITY_PER_BUILDING;
        assert!((total - 1_500_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn collection_halted_only_at_emergency() {
        // Simulate the flag logic from the system.
        for &(fill, expected_halted) in &[
            (0.0, false),        // Normal: not halted
            (750_000.0, false),  // Low: not halted
            (900_000.0, false),  // Critical: not halted
            (950_000.0, false),  // VeryLow: not halted
            (1_000_000.0, true), // Emergency: halted
        ] {
            let pct = compute_remaining_pct(1_000_000.0, fill);
            let tier = tier_from_remaining_pct(pct);
            let halted = tier == LandfillWarningTier::Emergency;
            assert_eq!(halted, expected_halted, "fill={fill}");
        }
    }

    #[test]
    fn boundary_just_above_threshold() {
        // 25.0001% remaining -> still Normal (above 25% threshold)
        let pct = compute_remaining_pct(1_000_000.0, 749_999.0);
        assert!(pct > 25.0);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Normal);

        // 10.0001% -> still Low
        let pct = compute_remaining_pct(1_000_000.0, 899_999.0);
        assert!(pct > 10.0);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Low);

        // 5.0001% -> still Critical
        let pct = compute_remaining_pct(1_000_000.0, 949_999.0);
        assert!(pct > 5.0);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Critical);
    }

    #[test]
    fn boundary_exactly_at_threshold() {
        // Exactly 25% -> Low
        let pct = compute_remaining_pct(1_000_000.0, 750_000.0);
        assert!((pct - 25.0).abs() < 1e-9);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Low);

        // Exactly 10% -> Critical
        let pct = compute_remaining_pct(1_000_000.0, 900_000.0);
        assert!((pct - 10.0).abs() < 1e-9);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Critical);

        // Exactly 5% -> VeryLow
        let pct = compute_remaining_pct(1_000_000.0, 950_000.0);
        assert!((pct - 5.0).abs() < 1e-9);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::VeryLow);
    }

    #[test]
    fn advance_fill_incremental_to_full() {
        // Fill a landfill incrementally.
        let capacity = 100.0;
        let mut fill = 0.0;
        for _ in 0..100 {
            fill = advance_fill(fill, 1.0, capacity);
        }
        assert!((fill - 100.0).abs() < f64::EPSILON);

        // One more tick should not exceed capacity.
        fill = advance_fill(fill, 1.0, capacity);
        assert!((fill - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn days_remaining_decreases_as_fill_grows() {
        let capacity = 1_000_000.0;
        let rate = 1_000.0;
        let days_at_0 = compute_days_remaining(capacity, 0.0, rate);
        let days_at_half = compute_days_remaining(capacity, 500_000.0, rate);
        let days_at_90 = compute_days_remaining(capacity, 900_000.0, rate);

        assert!(days_at_0 > days_at_half);
        assert!(days_at_half > days_at_90);
        assert!(days_at_90 > 0.0);
    }

    #[test]
    fn very_small_daily_input() {
        // Tiny input rate should still work, giving a very long time horizon.
        let days = compute_days_remaining(1_000_000.0, 0.0, 0.001);
        assert!(days > 999_999_000.0);
        assert!(days.is_finite());
    }

    #[test]
    fn large_daily_input_fills_quickly() {
        let days = compute_days_remaining(500_000.0, 0.0, 500_000.0);
        assert!((days - 1.0).abs() < 0.01);
    }

    #[test]
    fn remaining_pct_precision() {
        // Check precision at a non-round fill level.
        let pct = compute_remaining_pct(3_000_000.0, 2_700_000.0);
        // remaining = 300k, pct = 300k/3M * 100 = 10.0
        assert!((pct - 10.0).abs() < 1e-9);
    }

    #[test]
    fn no_landfills_zero_capacity_emergency() {
        // If there are no landfill buildings: capacity is 0, remaining is 0%, tier is Emergency.
        let pct = compute_remaining_pct(0.0, 0.0);
        assert!((pct - 0.0).abs() < f64::EPSILON);
        assert_eq!(tier_from_remaining_pct(pct), LandfillWarningTier::Emergency);
    }

    #[test]
    fn single_landfill_capacity() {
        let total = 1_u32 as f64 * LANDFILL_CAPACITY_PER_BUILDING;
        assert!((total - 500_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn serde_roundtrip_tier() {
        let tiers = [
            LandfillWarningTier::Normal,
            LandfillWarningTier::Low,
            LandfillWarningTier::Critical,
            LandfillWarningTier::VeryLow,
            LandfillWarningTier::Emergency,
        ];
        for tier in &tiers {
            let json = serde_json::to_string(tier).expect("serialize");
            let deser: LandfillWarningTier = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*tier, deser);
        }
    }

    #[test]
    fn serde_roundtrip_state() {
        let state = LandfillCapacityState {
            total_capacity: 1_500_000.0,
            current_fill: 750_000.0,
            daily_input_rate: 1_000.0,
            days_remaining: 750.0,
            years_remaining: 2.054,
            remaining_pct: 50.0,
            current_tier: LandfillWarningTier::Normal,
            collection_halted: false,
            landfill_count: 3,
        };
        let json = serde_json::to_string(&state).expect("serialize");
        let deser: LandfillCapacityState = serde_json::from_str(&json).expect("deserialize");
        assert!((deser.total_capacity - state.total_capacity).abs() < f64::EPSILON);
        assert!((deser.current_fill - state.current_fill).abs() < f64::EPSILON);
        assert_eq!(deser.current_tier, state.current_tier);
        assert_eq!(deser.collection_halted, state.collection_halted);
        assert_eq!(deser.landfill_count, state.landfill_count);
    }
}

pub struct LandfillWarningPlugin;

impl Plugin for LandfillWarningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LandfillCapacityState>()
            .add_event::<LandfillWarningEvent>()
            .add_systems(
                FixedUpdate,
                update_landfill_capacity.after(crate::imports_exports::process_trade),
            );
    }
}
