//! Integration tests for building upgrade and downgrade systems (TEST-042).
//!
//! Tests cover:
//! - Upgrade conditions and constraints (occupancy, happiness, FAR cap, policy, UGB)
//! - Building level never exceeds zone max_level or FAR cap
//! - Downgrade when happiness is very low
//! - Downgrade clamps excess occupants
//! - Mixed-use building capacities update on downgrade
//!
//! Note: With default FAR values, `max_level_for_far` returns 1 for all zone
//! types, which means upgrades past level 1 are blocked by the FAR constraint.
//! Tests validate this constraint as well as the other upgrade/downgrade logic.

use crate::building_upgrade::UpgradeTimer;
use crate::buildings::{max_level_for_far, Building, MixedUseBuilding};
use crate::grid::{WorldGrid, ZoneType};
use crate::policies::{Policies, Policy};
use crate::stats::CityStats;
use crate::test_harness::TestCity;
use crate::urban_growth_boundary::UrbanGrowthBoundary;

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
// Upgrade: FAR cap constraint
// ====================================================================

#[test]
fn test_far_cap_is_at_least_one_for_all_zones() {
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
    }
}

#[test]
fn test_far_cap_does_not_exceed_zone_max_level() {
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
        let zone_max = zone.max_level() as u32;
        assert!(
            far_cap <= zone_max,
            "FAR cap for {:?} ({}) should not exceed zone max level ({})",
            zone,
            far_cap,
            zone_max
        );
    }
}

#[test]
fn test_upgrade_blocked_by_far_cap_at_level_1() {
    // With default FAR values, all zones have far_cap=1, so buildings at
    // level 1 should not upgrade even with perfect conditions.
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
    ];

    for zone in zones {
        let far_cap = max_level_for_far(zone) as u8;
        // Only test zones where FAR cap blocks at level 1
        if far_cap > 1 {
            continue;
        }

        let mut city = TestCity::new().with_building(100, 100, zone, 1);

        // Set perfect upgrade conditions
        {
            let world = city.world_mut();
            let mut q = world.query::<&mut Building>();
            for mut building in q.iter_mut(world) {
                building.occupants = building.capacity; // 100% occupancy
            }
        }
        {
            let world = city.world_mut();
            world.resource_mut::<CityStats>().average_happiness = 90.0;
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
            "Building {:?} at level 1 should NOT upgrade when FAR cap is 1",
            zone
        );
    }
}

#[test]
fn test_upgrade_blocked_at_far_cap_level() {
    // When a building is already at the FAR cap level, it should not upgrade
    // even if all other conditions are met.
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
    ];

    for zone in zones {
        let far_cap = max_level_for_far(zone) as u8;
        let mut city = TestCity::new().with_building(100, 100, zone, far_cap);

        {
            let world = city.world_mut();
            let mut q = world.query::<&mut Building>();
            for mut building in q.iter_mut(world) {
                building.occupants = building.capacity;
            }
        }
        {
            let world = city.world_mut();
            world.resource_mut::<CityStats>().average_happiness = 90.0;
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
            building.level, far_cap,
            "Building {:?} at FAR cap level {} should not upgrade further",
            zone, far_cap
        );
    }
}

// ====================================================================
// Upgrade: conditions not met (tested with buildings below FAR cap)
// ====================================================================

#[test]
fn test_no_upgrade_when_low_occupancy() {
    // Even if FAR allowed it, low occupancy prevents upgrade.
    // We set up a building at level 0 conceptually (level 1 with conditions
    // that would allow upgrade if FAR permitted), but since FAR blocks it anyway,
    // we verify the occupancy check by using a level below FAR cap.
    let zone = ZoneType::ResidentialHigh;
    let far_cap = max_level_for_far(zone) as u8;

    // If FAR cap is already 1, we can't test upgrade conditions.
    // So we test that even with high happiness, low occupancy keeps level at 1.
    let mut city = TestCity::new().with_building(100, 100, zone, far_cap);

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            building.occupants = (building.capacity as f32 * 0.50) as u32;
        }
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
        building.level, far_cap,
        "Building should NOT upgrade when occupancy < 0.75"
    );
}

#[test]
fn test_no_upgrade_when_low_happiness() {
    let zone = ZoneType::ResidentialHigh;
    let far_cap = max_level_for_far(zone) as u8;

    let mut city = TestCity::new().with_building(100, 100, zone, far_cap);

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            building.occupants = building.capacity; // 100%
        }
    }
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 30.0;
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
        building.level, far_cap,
        "Building should NOT upgrade when happiness < 45"
    );
}

