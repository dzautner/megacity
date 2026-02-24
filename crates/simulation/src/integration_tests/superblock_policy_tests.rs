//! Integration tests for Barcelona Superblock District Policy (TRAF-009).

use crate::grid::RoadType;
use crate::superblock::{Superblock, SuperblockState};
use crate::superblock_policy::SuperblockPolicyState;
use crate::test_harness::TestCity;

// =============================================================================
// Activation and road conversion
// =============================================================================

#[test]
fn test_superblock_policy_activation_converts_interior_roads_to_path() {
    let mut city = TestCity::new()
        // Place a grid of roads from (50,50) to (56,56)
        .with_road(50, 50, 56, 50, RoadType::Local)
        .with_road(50, 51, 56, 51, RoadType::Local)
        .with_road(50, 52, 56, 52, RoadType::Avenue)
        .with_road(50, 53, 56, 53, RoadType::Local)
        .with_road(50, 54, 56, 54, RoadType::Local)
        .with_road(50, 55, 56, 55, RoadType::Local)
        .with_road(50, 56, 56, 56, RoadType::Local);

    // Designate a 5x5 superblock (50,50)-(54,54)
    {
        let world = city.world_mut();
        let mut sb_state = world.resource_mut::<SuperblockState>();
        sb_state.add_superblock(Superblock::new(50, 50, 54, 54, "Test".to_string()));
    }

    // Activate the policy
    {
        let world = city.world_mut();
        world.resource_scope(|world, sb_state: bevy::prelude::Mut<SuperblockState>| {
            world.resource_scope(|world, mut policy: bevy::prelude::Mut<SuperblockPolicyState>| {
                world.resource_scope(|_world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                    let result = policy.activate(0, &sb_state, &mut grid);
                    assert!(result, "should successfully activate superblock policy");
                });
            });
        });
    }

    // Verify interior roads (51,51), (52,52), (53,53) are now Path
    let grid = city.grid();
    assert_eq!(
        grid.get(52, 52).road_type,
        RoadType::Path,
        "interior road should be converted to Path"
    );
    assert_eq!(
        grid.get(51, 51).road_type,
        RoadType::Path,
        "interior road should be converted to Path"
    );

    // Verify perimeter roads remain unchanged
    assert_eq!(
        grid.get(50, 50).road_type,
        RoadType::Local,
        "perimeter road should remain Local"
    );
    assert_eq!(
        grid.get(54, 54).road_type,
        RoadType::Local,
        "perimeter road should remain Local"
    );
}

#[test]
fn test_superblock_policy_revert_restores_original_roads() {
    let mut city = TestCity::new()
        .with_road(50, 50, 56, 50, RoadType::Local)
        .with_road(50, 51, 56, 51, RoadType::Local)
        .with_road(50, 52, 56, 52, RoadType::Avenue)
        .with_road(50, 53, 56, 53, RoadType::Local)
        .with_road(50, 54, 56, 54, RoadType::Local)
        .with_road(50, 55, 56, 55, RoadType::Local)
        .with_road(50, 56, 56, 56, RoadType::Local);

    // Designate and activate
    {
        let world = city.world_mut();
        let mut sb_state = world.resource_mut::<SuperblockState>();
        sb_state.add_superblock(Superblock::new(50, 50, 54, 54, "Test".to_string()));
    }
    {
        let world = city.world_mut();
        world.resource_scope(|world, sb_state: bevy::prelude::Mut<SuperblockState>| {
            world.resource_scope(|world, mut policy: bevy::prelude::Mut<SuperblockPolicyState>| {
                world.resource_scope(|_world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                    policy.activate(0, &sb_state, &mut grid);
                });
            });
        });
    }

    // Interior should be Path now
    assert_eq!(city.grid().get(52, 52).road_type, RoadType::Path);

    // Revert the policy
    {
        let world = city.world_mut();
        world.resource_scope(|world, mut policy: bevy::prelude::Mut<SuperblockPolicyState>| {
            world.resource_scope(|_world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                let result = policy.revert(0, &mut grid);
                assert!(result, "should successfully revert superblock policy");
            });
        });
    }

    // Interior should be restored to Avenue (row 52 was Avenue)
    let grid = city.grid();
    assert_eq!(
        grid.get(52, 52).road_type,
        RoadType::Avenue,
        "reverted interior road should restore to Avenue"
    );
}

// =============================================================================
// Duplicate activation rejected
// =============================================================================

#[test]
fn test_superblock_policy_reject_double_activation() {
    let mut city = TestCity::new()
        .with_road(50, 50, 56, 56, RoadType::Local);

    {
        let world = city.world_mut();
        let mut sb_state = world.resource_mut::<SuperblockState>();
        sb_state.add_superblock(Superblock::new(50, 50, 54, 54, "Test".to_string()));
    }

    // First activation succeeds
    {
        let world = city.world_mut();
        world.resource_scope(|world, sb_state: bevy::prelude::Mut<SuperblockState>| {
            world.resource_scope(|world, mut policy: bevy::prelude::Mut<SuperblockPolicyState>| {
                world.resource_scope(|_world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                    assert!(policy.activate(0, &sb_state, &mut grid));
                });
            });
        });
    }

    // Second activation fails
    {
        let world = city.world_mut();
        world.resource_scope(|world, sb_state: bevy::prelude::Mut<SuperblockState>| {
            world.resource_scope(|world, mut policy: bevy::prelude::Mut<SuperblockPolicyState>| {
                world.resource_scope(|_world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                    assert!(
                        !policy.activate(0, &sb_state, &mut grid),
                        "duplicate activation should be rejected"
                    );
                });
            });
        });
    }
}

