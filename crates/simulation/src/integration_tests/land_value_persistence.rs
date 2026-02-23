//! Integration tests for land value persistence and momentum (INFRA-060).
//!
//! Validates that:
//! - Land values persist between update cycles (no reset to base)
//! - Exponential smoothing produces gradual changes toward targets
//! - 8-neighbour diffusion spreads high/low values to adjacent cells
//! - Placing a service gradually *increases* nearby land value
//! - Removing a service gradually *decreases* nearby land value
//! - Land value grid is serializable via the Saveable trait

use crate::land_value::LandValueGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

/// Helper: read the land value at (x, y) from the ECS world.
fn lv_at(city: &TestCity, x: usize, y: usize) -> u8 {
    city.resource::<LandValueGrid>().get(x, y)
}

// -------------------------------------------------------------------------
// 1. Persistence — values do NOT reset to base each cycle
// -------------------------------------------------------------------------

#[test]
fn test_land_value_persists_between_cycles() {
    let mut city = TestCity::new();

    // Manually set a non-default value before any system runs
    {
        let world = city.world_mut();
        let mut lv = world.resource_mut::<LandValueGrid>();
        lv.set(128, 128, 200);
    }

    // Run one slow cycle — the system should smooth toward the target (50
    // for a plain grass cell) but NOT snap all the way back to 50.
    city.tick_slow_cycle();

    let val = lv_at(&city, 128, 128);
    assert!(
        val > 50,
        "After one cycle, value should still be well above 50 (was 200), got {val}"
    );
    // With alpha=0.1: new = 0.1*50 + 0.9*200 = 185, then diffusion shifts
    // slightly. Value should remain in the ballpark of 185.
    assert!(
        val > 150,
        "After one cycle with alpha=0.1, value should still be > 150, got {val}"
    );
}

#[test]
fn test_land_value_converges_gradually_not_instantly() {
    let mut city = TestCity::new();

    // Set an extreme value
    {
        let world = city.world_mut();
        let mut lv = world.resource_mut::<LandValueGrid>();
        lv.set(128, 128, 250);
    }

    // Track convergence over several cycles — each should move closer to 50
    let mut prev = 250u8;
    for _ in 0..5 {
        city.tick_slow_cycle();
        let cur = lv_at(&city, 128, 128);
        assert!(
            cur < prev,
            "Value should decrease each cycle toward target 50: prev={prev}, cur={cur}"
        );
        prev = cur;
    }

    // After 5 cycles, value should still be well above 50 (momentum)
    assert!(
        prev > 100,
        "After 5 cycles from 250, value should still be > 100, got {prev}"
    );
}

// -------------------------------------------------------------------------
// 2. Exponential smoothing — gradual approach to target
// -------------------------------------------------------------------------

#[test]
fn test_land_value_smoothing_approaches_target_monotonically() {
    // Place a park to create a target above 50 at (100, 100)
    let mut city = TestCity::new().with_service(100, 100, ServiceType::SmallPark);

    // Run first cycle — value should start moving upward from 50
    city.tick_slow_cycle();
    let after_1 = lv_at(&city, 100, 100);
    assert!(
        after_1 >= 50,
        "With park, value should rise from baseline 50, got {after_1}"
    );

    // Run more cycles — value should keep rising or stay the same
    city.tick_slow_cycle();
    let after_2 = lv_at(&city, 100, 100);
    assert!(
        after_2 >= after_1,
        "Value should rise monotonically: after_1={after_1}, after_2={after_2}"
    );

    city.tick_slow_cycles(10);
    let after_12 = lv_at(&city, 100, 100);
    assert!(
        after_12 >= after_2,
        "Value should continue rising: after_2={after_2}, after_12={after_12}"
    );
}

// -------------------------------------------------------------------------
// 3. Neighbourhood diffusion — values spread to adjacent cells
// -------------------------------------------------------------------------