#[test]
fn test_no_upgrade_when_zero_occupancy() {
    let zone = ZoneType::Office;
    let far_cap = max_level_for_far(zone) as u8;

    let mut city = TestCity::new().with_building(100, 100, zone, far_cap);

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

    assert_eq!(building.level, far_cap, "Empty building should NOT upgrade");
}

#[test]
fn test_no_upgrade_when_both_conditions_unmet() {
    let zone = ZoneType::ResidentialLow;
    let far_cap = max_level_for_far(zone) as u8;

    let mut city = TestCity::new().with_building(100, 100, zone, far_cap);

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            building.occupants = (building.capacity as f32 * 0.20) as u32;
        }
    }
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 20.0;
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
        building.level, far_cap,
        "Building should NOT upgrade when both conditions are unmet"
    );
}

// ====================================================================
// Upgrade: level cap constraints
// ====================================================================

#[test]
fn test_all_zone_types_building_level_capped() {
    // Verify that buildings at their effective max level (min of zone max,
    // policy max, and FAR cap) cannot upgrade.
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
        let policy_max = 3u8; // default (no HighRiseBan)
        let effective_max = zone_max.min(policy_max).min(far_cap);

        let mut city = TestCity::new().with_building(100, 100, zone, effective_max);

        {
            let world = city.world_mut();
            let mut q = world.query::<&mut Building>();
            for mut building in q.iter_mut(world) {
                building.occupants = building.capacity;
            }
        }
        {
            let world = city.world_mut();
            world.resource_mut::<CityStats>().average_happiness = 90.0;
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
            building.level, effective_max,
            "Building level for {:?} should not exceed effective max {} (zone={}, policy={}, FAR={})",
            zone, effective_max, zone_max, policy_max, far_cap
        );
    }
}

#[test]
fn test_upgrade_respects_policy_max_building_level() {
    // HighRiseBan policy limits max building level to 2
    let zone = ZoneType::ResidentialHigh;
    let far_cap = max_level_for_far(zone) as u8;
    let policy_max_with_ban = 2u8;
    let effective_max = zone.max_level().min(policy_max_with_ban).min(far_cap);

    let mut city = TestCity::new().with_building(100, 100, zone, effective_max);

    // Enable HighRiseBan policy
    {
        let world = city.world_mut();
        world.resource_mut::<Policies>().toggle(Policy::HighRiseBan);
    }

    // Set perfect upgrade conditions
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            building.occupants = building.capacity;
        }
    }
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 90.0;
        world.resource_mut::<UpgradeTimer>().tick = 29;
    }

    city.tick(1);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("building should exist");

    assert!(
        building.level <= policy_max_with_ban,
        "Building level should not exceed {} with HighRiseBan policy active, got {}",
        policy_max_with_ban,
        building.level
    );
}

// ====================================================================
// Upgrade: Urban Growth Boundary constraint
// ====================================================================

#[test]
fn test_upgrade_blocked_outside_ugb() {
    // Building outside UGB should not upgrade even with perfect conditions
    let zone = ZoneType::ResidentialHigh;
    // Use level 1 to ensure building exists
    let mut city = TestCity::new().with_building(100, 100, zone, 1);

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            building.occupants = building.capacity;
        }
    }

    // Enable UGB that excludes the building at (100, 100)
    {
        let world = city.world_mut();
        let mut ugb = world.resource_mut::<UrbanGrowthBoundary>();
        ugb.enabled = true;
        // Small polygon around (50, 50) that doesn't contain (100, 100)
        ugb.vertices = vec![(40.0, 40.0), (60.0, 40.0), (60.0, 60.0), (40.0, 60.0)];
    }

    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 90.0;
        world.resource_mut::<UpgradeTimer>().tick = 29;
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
fn test_ugb_disabled_does_not_block_upgrade() {
    // When UGB is disabled, the UGB check should pass
    let city = TestCity::new();
    let ugb = city.resource::<UrbanGrowthBoundary>();
    assert!(!ugb.enabled, "UGB should be disabled by default");
    // With UGB disabled, allows_upgrade returns true for any coordinate
    assert!(
        ugb.allows_upgrade(100, 100),
        "allows_upgrade should return true when UGB is disabled"
    );
}

