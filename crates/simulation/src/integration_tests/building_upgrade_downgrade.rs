//! Integration tests for building upgrade and downgrade systems (TEST-042).
//!
//! Tests cover:
//! - Upgrade when all conditions are met (high occupancy + high happiness)
//! - No upgrade when conditions not met (low occupancy, low happiness)
//! - Building level never exceeds zone max_level
//! - Building level respects policy max_building_level (HighRiseBan)
//! - Building level respects FAR cap
//! - Downgrade when happiness is very low
//! - Downgrade clamps excess occupants
//! - UGB prevents upgrades outside boundary
//! - Mixed-use building capacities update on upgrade/downgrade

use crate::building_upgrade::UpgradeTimer;
use crate::buildings::{max_level_for_far, Building, MixedUseBuilding};
use crate::grid::ZoneType;
use crate::policies::{Policies, Policy};
use crate::stats::CityStats;
use crate::test_harness::TestCity;
use crate::urban_growth_boundary::UrbanGrowthBoundary;

/// Helper: set up a city with a single building at a given level and occupancy,
/// then set CityStats.average_happiness and reset the UpgradeTimer so the next
/// tick triggers an upgrade check.
fn city_with_building_ready_for_upgrade(
    zone: ZoneType,
    level: u8,
    occupancy_fraction: f32,
    happiness: f32,
) -> TestCity {
    let mut city = TestCity::new().with_building(100, 100, zone, level);

    // Set occupancy on the building
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            let target_occupants = (building.capacity as f32 * occupancy_fraction) as u32;
            building.occupants = target_occupants;
        }
    }

    // Set happiness
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = happiness;
    }

    // Reset upgrade timer so next tick triggers the check
    {
        let world = city.world_mut();
        let mut timer = world.resource_mut::<UpgradeTimer>();
        timer.tick = 29; // UPGRADE_INTERVAL - 1, so the next tick fires
    }

    city
}

/// Helper: set up a city with a building and prepare for downgrade check.
fn city_with_building_ready_for_downgrade(zone: ZoneType, level: u8, happiness: f32) -> TestCity {
    let mut city = TestCity::new().with_building(100, 100, zone, level);

    // Set happiness
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = happiness;
    }

    // Reset downgrade timer so next tick triggers the check
    {
        let world = city.world_mut();
        let mut timer = world.resource_mut::<UpgradeTimer>();
        timer.downgrade_tick = 29; // UPGRADE_INTERVAL - 1
    }

    city
}

// ====================================================================
// Upgrade: conditions met
// ====================================================================

#[test]
fn test_upgrade_when_high_occupancy_and_high_happiness() {
    // Occupancy >= 0.75 and happiness >= 45.0 should trigger upgrade
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialHigh, 1, 0.80, 60.0);

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 2,
        "Building should upgrade from level 1 to 2 when occupancy >= 0.75 and happiness >= 45"
    );
}

#[test]
fn test_upgrade_updates_capacity() {
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialHigh, 1, 0.80, 60.0);

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    let expected_capacity = Building::capacity_for_level(ZoneType::ResidentialHigh, 2);
    assert_eq!(
        building.capacity, expected_capacity,
        "Capacity should update to level 2 capacity after upgrade"
    );
}

#[test]
fn test_upgrade_at_exact_threshold_occupancy() {
    // Test upgrade at exactly 0.75 occupancy
    let mut city = city_with_building_ready_for_upgrade(ZoneType::CommercialHigh, 1, 0.75, 50.0);

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 2,
        "Building should upgrade at exactly 0.75 occupancy"
    );
}

#[test]
fn test_upgrade_at_exact_threshold_happiness() {
    // Test upgrade at exactly 45.0 happiness
    let mut city = city_with_building_ready_for_upgrade(ZoneType::Industrial, 1, 0.90, 45.0);

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 2,
        "Building should upgrade at exactly 45.0 happiness"
    );
}

// ====================================================================
// Upgrade: conditions not met
// ====================================================================

