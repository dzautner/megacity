use std::collections::HashMap;

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

// =============================================================================
// Achievement Definition
// =============================================================================

/// All achievements a player can unlock in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum Achievement {
    // Population milestones
    Population1K,
    Population5K,
    Population10K,
    Population50K,
    Population100K,
    Population500K,
    Population1M,

    // Economic
    Millionaire,      // Treasury reaches $1M
    TradePositive100, // Positive trade balance for 100 consecutive ticks

    // Services
    FullPowerCoverage, // 100% power coverage
    FullWaterCoverage, // 100% water coverage

    // Happiness
    HappyCity,    // Average happiness above 80%
    EuphoricCity, // Average happiness above 90%

    // Infrastructure
    RoadBuilder500, // 500 road cells
    HighwayBuilder, // Build a highway (have highway road type)
    RoadDiversity,  // Have all road types (Local, Avenue, Boulevard, Highway, OneWay, Path)

    // Special
    DisasterSurvivor,       // Survive a disaster (disaster resolves)
    FullEmployment,         // Reach 0% unemployment (rounded)
    DiverseSpecializations, // Have all 6 specialization scores above 20

    // Environmental (POLL-021)
    GreenCity,   // Environmental score > 80
    EcoChampion, // Environmental score > 95
}

impl Achievement {
    /// All achievement variants for iteration.
    pub const ALL: &'static [Achievement] = &[
        Achievement::Population1K,
        Achievement::Population5K,
        Achievement::Population10K,
        Achievement::Population50K,
        Achievement::Population100K,
        Achievement::Population500K,
        Achievement::Population1M,
        Achievement::Millionaire,
        Achievement::TradePositive100,
        Achievement::FullPowerCoverage,
        Achievement::FullWaterCoverage,
        Achievement::HappyCity,
        Achievement::EuphoricCity,
        Achievement::RoadBuilder500,
        Achievement::HighwayBuilder,
        Achievement::RoadDiversity,
        Achievement::DisasterSurvivor,
        Achievement::FullEmployment,
        Achievement::DiverseSpecializations,
        Achievement::GreenCity,
        Achievement::EcoChampion,
    ];

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Achievement::Population1K => "Village to Town",
            Achievement::Population5K => "Growing Community",
            Achievement::Population10K => "Cityhood",
            Achievement::Population50K => "Metro Area",
            Achievement::Population100K => "Major City",
            Achievement::Population500K => "Gigapolis",
            Achievement::Population1M => "Megacity",
            Achievement::Millionaire => "City Millionaire",
            Achievement::TradePositive100 => "Trade Surplus",
            Achievement::FullPowerCoverage => "Fully Powered",
            Achievement::FullWaterCoverage => "Water For All",
            Achievement::HappyCity => "Happy City",
            Achievement::EuphoricCity => "Euphoric Metropolis",
            Achievement::RoadBuilder500 => "Road Builder",
            Achievement::HighwayBuilder => "Highway Engineer",
            Achievement::RoadDiversity => "Road Architect",
            Achievement::DisasterSurvivor => "Disaster Survivor",
            Achievement::FullEmployment => "Full Employment",
            Achievement::DiverseSpecializations => "Renaissance City",
            Achievement::GreenCity => "Green City",
            Achievement::EcoChampion => "Eco Champion",
        }
    }

    /// Description of what the player must do.
    pub fn description(self) -> &'static str {
        match self {
            Achievement::Population1K => "Reach 1,000 population",
            Achievement::Population5K => "Reach 5,000 population",
            Achievement::Population10K => "Reach 10,000 population",
            Achievement::Population50K => "Reach 50,000 population",
            Achievement::Population100K => "Reach 100,000 population",
            Achievement::Population500K => "Reach 500,000 population",
            Achievement::Population1M => "Reach 1,000,000 population",
            Achievement::Millionaire => "Accumulate $1,000,000 in treasury",
            Achievement::TradePositive100 => "Maintain positive trade balance for 100 ticks",
            Achievement::FullPowerCoverage => "Achieve 100% power coverage",
            Achievement::FullWaterCoverage => "Achieve 100% water coverage",
            Achievement::HappyCity => "Average happiness above 80%",
            Achievement::EuphoricCity => "Average happiness above 90%",
            Achievement::RoadBuilder500 => "Build 500 road cells",
            Achievement::HighwayBuilder => "Build a highway",
            Achievement::RoadDiversity => "Use all 6 road types in your city",
            Achievement::DisasterSurvivor => "Survive a disaster",
            Achievement::FullEmployment => "Reach 0% unemployment",
            Achievement::DiverseSpecializations => "Score above 20 in all 6 specializations",
            Achievement::GreenCity => "Environmental score above 80",
            Achievement::EcoChampion => "Environmental score above 95",
        }
    }

    /// The reward given when this achievement is unlocked.
    pub fn reward(self) -> AchievementReward {
        match self {
            Achievement::Population1K => AchievementReward::TreasuryBonus(5_000.0),
            Achievement::Population5K => AchievementReward::TreasuryBonus(15_000.0),
            Achievement::Population10K => AchievementReward::TreasuryBonus(30_000.0),
            Achievement::Population50K => AchievementReward::TreasuryBonus(75_000.0),
            Achievement::Population100K => AchievementReward::TreasuryBonus(150_000.0),
            Achievement::Population500K => AchievementReward::TreasuryBonus(500_000.0),
            Achievement::Population1M => AchievementReward::TreasuryBonus(1_000_000.0),
            Achievement::Millionaire => AchievementReward::DevelopmentPoints(5),
            Achievement::TradePositive100 => AchievementReward::TreasuryBonus(50_000.0),
            Achievement::FullPowerCoverage => AchievementReward::DevelopmentPoints(3),
            Achievement::FullWaterCoverage => AchievementReward::DevelopmentPoints(3),
            Achievement::HappyCity => AchievementReward::TreasuryBonus(25_000.0),
            Achievement::EuphoricCity => AchievementReward::TreasuryBonus(100_000.0),
            Achievement::RoadBuilder500 => AchievementReward::TreasuryBonus(10_000.0),
            Achievement::HighwayBuilder => AchievementReward::DevelopmentPoints(2),
            Achievement::RoadDiversity => AchievementReward::DevelopmentPoints(3),
            Achievement::DisasterSurvivor => AchievementReward::TreasuryBonus(50_000.0),
            Achievement::FullEmployment => AchievementReward::TreasuryBonus(40_000.0),
            Achievement::DiverseSpecializations => AchievementReward::DevelopmentPoints(5),
            Achievement::GreenCity => AchievementReward::TreasuryBonus(30_000.0),
            Achievement::EcoChampion => AchievementReward::TreasuryBonus(100_000.0),
        }
    }

    /// Total number of achievements.
    pub fn total_count() -> usize {
        Self::ALL.len()
    }
}

