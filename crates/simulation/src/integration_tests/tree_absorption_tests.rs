//! Integration tests for POLL-018: Tree and Green Space Pollution Absorption Enhancement.

use crate::districts::DISTRICTS_X;
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

    // Plant a tree at (128, 128)
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(128, 128, true);
    }

    // Run several slow cycles to grow maturity
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

    // Plant a tree
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

    // Plant and grow a tree
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(128, 128, true);
    }
    city.tick_slow_cycles(40);

    // Verify it has maturity
    {
        let mat = city.resource::<TreeMaturityGrid>().get(128, 128);
        assert!(mat > 0.0, "tree should have grown, got {}", mat);
    }

    // Remove the tree
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
fn test_mature_tree_reduces_pollution_percentage() {
    let mut city = TestCity::new();

    // Plant a mature tree and set high pollution nearby
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(128, 128, true);
        world
            .resource_mut::<TreeMaturityGrid>()
            .set(128, 128, 1.0);
    }
    {
        let world = city.world_mut();
        let mut pol = world.resource_mut::<PollutionGrid>();
        for dy in -3i32..=3 {
            for dx in -3i32..=3 {
                let x = (128 + dx) as usize;
                let y = (128 + dy) as usize;
                pol.set(x, y, 200);
            }
        }
    }

    city.tick_slow_cycles(1);

    // Check that pollution at the tree center is reduced
    let pol = city.resource::<PollutionGrid>();
    let center_pol = pol.get(128, 128);
    assert!(
        center_pol < 200,
        "pollution at tree center should be reduced from 200, got {}",
        center_pol
    );

    // Check that nearby cells also have reduced pollution
    let nearby_pol = pol.get(129, 128);
    assert!(
        nearby_pol < 200,
        "pollution near tree should be reduced, got {}",
        nearby_pol
    );
}

#[test]
fn test_immature_tree_has_reduced_effect() {
    let mut city = TestCity::new();

    // Plant two trees: one mature, one immature
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(100, 100, true);
    }
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(200, 200, true);
    }
    {
        let world = city.world_mut();
        world
            .resource_mut::<TreeMaturityGrid>()
            .set(100, 100, 1.0);
    }
    {
        let world = city.world_mut();
        world
            .resource_mut::<TreeMaturityGrid>()
            .set(200, 200, 0.2);
    }
    {
        let world = city.world_mut();
        let mut pol = world.resource_mut::<PollutionGrid>();
        pol.set(100, 100, 200);
        pol.set(200, 200, 200);
    }

    city.tick_slow_cycles(1);

    let pol = city.resource::<PollutionGrid>();
    let mature_pol = pol.get(100, 100);
    let immature_pol = pol.get(200, 200);

    // The mature tree should reduce pollution more than the immature one
    assert!(
        mature_pol < immature_pol,
        "mature tree ({}) should filter more than immature tree ({})",
        mature_pol,
        immature_pol
    );
}

// ---------------------------------------------------------------------------
// Green space cluster bonus
// ---------------------------------------------------------------------------

#[test]
fn test_green_space_cluster_bonus() {
    let mut city = TestCity::new();

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

    // Solitary tree at (200, 200), mature
    {
        let world = city.world_mut();
        world.resource_mut::<TreeGrid>().set(200, 200, true);
        world
            .resource_mut::<TreeMaturityGrid>()
            .set(200, 200, 1.0);
    }

    // Set equal pollution at both locations
    {
        let world = city.world_mut();
        let mut pol = world.resource_mut::<PollutionGrid>();
        pol.set(50, 50, 200);
        pol.set(200, 200, 200);
    }

    city.tick_slow_cycles(1);

    let pol = city.resource::<PollutionGrid>();
    let cluster_pol = pol.get(50, 50);
    let solo_pol = pol.get(200, 200);

    // Cluster should filter more aggressively
    assert!(
        cluster_pol < solo_pol,
        "cluster center ({}) should have less pollution than solo tree ({})",
        cluster_pol,
        solo_pol
    );
}

// ---------------------------------------------------------------------------
// Noise reduction
// ---------------------------------------------------------------------------

#[test]
fn test_tree_reduces_noise_pollution() {
    let mut city = TestCity::new();

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

    // Fill an entire district with mature trees
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

    // Plant 100 mature trees
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