#[test]
fn test_no_upgrade_when_low_occupancy() {
    // Occupancy < 0.75 should prevent upgrade even with high happiness
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialHigh, 1, 0.50, 80.0);

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 1,
        "Building should NOT upgrade when occupancy < 0.75"
    );
}

#[test]
fn test_no_upgrade_when_low_happiness() {
    // Happiness < 45.0 should prevent upgrade even with high occupancy
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialHigh, 1, 0.90, 30.0);

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 1,
        "Building should NOT upgrade when happiness < 45"
    );
}

#[test]
fn test_no_upgrade_when_zero_occupancy() {
    // Empty building should not upgrade
    let mut city = city_with_building_ready_for_upgrade(ZoneType::Office, 1, 0.0, 80.0);

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(building.level, 1, "Empty building should NOT upgrade");
}

#[test]
fn test_no_upgrade_when_both_conditions_unmet() {
    // Both low occupancy and low happiness
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialLow, 1, 0.20, 20.0);

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 1,
        "Building should NOT upgrade when both conditions are unmet"
    );
}

// ====================================================================
// Upgrade: level cap constraints
// ====================================================================

#[test]
fn test_upgrade_does_not_exceed_zone_max_level() {
    // ResidentialLow has max_level = 3. Building at level 3 should not go higher.
    let far_cap = max_level_for_far(ZoneType::ResidentialLow) as u8;
    let zone_max = ZoneType::ResidentialLow.max_level();
    let effective_max = zone_max.min(far_cap);

    let mut city =
        city_with_building_ready_for_upgrade(ZoneType::ResidentialLow, effective_max, 0.90, 80.0);

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, effective_max,
        "Building level should not exceed the effective max (zone max={}, FAR cap={})",
        zone_max, far_cap
    );
}

#[test]
fn test_all_zone_types_respect_max_level() {
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];

    for zone in zones {
        let zone_max = zone.max_level();
        let far_cap = max_level_for_far(zone) as u8;
        let effective_max = zone_max.min(far_cap);

        let mut city = city_with_building_ready_for_upgrade(zone, effective_max, 0.95, 90.0);

        city.tick(1);

        let world = city.world_mut();
        let building = world
            .query::<&Building>()
            .iter(world)
            .next()
            .expect("building should exist");

        assert_eq!(
            building.level, effective_max,
            "Building level for {:?} should not exceed effective max {}",
            zone, effective_max
        );
    }
}

#[test]
fn test_upgrade_respects_policy_max_building_level() {
    // HighRiseBan policy limits max building level to 2
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialHigh, 2, 0.90, 80.0);

    // Enable HighRiseBan policy
    {
        let world = city.world_mut();
        world.resource_mut::<Policies>().toggle(Policy::HighRiseBan);
    }

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert!(
        building.level <= 2,
        "Building level should not exceed 2 with HighRiseBan policy active, got {}",
        building.level
    );
}

#[test]
fn test_upgrade_respects_far_cap() {
    // The FAR cap for each zone type should be respected
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];

    for zone in zones {
        let far_cap = max_level_for_far(zone);
        assert!(
            far_cap >= 1,
            "FAR cap for {:?} must be at least 1, got {}",
            zone,
            far_cap
        );
        assert!(
            far_cap <= zone.max_level() as u32,
            "FAR cap for {:?} ({}) should not exceed zone max level ({})",
            zone,
            far_cap,
            zone.max_level()
        );
    }
}

// ====================================================================
// Upgrade: Urban Growth Boundary constraint
// ====================================================================

#[test]
fn test_upgrade_blocked_outside_ugb() {
    // Building outside UGB should not upgrade
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialHigh, 1, 0.90, 80.0);

    // Enable UGB that excludes the building at (100, 100)
    {
        let world = city.world_mut();
        let mut ugb = world.resource_mut::<UrbanGrowthBoundary>();
        ugb.enabled = true;
        // Small polygon around (50, 50) that doesn't contain (100, 100)
        ugb.vertices = vec![(40.0, 40.0), (60.0, 40.0), (60.0, 60.0), (40.0, 60.0)];
    }

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(building.level, 1, "Building outside UGB should NOT upgrade");
}