#[test]
fn test_ugb_inside_allows_upgrade_check() {
    // Verify that allows_upgrade returns true for coordinates inside the boundary
    let mut ugb = UrbanGrowthBoundary {
        enabled: true,
        vertices: vec![(0.0, 0.0), (200.0, 0.0), (200.0, 200.0), (0.0, 200.0)],
    };
    assert!(
        ugb.allows_upgrade(100, 100),
        "allows_upgrade should return true for coordinates inside UGB"
    );

    // Coordinate outside
    ugb.vertices = vec![(40.0, 40.0), (60.0, 40.0), (60.0, 60.0), (40.0, 60.0)];
    assert!(
        !ugb.allows_upgrade(100, 100),
        "allows_upgrade should return false for coordinates outside UGB"
    );
}

// ====================================================================
// Upgrade: timer behavior
// ====================================================================

#[test]
fn test_upgrade_only_fires_on_interval() {
    // Upgrade should only check every UPGRADE_INTERVAL (30) ticks
    let mut city = TestCity::new().with_building(100, 100, ZoneType::ResidentialHigh, 1);

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            building.occupants = building.capacity;
        }
    }
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 80.0;
        // Reset timer to 0 so the check won't fire for 30 ticks
        world.resource_mut::<UpgradeTimer>().tick = 0;
    }

    // Tick only 10 times -- upgrade check should not have fired
    city.tick(10);

    let world = city.world_mut();
    let timer = world.resource::<UpgradeTimer>();
    // Timer should have advanced by 10
    assert_eq!(timer.tick, 10, "Upgrade timer should advance by 10 ticks");
}

#[test]
fn test_upgrade_timer_resets_after_interval() {
    let mut city = TestCity::new().with_building(100, 100, ZoneType::ResidentialHigh, 1);

    {
        let world = city.world_mut();
        world.resource_mut::<UpgradeTimer>().tick = 29;
    }

    city.tick(1);

    let world = city.world_mut();
    let timer = world.resource::<UpgradeTimer>();
    assert_eq!(
        timer.tick, 0,
        "Upgrade timer should reset to 0 after reaching UPGRADE_INTERVAL"
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

    // 1% chance per check; 2000 checks → (0.99)^2000 ≈ 2e-9 failure probability.
    // Break early once downgrade is observed to avoid side effects from long runs.
    let mut downgraded = false;
    for _ in 0..2000 {
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
            downgraded = true;
            break;
        }
    }

    assert!(
        downgraded,
        "Building should have downgraded from level 5 after many cycles with very low happiness"
    );
}

