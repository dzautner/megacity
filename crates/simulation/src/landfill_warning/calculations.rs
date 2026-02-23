use super::types::LandfillWarningTier;

// =============================================================================
// Constants
// =============================================================================

/// Each Landfill service building provides this many tons of capacity.
pub const LANDFILL_CAPACITY_PER_BUILDING: f64 = 500_000.0;

/// Threshold for Low warning tier: 25% remaining capacity.
pub(crate) const LOW_THRESHOLD_PCT: f64 = 25.0;

/// Threshold for Critical warning tier: 10% remaining capacity.
pub(crate) const CRITICAL_THRESHOLD_PCT: f64 = 10.0;

/// Threshold for VeryLow warning tier: 5% remaining capacity.
pub(crate) const VERY_LOW_THRESHOLD_PCT: f64 = 5.0;

/// Days per year for years_remaining calculation.
pub(crate) const DAYS_PER_YEAR: f32 = 365.0;

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
/// Returns 0.0 when total_capacity is zero (no landfills built yet).
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
