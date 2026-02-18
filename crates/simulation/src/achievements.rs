use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::disasters::ActiveDisaster;
use crate::economy::CityBudget;
use crate::education_jobs::EmploymentStats;
use crate::events::{CityEvent, CityEventType, EventJournal};
use crate::grid::{RoadType, WorldGrid};
use crate::natural_resources::ResourceBalance;
use crate::production::CityGoods;
use crate::specialization::{CitySpecialization, CitySpecializations};
use crate::stats::CityStats;
use crate::time_of_day::GameClock;
use crate::SlowTickTimer;
use crate::TickCounter;

// =============================================================================
// Achievement Definition
// =============================================================================

/// All achievements a player can unlock in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    Millionaire,              // Treasury reaches $1M
    TradePositive100,         // Positive trade balance for 100 consecutive ticks

    // Services
    FullPowerCoverage,        // 100% power coverage
    FullWaterCoverage,        // 100% water coverage

    // Happiness
    HappyCity,                // Average happiness above 80%
    EuphoricCity,             // Average happiness above 90%

    // Infrastructure
    RoadBuilder500,           // 500 road cells
    HighwayBuilder,           // Build a highway (have highway road type)
    RoadDiversity,            // Have all road types (Local, Avenue, Boulevard, Highway, OneWay, Path)

    // Special
    DisasterSurvivor,         // Survive a disaster (disaster resolves)
    FullEmployment,           // Reach 0% unemployment (rounded)
    DiverseSpecializations,   // Have all 6 specialization scores above 20
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
    ];

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Achievement::Population1K => "Village to Town",
            Achievement::Population5K => "Growing Community",
            Achievement::Population10K => "Cityhood",
            Achievement::Population50K => "Metro Area",
            Achievement::Population100K => "Major City",
            Achievement::Population500K => "Megalopolis",
            Achievement::Population1M => "World Capital",
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
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AchievementTracker {
    /// Maps unlocked achievements to the tick at which they were unlocked.
    pub unlocked: HashMap<Achievement, u64>,
    /// Counter for consecutive ticks with positive trade balance (for TradePositive100).
    pub positive_trade_ticks: u32,
    /// Whether a disaster has been survived (disaster was active and then resolved).
    pub had_active_disaster: bool,
}

