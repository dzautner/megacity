use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::land_value::LandValueGrid;
use crate::test_harness::TestCity;
use crate::undo_redo::{ActionHistory, CityAction};
use bevy::prelude::Mut;

// ====================================================================
// Undo/Redo integration tests (UX-001)
// ====================================================================

#[test]
fn test_undo_grid_road_restores_cell_and_treasury() {
    let mut city = TestCity::new().with_road(5, 5, 5, 10, RoadType::Local);

    city.tick(1);

    let cost = {
        let grid = city.grid();
        assert_eq!(grid.get(5, 5).cell_type, CellType::Road);
        let budget = city.budget();
        10_000.0 - budget.treasury
    };

    // Push an undo action for the road
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        let grid = world.resource::<WorldGrid>();
        let road_type = grid.get(5, 5).road_type;
        history.push(CityAction::PlaceGridRoad {
            x: 5,
            y: 5,
            road_type,
            cost,
        });
    });

    // Now undo it
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        if let Some(action) = history.pop_undo() {
            match &action {
                CityAction::PlaceGridRoad { x, y, cost, .. } => {
                    let mut grid = world.resource_mut::<WorldGrid>();
                    grid.get_mut(*x, *y).cell_type = CellType::Grass;
                    let mut budget = world.resource_mut::<CityBudget>();
                    budget.treasury += cost;
                }
                _ => panic!("Expected PlaceGridRoad action"),
            }
            history.push_redo(action);
        }
    });

    let grid = city.grid();
    assert_eq!(
        grid.get(5, 5).cell_type,
        CellType::Grass,
        "Road should be removed after undo"
    );

    let budget = city.budget();
    assert!(
        (budget.treasury - 10_000.0).abs() < 1.0,
        "Treasury should be restored after undo"
    );
}

#[test]
fn test_redo_grid_road_replaces_road_and_deducts() {
    let mut city = TestCity::new();

    let cost = 5.0;
    let road_type = RoadType::Local;

    // Push and then undo a road action
    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        history.push(CityAction::PlaceGridRoad {
            x: 3,
            y: 3,
            road_type,
            cost,
        });
        if let Some(action) = history.pop_undo() {
            history.push_redo(action);
        }
    });

    // Redo it manually
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        if let Some(action) = history.pop_redo() {
            match &action {
                CityAction::PlaceGridRoad {
                    x,
                    y,
                    road_type,
                    cost,
                    ..
                } => {
                    let mut grid = world.resource_mut::<WorldGrid>();
                    grid.get_mut(*x, *y).cell_type = CellType::Road;
                    grid.get_mut(*x, *y).road_type = *road_type;
                    let mut budget = world.resource_mut::<CityBudget>();
                    budget.treasury -= cost;
                }
                _ => panic!("Expected PlaceGridRoad action"),
            }
            history.push_undo_no_clear(action);
        }
    });

    let grid = city.grid();
    assert_eq!(
        grid.get(3, 3).cell_type,
        CellType::Road,
        "Road should be placed after redo"
    );
    assert_eq!(grid.get(3, 3).road_type, road_type);
}

#[test]
fn test_undo_zone_placement_clears_zone() {
    let mut city = TestCity::new()
        .with_road(5, 5, 5, 10, RoadType::Local)
        .with_zone(6, 5, ZoneType::ResidentialLow);

    city.tick(1);

    // Push zone action
    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        history.push(CityAction::PlaceZone {
            cells: vec![(6, 5, ZoneType::ResidentialLow)],
            cost: 0.0,
        });
    });

    // Undo the zone
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        if let Some(action) = history.pop_undo() {
            match &action {
                CityAction::PlaceZone { cells, .. } => {
                    let mut grid = world.resource_mut::<WorldGrid>();
                    for (x, y, _) in cells {
                        grid.get_mut(*x, *y).zone = ZoneType::None;
                    }
                }
                _ => panic!("Expected PlaceZone action"),
            }
            history.push_redo(action);
        }
    });

    let grid = city.grid();
    assert_eq!(
        grid.get(6, 5).zone,
        ZoneType::None,
        "Zone should be cleared after undo"
    );
}

