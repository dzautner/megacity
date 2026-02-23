use bevy::prelude::*;

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

use super::types::{Achievement, AchievementNotification, AchievementReward, AchievementTracker};

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
    if !tracker.is_unlocked(Achievement::Millionaire)
        && budget.treasury >= 1_000_000.0
        && tracker.unlock(Achievement::Millionaire, current_tick)
    {
        newly_unlocked.push(Achievement::Millionaire);
    }

    // --- Trade balance tracking ---
    let trade_balance = resource_balance.trade_balance() + city_goods.trade_balance;
    if trade_balance > 0.0 {
        tracker.positive_trade_ticks += 1;
    } else {
        tracker.positive_trade_ticks = 0;
    }
    if !tracker.is_unlocked(Achievement::TradePositive100)
        && tracker.positive_trade_ticks >= 100
        && tracker.unlock(Achievement::TradePositive100, current_tick)
    {
        newly_unlocked.push(Achievement::TradePositive100);
    }

    // --- Services: Full power/water coverage ---
    check_utility_coverage(&grid, current_tick, &mut tracker, &mut newly_unlocked);

    // --- Happiness ---
    if !tracker.is_unlocked(Achievement::HappyCity)
        && stats.population > 0
        && stats.average_happiness >= 80.0
        && tracker.unlock(Achievement::HappyCity, current_tick)
    {
        newly_unlocked.push(Achievement::HappyCity);
    }
    if !tracker.is_unlocked(Achievement::EuphoricCity)
        && stats.population > 0
        && stats.average_happiness >= 90.0
        && tracker.unlock(Achievement::EuphoricCity, current_tick)
    {
        newly_unlocked.push(Achievement::EuphoricCity);
    }

    // --- Infrastructure ---
    if !tracker.is_unlocked(Achievement::RoadBuilder500)
        && stats.road_cells >= 500
        && tracker.unlock(Achievement::RoadBuilder500, current_tick)
    {
        newly_unlocked.push(Achievement::RoadBuilder500);
    }

    check_road_types(&grid, current_tick, &mut tracker, &mut newly_unlocked);

    // --- Employment ---
    if !tracker.is_unlocked(Achievement::FullEmployment)
        && stats.population > 100
        && employment_stats.unemployment_rate < 0.005 // effectively 0% when rounded
        && tracker.unlock(Achievement::FullEmployment, current_tick)
    {
        newly_unlocked.push(Achievement::FullEmployment);
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
        && tracker.unlock(Achievement::DisasterSurvivor, current_tick)
    {
        newly_unlocked.push(Achievement::DisasterSurvivor);
    }

    // --- Diverse specializations ---
    if !tracker.is_unlocked(Achievement::DiverseSpecializations) {
        let all_above_20 = CitySpecialization::ALL
            .iter()
            .all(|&spec| specializations.get(spec).score >= 20.0);
        if all_above_20 && tracker.unlock(Achievement::DiverseSpecializations, current_tick) {
            newly_unlocked.push(Achievement::DiverseSpecializations);
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

pub(crate) fn check_population(
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
        if !tracker.is_unlocked(achievement)
            && stats.population >= threshold
            && tracker.unlock(achievement, tick)
        {
            newly_unlocked.push(achievement);
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

        if !tracker.is_unlocked(Achievement::FullPowerCoverage)
            && power_coverage >= 0.999
            && tracker.unlock(Achievement::FullPowerCoverage, tick)
        {
            newly_unlocked.push(Achievement::FullPowerCoverage);
        }
        if !tracker.is_unlocked(Achievement::FullWaterCoverage)
            && water_coverage >= 0.999
            && tracker.unlock(Achievement::FullWaterCoverage, tick)
        {
            newly_unlocked.push(Achievement::FullWaterCoverage);
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

    if !tracker.is_unlocked(Achievement::HighwayBuilder)
        && has_highway
        && tracker.unlock(Achievement::HighwayBuilder, tick)
    {
        newly_unlocked.push(Achievement::HighwayBuilder);
    }

    // All 6 road types present
    if !tracker.is_unlocked(Achievement::RoadDiversity)
        && road_types_found == 0b111111
        && tracker.unlock(Achievement::RoadDiversity, tick)
    {
        newly_unlocked.push(Achievement::RoadDiversity);
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