// =============================================================================
// Happiness bonus
// =============================================================================

#[test]
fn test_superblock_policy_happiness_bonus_for_interior() {
    let mut city = TestCity::new()
        .with_road(50, 50, 56, 56, RoadType::Local);

    {
        let world = city.world_mut();
        let mut sb_state = world.resource_mut::<SuperblockState>();
        sb_state.add_superblock(Superblock::new(50, 50, 54, 54, "Test".to_string()));
    }
    {
        let world = city.world_mut();
        world.resource_scope(|world, sb_state: bevy::prelude::Mut<SuperblockState>| {
            world.resource_scope(|world, mut policy: bevy::prelude::Mut<SuperblockPolicyState>| {
                world.resource_scope(|_world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                    policy.activate(0, &sb_state, &mut grid);
                });
            });
        });
    }

    let sb_state = city.resource::<SuperblockState>();
    let policy = city.resource::<SuperblockPolicyState>();

    // Interior cell should get happiness bonus
    let bonus = policy.cell_happiness_bonus(52, 52, sb_state);
    assert!(
        bonus >= 8.0 && bonus <= 12.0,
        "happiness bonus should be in [8, 12] range, got {bonus}"
    );

    // Perimeter cell should get no bonus
    let perimeter_bonus = policy.cell_happiness_bonus(50, 50, sb_state);
    assert!(
        perimeter_bonus.abs() < f32::EPSILON,
        "perimeter cell should get no happiness bonus"
    );

    // Outside cell should get no bonus
    let outside_bonus = policy.cell_happiness_bonus(40, 40, sb_state);
    assert!(
        outside_bonus.abs() < f32::EPSILON,
        "outside cell should get no happiness bonus"
    );
}

// =============================================================================
// Resource initialization
// =============================================================================

#[test]
fn test_superblock_policy_resource_initialized() {
    let city = TestCity::new();
    city.assert_resource_exists::<SuperblockPolicyState>();
}

#[test]
fn test_superblock_policy_default_state_is_empty() {
    let city = TestCity::new();
    let policy = city.resource::<SuperblockPolicyState>();
    assert!(policy.entries.is_empty());
    assert_eq!(policy.active_count(), 0);
    assert!(policy.happiness_bonus.abs() < f32::EPSILON);
    assert!(policy.monthly_cost.abs() < f64::EPSILON);
    assert!((policy.perimeter_congestion - 1.0).abs() < f32::EPSILON);
}

// =============================================================================
// Perimeter congestion
// =============================================================================

#[test]
fn test_superblock_policy_perimeter_congestion_increases_with_active_count() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_road(50, 51, 70, 51, RoadType::Local)
        .with_road(50, 52, 70, 52, RoadType::Local)
        .with_road(50, 53, 70, 53, RoadType::Local)
        .with_road(50, 54, 70, 54, RoadType::Local)
        .with_road(50, 55, 70, 55, RoadType::Local)
        .with_road(50, 56, 70, 56, RoadType::Local)
        .with_road(50, 57, 70, 57, RoadType::Local)
        .with_road(50, 58, 70, 58, RoadType::Local)
        .with_road(50, 59, 70, 59, RoadType::Local)
        .with_road(50, 60, 70, 60, RoadType::Local);

    // Add two superblocks
    {
        let world = city.world_mut();
        let mut sb_state = world.resource_mut::<SuperblockState>();
        sb_state.add_superblock(Superblock::new(50, 50, 54, 54, "A".to_string()));
        sb_state.add_superblock(Superblock::new(60, 50, 64, 54, "B".to_string()));
    }

    // Activate first
    {
        let world = city.world_mut();
        world.resource_scope(|world, sb_state: bevy::prelude::Mut<SuperblockState>| {
            world.resource_scope(|world, mut policy: bevy::prelude::Mut<SuperblockPolicyState>| {
                world.resource_scope(|_world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                    policy.activate(0, &sb_state, &mut grid);
                });
            });
        });
    }

    let congestion_1 = city.resource::<SuperblockPolicyState>().perimeter_congestion;

    // Activate second
    {
        let world = city.world_mut();
        world.resource_scope(|world, sb_state: bevy::prelude::Mut<SuperblockState>| {
            world.resource_scope(|world, mut policy: bevy::prelude::Mut<SuperblockPolicyState>| {
                world.resource_scope(|_world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                    policy.activate(1, &sb_state, &mut grid);
                });
            });
        });
    }

    let congestion_2 = city.resource::<SuperblockPolicyState>().perimeter_congestion;

    assert!(
        congestion_2 > congestion_1,
        "more active superblocks should increase perimeter congestion: {congestion_1} vs {congestion_2}"
    );
}

// =============================================================================
// Saveable
// =============================================================================

#[test]
fn test_superblock_policy_saveable_skips_default() {
    use crate::Saveable;
    let state = SuperblockPolicyState::default();
    assert!(state.save_to_bytes().is_none());
}

#[test]
fn test_superblock_policy_saveable_key() {
    use crate::Saveable;
    assert_eq!(SuperblockPolicyState::SAVE_KEY, "superblock_policy");
}
