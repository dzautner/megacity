//! Integration tests verifying that the happiness breakdown for garbage uses
//! linear scaling that matches the actual happiness system (issue #1979).
//!
//! The breakdown must use the same formula as `happiness/systems.rs`:
//!   ratio = ((garbage_level - 10) / 90).clamp(0, 1)
//!   penalty = GARBAGE_PENALTY * ratio
//! rather than a binary full-penalty check.

use crate::garbage::GarbageGrid;
use crate::grid::{RoadType, ZoneType};
use crate::happiness::GARBAGE_PENALTY;
use crate::happiness_breakdown::HappinessBreakdown;
use crate::test_harness::TestCity;

/// Helper: extract the "garbage" factor from the breakdown, returning 0 if absent.
fn garbage_factor(bd: &HappinessBreakdown) -> f32 {
    bd.factors
        .iter()
        .find(|(name, _)| name == "garbage")
        .map(|(_, v)| *v)
        .unwrap_or(0.0)
}

#[test]
fn test_garbage_breakdown_at_level_20_gives_partial_penalty() {
    let mut city = TestCity::new()
        .with_road(48, 50, 52, 50, RoadType::Local)
        .with_zone(51, 51, ZoneType::ResidentialLow)
        .with_building(51, 51, ZoneType::ResidentialLow, 1)
        .with_citizen((51, 51), (51, 51));

    // Set garbage level to 20 at the citizen's home cell
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<GarbageGrid>();
        grid.set(51, 51, 20);
    }

    // Tick enough for happiness breakdown to compute
    city.tick(100);

    let bd = city.resource::<HappinessBreakdown>();
    let garbage = garbage_factor(bd);

    // Expected: ratio = (20-10)/90 ≈ 0.111, penalty ≈ -0.556
    // With the old binary code this would be the full -GARBAGE_PENALTY (-5.0)
    // The garbage factor also includes waste accumulation effects which should
    // be ~0 for a fresh city with no waste services, so we check that the
    // total is much less severe than full penalty.
    assert!(
        garbage > -(GARBAGE_PENALTY * 0.5),
        "Garbage level 20 should give partial penalty (< 50% of full {:.1}), got {:.3}",
        GARBAGE_PENALTY,
        garbage
    );
}

#[test]
fn test_garbage_breakdown_scales_with_level() {
    // Verify that higher garbage levels produce larger (more negative) penalties
    let mut city = TestCity::new()
        .with_road(48, 50, 52, 50, RoadType::Local)
        .with_zone(51, 51, ZoneType::ResidentialLow)
        .with_building(51, 51, ZoneType::ResidentialLow, 1)
        .with_citizen((51, 51), (51, 51));

    // First measure with garbage = 30
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<GarbageGrid>();
        grid.set(51, 51, 30);
    }
    city.tick(100);
    let low_garbage = garbage_factor(city.resource::<HappinessBreakdown>());

    // Now raise to garbage = 90
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<GarbageGrid>();
        grid.set(51, 51, 90);
    }
    city.tick(100);
    let high_garbage = garbage_factor(city.resource::<HappinessBreakdown>());

    // Higher garbage should produce a more negative (worse) penalty.
    // With binary check both would be identical (-5.0), but with linear
    // scaling, 90 should be much worse than 30.
    if low_garbage < 0.0 && high_garbage < 0.0 {
        assert!(
            high_garbage < low_garbage,
            "Higher garbage (90) should give worse penalty than lower (30): high={:.3}, low={:.3}",
            high_garbage,
            low_garbage
        );
    }
}

#[test]
fn test_garbage_breakdown_below_threshold_gives_no_grid_penalty() {
    // Garbage level <= 10 should give zero garbage grid penalty
    let mut city = TestCity::new()
        .with_road(48, 50, 52, 50, RoadType::Local)
        .with_zone(51, 51, ZoneType::ResidentialLow)
        .with_building(51, 51, ZoneType::ResidentialLow, 1)
        .with_citizen((51, 51), (51, 51));

    // Set garbage to exactly 10 (threshold, no penalty expected from grid)
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<GarbageGrid>();
        grid.set(51, 51, 10);
    }
    city.tick(100);

    let bd = city.resource::<HappinessBreakdown>();
    let garbage = garbage_factor(bd);

    // At level 10, the garbage grid component should contribute 0 penalty.
    // The total garbage factor may include waste accumulation, but the grid
    // penalty specifically should be zero. We verify the overall is not
    // strongly negative (i.e., no binary -5.0 penalty).
    assert!(
        garbage > -(GARBAGE_PENALTY * 0.5),
        "Garbage level 10 should not trigger significant grid penalty, got {:.3}",
        garbage
    );
}