impl Default for AchievementTracker {
    fn default() -> Self {
        Self {
            unlocked: HashMap::new(),
            positive_trade_ticks: 0,
            had_active_disaster: false,
        }
    }
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
    fn unlock(&mut self, achievement: Achievement, tick: u64) -> bool {
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

// =============================================================================
// Check Achievements System
// =============================================================================

/// System that checks achievement conditions every 100 ticks (via SlowTickTimer).
/// When an achievement unlocks, it applies the reward and logs to the EventJournal.
#[allow(clippy::too_many_arguments)]
pub fn check_achievements(
    slow_timer: Res<SlowTickTimer>,
    tick: Res<TickCounter>,
    clock: Res<GameClock>,
    stats: Res<CityStats>,
    grid: Res<WorldGrid>,
    employment_stats: Res<EmploymentStats>,
    resource_balance: Res<ResourceBalance>,
    city_goods: Res<CityGoods>,
    specializations: Res<CitySpecializations>,
    active_disaster: Res<ActiveDisaster>,
    mut tracker: ResMut<AchievementTracker>,
    mut notifications: ResMut<AchievementNotification>,
    mut journal: ResMut<EventJournal>,
    mut budget: ResMut<CityBudget>,
    mut unlock_state: ResMut<crate::unlocks::UnlockState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let current_tick = tick.0;
    let mut newly_unlocked: Vec<Achievement> = Vec::new();

    // --- Population milestones ---
    check_population(&stats, current_tick, &mut tracker, &mut newly_unlocked);

    // --- Economic: Millionaire ---
    if !tracker.is_unlocked(Achievement::Millionaire) && budget.treasury >= 1_000_000.0 {
        if tracker.unlock(Achievement::Millionaire, current_tick) {
            newly_unlocked.push(Achievement::Millionaire);
        }
    }

    // --- Trade balance tracking ---
    let trade_balance = resource_balance.trade_balance() + city_goods.trade_balance;
    if trade_balance > 0.0 {
        tracker.positive_trade_ticks += 1;
    } else {
        tracker.positive_trade_ticks = 0;
    }
    if !tracker.is_unlocked(Achievement::TradePositive100) && tracker.positive_trade_ticks >= 100 {
        if tracker.unlock(Achievement::TradePositive100, current_tick) {
            newly_unlocked.push(Achievement::TradePositive100);
        }
    }

    // --- Services: Full power/water coverage ---
    check_utility_coverage(&grid, current_tick, &mut tracker, &mut newly_unlocked);

    // --- Happiness ---
    if !tracker.is_unlocked(Achievement::HappyCity)
        && stats.population > 0
        && stats.average_happiness >= 80.0
    {
        if tracker.unlock(Achievement::HappyCity, current_tick) {
            newly_unlocked.push(Achievement::HappyCity);
        }
    }
    if !tracker.is_unlocked(Achievement::EuphoricCity)
        && stats.population > 0
        && stats.average_happiness >= 90.0
    {
        if tracker.unlock(Achievement::EuphoricCity, current_tick) {
            newly_unlocked.push(Achievement::EuphoricCity);
        }
    }

    // --- Infrastructure ---
    if !tracker.is_unlocked(Achievement::RoadBuilder500) && stats.road_cells >= 500 {
        if tracker.unlock(Achievement::RoadBuilder500, current_tick) {
            newly_unlocked.push(Achievement::RoadBuilder500);
        }
    }

    check_road_types(&grid, current_tick, &mut tracker, &mut newly_unlocked);

    // --- Employment ---
    if !tracker.is_unlocked(Achievement::FullEmployment)
        && stats.population > 100
        && employment_stats.unemployment_rate < 0.005 // effectively 0% when rounded
    {
        if tracker.unlock(Achievement::FullEmployment, current_tick) {
            newly_unlocked.push(Achievement::FullEmployment);
        }
    }

    // --- Disaster survivor ---
    // Track if a disaster is currently active
    if active_disaster.current.is_some() {
        tracker.had_active_disaster = true;
    }
    // If we had one and it resolved, grant the achievement
    if !tracker.is_unlocked(Achievement::DisasterSurvivor)
        && tracker.had_active_disaster
        && active_disaster.current.is_none()
    {
        if tracker.unlock(Achievement::DisasterSurvivor, current_tick) {
            newly_unlocked.push(Achievement::DisasterSurvivor);
        }
    }

    // --- Diverse specializations ---
    if !tracker.is_unlocked(Achievement::DiverseSpecializations) {
        let all_above_20 = CitySpecialization::ALL
            .iter()
            .all(|&spec| specializations.get(spec).score >= 20.0);
        if all_above_20 {
            if tracker.unlock(Achievement::DiverseSpecializations, current_tick) {
                newly_unlocked.push(Achievement::DiverseSpecializations);
            }
        }
    }

    // --- Apply rewards and log ---
    for &achievement in &newly_unlocked {
        let reward = achievement.reward();
        apply_reward(&reward, &mut budget, &mut unlock_state);

        journal.push(CityEvent {
            event_type: CityEventType::MilestoneReached(format!(
                "Achievement: {}",
                achievement.name()
            )),
            day: clock.day,
            hour: clock.hour,
            description: format!(
                "Achievement unlocked: {} - {} (Reward: {})",
                achievement.name(),
                achievement.description(),
                reward.description(),
            ),
        });
    }

    notifications.recent_unlocks.extend(newly_unlocked);
}

// =============================================================================
// Internal Helpers
// =============================================================================

fn check_population(
    stats: &CityStats,
    tick: u64,
    tracker: &mut AchievementTracker,
    newly_unlocked: &mut Vec<Achievement>,
) {
    let milestones = [
        (1_000, Achievement::Population1K),
        (5_000, Achievement::Population5K),
        (10_000, Achievement::Population10K),
        (50_000, Achievement::Population50K),
        (100_000, Achievement::Population100K),
        (500_000, Achievement::Population500K),
        (1_000_000, Achievement::Population1M),
    ];

    for &(threshold, achievement) in &milestones {
        if !tracker.is_unlocked(achievement) && stats.population >= threshold {
            if tracker.unlock(achievement, tick) {
                newly_unlocked.push(achievement);
            }
        }
    }
}

fn check_utility_coverage(
    grid: &WorldGrid,
    tick: u64,
    tracker: &mut AchievementTracker,
    newly_unlocked: &mut Vec<Achievement>,
) {
    use crate::grid::{CellType, ZoneType};

    let mut total_zoned = 0u32;
    let mut powered = 0u32;
    let mut watered = 0u32;

    for cell in &grid.cells {
        if cell.cell_type == CellType::Grass && cell.zone != ZoneType::None {
            total_zoned += 1;
            if cell.has_power {
                powered += 1;
            }
            if cell.has_water {
                watered += 1;
            }
        }
    }

    if total_zoned > 0 {
        let power_coverage = powered as f32 / total_zoned as f32;
        let water_coverage = watered as f32 / total_zoned as f32;

        if !tracker.is_unlocked(Achievement::FullPowerCoverage) && power_coverage >= 0.999 {
            if tracker.unlock(Achievement::FullPowerCoverage, tick) {
                newly_unlocked.push(Achievement::FullPowerCoverage);
            }
        }
        if !tracker.is_unlocked(Achievement::FullWaterCoverage) && water_coverage >= 0.999 {
            if tracker.unlock(Achievement::FullWaterCoverage, tick) {
                newly_unlocked.push(Achievement::FullWaterCoverage);
            }
        }
    }
}

fn check_road_types(
    grid: &WorldGrid,
    tick: u64,
    tracker: &mut AchievementTracker,
    newly_unlocked: &mut Vec<Achievement>,
) {
    let mut has_highway = false;
    let mut road_types_found: u8 = 0;

    for cell in &grid.cells {
        if cell.cell_type == crate::grid::CellType::Road {
            match cell.road_type {
                RoadType::Local => road_types_found |= 0b000001,
                RoadType::Avenue => road_types_found |= 0b000010,
                RoadType::Boulevard => road_types_found |= 0b000100,
                RoadType::Highway => {
                    road_types_found |= 0b001000;
                    has_highway = true;
                }
                RoadType::OneWay => road_types_found |= 0b010000,
                RoadType::Path => road_types_found |= 0b100000,
            }
        }
    }

    if !tracker.is_unlocked(Achievement::HighwayBuilder) && has_highway {
        if tracker.unlock(Achievement::HighwayBuilder, tick) {
            newly_unlocked.push(Achievement::HighwayBuilder);
        }
    }

    // All 6 road types present
    if !tracker.is_unlocked(Achievement::RoadDiversity) && road_types_found == 0b111111 {
        if tracker.unlock(Achievement::RoadDiversity, tick) {
            newly_unlocked.push(Achievement::RoadDiversity);
        }
    }
}

fn apply_reward(
    reward: &AchievementReward,
    budget: &mut ResMut<CityBudget>,
    unlock_state: &mut ResMut<crate::unlocks::UnlockState>,
) {
    match reward {
        AchievementReward::TreasuryBonus(amount) => {
            budget.treasury += amount;
        }
        AchievementReward::DevelopmentPoints(pts) => {
            unlock_state.development_points += pts;
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_tracker_default() {
        let tracker = AchievementTracker::default();
        assert_eq!(tracker.unlocked_count(), 0);
        assert!(!tracker.is_unlocked(Achievement::Population1K));
    }

    #[test]
    fn test_achievement_unlock() {
        let mut tracker = AchievementTracker::default();
        assert!(tracker.unlock(Achievement::Population1K, 100));
        assert!(tracker.is_unlocked(Achievement::Population1K));
        assert_eq!(tracker.unlocked_count(), 1);
        // Unlocking again should return false
        assert!(!tracker.unlock(Achievement::Population1K, 200));
        assert_eq!(tracker.unlocked_count(), 1);
    }

    #[test]
    fn test_all_achievements_count() {
        assert_eq!(Achievement::total_count(), 19);
    }

    #[test]
    fn test_all_achievements_have_names() {
        for &a in Achievement::ALL {
            assert!(!a.name().is_empty());
            assert!(!a.description().is_empty());
        }
    }

    #[test]
    fn test_reward_description() {
        let treasury = AchievementReward::TreasuryBonus(50_000.0);
        assert!(treasury.description().contains("$50K"));

        let dp = AchievementReward::DevelopmentPoints(3);
        assert!(dp.description().contains("3 development points"));

        let big = AchievementReward::TreasuryBonus(1_500_000.0);
        assert!(big.description().contains("$1.5M"));
    }

    #[test]
    fn test_notification_take() {
        let mut notif = AchievementNotification::default();
        notif.recent_unlocks.push(Achievement::Population1K);
        notif.recent_unlocks.push(Achievement::HappyCity);
        let taken = notif.take();
        assert_eq!(taken.len(), 2);
        assert!(notif.recent_unlocks.is_empty());
    }

    #[test]
    fn test_check_population_milestones() {
        let mut tracker = AchievementTracker::default();
        let mut newly = Vec::new();

        let stats = CityStats {
            population: 6_000,
            ..Default::default()
        };

        check_population(&stats, 42, &mut tracker, &mut newly);

        // Should unlock 1K and 5K but not 10K
        assert!(tracker.is_unlocked(Achievement::Population1K));
        assert!(tracker.is_unlocked(Achievement::Population5K));
        assert!(!tracker.is_unlocked(Achievement::Population10K));
        assert_eq!(newly.len(), 2);
    }

    #[test]
    fn test_trade_positive_counter() {
        let mut tracker = AchievementTracker::default();
        // Simulate 99 positive ticks -> no unlock
        tracker.positive_trade_ticks = 99;
        assert!(!tracker.is_unlocked(Achievement::TradePositive100));

        // 100th tick
        tracker.positive_trade_ticks = 100;
        tracker.unlock(Achievement::TradePositive100, 500);
        assert!(tracker.is_unlocked(Achievement::TradePositive100));
    }

    #[test]
    fn test_disaster_survivor_logic() {
        let mut tracker = AchievementTracker::default();
        // No disaster seen yet
        assert!(!tracker.had_active_disaster);

        // Mark that a disaster was active
        tracker.had_active_disaster = true;

        // Disaster resolved (current is None), unlock
        let newly = tracker.unlock(Achievement::DisasterSurvivor, 300);
        assert!(newly);
        assert!(tracker.is_unlocked(Achievement::DisasterSurvivor));
    }
}
