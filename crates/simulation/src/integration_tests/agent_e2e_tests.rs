//! End-to-end integration tests for the agent protocol flow (issue #1920).
//!
//! Verifies the full action lifecycle: push action via ActionQueue with
//! ActionSource::Agent, tick the simulation, and verify the world state
//! changed as expected.

use crate::budget::ExtendedBudget;
use crate::game_actions::queue::ActionSource;
use crate::game_actions::result_log::ActionResultLog;
use crate::game_actions::{ActionQueue, ActionResult, GameAction};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::observation_builder::CurrentObservation;
use crate::replay::{ReplayPlayer, ReplayRecorder};
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::utilities::UtilitySource;
use crate::utilities::UtilityType;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// 1. PlaceRoadLine modifies grid
// ---------------------------------------------------------------------------

#[test]
fn test_agent_place_road_modifies_grid() {
    let mut city = TestCity::new();

    // Push a PlaceRoadLine action via the agent source
    {
        let world = city.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            tick,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (100, 100),
                end: (110, 100),
                road_type: RoadType::Avenue,
            },
        );
    }

    city.tick(1);

    // Verify all cells along the line are roads
    {
        let grid = city.grid();
        for x in 100..=110 {
            assert_eq!(
                grid.get(x, 100).cell_type,
                CellType::Road,
                "cell ({x}, 100) should be a road after PlaceRoadLine"
            );
        }
    }

    // Verify success was logged
    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].1, ActionResult::Success);
}

// ---------------------------------------------------------------------------
// 2. PlaceUtility spawns entity
// ---------------------------------------------------------------------------

#[test]
fn test_agent_place_utility_spawns_entity() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            tick,
            ActionSource::Agent,
            GameAction::PlaceUtility {
                pos: (80, 80),
                utility_type: UtilityType::PowerPlant,
            },
        );
    }

    city.tick(1);

    // Verify success logged
    let log = city.resource::<ActionResultLog>();
    assert_eq!(log.last_n(1)[0].1, ActionResult::Success);

    // Verify grid cell has building_id
    let cell = city.cell(80, 80);
    assert!(
        cell.building_id.is_some(),
        "grid cell (80,80) should have building_id after PlaceUtility"
    );

    // Verify UtilitySource entity exists with correct type
    let entity = cell.building_id.unwrap();
    let world = city.world_mut();
    let source = world
        .get::<UtilitySource>(entity)
        .expect("entity should have UtilitySource component");
    assert_eq!(source.utility_type, UtilityType::PowerPlant);
    assert_eq!(source.grid_x, 80);
    assert_eq!(source.grid_y, 80);
}

// ---------------------------------------------------------------------------
// 3. PlaceService spawns entity
// ---------------------------------------------------------------------------

#[test]
fn test_agent_place_service_spawns_entity() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            tick,
            ActionSource::Agent,
            GameAction::PlaceService {
                pos: (90, 90),
                service_type: ServiceType::FireStation,
            },
        );
    }

    city.tick(1);

    // Verify success logged
    let log = city.resource::<ActionResultLog>();
    assert_eq!(log.last_n(1)[0].1, ActionResult::Success);

    // Verify grid cell has building_id
    let cell = city.cell(90, 90);
    assert!(
        cell.building_id.is_some(),
        "grid cell (90,90) should have building_id after PlaceService"
    );

    // Verify ServiceBuilding entity exists via query
    let world = city.world_mut();
    let mut query = world.query::<&ServiceBuilding>();
    let services: Vec<&ServiceBuilding> = query.iter(world).collect();
    assert!(
        !services.is_empty(),
        "should have spawned at least one ServiceBuilding"
    );

    // Find the one at (90, 90)
    let found = services
        .iter()
        .any(|s| s.grid_x == 90 && s.grid_y == 90 && s.service_type == ServiceType::FireStation);
    assert!(
        found,
        "should find a FireStation ServiceBuilding at (90,90)"
    );
}

