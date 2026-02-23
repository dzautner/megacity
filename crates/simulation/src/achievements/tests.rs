#[cfg(test)]
mod tests {
    use crate::achievements::systems::check_population;
    use crate::achievements::types::*;
    use crate::stats::CityStats;

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