#[test]
fn test_push_clears_redo_stack() {
    let mut city = TestCity::new();

    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        history.push(CityAction::PlaceGridRoad {
            x: 1,
            y: 1,
            road_type: RoadType::Local,
            cost: 5.0,
        });
        if let Some(action) = history.pop_undo() {
            history.push_redo(action);
        }
        assert!(history.can_redo(), "Should be able to redo after undo");

        // Pushing a new action should clear the redo stack
        history.push(CityAction::PlaceGridRoad {
            x: 2,
            y: 2,
            road_type: RoadType::Local,
            cost: 5.0,
        });
        assert!(
            !history.can_redo(),
            "Redo stack should be cleared after push"
        );
    });
}

#[test]
fn test_action_history_limit_100() {
    let mut city = TestCity::new();

    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        for i in 0..120 {
            history.push(CityAction::PlaceGridRoad {
                x: i % 256,
                y: 0,
                road_type: RoadType::Local,
                cost: 5.0,
            });
        }
        assert_eq!(
            history.undo_stack.len(),
            100,
            "History should be capped at 100 actions"
        );
    });
}

#[test]
fn test_undo_composite_action() {
    let mut city = TestCity::new();

    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        let composite = CityAction::Composite(vec![
            CityAction::PlaceGridRoad {
                x: 0,
                y: 0,
                road_type: RoadType::Local,
                cost: 5.0,
            },
            CityAction::PlaceGridRoad {
                x: 1,
                y: 0,
                road_type: RoadType::Local,
                cost: 5.0,
            },
        ]);
        history.push(composite);
        assert_eq!(
            history.undo_stack.len(),
            1,
            "Composite counts as one action"
        );

        let undone = history.pop_undo();
        assert!(undone.is_some());
        if let Some(CityAction::Composite(actions)) = &undone {
            assert_eq!(actions.len(), 2, "Composite should contain 2 sub-actions");
        } else {
            panic!("Expected Composite action");
        }
        assert_eq!(history.undo_stack.len(), 0);
    });
}

#[test]
fn test_undo_bulldoze_road_restores_road() {
    let mut city = TestCity::new().with_road(4, 4, 4, 9, RoadType::Local);

    city.tick(1);

    // Get road type before bulldozing
    let road_type = city.grid().get(4, 4).road_type;

    // Simulate bulldozing by clearing the road
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(4, 4).cell_type = CellType::Grass;
    }

    // Record the bulldoze action and undo it
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        history.push(CityAction::BulldozeRoad {
            x: 4,
            y: 4,
            road_type,
            refund: 2.0,
        });

        // Undo the bulldoze (should restore the road)
        if let Some(action) = history.pop_undo() {
            match &action {
                CityAction::BulldozeRoad {
                    x,
                    y,
                    road_type,
                    refund,
                } => {
                    let mut grid = world.resource_mut::<WorldGrid>();
                    grid.get_mut(*x, *y).cell_type = CellType::Road;
                    grid.get_mut(*x, *y).road_type = *road_type;
                    let mut budget = world.resource_mut::<CityBudget>();
                    budget.treasury -= refund;
                }
                _ => panic!("Expected BulldozeRoad action"),
            }
            history.push_redo(action);
        }
    });

    let grid = city.grid();
    assert_eq!(
        grid.get(4, 4).cell_type,
        CellType::Road,
        "Road should be restored after undoing bulldoze"
    );
    assert_eq!(grid.get(4, 4).road_type, road_type);
}

// Metro transit system (TRAF-006)
// ===========================================================================

#[test]
fn test_metro_station_placement_and_ridership() {
    use crate::metro_transit::MetroTransitState;

    let mut city = TestCity::new()
        .with_road(10, 50, 90, 50, RoadType::Avenue)
        .with_building(12, 48, ZoneType::ResidentialLow, 2)
        .with_building(88, 48, ZoneType::CommercialLow, 2)
        .with_citizen((12, 48), (88, 48));

    // Place metro stations and create a line
    {
        let world = city.world_mut();
        let mut metro = world.resource_mut::<MetroTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = metro
            .add_station(15, 50, "West Station".to_string(), &grid)
            .expect("station 1 should be placed");
        let s2 = metro
            .add_station(85, 50, "East Station".to_string(), &grid)
            .expect("station 2 should be placed");
        metro
            .add_line("Red Line".to_string(), vec![s1, s2])
            .expect("line should be created");
    }

    // Run a few slow cycles to accumulate ridership stats
    city.tick_slow_cycles(3);

    let metro = city.resource::<MetroTransitState>();
    assert_eq!(metro.stats.total_stations, 2, "should have 2 stations");
    assert_eq!(metro.stats.total_lines, 1, "should have 1 operational line");
}