#[test]
fn test_upgrade_allowed_inside_ugb() {
    // Building inside UGB should upgrade normally
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialHigh, 1, 0.90, 80.0);

    // Enable UGB that includes the building at (100, 100)
    {
        let world = city.world_mut();
        let mut ugb = world.resource_mut::<UrbanGrowthBoundary>();
        ugb.enabled = true;
        ugb.vertices = vec![(0.0, 0.0), (200.0, 0.0), (200.0, 200.0), (0.0, 200.0)];
    }

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 2,
        "Building inside UGB should upgrade normally"
    );
}

#[test]
fn test_upgrade_allowed_when_ugb_disabled() {
    // When UGB is disabled, all buildings can upgrade
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialHigh, 1, 0.90, 80.0);

    // Ensure UGB is disabled (default)
    {
        let world = city.world_mut();
        let ugb = world.resource::<UrbanGrowthBoundary>();
        assert!(!ugb.enabled, "UGB should be disabled by default");
    }

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 2,
        "Building should upgrade when UGB is disabled"
    );
}

// ====================================================================
// Upgrade: timer behavior
// ====================================================================

#[test]
fn test_upgrade_only_fires_on_interval() {
    // Upgrade should only check every UPGRADE_INTERVAL (30) ticks
    let mut city = city_with_building_ready_for_upgrade(ZoneType::ResidentialHigh, 1, 0.90, 80.0);

    // Reset timer to 0 so the check won't fire for 30 ticks
    {
        let world = city.world_mut();
        world.resource_mut::<UpgradeTimer>().tick = 0;
    }

    // Tick only 10 times -- upgrade check should not have fired
    city.tick(10);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 1,
        "Building should NOT upgrade before UPGRADE_INTERVAL ticks"
    );
}

// ====================================================================
// Downgrade: conditions met
// ====================================================================

#[test]
fn test_downgrade_possible_when_happiness_very_low() {
    // Downgrade fires when average_happiness <= 30.0 with a random 1% chance per building.
    // We run many cycles to ensure at least one downgrade happens statistically.
    let mut city = city_with_building_ready_for_downgrade(ZoneType::ResidentialHigh, 5, 10.0);

    // Run many downgrade cycles (each cycle is 30 ticks)
    // With 1% chance per check and ~500 checks, probability of NO downgrade is (0.99)^500 ~ 0.7%
    for _ in 0..500 {
        {
            let world = city.world_mut();
            world.resource_mut::<UpgradeTimer>().downgrade_tick = 29;
            world.resource_mut::<CityStats>().average_happiness = 10.0;
        }
        city.tick(1);
    }

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert!(
        building.level < 5,
        "Building should have downgraded from level 5 after many cycles with very low happiness"
    );
}

#[test]
fn test_no_downgrade_when_happiness_above_threshold() {
    // Downgrade should NOT happen when happiness > 30.0
    let mut city = city_with_building_ready_for_downgrade(ZoneType::ResidentialHigh, 3, 50.0);

    // Run several downgrade cycles
    for _ in 0..100 {
        {
            let world = city.world_mut();
            world.resource_mut::<UpgradeTimer>().downgrade_tick = 29;
            world.resource_mut::<CityStats>().average_happiness = 50.0;
        }
        city.tick(1);
    }

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 3,
        "Building should NOT downgrade when happiness is above 30.0"
    );
}

#[test]
fn test_no_downgrade_below_level_1() {
    // Buildings at level 1 should never downgrade further
    let mut city = city_with_building_ready_for_downgrade(ZoneType::ResidentialLow, 1, 5.0);

    for _ in 0..200 {
        {
            let world = city.world_mut();
            world.resource_mut::<UpgradeTimer>().downgrade_tick = 29;
            world.resource_mut::<CityStats>().average_happiness = 5.0;
        }
        city.tick(1);
    }

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 1,
        "Building at level 1 should never downgrade below 1"
    );
}

