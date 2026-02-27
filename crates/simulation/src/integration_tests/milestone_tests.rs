//! Integration tests for the 12-tier milestone system (MILE-001).
//!
//! Verifies milestone progression, auto-unlocking, DP rewards,
//! progress tracking, notifications, and save/load round-trips.

use crate::milestones::{MilestoneProgress, MilestoneTier};
use crate::notifications::NotificationLog;
use crate::test_harness::TestCity;
use crate::unlocks::{UnlockNode, UnlockState};
use crate::virtual_population::VirtualPopulation;
use crate::Saveable;

// =============================================================================
// Milestone Tier Definitions
// =============================================================================

#[test]
fn test_milestone_tier_count() {
    assert_eq!(MilestoneTier::ALL.len(), 12);
}

#[test]
fn test_milestone_tier_population_thresholds() {
    assert_eq!(MilestoneTier::Hamlet.required_population(), 0);
    assert_eq!(MilestoneTier::SmallSettlement.required_population(), 240);
    assert_eq!(MilestoneTier::Village.required_population(), 1_200);
    assert_eq!(MilestoneTier::LargeVillage.required_population(), 2_600);
    assert_eq!(MilestoneTier::Town.required_population(), 5_000);
    assert_eq!(MilestoneTier::LargeTown.required_population(), 7_500);
    assert_eq!(MilestoneTier::SmallCity.required_population(), 12_000);
    assert_eq!(MilestoneTier::City.required_population(), 20_000);
    assert_eq!(MilestoneTier::LargeCity.required_population(), 36_000);
    assert_eq!(MilestoneTier::Metropolis.required_population(), 50_000);
    assert_eq!(MilestoneTier::LargeMetropolis.required_population(), 65_000);
    assert_eq!(MilestoneTier::Megalopolis.required_population(), 80_000);
}

#[test]
fn test_milestone_tier_names() {
    assert_eq!(MilestoneTier::Hamlet.name(), "Hamlet");
    assert_eq!(MilestoneTier::SmallSettlement.name(), "Small Settlement");
    assert_eq!(MilestoneTier::Village.name(), "Village");
    assert_eq!(MilestoneTier::LargeVillage.name(), "Large Village");
    assert_eq!(MilestoneTier::Town.name(), "Town");
    assert_eq!(MilestoneTier::LargeTown.name(), "Large Town");
    assert_eq!(MilestoneTier::SmallCity.name(), "Township");
    assert_eq!(MilestoneTier::City.name(), "City");
    assert_eq!(MilestoneTier::LargeCity.name(), "Grand City");
    assert_eq!(MilestoneTier::Metropolis.name(), "Metropolis");
    assert_eq!(MilestoneTier::LargeMetropolis.name(), "Conurbation");
    assert_eq!(MilestoneTier::Megalopolis.name(), "Megalopolis");
}

#[test]
fn test_milestone_tier_ordering_is_increasing() {
    let pops: Vec<u32> = MilestoneTier::ALL
        .iter()
        .map(|t| t.required_population())
        .collect();
    for i in 1..pops.len() {
        assert!(
            pops[i] > pops[i - 1],
            "Tier {} pop ({}) should be > tier {} pop ({})",
            i,
            pops[i],
            i - 1,
            pops[i - 1]
        );
    }
}

#[test]
fn test_milestone_tier_next() {
    assert_eq!(
        MilestoneTier::Hamlet.next(),
        Some(MilestoneTier::SmallSettlement)
    );
    assert_eq!(
        MilestoneTier::SmallSettlement.next(),
        Some(MilestoneTier::Village)
    );
    assert_eq!(MilestoneTier::Megalopolis.next(), None);
}

#[test]
fn test_milestone_tier_index() {
    assert_eq!(MilestoneTier::Hamlet.index(), 0);
    assert_eq!(MilestoneTier::SmallSettlement.index(), 1);
    assert_eq!(MilestoneTier::Megalopolis.index(), 11);
}

#[test]
fn test_milestone_tier_dp_rewards() {
    assert_eq!(MilestoneTier::Hamlet.dp_reward(), 0);
    assert_eq!(MilestoneTier::SmallSettlement.dp_reward(), 2);
    assert_eq!(MilestoneTier::Village.dp_reward(), 2);
    assert_eq!(MilestoneTier::LargeVillage.dp_reward(), 3);
    assert_eq!(MilestoneTier::Megalopolis.dp_reward(), 6);
}

// =============================================================================
// Milestone Unlocks per Tier
// =============================================================================

#[test]
fn test_milestone_hamlet_unlocks_basics() {
    let unlocks = MilestoneTier::Hamlet.unlocks();
    assert!(unlocks.contains(&UnlockNode::BasicRoads));
    assert!(unlocks.contains(&UnlockNode::ResidentialZoning));
    assert!(unlocks.contains(&UnlockNode::CommercialZoning));
    assert!(unlocks.contains(&UnlockNode::IndustrialZoning));
    assert!(unlocks.contains(&UnlockNode::BasicPower));
    assert!(unlocks.contains(&UnlockNode::BasicWater));
}