#[test]
fn test_metro_land_value_boost() {
    use crate::metro_transit::MetroTransitState;

    let mut city = TestCity::new();

    // Record baseline land value at station location
    let baseline = city.resource::<LandValueGrid>().get(50, 50);

    // Place a metro station
    {
        let world = city.world_mut();
        let mut metro = world.resource_mut::<MetroTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = metro
            .add_station(50, 50, "Central".to_string(), &grid)
            .expect("station should be placed");
        let s2 = metro
            .add_station(80, 50, "East".to_string(), &grid)
            .expect("station should be placed");
        metro.add_line("Blue Line".to_string(), vec![s1, s2]);
    }

    // Run slow tick so land value boost is applied
    city.tick_slow_cycle();

    let boosted = city.resource::<LandValueGrid>().get(50, 50);
    assert!(
        boosted > baseline,
        "land value at station ({}) should exceed baseline ({})",
        boosted,
        baseline
    );
}

#[test]
fn test_metro_maintenance_cost() {
    use crate::metro_transit::MetroTransitState;

    let mut city = TestCity::new().with_budget(100_000.0);

    // Place stations and line
    {
        let world = city.world_mut();
        let mut metro = world.resource_mut::<MetroTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = metro.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = metro.add_station(20, 20, "B".to_string(), &grid).unwrap();
        metro.add_line("Red".to_string(), vec![s1, s2]);
    }

    let cost = city
        .resource::<MetroTransitState>()
        .total_monthly_maintenance();
    assert!(
        cost > 0.0,
        "metro maintenance cost should be positive, got {}",
        cost
    );

    // Verify the cost is: 2 stations * $500/week * 4 + 1 line * $1200/week * 4
    let expected = 2.0 * 500.0 * 4.0 + 1.0 * 1200.0 * 4.0;
    assert!(
        (cost - expected).abs() < 0.01,
        "expected cost {}, got {}",
        expected,
        cost
    );
}

#[test]
fn test_metro_station_on_water_rejected() {
    use crate::metro_transit::MetroTransitState;

    // Create a grid with water at (50,50)
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(50, 50).cell_type = CellType::Water;

    let mut metro = MetroTransitState::default();

    // Placing on water should fail
    let result = metro.add_station(50, 50, "Aquatic".to_string(), &grid);
    assert!(
        result.is_none(),
        "should not be able to place station on water"
    );

    // Placing on grass should succeed
    let result = metro.add_station(51, 50, "Dry".to_string(), &grid);
    assert!(result.is_some(), "should place station on grass");
}

#[test]
fn test_metro_travel_time_estimation() {
    use crate::metro_transit::MetroTransitState;

    let mut metro = MetroTransitState::default();
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    let s1 = metro
        .add_station(50, 50, "West".to_string(), &grid)
        .unwrap();
    let s2 = metro
        .add_station(100, 50, "East".to_string(), &grid)
        .unwrap();
    metro.add_line("Green".to_string(), vec![s1, s2]);

    // Travel from near station 1 to near station 2
    let time = metro
        .estimate_travel_time(52, 50, 98, 50)
        .expect("should have a viable route");
    assert!(time > 0.0, "travel time should be positive");
    assert!(
        time < 0.5,
        "metro trip should be under 30 min for ~50 cells"
    );

    // Travel from far away should fail
    let no_route = metro.estimate_travel_time(200, 200, 98, 50);
    assert!(
        no_route.is_none(),
        "should fail when too far from any station"
    );
}

#[test]
fn test_metro_saveable_roundtrip_integration() {
    use crate::metro_transit::MetroTransitState;
    use crate::Saveable;

    let mut state = MetroTransitState::default();
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    state.add_station(10, 10, "Alpha".to_string(), &grid);
    state.add_station(50, 50, "Beta".to_string(), &grid);
    state.add_line("Red".to_string(), vec![0, 1]);

    // Save
    let bytes = state.save_to_bytes().expect("should produce bytes");
    assert!(!bytes.is_empty());

    // Load
    let restored = MetroTransitState::load_from_bytes(&bytes);
    assert_eq!(restored.stations.len(), 2);
    assert_eq!(restored.lines.len(), 1);
    assert_eq!(restored.lines[0].name, "Red");
}
