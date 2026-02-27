//! Integration tests for the game action executor system (#1873).
//!
//! Each test pushes actions into the [`ActionQueue`], ticks the simulation, and
//! verifies the expected world mutations via the [`ActionResultLog`].

use crate::budget::ExtendedBudget;
use crate::game_actions::queue::ActionSource;
use crate::game_actions::result_log::ActionResultLog;
use crate::game_actions::{ActionError, ActionQueue, ActionResult, GameAction};
use crate::grid::{CellType, RoadType, ZoneType};
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

// -----------------------------------------------------------------------
// Road placement
// -----------------------------------------------------------------------

#[test]
fn test_executor_place_road_creates_road_cells() {
    let mut city = TestCity::new().with_budget(100_000.0);

    // Push a road action
    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (10, 10),
                end: (15, 10),
                road_type: RoadType::Local,
            },
        );
    }

    city.tick(1);

    // Verify cells are road
    for x in 10..=15 {
        assert_eq!(
            city.cell(x, 10).cell_type,
            CellType::Road,
            "Cell ({x}, 10) should be a road"
        );
    }

    // Verify success logged
    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].1, ActionResult::Success);
}

// -----------------------------------------------------------------------
// Zone placement
// -----------------------------------------------------------------------

#[test]
fn test_executor_zone_rect_sets_zone_type() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(10, 10, 15, 10, RoadType::Local);

    // Zone cells adjacent to the road
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

    // Cells at y=9 should be zoned (they are adjacent to road at y=10)
    for x in 10..=15 {
        assert_eq!(
            city.cell(x, 9).zone,
            ZoneType::ResidentialLow,
            "Cell ({x}, 9) should be ResidentialLow"
        );
    }
}

// -----------------------------------------------------------------------
// Bulldoze
// -----------------------------------------------------------------------

#[test]
fn test_executor_bulldoze_clears_road_cells() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(20, 20, 25, 20, RoadType::Local);

    // Confirm roads exist
    for x in 20..=25 {
        assert_eq!(city.cell(x, 20).cell_type, CellType::Road);
    }

    let budget_before = city.budget().treasury;

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Player,
            GameAction::BulldozeRect {
                min: (20, 20),
                max: (25, 20),
            },
        );
    }

    city.tick(1);

    for x in 20..=25 {
        assert_eq!(
            city.cell(x, 20).cell_type,
            CellType::Grass,
            "Cell ({x}, 20) should be grass after bulldoze"
        );
    }

    // Should have received a refund
    assert!(
        city.budget().treasury > budget_before,
        "Treasury should increase from bulldoze refund"
    );
}

// -----------------------------------------------------------------------
// Tax rates
// -----------------------------------------------------------------------

#[test]
fn test_executor_set_tax_rates_updates_extended_budget() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::SetTaxRates {
                residential: 0.15,
                commercial: 0.20,
                industrial: 0.12,
                office: 0.08,
            },
        );
    }

    city.tick(1);

    let ext = city.resource::<ExtendedBudget>();
    assert!((ext.zone_taxes.residential - 0.15).abs() < f32::EPSILON);
    assert!((ext.zone_taxes.commercial - 0.20).abs() < f32::EPSILON);
    assert!((ext.zone_taxes.industrial - 0.12).abs() < f32::EPSILON);
    assert!((ext.zone_taxes.office - 0.08).abs() < f32::EPSILON);
}

// -----------------------------------------------------------------------
// Insufficient funds
// -----------------------------------------------------------------------

#[test]
fn test_executor_insufficient_funds_returns_error() {
    let mut city = TestCity::new().with_budget(0.0);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (5, 5),
                end: (50, 5),
                road_type: RoadType::Highway,
            },
        );
    }

    city.tick(1);

    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(
        last[0].1,
        ActionResult::Error(ActionError::InsufficientFunds)
    );
}

// -----------------------------------------------------------------------
// Out of bounds
// -----------------------------------------------------------------------

#[test]
fn test_executor_out_of_bounds_returns_error() {
    let mut city = TestCity::new().with_budget(100_000.0);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (300, 300),
                end: (310, 300),
                road_type: RoadType::Local,
            },
        );
    }

    city.tick(1);

    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0].1, ActionResult::Error(ActionError::OutOfBounds));
}

// -----------------------------------------------------------------------
// Speed & pause
// -----------------------------------------------------------------------

#[test]
fn test_executor_set_speed_and_pause() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(0, ActionSource::Agent, GameAction::SetSpeed { speed: 3 });
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::SetPaused { paused: true },
        );
    }

    city.tick(1);

    let clock = city.resource::<GameClock>();
    assert!((clock.speed - 3.0).abs() < f32::EPSILON);
    assert!(clock.paused);
}

// -----------------------------------------------------------------------
// Road placement deducts treasury
// -----------------------------------------------------------------------

#[test]
fn test_executor_road_placement_deducts_cost() {
    let mut city = TestCity::new().with_budget(1_000.0);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceRoadLine {
                start: (10, 10),
                end: (12, 10),
                road_type: RoadType::Local,
            },
        );
    }

    city.tick(1);

    // 3 cells * $10 per Local road = $30
    let expected = 1_000.0 - 30.0;
    assert!(
        (city.budget().treasury - expected).abs() < 0.01,
        "Treasury should be ~{expected} but was {}",
        city.budget().treasury
    );
}

// -----------------------------------------------------------------------
// Utility placement insufficient funds
// -----------------------------------------------------------------------

#[test]
fn test_executor_place_utility_insufficient_funds() {
    let mut city = TestCity::new().with_budget(1.0);

    {
        let world = city.world_mut();
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.push(
            0,
            ActionSource::Agent,
            GameAction::PlaceUtility {
                pos: (50, 50),
                utility_type: crate::utilities::UtilityType::NuclearPlant,
            },
        );
    }

    city.tick(1);

    let log = city.resource::<ActionResultLog>();
    let last = log.last_n(1);
    assert_eq!(last[0].1, ActionResult::Error(ActionError::InsufficientFunds));
}
