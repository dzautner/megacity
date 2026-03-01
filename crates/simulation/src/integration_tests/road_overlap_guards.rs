//! Guards against illegal road/building overlap in the action executor.

use crate::game_actions::queue::ActionSource;
use crate::game_actions::result_log::ActionResultLog;
use crate::game_actions::{ActionError, ActionQueue, ActionResult, GameAction};
use crate::grid::{CellType, RoadType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

#[test]
fn test_place_utility_on_road_returns_blocked_by_road() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(20, 20, 30, 20, RoadType::Local);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceUtility {
                pos: (25, 20),
                utility_type: UtilityType::PowerPlant,
            },
        );
    }

    city.tick(1);

    assert_eq!(
        city.cell(25, 20).building_id,
        None,
        "utility should not be placeable on a road cell"
    );
    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].1, ActionResult::Error(ActionError::BlockedByRoad));
}

#[test]
fn test_place_service_on_road_returns_blocked_by_road() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(40, 40, 50, 40, RoadType::Local);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceService {
                pos: (45, 40),
                service_type: ServiceType::FireStation,
            },
        );
    }

    city.tick(1);

    assert_eq!(
        city.cell(45, 40).building_id,
        None,
        "service should not be placeable on a road cell"
    );
    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last[0].1, ActionResult::Error(ActionError::BlockedByRoad));
}

#[test]
fn test_place_multi_cell_service_overlapping_road_returns_blocked_by_road() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(60, 60, 70, 60, RoadType::Local);

    // FireHQ has a 3x3 footprint. Top-left (62,59) overlaps road row y=60.
    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceService {
                pos: (62, 59),
                service_type: ServiceType::FireHQ,
            },
        );
    }

    city.tick(1);

    for y in 59..=61 {
        for x in 62..=64 {
            assert_eq!(
                city.cell(x, y).building_id,
                None,
                "no footprint cell should receive a building when overlap is rejected"
            );
        }
    }
    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last[0].1, ActionResult::Error(ActionError::BlockedByRoad));
}

#[test]
fn test_place_road_line_does_not_overwrite_existing_building_cell() {
    let mut city = TestCity::new().with_budget(100_000.0);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceUtility {
                pos: (80, 80),
                utility_type: UtilityType::PowerPlant,
            },
        );
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (75, 80),
                end: (85, 80),
                road_type: RoadType::Local,
            },
        );
    }

    city.tick(1);

    assert_eq!(
        city.cell(80, 80).cell_type,
        CellType::Grass,
        "road placement should not overwrite an occupied building cell"
    );
    assert!(
        city.cell(80, 80).building_id.is_some(),
        "the original building should remain on the occupied cell"
    );
    assert_eq!(
        city.cell(79, 80).cell_type,
        CellType::Road,
        "adjacent unoccupied cells should still receive road placement"
    );
    assert_eq!(city.cell(81, 80).cell_type, CellType::Road);
}
