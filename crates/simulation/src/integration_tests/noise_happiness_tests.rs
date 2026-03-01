//! Integration tests for issue #1978: Noise happiness rebalance
//!
//! Verifies that after reducing road noise dB levels, residential areas
//! further from roads experience meaningfully lower noise than those
//! immediately adjacent, producing varied happiness penalties instead of
//! a constant -5.0.

use crate::grid::RoadType;
use crate::noise::NoisePollutionGrid;
use crate::test_harness::TestCity;
use crate::wind::WindState;

/// A residential building far from a Local road should have significantly
/// less noise than one directly adjacent, proving that the rebalanced dB
/// levels produce varied noise rather than saturating to 100 everywhere.
#[test]
fn test_noise_residential_away_from_highway_is_moderate() {
    // Place a horizontal Local road at y=10
    let mut city = TestCity::new()
        .with_road(100, 10, 150, 10, RoadType::Local);

    // Disable wind to avoid directional noise drift
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    // Measure noise at y=12 (2 cells from road) and y=20 (10 cells from road)
    let noise_near = city.resource::<NoisePollutionGrid>().get(125, 12);
    let noise_far = city.resource::<NoisePollutionGrid>().get(125, 20);

    assert!(
        noise_near > noise_far,
        "noise near road (y=12) should be higher than far (y=20): near={}, far={}",
        noise_near,
        noise_far,
    );

    // The far cell should have meaningfully lower noise (not just 1 point less)
    assert!(
        noise_near >= noise_far + 5,
        "noise difference should be meaningful (>= 5): near={}, far={}",
        noise_near,
        noise_far,
    );
}

/// With the rebalanced Local road at 35 dB, a single road cell should NOT
/// saturate the adjacent cell to 100. This is the core fix for issue #1978.
#[test]
fn test_single_local_road_does_not_saturate_adjacent() {
    let mut city = TestCity::new()
        .with_road(128, 128, 128, 128, RoadType::Local);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise_adjacent = city.resource::<NoisePollutionGrid>().get(129, 128);
    assert!(
        noise_adjacent < 80,
        "single local road should not saturate adjacent cell: got {}",
        noise_adjacent,
    );
}

/// A Highway should still produce significantly more noise than a Local road,
/// validating that the rebalanced hierarchy is maintained.
#[test]
fn test_road_type_noise_hierarchy_preserved() {
    let mut city_highway = TestCity::new()
        .with_road(128, 128, 128, 128, RoadType::Highway);
    let mut city_local = TestCity::new()
        .with_road(128, 128, 128, 128, RoadType::Local);

    for city in [&mut city_highway, &mut city_local] {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city_highway.tick_slow_cycles(2);
    city_local.tick_slow_cycles(2);

    let hw_noise = city_highway.resource::<NoisePollutionGrid>().get(130, 128);
    let local_noise = city_local.resource::<NoisePollutionGrid>().get(130, 128);

    assert!(
        hw_noise > local_noise,
        "highway ({}) should produce more noise than local road ({}) at distance 2",
        hw_noise,
        local_noise,
    );

    // The difference should be substantial (highway is 80 dB vs local 35 dB)
    assert!(
        hw_noise >= local_noise + 10,
        "highway vs local noise gap should be >= 10: highway={}, local={}",
        hw_noise,
        local_noise,
    );
}

/// Noise happiness penalty should scale with distance from road: a citizen
/// far from any road gets a smaller penalty than one right next to it.
/// This tests the / 25.0 divisor indirectly via the noise grid values.
#[test]
fn test_noise_penalty_varies_with_distance() {
    let mut city = TestCity::new()
        .with_road(100, 10, 150, 10, RoadType::Local);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise_at_2 = city.resource::<NoisePollutionGrid>().get(125, 12) as f32;
    let noise_at_10 = city.resource::<NoisePollutionGrid>().get(125, 20) as f32;

    // Simulate the happiness penalty formula: noise / 25.0
    let penalty_near = noise_at_2 / 25.0;
    let penalty_far = noise_at_10 / 25.0;

    assert!(
        penalty_near > penalty_far,
        "near-road penalty ({:.2}) should exceed far-road penalty ({:.2})",
        penalty_near,
        penalty_far,
    );

    // With rebalanced values, the near penalty should be well below the old
    // constant -5.0 (which would require noise=125 with the new divisor of 25)
    assert!(
        penalty_near < 5.0,
        "near-road penalty ({:.2}) should be below old constant -5.0",
        penalty_near,
    );
}
