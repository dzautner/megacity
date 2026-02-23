//! Recycling program tier definitions and policy parameters.

use serde::{Deserialize, Serialize};

/// Recycling program tiers available to the player.
///
/// Each tier represents a progressively more ambitious (and costly) recycling
/// program. The player selects a tier as a city-wide policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RecyclingTier {
    /// No formal program; only scavenging / informal recycling (~5% diversion).
    #[default]
    None,
    /// Voluntary drop-off centres; moderate participation (~15% diversion).
    VoluntaryDropoff,
    /// Curbside collection with basic sorting (~30% diversion).
    CurbsideBasic,
    /// Curbside collection with multi-stream sorting (~45% diversion).
    CurbsideSort,
    /// Single-stream (commingled) curbside collection (~40% diversion).
    SingleStream,
    /// Variable-rate pricing; pay by weight/volume (~50% diversion).
    PayAsYouThrow,
    /// Ambitious zero-waste goal with composting + reuse (~60% diversion).
    ZeroWaste,
}

impl RecyclingTier {
    /// Fraction of waste stream diverted from landfill (0.0..=1.0).
    pub fn diversion_rate(self) -> f32 {
        match self {
            Self::None => 0.05,
            Self::VoluntaryDropoff => 0.15,
            Self::CurbsideBasic => 0.30,
            Self::CurbsideSort => 0.45,
            Self::SingleStream => 0.40,
            Self::PayAsYouThrow => 0.50,
            Self::ZeroWaste => 0.60,
        }
    }

    /// Fraction of households participating in the program (0.0..=1.0).
    pub fn participation_rate(self) -> f32 {
        match self {
            Self::None => 0.05,
            Self::VoluntaryDropoff => 0.25,
            Self::CurbsideBasic => 0.60,
            Self::CurbsideSort => 0.70,
            Self::SingleStream => 0.80,
            Self::PayAsYouThrow => 0.85,
            Self::ZeroWaste => 0.95,
        }
    }

    /// Annual cost per household in dollars.
    pub fn cost_per_household_year(self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::VoluntaryDropoff => 15.0,
            Self::CurbsideBasic => 60.0,
            Self::CurbsideSort => 90.0,
            Self::SingleStream => 75.0,
            Self::PayAsYouThrow => 50.0, // lower because variable rate shifts cost to users
            Self::ZeroWaste => 120.0,
        }
    }

    /// Contamination rate: fraction of the recycling stream that is actually
    /// non-recyclable waste and must go to landfill (0.0..=1.0).
    ///
    /// Higher convenience programs (single-stream) have worse contamination.
    pub fn contamination_rate(self) -> f32 {
        match self {
            Self::None => 0.30,
            Self::VoluntaryDropoff => 0.20,
            Self::CurbsideBasic => 0.20,
            Self::CurbsideSort => 0.15,
            Self::SingleStream => 0.25,
            Self::PayAsYouThrow => 0.18,
            Self::ZeroWaste => 0.15,
        }
    }

    /// Revenue potential multiplier relative to baseline commodity prices.
    /// Higher tiers with better sorting yield cleaner material and higher prices.
    pub fn revenue_potential(self) -> f32 {
        match self {
            Self::None => 0.3,
            Self::VoluntaryDropoff => 0.5,
            Self::CurbsideBasic => 0.7,
            Self::CurbsideSort => 0.9,
            Self::SingleStream => 0.65,
            Self::PayAsYouThrow => 0.8,
            Self::ZeroWaste => 1.0,
        }
    }

    /// Human-readable name for the tier.
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "No Program",
            Self::VoluntaryDropoff => "Voluntary Drop-off",
            Self::CurbsideBasic => "Curbside Basic",
            Self::CurbsideSort => "Curbside Multi-Sort",
            Self::SingleStream => "Single Stream",
            Self::PayAsYouThrow => "Pay-As-You-Throw",
            Self::ZeroWaste => "Zero Waste Goal",
        }
    }

    /// All tiers in order of increasing ambition.
    pub fn all() -> &'static [RecyclingTier] {
        &[
            Self::None,
            Self::VoluntaryDropoff,
            Self::CurbsideBasic,
            Self::CurbsideSort,
            Self::SingleStream,
            Self::PayAsYouThrow,
            Self::ZeroWaste,
        ]
    }
}
