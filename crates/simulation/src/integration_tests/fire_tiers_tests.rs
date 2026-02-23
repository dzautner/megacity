//! Integration tests for the fire service multi-tier system (SVC-004).
//!
//! Tests that:
//! - Fire tier coverage grid correctly maps service types to tiers
//! - Higher-tier stations provide faster fire suppression
//! - FireTiersState tracks extinguishment stats
//! - Coverage grid respects the "highest tier wins" rule

use bevy::prelude::*;

use crate::buildings::Building;
use crate::fire::{FireGrid, OnFire};
use crate::fire_tiers::{FireTier, FireTierCoverageGrid, FireTiersState};
use crate::grid::ZoneType;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Test 1: FireHouse places Small tier coverage
// ====================================================================

#[test]
fn test_fire_house_provides_small_tier_coverage() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::FireHouse);

    // Tick to let the coverage system run.
    city.tick(2);

    let world = city.world_mut();
    let tier_grid = world.resource::<FireTierCoverageGrid>();
    // The service building itself should have Small coverage.
    assert_eq!(
        tier_grid.get(80, 80),
        Some(FireTier::Small),
        "FireHouse should provide Small tier coverage at its location"
    );
}

// ====================================================================
// Test 2: FireStation places Standard tier coverage
// ====================================================================

#[test]
fn test_fire_station_provides_standard_tier_coverage() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::FireStation);

    city.tick(2);

    let world = city.world_mut();
    let tier_grid = world.resource::<FireTierCoverageGrid>();
    assert_eq!(
        tier_grid.get(80, 80),
        Some(FireTier::Standard),
        "FireStation should provide Standard tier coverage"
    );
}

// ====================================================================
// Test 3: FireHQ places Headquarters tier coverage
// ====================================================================

#[test]
fn test_fire_hq_provides_headquarters_tier_coverage() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::FireHQ);

    city.tick(2);

    let world = city.world_mut();
    let tier_grid = world.resource::<FireTierCoverageGrid>();
    assert_eq!(
        tier_grid.get(80, 80),
        Some(FireTier::Headquarters),
        "FireHQ should provide Headquarters tier coverage"
    );
}

// ====================================================================
// Test 4: Higher tier wins when coverage overlaps
// ====================================================================

#[test]
fn test_highest_tier_wins_on_overlap() {
    // Place a FireHouse and FireHQ near each other so coverage overlaps.
    let mut city = TestCity::new()
        .with_service(80, 80, ServiceType::FireHouse)
        .with_service(82, 80, ServiceType::FireHQ);

    city.tick(2);

    let world = city.world_mut();
    let tier_grid = world.resource::<FireTierCoverageGrid>();
    // Cell 81,80 should be in range of both. Headquarters should dominate.
    assert_eq!(
        tier_grid.get(81, 80),
        Some(FireTier::Headquarters),
        "Overlapping coverage should use the highest tier"
    );
}

// ====================================================================
// Test 5: Building counts tracked in state
// ====================================================================

#[test]
fn test_fire_tier_building_counts() {
    let mut city = TestCity::new()
        .with_service(40, 40, ServiceType::FireHouse)
        .with_service(80, 80, ServiceType::FireStation)
        .with_service(120, 120, ServiceType::FireHQ);

    city.tick(2);

    let world = city.world_mut();
    let state = world.resource::<FireTiersState>();
    assert_eq!(state.count_small, 1);
    assert_eq!(state.count_standard, 1);
    assert_eq!(state.count_hq, 1);
}

// ====================================================================
// Test 6: Tier-based suppression reduces fire intensity
// ====================================================================

#[test]
fn test_tier_based_suppression_reduces_intensity() {
    let mut city = TestCity::new()
        .with_service(80, 80, ServiceType::FireHQ)
        .with_building(80, 81, ZoneType::Industrial, 1);

    city.tick(2);

    // Manually ignite the building.
    {
        let world = city.world_mut();
        let mut q = world.query::<(Entity, &Building)>();
        let entities: Vec<(Entity, usize, usize)> = q
            .iter(world)
            .map(|(e, b)| (e, b.grid_x, b.grid_y))
            .collect();
        for (e, gx, gy) in entities {
            if gx == 80 && gy == 81 {
                world.entity_mut(e).insert(OnFire {
                    intensity: 50.0,
                    ticks_burning: 0,
                });
            }
        }
    }

    // Run a few ticks -- HQ suppression rate is 4.0/tick.
    city.tick(5);

    // Check that fire intensity has decreased.
    {
        let world = city.world_mut();
        let fire_grid = world.resource::<FireGrid>();
        let intensity = fire_grid.get(80, 81);
        // After 5 ticks at 4.0/tick = 20.0 reduction from the tier system.
        // Plus the base extinguish_fires also runs at 2.0/tick = 10.0.
        // Combined: at least 30.0 reduction from 50.0 => should be <= 20.
        assert!(
            intensity <= 20,
            "Fire intensity should be significantly reduced by HQ suppression; got {}",
            intensity
        );
    }
}

// ====================================================================
// Test 7: No coverage means no tier-based suppression
// ====================================================================

#[test]
fn test_no_fire_service_no_tier_coverage() {
    let mut city = TestCity::new();

    city.tick(2);

    let world = city.world_mut();
    let tier_grid = world.resource::<FireTierCoverageGrid>();
    // No fire service placed => no coverage anywhere.
    assert_eq!(
        tier_grid.get(128, 128),
        None,
        "No fire service should mean no tier coverage"
    );
}

// ====================================================================
// Test 8: Suppression stats accumulate
// ====================================================================

#[test]
fn test_suppression_stats_accumulate() {
    let mut city = TestCity::new()
        .with_service(80, 80, ServiceType::FireStation)
        .with_building(80, 81, ZoneType::Industrial, 1);

    city.tick(2);

    // Ignite the building.
    {
        let world = city.world_mut();
        let mut q = world.query::<(Entity, &Building)>();
        let entities: Vec<(Entity, usize, usize)> = q
            .iter(world)
            .map(|(e, b)| (e, b.grid_x, b.grid_y))
            .collect();
        for (e, gx, gy) in entities {
            if gx == 80 && gy == 81 {
                world.entity_mut(e).insert(OnFire {
                    intensity: 10.0,
                    ticks_burning: 0,
                });
            }
        }
    }

    city.tick(10);

    let world = city.world_mut();
    let state = world.resource::<FireTiersState>();
    // The fire should have been extinguished (10.0 intensity, 2.0/tick = 5 ticks).
    assert!(
        state.extinguished_by_standard >= 1,
        "Standard tier should have extinguished at least one fire"
    );
}