// ---------------------------------------------------------------------------
// 4. ZoneRect zones cells adjacent to road
// ---------------------------------------------------------------------------

#[test]
fn test_agent_zone_rect_zones_adjacent_cells() {
    // Place a horizontal road first, then zone cells adjacent to it
    let mut city = TestCity::new();

    // Place road from (50,50) to (60,50) via action
    {
        let world = city.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            tick,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (50, 50),
                end: (60, 50),
                road_type: RoadType::Local,
            },
        );
    }
    city.tick(1);

    // Verify road placed
    assert_eq!(
        city.grid().get(55, 50).cell_type,
        CellType::Road,
        "road should exist at (55,50)"
    );

    // Zone a rect that includes cells above and below the road
    // The rect covers y=48..52, x=50..60
    // Only cells adjacent to road (y=49 and y=51) should get zoned
    // Cells ON the road (y=50) should NOT get zoned
    // Cells NOT adjacent to road (y=48, y=52) should NOT get zoned
    {
        let world = city.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            tick,
            ActionSource::Agent,
            GameAction::ZoneRect {
                min: (50, 48),
                max: (60, 52),
                zone_type: ZoneType::ResidentialLow,
            },
        );
    }
    city.tick(1);

    let grid = city.grid();

    // Cells adjacent to road (y=49 and y=51) should be zoned
    for x in 50..=60 {
        assert_eq!(
            grid.get(x, 49).zone,
            ZoneType::ResidentialLow,
            "cell ({x}, 49) should be zoned ResidentialLow (adjacent to road)"
        );
        assert_eq!(
            grid.get(x, 51).zone,
            ZoneType::ResidentialLow,
            "cell ({x}, 51) should be zoned ResidentialLow (adjacent to road)"
        );
    }

    // Road cells should NOT be zoned (they stay as Road cell type)
    for x in 50..=60 {
        assert_eq!(
            grid.get(x, 50).cell_type,
            CellType::Road,
            "cell ({x}, 50) should still be Road"
        );
    }

    // Non-adjacent cells (y=48, y=52) should NOT be zoned
    for x in 50..=60 {
        assert_eq!(
            grid.get(x, 48).zone,
            ZoneType::None,
            "cell ({x}, 48) should NOT be zoned (not adjacent to road)"
        );
        assert_eq!(
            grid.get(x, 52).zone,
            ZoneType::None,
            "cell ({x}, 52) should NOT be zoned (not adjacent to road)"
        );
    }
}

// ---------------------------------------------------------------------------
// 5. SetTaxRates updates ExtendedBudget
// ---------------------------------------------------------------------------

#[test]
fn test_agent_set_tax_rates() {
    let mut city = TestCity::new();

    // Verify default tax rates first
    let defaults = city.resource::<ExtendedBudget>().zone_taxes.clone();
    assert!(
        (defaults.residential - 0.10).abs() < f32::EPSILON,
        "default residential tax should be 0.10"
    );

    // Push SetTaxRates action
    {
        let world = city.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            tick,
            ActionSource::Agent,
            GameAction::SetTaxRates {
                residential: 0.15,
                commercial: 0.20,
                industrial: 0.12,
                office: 0.18,
            },
        );
    }
    city.tick(1);

    // Verify tax rates changed
    let extended = city.resource::<ExtendedBudget>();
    assert!(
        (extended.zone_taxes.residential - 0.15).abs() < f32::EPSILON,
        "residential tax should be 0.15, got {}",
        extended.zone_taxes.residential
    );
    assert!(
        (extended.zone_taxes.commercial - 0.20).abs() < f32::EPSILON,
        "commercial tax should be 0.20, got {}",
        extended.zone_taxes.commercial
    );
    assert!(
        (extended.zone_taxes.industrial - 0.12).abs() < f32::EPSILON,
        "industrial tax should be 0.12, got {}",
        extended.zone_taxes.industrial
    );
    assert!(
        (extended.zone_taxes.office - 0.18).abs() < f32::EPSILON,
        "office tax should be 0.18, got {}",
        extended.zone_taxes.office
    );

    // Verify success logged
    let log = city.resource::<ActionResultLog>();
    assert_eq!(log.last_n(1)[0].1, ActionResult::Success);
}