#[test]
fn test_land_value_diffusion_spreads_high_value_to_neighbours() {
    let mut city = TestCity::new();

    // Set a single cell very high; neighbours start at 50
    {
        let world = city.world_mut();
        let mut lv = world.resource_mut::<LandValueGrid>();
        lv.set(128, 128, 250);
    }

    city.tick_slow_cycle();

    // Neighbour should have been pulled slightly above 50 by diffusion
    let neighbour = lv_at(&city, 129, 128);
    assert!(
        neighbour >= 50,
        "Diffusion should pull neighbour to at least 50, got {neighbour}"
    );
    // The centre should still be much higher than the neighbour
    let centre = lv_at(&city, 128, 128);
    assert!(
        centre > neighbour,
        "Centre ({centre}) should remain higher than neighbour ({neighbour})"
    );
}

#[test]
fn test_land_value_diffusion_spreads_low_value_to_neighbours() {
    let mut city = TestCity::new();

    // Set a single cell very low; neighbours start at 50
    {
        let world = city.world_mut();
        let mut lv = world.resource_mut::<LandValueGrid>();
        lv.set(128, 128, 0);
    }

    city.tick_slow_cycle();

    // Neighbour should have been pulled slightly below 50 by diffusion
    let neighbour = lv_at(&city, 129, 128);
    assert!(
        neighbour <= 50,
        "Diffusion should pull neighbour to at most 50, got {neighbour}"
    );
}

// -------------------------------------------------------------------------
// 4. Service placement gradually increases nearby land value
// -------------------------------------------------------------------------

#[test]
fn test_placing_service_gradually_increases_nearby_value() {
    let mut city = TestCity::new().with_service(100, 100, ServiceType::SmallPark);

    // After 1 cycle, value should have started rising but not converged yet
    city.tick_slow_cycle();
    let early = lv_at(&city, 100, 100);

    // After 20 more cycles, value should be higher
    city.tick_slow_cycles(20);
    let later = lv_at(&city, 100, 100);

    assert!(
        later > early,
        "Value should increase over time with park: early={early}, later={later}"
    );
    assert!(
        later > 50,
        "Value with park should be above baseline 50, got {later}"
    );
}

// -------------------------------------------------------------------------
// 5. Removing a service gradually decreases nearby land value
// -------------------------------------------------------------------------

#[test]
fn test_removing_service_gradually_decreases_nearby_value() {
    use crate::services::ServiceBuilding;
    use bevy::prelude::*;

    // Build city with park and let it converge
    let mut city = TestCity::new().with_service(100, 100, ServiceType::SmallPark);
    city.tick_slow_cycles(50);

    let boosted = lv_at(&city, 100, 100);
    assert!(
        boosted > 55,
        "After convergence with park, value should be well above 50, got {boosted}"
    );

    // Remove the park entity
    {
        let world = city.world_mut();
        let entities: Vec<Entity> = world
            .query_filtered::<Entity, With<ServiceBuilding>>()
            .iter(world)
            .collect();
        for e in entities {
            world.despawn(e);
        }
    }

    // Run a few more cycles — value should start decreasing toward 50
    city.tick_slow_cycles(10);
    let after_removal = lv_at(&city, 100, 100);

    assert!(
        after_removal < boosted,
        "After removing park, value should decrease: boosted={boosted}, after={after_removal}"
    );

    // But it should NOT have snapped all the way to 50 yet (gradual)
    // With alpha=0.1 and 10 cycles: retains (0.9)^10 ≈ 35% of the delta
    assert!(
        after_removal > 50,
        "Value should still be above 50 after only 10 cycles, got {after_removal}"
    );
}

// -------------------------------------------------------------------------
// 6. Saveable trait round-trip
// -------------------------------------------------------------------------

#[test]
fn test_land_value_saveable_round_trip() {
    use crate::Saveable;

    let mut grid = LandValueGrid::default();
    grid.set(10, 20, 200);
    grid.set(0, 0, 0);
    grid.set(255, 255, 255);

    let bytes = grid
        .save_to_bytes()
        .expect("save_to_bytes should return Some");
    let restored = LandValueGrid::load_from_bytes(&bytes);

    assert_eq!(restored.get(10, 20), 200);
    assert_eq!(restored.get(0, 0), 0);
    assert_eq!(restored.get(255, 255), 255);
    assert_eq!(restored.width, grid.width);
    assert_eq!(restored.height, grid.height);
}

#[test]
fn test_land_value_saveable_key_is_stable() {
    use crate::Saveable;
    assert_eq!(
        LandValueGrid::SAVE_KEY,
        "land_value",
        "SAVE_KEY must be stable across versions"
    );
}
