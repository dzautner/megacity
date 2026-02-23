/// Bitflags for service coverage packed into a single byte per cell.
pub const COVERAGE_HEALTH: u8 = 0b0000_0001;
pub const COVERAGE_EDUCATION: u8 = 0b0000_0010;
pub const COVERAGE_POLICE: u8 = 0b0000_0100;
pub const COVERAGE_PARK: u8 = 0b0000_1000;
pub const COVERAGE_ENTERTAINMENT: u8 = 0b0001_0000;
pub const COVERAGE_TELECOM: u8 = 0b0010_0000;
pub const COVERAGE_TRANSPORT: u8 = 0b0100_0000;
pub const COVERAGE_FIRE: u8 = 0b1000_0000;

// ---------------------------------------------------------------------------
// Update interval (ticks between happiness recalculations)
// ---------------------------------------------------------------------------
/// How often (in ticks) the happiness system recalculates. Reduced from the
/// original value for faster responsiveness.
pub const HAPPINESS_UPDATE_INTERVAL: u64 = 20;

// ---------------------------------------------------------------------------
// Base & linear bonuses / penalties
// ---------------------------------------------------------------------------
/// Base happiness before any factors. Reduced from 50 to accommodate new
/// wealth satisfaction and weather factors while keeping the overall range
/// balanced.
pub const BASE_HAPPINESS: f32 = 42.0;
pub const EMPLOYED_BONUS: f32 = 15.0;
pub const SHORT_COMMUTE_BONUS: f32 = 10.0;
pub const POWER_BONUS: f32 = 5.0;
pub const NO_POWER_PENALTY: f32 = 25.0;
pub const WATER_BONUS: f32 = 5.0;
pub const NO_WATER_PENALTY: f32 = 20.0;
pub const HEALTH_COVERAGE_BONUS: f32 = 5.0;
pub const EDUCATION_BONUS: f32 = 3.0;
pub const POLICE_BONUS: f32 = 5.0;
pub const PARK_BONUS: f32 = 8.0;
pub const ENTERTAINMENT_BONUS: f32 = 5.0;
pub const HIGH_TAX_PENALTY: f32 = 8.0;
pub const CONGESTION_PENALTY: f32 = 5.0;
pub const GARBAGE_PENALTY: f32 = 5.0;
pub const CRIME_PENALTY_MAX: f32 = 15.0;
pub const TELECOM_BONUS: f32 = 3.0;
pub const TRANSPORT_BONUS: f32 = 4.0;
pub const POOR_ROAD_PENALTY: f32 = 3.0;

/// Happiness penalty for homeless citizens (unsheltered).
pub const HOMELESS_PENALTY: f32 = 30.0;
/// Reduced happiness penalty for sheltered homeless citizens.
pub const SHELTERED_PENALTY: f32 = 10.0;

// ---------------------------------------------------------------------------
// Critical thresholds — below these, a severe penalty applies
// ---------------------------------------------------------------------------
/// No water at home: severe happiness penalty (on top of the normal NO_WATER_PENALTY).
pub const CRITICAL_NO_WATER_PENALTY: f32 = 25.0;
/// No power at home: severe happiness penalty (on top of the normal NO_POWER_PENALTY).
pub const CRITICAL_NO_POWER_PENALTY: f32 = 15.0;
/// Very low health (< 30): severe happiness penalty.
pub const CRITICAL_HEALTH_THRESHOLD: f32 = 30.0;
pub const CRITICAL_HEALTH_PENALTY: f32 = 20.0;
/// Very low needs satisfaction (< 0.2): severe happiness penalty.
pub const CRITICAL_NEEDS_THRESHOLD: f32 = 0.2;
pub const CRITICAL_NEEDS_PENALTY: f32 = 15.0;
/// Very high crime (> 200 out of 255): severe happiness penalty.
pub const CRITICAL_CRIME_THRESHOLD: f32 = 200.0;
pub const CRITICAL_CRIME_PENALTY: f32 = 10.0;