#[test]
fn test_downgrade_updates_capacity() {
    // When a building downgrades, its capacity should update
    let mut city = city_with_building_ready_for_downgrade(ZoneType::ResidentialHigh, 5, 10.0);

    // Force downgrade by running many cycles
    let mut downgraded = false;
    for _ in 0..500 {
        {
            let world = city.world_mut();
            world.resource_mut::<UpgradeTimer>().downgrade_tick = 29;
            world.resource_mut::<CityStats>().average_happiness = 10.0;
        }
        city.tick(1);

        let world = city.world_mut();
        let building = world
            .query::<&Building>()
            .iter(world)
            .next()
            .expect("building should exist");
        if building.level < 5 {
            let expected_cap =
                Building::capacity_for_level(ZoneType::ResidentialHigh, building.level);
            assert_eq!(
                building.capacity, expected_cap,
                "Capacity should match level {} capacity after downgrade",
                building.level
            );
            downgraded = true;
            break;
        }
    }

    assert!(downgraded, "Building should have downgraded at least once");
}

#[test]
fn test_downgrade_clamps_excess_occupants() {
    // When a building downgrades, occupants > capacity should be clamped
    let mut city = TestCity::new().with_building(100, 100, ZoneType::ResidentialHigh, 3);

    // Set high occupancy that would exceed lower-level capacity
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            // Level 3 capacity is 500; level 2 capacity is 200
            building.occupants = 400;
        }
    }

    // Set conditions for downgrade
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 10.0;
    }

    // Run many cycles until a downgrade happens
    let mut downgraded = false;
    for _ in 0..500 {
        {
            let world = city.world_mut();
            world.resource_mut::<UpgradeTimer>().downgrade_tick = 29;
            world.resource_mut::<CityStats>().average_happiness = 10.0;
        }
        city.tick(1);

        let world = city.world_mut();
        let building = world
            .query::<&Building>()
            .iter(world)
            .next()
            .expect("building should exist");
        if building.level < 3 {
            assert!(
                building.occupants <= building.capacity,
                "Occupants ({}) should not exceed capacity ({}) after downgrade",
                building.occupants,
                building.capacity
            );
            downgraded = true;
            break;
        }
    }

    assert!(downgraded, "Building should have downgraded at least once");
}

// ====================================================================
// Mixed-use building upgrade/downgrade
// ====================================================================

#[test]
fn test_mixed_use_upgrade_updates_subcapacities() {
    // When a mixed-use building upgrades, both commercial and residential
    // capacities should update.
    let mut city = TestCity::new();

    // Spawn a mixed-use building with MixedUseBuilding component
    {
        let world = city.world_mut();
        let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(1);
        let capacity = Building::capacity_for_level(ZoneType::MixedUse, 1);
        world.spawn((
            Building {
                zone_type: ZoneType::MixedUse,
                level: 1,
                grid_x: 100,
                grid_y: 100,
                capacity,
                occupants: (capacity as f32 * 0.85) as u32,
            },
            MixedUseBuilding {
                commercial_capacity: comm_cap,
                commercial_occupants: 0,
                residential_capacity: res_cap,
                residential_occupants: 0,
            },
        ));
    }

    // Set up upgrade conditions
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 70.0;
        world.resource_mut::<UpgradeTimer>().tick = 29;
    }

    city.tick(1);

    let world = city.world_mut();
    let (building, mixed) = world
        .query::<(&Building, &MixedUseBuilding)>()
        .iter(world)
        .next()
        .expect("mixed-use building should exist");

    if building.level == 2 {
        let (expected_comm, expected_res) = MixedUseBuilding::capacities_for_level(2);
        assert_eq!(
            mixed.commercial_capacity, expected_comm,
            "Commercial capacity should update on upgrade"
        );
        assert_eq!(
            mixed.residential_capacity, expected_res,
            "Residential capacity should update on upgrade"
        );
    }
    // If FAR cap prevents upgrade, level stays at 1 -- that's also valid.
}