#[test]
fn test_milestone_small_settlement_unlocks() {
    let unlocks = MilestoneTier::SmallSettlement.unlocks();
    assert!(unlocks.contains(&UnlockNode::HealthCare));
    assert!(unlocks.contains(&UnlockNode::DeathCare));
    assert!(unlocks.contains(&UnlockNode::BasicSanitation));
}

#[test]
fn test_milestone_village_unlocks() {
    let unlocks = MilestoneTier::Village.unlocks();
    assert!(unlocks.contains(&UnlockNode::FireService));
    assert!(unlocks.contains(&UnlockNode::PoliceService));
    assert!(unlocks.contains(&UnlockNode::ElementaryEducation));
}

#[test]
fn test_milestone_large_village_unlocks() {
    let unlocks = MilestoneTier::LargeVillage.unlocks();
    assert!(unlocks.contains(&UnlockNode::HighSchoolEducation));
    assert!(unlocks.contains(&UnlockNode::SmallParks));
    assert!(unlocks.contains(&UnlockNode::PolicySystem));
}

#[test]
fn test_milestone_town_unlocks() {
    let unlocks = MilestoneTier::Town.unlocks();
    assert!(unlocks.contains(&UnlockNode::PublicTransport));
    assert!(unlocks.contains(&UnlockNode::Landmarks));
}

#[test]
fn test_milestone_large_town_unlocks() {
    let unlocks = MilestoneTier::LargeTown.unlocks();
    assert!(unlocks.contains(&UnlockNode::HighDensityResidential));
    assert!(unlocks.contains(&UnlockNode::HighDensityCommercial));
    assert!(unlocks.contains(&UnlockNode::AdvancedTransport));
    assert!(unlocks.contains(&UnlockNode::OfficeZoning));
}

#[test]
fn test_milestone_megalopolis_unlocks() {
    let unlocks = MilestoneTier::Megalopolis.unlocks();
    assert!(unlocks.contains(&UnlockNode::InternationalAirports));
}

#[test]
fn test_all_unlock_nodes_appear_in_exactly_one_tier() {
    let mut found = std::collections::HashMap::new();
    for &tier in MilestoneTier::ALL {
        for &node in tier.unlocks() {
            found.entry(node).or_insert_with(Vec::new).push(tier);
        }
    }
    for &node in UnlockNode::all() {
        let tiers = found.get(&node);
        assert!(
            tiers.is_some(),
            "UnlockNode {:?} not assigned to any milestone tier",
            node
        );
        let tiers = tiers.unwrap();
        assert_eq!(
            tiers.len(),
            1,
            "UnlockNode {:?} assigned to {} tiers: {:?}",
            node,
            tiers.len(),
            tiers
        );
    }
}

// =============================================================================
// MilestoneProgress Resource
// =============================================================================

#[test]
fn test_milestone_progress_default() {
    let progress = MilestoneProgress::default();
    assert_eq!(progress.current_tier, MilestoneTier::Hamlet);
    assert!(progress.has_reached(MilestoneTier::Hamlet));
    assert!(!progress.has_reached(MilestoneTier::SmallSettlement));
}

#[test]
fn test_milestone_progress_next_tier() {
    let progress = MilestoneProgress::default();
    assert_eq!(progress.next_tier(), Some(MilestoneTier::SmallSettlement));
    assert_eq!(progress.next_tier_population(), Some(240));
}

#[test]
fn test_milestone_progress_fraction() {
    let progress = MilestoneProgress::default();
    // At 0 pop, 0% toward SmallSettlement (240)
    assert!((progress.progress_fraction(0) - 0.0).abs() < 0.01);
    // At 120 pop, 50% toward SmallSettlement
    assert!((progress.progress_fraction(120) - 0.5).abs() < 0.01);
    // At 240 pop, 100% toward SmallSettlement
    assert!((progress.progress_fraction(240) - 1.0).abs() < 0.01);
}

#[test]
fn test_milestone_progress_fraction_at_max_tier() {
    let mut progress = MilestoneProgress::default();
    progress.current_tier = MilestoneTier::Megalopolis;
    progress.reached_tiers = MilestoneTier::ALL.to_vec();
    // At max tier, progress is always 1.0
    assert!((progress.progress_fraction(100_000) - 1.0).abs() < 0.01);
}

#[test]
fn test_milestone_progress_resource_exists_in_test_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<MilestoneProgress>();
    let progress = city.resource::<MilestoneProgress>();
    assert_eq!(progress.current_tier, MilestoneTier::Hamlet);
}

// =============================================================================
// Milestone Progression System (via TestCity)
// =============================================================================