// ---------------------------------------------------------------------------
// 6. Observation includes recent action results
// ---------------------------------------------------------------------------

#[test]
fn test_agent_observation_has_action_results() {
    let mut city = TestCity::new();

    // Push an action so there's something in the log
    {
        let world = city.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            tick,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (30, 30),
                end: (40, 30),
                road_type: RoadType::Local,
            },
        );
    }

    // Tick to execute the action AND build the observation
    city.tick(1);

    // Check that the observation's recent_action_results is non-empty
    let obs = city.resource::<CurrentObservation>();
    assert!(
        !obs.observation.recent_action_results.is_empty(),
        "CurrentObservation.observation.recent_action_results should be non-empty \
         after an action was executed"
    );

    // The result should indicate success
    let entry = &obs.observation.recent_action_results[0];
    assert!(
        entry.success,
        "the PlaceRoadLine action should have succeeded"
    );
    assert!(
        entry.action_summary.contains("PlaceRoadLine"),
        "action summary should mention PlaceRoadLine, got: {}",
        entry.action_summary
    );
}

// ---------------------------------------------------------------------------
// 7. Replay records and replays
// ---------------------------------------------------------------------------

#[test]
fn test_agent_replay_records_and_replays() {
    // --- City 1: record ---
    let mut city1 = TestCity::new();

    // Start the recorder
    {
        let world = city1.world_mut();
        let mut recorder = world.resource_mut::<ReplayRecorder>();
        recorder.start(42, "E2ETest".to_string(), 0);
    }

    // Push a road action
    {
        let world = city1.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            tick,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (50, 50),
                end: (60, 50),
                road_type: RoadType::Local,
            },
        );
    }

    // Tick to let the record_actions system capture and the executor execute
    city1.tick(1);

    // Verify road was placed in city1
    assert_eq!(
        city1.grid().get(55, 50).cell_type,
        CellType::Road,
        "city1 should have road at (55,50)"
    );

    // Stop recording
    let replay = {
        let world = city1.world_mut();
        let tick = world.resource::<TickCounter>().0;
        let mut recorder = world.resource_mut::<ReplayRecorder>();
        recorder.stop(tick, 0)
    };

    assert!(
        !replay.entries.is_empty(),
        "replay should have recorded entries"
    );
    assert!(
        replay.validate().is_ok(),
        "replay file should be valid: {:?}",
        replay.validate()
    );

    // Verify the recorded entry contains our road action
    let has_road_action = replay.entries.iter().any(|e| {
        matches!(
            e.action,
            GameAction::PlaceRoadLine {
                start: (50, 50),
                end: (60, 50),
                ..
            }
        )
    });
    assert!(
        has_road_action,
        "replay should contain the PlaceRoadLine action"
    );

    // --- City 2: replay ---
    let mut city2 = TestCity::new();

    // Verify city2 has no road at (55,50) initially
    assert_eq!(
        city2.grid().get(55, 50).cell_type,
        CellType::Grass,
        "city2 should start with grass at (55,50)"
    );

    // Load the replay
    {
        let world = city2.world_mut();
        let mut player = world.resource_mut::<ReplayPlayer>();
        player.load(replay);
    }

    // Tick enough to let the replay player feed actions.
    // The replay entries have a tick value from city1. The feed_replay_actions
    // system matches on the current TickCounter. We need to advance city2 to
    // the tick where the action was recorded.
    // Since city1 had 1 tick before the action was pushed (from TestCity::new()
    // doing one update), let's tick generously to cover the recorded tick.
    city2.tick(5);

    // Verify the road was placed by replay
    let grid = city2.grid();
    assert_eq!(
        grid.get(55, 50).cell_type,
        CellType::Road,
        "replay should have placed the road at (55,50) in city2"
    );
}
