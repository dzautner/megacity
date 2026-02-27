//! Integration tests for executor entity spawning (issue #1904).
//!
//! Verifies that `PlaceUtility` and `PlaceService` game actions spawn the
//! correct ECS entities and set `building_id` on the grid cells.

use crate::game_actions::queue::ActionSource;
use crate::game_actions::result_log::ActionResultLog;
use crate::game_actions::{ActionError, ActionQueue, ActionResult, GameAction};
use crate::grid::{CellType, WorldGrid};
use crate::services::ServiceBuilding;
use crate::test_harness::TestCity;
use crate::utilities::{UtilitySource, UtilityType};

// -----------------------------------------------------------------------
// Utility placement — entity spawning
// -----------------------------------------------------------------------

#[test]
fn test_place_utility_via_action_spawns_entity() {
    let mut city = TestCity::new().with_budget(100_000.0);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceUtility {
                pos: (50, 50),
                utility_type: UtilityType::PowerPlant,
            },
        );
    }

    city.tick(1);

    // Verify success logged
    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].1, ActionResult::Success);

    // Verify grid cell has building_id
    let cell = city.cell(50, 50);
    assert!(
        cell.building_id.is_some(),
        "Grid cell (50,50) should have building_id after PlaceUtility"
    );

    // Verify UtilitySource entity exists with correct fields
    let entity = cell.building_id.unwrap();
    let world = city.world_mut();
    let source = world
        .get::<UtilitySource>(entity)
        .expect("Entity should have UtilitySource component");
    assert_eq!(source.utility_type, UtilityType::PowerPlant);
    assert_eq!(source.grid_x, 50);
    assert_eq!(source.grid_y, 50);
    assert!(source.range > 0, "Utility should have non-zero range");
}

// -----------------------------------------------------------------------
// Service placement — entity spawning
// -----------------------------------------------------------------------

#[test]
fn test_place_service_via_action_spawns_entity() {
    let mut city = TestCity::new().with_budget(100_000.0);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceService {
                pos: (60, 60),
                service_type: crate::services::ServiceType::FireStation,
            },
        );
    }

    city.tick(1);

    // Verify success logged
    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].1, ActionResult::Success);

    // Verify grid cell has building_id
    let cell = city.cell(60, 60);
    assert!(
        cell.building_id.is_some(),
        "Grid cell (60,60) should have building_id after PlaceService"
    );

    // Verify ServiceBuilding entity exists with correct fields
    let entity = cell.building_id.unwrap();
    let world = city.world_mut();
    let service = world
        .get::<ServiceBuilding>(entity)
        .expect("Entity should have ServiceBuilding component");
    assert_eq!(
        service.service_type,
        crate::services::ServiceType::FireStation
    );
    assert_eq!(service.grid_x, 60);
    assert_eq!(service.grid_y, 60);
    assert!(service.radius > 0.0, "Service should have non-zero radius");
}

// -----------------------------------------------------------------------
// Utility placement — cost deduction
// -----------------------------------------------------------------------

#[test]
fn test_place_utility_deducts_cost() {
    let initial_budget = 100_000.0;
    let mut city = TestCity::new().with_budget(initial_budget);

    let utility_type = UtilityType::PowerPlant;
    let expected_cost = crate::services::utility_cost(utility_type);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceUtility {
                pos: (40, 40),
                utility_type,
            },
        );
    }

    city.tick(1);

    let log = city.resource::<ActionResultLog>();
    assert_eq!(log.last_n(1)[0].1, ActionResult::Success);

    let expected_treasury = initial_budget - expected_cost;
    let actual_treasury = city.budget().treasury;
    assert!(
        (actual_treasury - expected_treasury).abs() < 0.01,
        "Treasury should be ~{expected_treasury} but was {actual_treasury}"
    );
}

// -----------------------------------------------------------------------
// Utility placement on water — error
// -----------------------------------------------------------------------

#[test]
fn test_place_utility_on_water_fails() {
    let mut city = TestCity::new().with_budget(100_000.0);

    // Set cell (70, 70) to water
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(70, 70).cell_type = CellType::Water;
    }

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceUtility {
                pos: (70, 70),
                utility_type: UtilityType::WaterTower,
            },
        );
    }

    city.tick(1);

    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].1, ActionResult::Error(ActionError::BlockedByWater));

    // Verify no entity spawned
    let cell = city.cell(70, 70);
    assert!(
        cell.building_id.is_none(),
        "Water cell should not have building_id"
    );
}

// -----------------------------------------------------------------------
// Service placement on occupied cell — error
// -----------------------------------------------------------------------

#[test]
fn test_place_service_on_occupied_cell_fails() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_utility(80, 80, UtilityType::PowerPlant);

    // Cell (80, 80) is now occupied by the utility
    assert!(
        city.cell(80, 80).building_id.is_some(),
        "Pre-condition: cell should be occupied"
    );

    let budget_before = city.budget().treasury;

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceService {
                pos: (80, 80),
                service_type: crate::services::ServiceType::Hospital,
            },
        );
    }

    city.tick(1);

    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].1, ActionResult::Error(ActionError::AlreadyExists));

    // Verify no funds were deducted
    assert!(
        (city.budget().treasury - budget_before).abs() < 0.01,
        "Treasury should not change on placement failure"
    );
}