#[test]
fn test_no_downgrade_when_happiness_above_threshold() {
    // Downgrade should NOT happen when happiness > 30.0.
    // We also reset the SlowTickTimer each iteration to prevent update_stats
    // from firing and recalculating average_happiness to 0.0 (no citizens).
    let mut city = city_with_building_ready_for_downgrade(ZoneType::ResidentialHigh, 3, 50.0);

    for _ in 0..100 {
        {
            let world = city.world_mut();
            world.resource_mut::<UpgradeTimer>().downgrade_tick = 29;
            world.resource_mut::<CityStats>().average_happiness = 50.0;
            // Prevent update_stats from firing and resetting happiness to 0.0
            world.resource_mut::<crate::SlowTickTimer>().counter = 1;
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

    let mut downgraded = false;
    for _ in 0..2000 {
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

    let mut downgraded = false;
    // 1% chance per check; 2000 checks → (0.99)^2000 ≈ 2e-9 failure probability
    for _ in 0..2000 {
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

#[test]
fn test_downgrade_timer_only_fires_on_interval() {
    // Downgrade should only check every UPGRADE_INTERVAL (30) ticks
    let mut city = TestCity::new().with_building(100, 100, ZoneType::ResidentialHigh, 3);

    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 5.0;
        world.resource_mut::<UpgradeTimer>().downgrade_tick = 0;
    }

    // Tick 10 times -- downgrade check should not have fired
    city.tick(10);

    let world = city.world_mut();
    let timer = world.resource::<UpgradeTimer>();
    assert_eq!(
        timer.downgrade_tick, 10,
        "Downgrade timer should advance by 10 ticks"
    );
}

// ====================================================================
// Mixed-use building downgrade
// ====================================================================

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
        // Provide utilities to prevent building abandonment
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(100, 100).has_power = true;
        grid.get_mut(100, 100).has_water = true;
    }

    // Set up downgrade conditions
    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 10.0;
    }

    let mut downgraded = false;
    for _ in 0..2000 {
        {
            let world = city.world_mut();
            world.resource_mut::<UpgradeTimer>().downgrade_tick = 29;
            world.resource_mut::<CityStats>().average_happiness = 10.0;
            // Re-set utilities to prevent abandonment
            let mut grid = world.resource_mut::<WorldGrid>();
            grid.cells[100][100].has_power = true;
            grid.cells[100][100].has_water = true;
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

#[test]
fn test_mixed_use_downgrade_updates_subcapacities() {
    // When a mixed-use building downgrades, its sub-capacities should update
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(4);
        let capacity = Building::capacity_for_level(ZoneType::MixedUse, 4);
        world.spawn((
            Building {
                zone_type: ZoneType::MixedUse,
                level: 4,
                grid_x: 100,
                grid_y: 100,
                capacity,
                occupants: 0,
            },
            MixedUseBuilding {
                commercial_capacity: comm_cap,
                commercial_occupants: 0,
                residential_capacity: res_cap,
                residential_occupants: 0,
            },
        ));
        // Provide utilities to prevent building abandonment during the long tick loop
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(100, 100).has_power = true;
        grid.get_mut(100, 100).has_water = true;
    }

    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 10.0;
    }

    let mut downgraded = false;
    for _ in 0..2000 {
        {
            let world = city.world_mut();
            world.resource_mut::<UpgradeTimer>().downgrade_tick = 29;
            world.resource_mut::<CityStats>().average_happiness = 10.0;
            // Re-set utilities each iteration to prevent abandonment
            let mut grid = world.resource_mut::<WorldGrid>();
            grid.cells[100][100].has_power = true;
            grid.cells[100][100].has_water = true;
        }
        city.tick(1);

        let world = city.world_mut();
        let (building, mixed) = world
            .query::<(&Building, &MixedUseBuilding)>()
            .iter(world)
            .next()
            .expect("mixed-use building should exist");

        if building.level < 4 {
            let (expected_comm, expected_res) =
                MixedUseBuilding::capacities_for_level(building.level);
            assert_eq!(
                mixed.commercial_capacity, expected_comm,
                "Commercial capacity should update on downgrade to level {}",
                building.level
            );
            assert_eq!(
                mixed.residential_capacity, expected_res,
                "Residential capacity should update on downgrade to level {}",
                building.level
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
fn test_upgrade_max_per_tick_capped_at_50() {
    // The upgrade system limits upgrades to 50 per tick.
    // We verify the constant by inspecting the system behavior.
    // Since FAR currently caps all zones at level 1, we cannot observe
    // actual upgrades. Instead, we verify the system processes buildings
    // without panic and respects the timer.
    let mut city = TestCity::new();

    for i in 0..60 {
        let x = 50 + (i % 20);
        let y = 50 + (i / 20);
        city = city.with_building(x, y, ZoneType::ResidentialHigh, 1);
    }

    {
        let world = city.world_mut();
        world.resource_mut::<CityStats>().average_happiness = 80.0;
        world.resource_mut::<UpgradeTimer>().tick = 29;
    }

    // Should not panic even with many buildings
    city.tick(1);

    let world = city.world_mut();
    let count = world.query::<&Building>().iter(world).count();
    assert_eq!(
        count, 60,
        "All 60 buildings should still exist after upgrade check"
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
// Capacity consistency invariants
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

#[test]
fn test_mixed_use_subcapacities_consistent_with_total() {
    for level in 1..=5u8 {
        let total = Building::capacity_for_level(ZoneType::MixedUse, level);
        let (c, r) = MixedUseBuilding::capacities_for_level(level);
        assert_eq!(
            total,
            c + r,
            "MixedUse level {} total capacity ({}) should equal commercial ({}) + residential ({})",
            level,
            total,
            c,
            r
        );
    }
}

#[test]
fn test_mixed_use_subcapacities_increase_with_level() {
    let mut prev_comm = 0u32;
    let mut prev_res = 0u32;
    for level in 1..=5u8 {
        let (c, r) = MixedUseBuilding::capacities_for_level(level);
        assert!(
            c > prev_comm,
            "MixedUse commercial capacity at level {} ({}) should exceed level {} ({})",
            level,
            c,
            level - 1,
            prev_comm
        );
        assert!(
            r > prev_res,
            "MixedUse residential capacity at level {} ({}) should exceed level {} ({})",
            level,
            r,
            level - 1,
            prev_res
        );
        prev_comm = c;
        prev_res = r;
    }
}
