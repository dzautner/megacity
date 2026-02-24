//! Integration tests for POLL-018: Tree and Green Space Pollution Absorption Enhancement.

use crate::districts::DISTRICTS_X;
use crate::grid::ZoneType;
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;
use crate::tree_absorption::{TreeCanopyStats, TreeMaturityGrid};
use crate::trees::TreeGrid;

// ---------------------------------------------------------------------------
// Tree maturity growth
// ---------------------------------------------------------------------------

#[test]
fn test_tree_maturity_grows_over_time() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(128, 128, true);
    }

    city.tick_slow_cycles(10);

    let maturity = city.resource::<TreeMaturityGrid>();
    let mat_value = maturity.get(128, 128);
    assert!(
        mat_value > 0.0,
        "tree maturity should grow after ticks, got {}",
        mat_value
    );
    assert!(
        mat_value < 1.0,
        "tree should not be fully mature after only 10 slow cycles, got {}",
        mat_value
    );
}

#[test]
fn test_tree_reaches_full_maturity() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(128, 128, true);
    }

    // Run enough slow cycles for full maturity (~72 slow cycles for 5 game-days)
    city.tick_slow_cycles(80);

    let maturity = city.resource::<TreeMaturityGrid>();
    let mat_value = maturity.get(128, 128);
    assert!(
        (mat_value - 1.0).abs() < 0.01,
        "tree should be fully mature after 80 slow cycles, got {}",
        mat_value
    );
}

#[test]
fn test_removed_tree_resets_maturity() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(128, 128, true);
    }
    city.tick_slow_cycles(40);

    {
        let mat = city.resource::<TreeMaturityGrid>().get(128, 128);
        assert!(mat > 0.0, "tree should have grown, got {}", mat);
    }

    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(128, 128, false);
    }

    city.tick_slow_cycles(1);

    let maturity = city.resource::<TreeMaturityGrid>();
    assert!(
        maturity.get(128, 128) < f32::EPSILON,
        "maturity should reset when tree removed, got {}",
        maturity.get(128, 128)
    );
}

// ---------------------------------------------------------------------------
// Pollution filtering (percentage-based)
// ---------------------------------------------------------------------------

#[test]
fn test_mature_tree_reduces_pollution_from_industrial() {
    // Place industrial building to generate pollution, then check tree effect
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 3);

    // Run one slow cycle without trees to measure baseline pollution
    city.tick_slow_cycle();
    let baseline_pol = city.resource::<PollutionGrid>().get(128, 128);
    assert!(
        baseline_pol > 0,
        "industrial building should generate pollution, got {}",
        baseline_pol
    );

    // Now plant a mature tree at the same location
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(128, 128, true);
        world
            .resource_mut::<TreeMaturityGrid>()
            .set(128, 128, 1.0);
    }

    // Run another slow cycle; tree absorption should reduce pollution
    city.tick_slow_cycle();

    let with_tree_pol = city.resource::<PollutionGrid>().get(128, 128);
    assert!(
        with_tree_pol < baseline_pol,
        "pollution with tree ({}) should be less than baseline ({})",
        with_tree_pol,
        baseline_pol
    );
}

#[test]
fn test_immature_tree_has_reduced_effect() {
    // Place two industrial buildings far apart to generate independent pollution
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 3)
        .with_building(200, 200, ZoneType::Industrial, 3);

    // Run one cycle to establish pollution
    city.tick_slow_cycle();
    let baseline_a = city.resource::<PollutionGrid>().get(50, 50);
    let baseline_b = city.resource::<PollutionGrid>().get(200, 200);
    assert!(baseline_a > 0, "should have pollution at A, got {}", baseline_a);
    assert!(baseline_b > 0, "should have pollution at B, got {}", baseline_b);

    // Plant mature tree at A, immature tree at B
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(50, 50, true);
        world.resource_mut::<TreeMaturityGrid>().set(50, 50, 1.0);
    }
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(200, 200, true);
        world
            .resource_mut::<TreeMaturityGrid>()
            .set(200, 200, 0.2);
    }

    city.tick_slow_cycle();

    let pol_a = city.resource::<PollutionGrid>().get(50, 50);
    let pol_b = city.resource::<PollutionGrid>().get(200, 200);

    // Both should have pollution reduced from baseline
    assert!(pol_a < baseline_a, "mature tree should reduce pollution at A");
    assert!(pol_b < baseline_b || pol_b <= baseline_b, "immature tree should not increase pollution at B");

    // Mature tree should have more reduction than immature
    // Calculate reduction percentages
    let reduction_a = (baseline_a as f32 - pol_a as f32) / baseline_a as f32;
    let reduction_b = (baseline_b as f32 - pol_b as f32) / baseline_b as f32;
    assert!(
        reduction_a > reduction_b,
        "mature tree reduction ({:.2}) should be greater than immature ({:.2})",
        reduction_a,
        reduction_b
    );
}

// ---------------------------------------------------------------------------
// Green space cluster bonus
// ---------------------------------------------------------------------------