// ---------------------------------------------------------------------------
// Diminishing returns curve parameters
// ---------------------------------------------------------------------------
/// Steepness parameter `k` for the diminishing returns function
/// `f(x) = 1 - exp(-k * x)`. Higher k = faster saturation.
pub const DIMINISHING_K_DEFAULT: f32 = 3.0;
/// Steepness for pollution/crime factors (negative impacts also diminish).
pub const DIMINISHING_K_NEGATIVE: f32 = 2.5;

// ---------------------------------------------------------------------------
// Weather happiness factor weights
// ---------------------------------------------------------------------------
/// Maximum weather happiness bonus (sunny spring day).
pub const WEATHER_HAPPINESS_MAX_BONUS: f32 = 5.0;
/// Maximum weather happiness penalty (storm in winter).
pub const WEATHER_HAPPINESS_MAX_PENALTY: f32 = 10.0;

// ---------------------------------------------------------------------------
// Wealth satisfaction factor
// ---------------------------------------------------------------------------
/// Maximum happiness bonus from wealth/savings satisfaction.
pub const WEALTH_SATISFACTION_MAX_BONUS: f32 = 8.0;
/// Savings threshold for "comfortable" wealth (per citizen).
pub const WEALTH_COMFORTABLE_SAVINGS: f32 = 10_000.0;
/// Penalty when savings are zero or negative.
pub const WEALTH_POVERTY_PENALTY: f32 = 5.0;

// ---------------------------------------------------------------------------
// Diminishing returns helper
// ---------------------------------------------------------------------------

/// Apply a diminishing returns curve: `f(x) = 1 - exp(-k * x)`.
///
/// - `x` should be in `[0.0, 1.0]` representing the normalized coverage/factor level.
/// - Returns a value in `[0.0, ~1.0)` where the first increments contribute the most.
///
/// This ensures that going from 0% to 25% coverage is far more impactful than
/// going from 75% to 100%.
#[inline]
pub fn diminishing_returns(x: f32, k: f32) -> f32 {
    1.0 - (-k * x.clamp(0.0, 1.0)).exp()
}

/// Count how many service coverage flags are set for a cell and return
/// a normalized ratio (0.0 to 1.0). Used for applying diminishing returns
/// to aggregate service coverage.
#[inline]
pub fn service_coverage_ratio(coverage_flags: u8) -> f32 {
    let count = coverage_flags.count_ones();
    // 8 possible service types
    count as f32 / 8.0
}

/// Wealth satisfaction factor: returns a value in `[-WEALTH_POVERTY_PENALTY, +WEALTH_SATISFACTION_MAX_BONUS]`.
/// Uses diminishing returns so the first few thousand in savings matter most.
#[inline]
pub fn wealth_satisfaction(savings: f32) -> f32 {
    if savings <= 0.0 {
        -WEALTH_POVERTY_PENALTY
    } else {
        let ratio = (savings / WEALTH_COMFORTABLE_SAVINGS).clamp(0.0, 1.0);
        diminishing_returns(ratio, DIMINISHING_K_DEFAULT) * WEALTH_SATISFACTION_MAX_BONUS
    }
}

/// Enhanced weather happiness factor with diminishing returns on extreme weather.
/// Takes the raw weather modifier and applies bounds + scaling.
#[inline]
pub fn weather_happiness_factor(raw_modifier: f32) -> f32 {
    if raw_modifier >= 0.0 {
        let ratio = (raw_modifier / 5.0).clamp(0.0, 1.0);
        diminishing_returns(ratio, DIMINISHING_K_DEFAULT) * WEATHER_HAPPINESS_MAX_BONUS
    } else {
        let ratio = (-raw_modifier / 10.0).clamp(0.0, 1.0);
        -diminishing_returns(ratio, DIMINISHING_K_NEGATIVE) * WEATHER_HAPPINESS_MAX_PENALTY
    }
}
