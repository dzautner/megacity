use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Wind Damage Tiers (Beaufort-inspired)
// =============================================================================

/// Beaufort-inspired wind damage classification based on normalized wind speed [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WindDamageTier {
    /// Speed 0.0 - 0.15: No damage.
    #[default]
    Calm,
    /// Speed 0.15 - 0.3: No damage, light wind.
    Breezy,
    /// Speed 0.3 - 0.45: Minor effects, no structural damage.
    Strong,
    /// Speed 0.45 - 0.6: Light structural risk begins.
    Gale,
    /// Speed 0.6 - 0.75: Moderate damage, power lines at risk.
    Storm,
    /// Speed 0.75 - 0.9: Significant damage, trees knocked down.
    Severe,
    /// Speed 0.9 - 0.95: Extreme damage to structures.
    HurricaneForce,
    /// Speed > 0.95: Catastrophic damage.
    Extreme,
}

impl WindDamageTier {
    /// Classify a normalized wind speed [0, 1] into a damage tier.
    pub fn from_speed(speed: f32) -> Self {
        if speed < 0.15 {
            WindDamageTier::Calm
        } else if speed < 0.3 {
            WindDamageTier::Breezy
        } else if speed < 0.45 {
            WindDamageTier::Strong
        } else if speed < 0.6 {
            WindDamageTier::Gale
        } else if speed < 0.75 {
            WindDamageTier::Storm
        } else if speed < 0.9 {
            WindDamageTier::Severe
        } else if speed < 0.95 {
            WindDamageTier::HurricaneForce
        } else {
            WindDamageTier::Extreme
        }
    }

    /// Human-readable label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            WindDamageTier::Calm => "Calm",
            WindDamageTier::Breezy => "Breezy",
            WindDamageTier::Strong => "Strong",
            WindDamageTier::Gale => "Gale",
            WindDamageTier::Storm => "Storm",
            WindDamageTier::Severe => "Severe",
            WindDamageTier::HurricaneForce => "Hurricane Force",
            WindDamageTier::Extreme => "Extreme",
        }
    }
}

// =============================================================================
// Damage formulas
// =============================================================================

/// Wind damage threshold: damage begins above this normalized speed.
pub(crate) const WIND_DAMAGE_THRESHOLD: f32 = 0.4;

/// Power outage threshold: outage probability begins above this speed.
const POWER_OUTAGE_THRESHOLD: f32 = 0.6;

/// Tree knockdown threshold: tree damage begins above this speed.
const TREE_KNOCKDOWN_THRESHOLD: f32 = 0.6;

/// Calculate wind damage amount using cubic formula.
/// Returns 0.0 for speeds <= 0.4, otherwise `(speed - 0.4)^3 * 1000`.
pub fn wind_damage_amount(speed: f32) -> f32 {
    if speed <= WIND_DAMAGE_THRESHOLD {
        return 0.0;
    }
    let excess = speed - WIND_DAMAGE_THRESHOLD;
    excess * excess * excess * 1000.0
}

/// Calculate power outage probability based on wind speed.
/// Returns 0.0 for speeds <= 0.6, scaling up to ~1.0 at extreme speeds.
/// Formula: `((speed - 0.6) / 0.4)^2` clamped to [0, 1].
pub fn power_outage_probability(speed: f32) -> f32 {
    if speed <= POWER_OUTAGE_THRESHOLD {
        return 0.0;
    }
    let factor = (speed - POWER_OUTAGE_THRESHOLD) / 0.4;
    (factor * factor).min(1.0)
}

/// Calculate tree knockdown probability based on wind speed.
/// Returns 0.0 for speeds <= 0.6, scaling up for higher speeds.
/// Formula: `((speed - 0.6) / 0.4)^2 * 0.1` per tree per update tick.
pub fn tree_knockdown_probability(speed: f32) -> f32 {
    if speed <= TREE_KNOCKDOWN_THRESHOLD {
        return 0.0;
    }
    let factor = (speed - TREE_KNOCKDOWN_THRESHOLD) / 0.4;
    (factor * factor * 0.1).min(1.0)
}

// =============================================================================
// Wind Damage State (resource)
// =============================================================================

/// Resource tracking accumulated wind damage during a storm.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct WindDamageState {
    /// Current wind damage tier classification.
    #[serde(default)]
    pub current_tier: WindDamageTier,
    /// Accumulated building damage this storm (cumulative damage points).
    #[serde(default)]
    pub accumulated_building_damage: f32,
    /// Number of trees knocked down during this storm.
    #[serde(default)]
    pub trees_knocked_down: u32,
    /// Whether a power outage is currently active due to wind.
    #[serde(default)]
    pub power_outage_active: bool,
}

impl Default for WindDamageState {
    fn default() -> Self {
        Self {
            current_tier: WindDamageTier::Calm,
            accumulated_building_damage: 0.0,
            trees_knocked_down: 0,
            power_outage_active: false,
        }
    }
}

// =============================================================================
// Wind Damage Event
// =============================================================================

/// Event fired when wind damage occurs, for notification to other systems.
#[derive(Event, Debug, Clone)]
pub struct WindDamageEvent {
    /// The damage tier that triggered this event.
    pub tier: WindDamageTier,
    /// Amount of building damage dealt this tick.
    pub building_damage: f32,
    /// Number of trees knocked down this tick.
    pub trees_knocked: u32,
    /// Whether power outage was triggered.
    pub power_outage: bool,
}

// =============================================================================
// Deterministic pseudo-random (splitmix64, matching wind.rs pattern)
// =============================================================================

pub(crate) fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Returns a deterministic pseudo-random f32 in [0.0, 1.0) based on seed.
pub(crate) fn rand_f32(seed: u64) -> f32 {
    let hash = splitmix64(seed);
    (hash % 1_000_000) as f32 / 1_000_000.0
}