#[test]
fn test_green_space_cluster_bonus() {
    // Place identical industrial buildings near two tree groups
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 3)
        .with_building(200, 200, ZoneType::Industrial, 3);

    // Run one cycle to establish baseline pollution
    city.tick_slow_cycle();
    let baseline_cluster = city.resource::<PollutionGrid>().get(50, 50);
    let baseline_solo = city.resource::<PollutionGrid>().get(200, 200);
    assert!(baseline_cluster > 0, "cluster site should have pollution");
    assert!(baseline_solo > 0, "solo site should have pollution");

    // Create a cluster of trees (exceeds threshold of 10) at (50,50)
    {
        let world = city.world_mut();
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let x = (50 + dx) as usize;
                let y = (50 + dy) as usize;
                world.resource_mut::<TreeGrid>().set(x, y, true);
                world.resource_mut::<TreeMaturityGrid>().set(x, y, 1.0);
            }
        }
    }

    // Solitary mature tree at (200, 200)
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(200, 200, true);
        world
            .resource_mut::<TreeMaturityGrid>()
            .set(200, 200, 1.0);
    }

    city.tick_slow_cycle();

    let cluster_pol = city.resource::<PollutionGrid>().get(50, 50);
    let solo_pol = city.resource::<PollutionGrid>().get(200, 200);

    // Both should reduce pollution
    assert!(cluster_pol < baseline_cluster, "cluster should reduce pollution");
    assert!(solo_pol < baseline_solo || solo_pol <= baseline_solo, "solo tree should not increase pollution");

    // Cluster should have greater reduction percentage
    let cluster_reduction = (baseline_cluster as f32 - cluster_pol as f32) / baseline_cluster as f32;
    let solo_reduction = (baseline_solo as f32 - solo_pol as f32) / baseline_solo as f32;
    assert!(
        cluster_reduction > solo_reduction,
        "cluster reduction ({:.2}) should exceed solo reduction ({:.2})",
        cluster_reduction,
        solo_reduction
    );
}

// ---------------------------------------------------------------------------
// Noise reduction
// ---------------------------------------------------------------------------

#[test]
fn test_tree_reduces_noise_pollution() {
    let mut city = TestCity::new();

    // Set noise pollution manually - noise is not rebuilt from scratch each tick
    // like air pollution, so manual values persist.
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(128, 128, true);
        world
            .resource_mut::<TreeMaturityGrid>()
            .set(128, 128, 1.0);
    }
    {
        let world = city.world_mut();
        world
            .resource_mut::<NoisePollutionGrid>()
            .set(128, 128, 150);
    }

    city.tick_slow_cycles(1);

    let noise = city.resource::<NoisePollutionGrid>();
    let val = noise.get(128, 128);
    assert!(
        val < 150,
        "noise should be reduced by tree from 150, got {}",
        val
    );
}

// ---------------------------------------------------------------------------
// Canopy stats and CO2 absorption
// ---------------------------------------------------------------------------

#[test]
fn test_canopy_percentage_computed() {
    let mut city = TestCity::new();

    let dist_x = 4;
    let dist_y = 4;
    let district_size = crate::districts::DISTRICT_SIZE;
    {
        let world = city.world_mut();
        for ly in 0..district_size {
            for lx in 0..district_size {
                let gx = dist_x * district_size + lx;
                let gy = dist_y * district_size + ly;
                world.resource_mut::<TreeGrid>().set(gx, gy, true);
                world.resource_mut::<TreeMaturityGrid>().set(gx, gy, 1.0);
            }
        }
    }

    city.tick_slow_cycles(1);

    let stats = city.resource::<TreeCanopyStats>();
    let canopy = stats.district_canopy(dist_x, dist_y);
    assert!(
        (canopy - 1.0).abs() < 0.01,
        "fully treed district should have ~100% canopy, got {}",
        canopy
    );
}

#[test]
fn test_co2_absorption_tracks_mature_trees() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        for i in 0..100usize {
            let x = 10 + (i % 10);
            let y = 10 + (i / 10);
            world.resource_mut::<TreeGrid>().set(x, y, true);
            world.resource_mut::<TreeMaturityGrid>().set(x, y, 1.0);
        }
    }

    city.tick_slow_cycles(1);

    let stats = city.resource::<TreeCanopyStats>();
    let expected_co2 = 100.0 * 48.0;
    let diff = (stats.total_co2_absorption_lbs_per_year - expected_co2).abs();
    assert!(
        diff < 1.0,
        "100 mature trees should absorb ~{} lbs CO2/year, got {}",
        expected_co2,
        stats.total_co2_absorption_lbs_per_year
    );
}

#[test]
fn test_empty_city_has_no_canopy() {
    let mut city = TestCity::new();
    city.tick_slow_cycles(1);

    let stats = city.resource::<TreeCanopyStats>();
    assert!(
        stats.total_co2_absorption_lbs_per_year < f32::EPSILON,
        "empty city should have zero CO2 absorption"
    );
    for dx in 0..DISTRICTS_X {
        assert!(
            stats.district_canopy(dx, 0) < f32::EPSILON,
            "empty city should have zero canopy"
        );
    }
}