// =============================================================================
// Achievement Reward
// =============================================================================

/// Reward granted when an achievement is unlocked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementReward {
    /// Add a flat amount to the city treasury.
    TreasuryBonus(f64),
    /// Award development points for the unlock tree.
    DevelopmentPoints(u32),
}

impl AchievementReward {
    pub fn description(&self) -> String {
        match self {
            AchievementReward::TreasuryBonus(amount) => {
                if *amount >= 1_000_000.0 {
                    format!("+${:.1}M treasury", amount / 1_000_000.0)
                } else if *amount >= 1_000.0 {
                    format!("+${:.0}K treasury", amount / 1_000.0)
                } else {
                    format!("+${:.0} treasury", amount)
                }
            }
            AchievementReward::DevelopmentPoints(pts) => {
                format!("+{} development points", pts)
            }
        }
    }
}

// =============================================================================
// Achievement Tracker Resource
// =============================================================================

/// Tracks which achievements the player has unlocked and when.
#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct AchievementTracker {
    /// Maps unlocked achievements to the tick at which they were unlocked.
    pub unlocked: HashMap<Achievement, u64>,
    /// Counter for consecutive ticks with positive trade balance (for TradePositive100).
    pub positive_trade_ticks: u32,
    /// Whether a disaster has been survived (disaster was active and then resolved).
    pub had_active_disaster: bool,
}

impl AchievementTracker {
    /// Returns true if the given achievement is unlocked.
    pub fn is_unlocked(&self, achievement: Achievement) -> bool {
        self.unlocked.contains_key(&achievement)
    }

    /// Returns the number of unlocked achievements.
    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    /// Unlock an achievement, returning true if it was newly unlocked.
    pub(crate) fn unlock(&mut self, achievement: Achievement, tick: u64) -> bool {
        if self.unlocked.contains_key(&achievement) {
            return false;
        }
        self.unlocked.insert(achievement, tick);
        true
    }
}

// =============================================================================
// Achievement Notification Resource
// =============================================================================

/// Resource for passing recently unlocked achievements to the UI for popup display.
#[derive(Resource, Default, Debug, Clone)]
pub struct AchievementNotification {
    /// List of achievements unlocked since the UI last read them.
    pub recent_unlocks: Vec<Achievement>,
}

impl AchievementNotification {
    /// Take all pending notifications, clearing the internal list.
    pub fn take(&mut self) -> Vec<Achievement> {
        std::mem::take(&mut self.recent_unlocks)
    }
}