#[test]
fn test_milestone_progression_at_240_pop_unlocks_healthcare() {
    let mut city = TestCity::new();
    city.world_mut()
        .resource_mut::<VirtualPopulation>()
        .total_virtual = 240;
    city.tick_slow_cycle();

    let progress = city.resource::<MilestoneProgress>();
    assert!(progress.has_reached(MilestoneTier::SmallSettlement));
    assert_eq!(progress.current_tier, MilestoneTier::SmallSettlement);

    let unlocks = city.resource::<UnlockState>();
    assert!(unlocks.is_unlocked(UnlockNode::HealthCare));
    assert!(unlocks.is_unlocked(UnlockNode::DeathCare));
    assert!(unlocks.is_unlocked(UnlockNode::BasicSanitation));
}

#[test]
fn test_milestone_progression_at_1200_pop_unlocks_fire_police() {
    let mut city = TestCity::new();
    city.world_mut()
        .resource_mut::<VirtualPopulation>()
        .total_virtual = 1_200;
    city.tick_slow_cycle();

    let progress = city.resource::<MilestoneProgress>();
    assert!(progress.has_reached(MilestoneTier::Village));

    let unlocks = city.resource::<UnlockState>();
    assert!(unlocks.is_unlocked(UnlockNode::FireService));
    assert!(unlocks.is_unlocked(UnlockNode::PoliceService));
    assert!(unlocks.is_unlocked(UnlockNode::ElementaryEducation));
    // Also should have Small Settlement unlocks
    assert!(unlocks.is_unlocked(UnlockNode::HealthCare));
}

#[test]
fn test_milestone_progression_awards_dp() {
    let mut city = TestCity::new();
    let initial_dp = city.resource::<UnlockState>().development_points;

    city.world_mut()
        .resource_mut::<VirtualPopulation>()
        .total_virtual = 240;
    city.tick_slow_cycle();

    let dp_after = city.resource::<UnlockState>().development_points;
    assert_eq!(
        dp_after,
        initial_dp + 2,
        "Should gain 2 DP at SmallSettlement milestone"
    );
}

#[test]
fn test_milestone_progression_multiple_tiers() {
    let mut city = TestCity::new();
    let initial_dp = city.resource::<UnlockState>().development_points;

    // Jump to 2,600 pop — should trigger tiers 1, 2, and 3
    city.world_mut()
        .resource_mut::<VirtualPopulation>()
        .total_virtual = 2_600;
    city.tick_slow_cycle();

    let progress = city.resource::<MilestoneProgress>();
    assert!(progress.has_reached(MilestoneTier::SmallSettlement));
    assert!(progress.has_reached(MilestoneTier::Village));
    assert!(progress.has_reached(MilestoneTier::LargeVillage));
    assert_eq!(progress.current_tier, MilestoneTier::LargeVillage);

    let dp_after = city.resource::<UnlockState>().development_points;
    // Tier 1: +2, Tier 2: +2, Tier 3: +3 = +7
    assert_eq!(dp_after, initial_dp + 7);
}

#[test]
fn test_milestone_does_not_retrigger() {
    let mut city = TestCity::new();

    city.world_mut()
        .resource_mut::<VirtualPopulation>()
        .total_virtual = 240;
    city.tick_slow_cycle();
    let dp_after_first = city.resource::<UnlockState>().development_points;

    // Run again at same pop
    city.tick_slow_cycle();
    let dp_after_second = city.resource::<UnlockState>().development_points;
    assert_eq!(
        dp_after_first, dp_after_second,
        "Milestone should not re-trigger"
    );
}

#[test]
fn test_milestone_notification_emitted() {
    let mut city = TestCity::new();

    city.world_mut()
        .resource_mut::<VirtualPopulation>()
        .total_virtual = 240;
    city.tick_slow_cycle();

    let log = city.resource::<NotificationLog>();
    let milestone_notifs: Vec<_> = log
        .active
        .iter()
        .filter(|n| n.text.contains("Milestone reached"))
        .collect();
    assert!(
        !milestone_notifs.is_empty(),
        "Should have at least one milestone notification"
    );
    assert!(milestone_notifs[0].text.contains("Small Settlement"));
}

// =============================================================================
// Save/Load Round-Trip
// =============================================================================

#[test]
fn test_milestone_progress_save_round_trip() {
    let mut progress = MilestoneProgress::default();
    progress.current_tier = MilestoneTier::Town;
    progress.tier_reached_pop = 5_200;
    progress.reached_tiers = vec![
        MilestoneTier::Hamlet,
        MilestoneTier::SmallSettlement,
        MilestoneTier::Village,
        MilestoneTier::LargeVillage,
        MilestoneTier::Town,
    ];

    let bytes = progress.save_to_bytes().expect("Should serialize");
    let loaded = MilestoneProgress::load_from_bytes(&bytes);

    assert_eq!(loaded.current_tier, MilestoneTier::Town);
    assert_eq!(loaded.tier_reached_pop, 5_200);
    assert_eq!(loaded.reached_tiers.len(), 5);
    assert!(loaded.has_reached(MilestoneTier::Town));
}

#[test]
fn test_milestone_progress_save_skips_default() {
    let progress = MilestoneProgress::default();
    assert!(
        progress.save_to_bytes().is_none(),
        "Default state should skip save"
    );
}
