//! Integration tests for ZoneRect returning error when no cells are zoned (#1976).

use crate::game_actions::queue::ActionSource;
use crate::game_actions::result_log::ActionResultLog;
use crate::game_actions::{ActionError, ActionQueue, ActionResult, GameAction};
use crate::grid::{RoadType, ZoneType};
use crate::test_harness::TestCity;

#[test]
fn test_zone_rect_no_roads_returns_no_cells_zoned_error() {
    let mut city = TestCity::new().with_budget(100_000.0);

    // Zone an area with no roads nearby — should fail
    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::ZoneRect {
                min: (50, 50),
                max: (55, 55),
                zone_type: ZoneType::ResidentialLow,
            },
        );
    }

    city.tick(1);

    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(
        last[0].1,
        ActionResult::Error(ActionError::NoCellsZoned),
        "ZoneRect with no road-adjacent cells should return NoCellsZoned error"
    );
}

#[test]
fn test_zone_rect_adjacent_to_road_succeeds() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(10, 10, 15, 10, RoadType::Local);

    // Zone cells adjacent to the road — should succeed
    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::ZoneRect {
                min: (10, 9),
                max: (15, 9),
                zone_type: ZoneType::ResidentialLow,
            },
        );
    }

    city.tick(1);

    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert!(
        last[0].1.is_success(),
        "ZoneRect adjacent to road should succeed, got {:?}",
        last[0].1
    );
}

#[test]
fn test_zone_rect_far_from_road_returns_error() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(10, 10, 15, 10, RoadType::Local);

    // Zone cells far from any road — should fail
    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::ZoneRect {
                min: (100, 100),
                max: (105, 105),
                zone_type: ZoneType::Commercial,
            },
        );
    }

    city.tick(1);

    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(
        last[0].1,
        ActionResult::Error(ActionError::NoCellsZoned),
        "ZoneRect far from any road should return NoCellsZoned error"
    );
}