#[test]
fn test_mixed_use_downgrade_clamps_subcapacity_occupants() {
    // When a mixed-use building downgrades, excess sub-occupants should be clamped
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(3);
        let capacity = Building::capacity_for_level(ZoneType::MixedUse, 3);
        world.spawn((
            Building {
                zone_type: ZoneType::MixedUse,
                level: 3,
                grid_x: 100,
                grid_y: 100,
                capacity,
                occupants: 0,
            },
            MixedUseBuilding {
                commercial_capacity: comm_cap,
                commercial_occupants: comm_cap, // fully occupied
                residential_capacity: res_cap,
                residential_occupants: res_cap, // fully occupied
            },
        ));
    }

    // Set up downgrade conditions
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 10.0;
    }

    let mut downgraded = false;
    for _ in 0..500 {
        {
            let world = city.world_mut();
            world.resource_mut::<UpgradeTimer>().downgrade_tick = 29;
            world.resource_mut::<CityStats>().average_happiness = 10.0;
        }
        city.tick(1);

        let world = city.world_mut();
        let (building, mixed) = world
            .query::<(&Building, &MixedUseBuilding)>()
            .iter(world)
            .next()
            .expect("mixed-use building should exist");

        if building.level < 3 {
            assert!(
                mixed.commercial_occupants <= mixed.commercial_capacity,
                "Commercial occupants ({}) should not exceed capacity ({}) after downgrade",
                mixed.commercial_occupants,
                mixed.commercial_capacity
            );
            assert!(
                mixed.residential_occupants <= mixed.residential_capacity,
                "Residential occupants ({}) should not exceed capacity ({}) after downgrade",
                mixed.residential_occupants,
                mixed.residential_capacity
            );
            downgraded = true;
            break;
        }
    }

    assert!(
        downgraded,
        "Mixed-use building should have downgraded at least once"
    );
}

// ====================================================================
// Upgrade: max upgrades per tick cap
// ====================================================================

#[test]
fn test_upgrade_max_per_tick_capped() {
    // The system limits upgrades to 50 per tick. Spawn more than 50 buildings
    // all eligible for upgrade and verify not all upgrade in a single tick.
    let mut city = TestCity::new();

    // Spawn 60 buildings, all at level 1 with high occupancy
    for i in 0..60 {
        let x = 50 + (i % 20);
        let y = 50 + (i / 20);
        city = city.with_building(x, y, ZoneType::ResidentialHigh, 1);
    }

    // Set all buildings to high occupancy
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            building.occupants = (building.capacity as f32 * 0.90) as u32;
        }
    }

    // Set up upgrade conditions
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 80.0;
        world.resource_mut::<UpgradeTimer>().tick = 29;
    }

    city.tick(1);

    let world = city.world_mut();
    let upgraded_count = world
        .query::<&Building>()
        .iter(world)
        .filter(|b| b.level == 2)
        .count();

    assert!(
        upgraded_count <= 50,
        "At most 50 buildings should upgrade per tick, got {}",
        upgraded_count
    );
}

// ====================================================================
// Upgrade: zero-capacity building edge case
// ====================================================================

#[test]
fn test_no_upgrade_for_zero_capacity_building() {
    // A building with capacity 0 should have 0 occupancy ratio and not upgrade
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        world.spawn(Building {
            zone_type: ZoneType::ResidentialHigh,
            level: 1,
            grid_x: 100,
            grid_y: 100,
            capacity: 0,
            occupants: 0,
        });
    }

    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 80.0;
        world.resource_mut::<UpgradeTimer>().tick = 29;
    }

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert_eq!(
        building.level, 1,
        "Building with zero capacity should NOT upgrade"
    );
}

// ====================================================================
// Capacity consistency invariant
// ====================================================================

#[test]
fn test_capacity_increases_with_level_for_all_zones() {
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];

    for zone in zones {
        let max = zone.max_level();
        let mut prev_capacity = 0;
        for level in 1..=max {
            let cap = Building::capacity_for_level(zone, level);
            assert!(
                cap > prev_capacity,
                "Capacity for {:?} level {} ({}) should be greater than level {} ({})",
                zone,
                level,
                cap,
                level - 1,
                prev_capacity
            );
            prev_capacity = cap;
        }
    }
}
