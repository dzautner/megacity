use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::land_value::LandValueGrid;
use crate::test_harness::TestCity;

// ====================================================================
// Train/Rail Transit (TRAF-012) integration tests
// ====================================================================

#[test]
fn test_train_transit_create_line_and_ridership() {
    use crate::train_transit::TrainTransitState;

    let mut city = TestCity::new()
        .with_road(10, 50, 90, 50, RoadType::Avenue)
        .with_building(12, 48, ZoneType::ResidentialLow, 2)
        .with_building(88, 48, ZoneType::CommercialLow, 2)
        .with_citizen((12, 48), (88, 48));

    // Place train stations and create a line
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<TrainTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = state
            .add_station(15, 50, "West Terminal".to_string(), &grid)
            .expect("station 1 should be placed");
        let s2 = state
            .add_station(85, 50, "East Terminal".to_string(), &grid)
            .expect("station 2 should be placed");
        state
            .add_line("Express Line".to_string(), vec![s1, s2])
            .expect("line should be created");
    }

    // Run a few slow cycles to accumulate ridership stats
    city.tick_slow_cycles(3);

    let state = city.resource::<TrainTransitState>();
    assert_eq!(state.stats.total_stations, 2, "should have 2 stations");
    assert_eq!(
        state.stats.total_active_lines, 1,
        "should have 1 active line"
    );
    assert_eq!(state.lines.len(), 1, "should have 1 line");
    assert!(state.lines[0].active, "line should be active");
}

#[test]
fn test_train_station_boosts_land_value() {
    use crate::train_transit::TrainTransitState;

    let mut city = TestCity::new();

    // Record baseline land value at station location
    let baseline = city.resource::<LandValueGrid>().get(50, 50);

    // Place train stations
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<TrainTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = state
            .add_station(50, 50, "Central".to_string(), &grid)
            .expect("station should be placed");
        let s2 = state
            .add_station(80, 50, "East".to_string(), &grid)
            .expect("station should be placed");
        state.add_line("Commuter Line".to_string(), vec![s1, s2]);
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
fn test_train_operating_cost_deduction() {
    use crate::train_transit::TrainTransitState;

    let mut city = TestCity::new().with_budget(100_000.0);

    // Place stations and line
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<TrainTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);
    }

    let cost = city.resource::<TrainTransitState>().total_weekly_cost();
    assert!(
        cost > 0.0,
        "train operating cost should be positive, got {}",
        cost
    );

    // Verify the cost is: 2 stations * $800/week + 1 line * $2000/week = $3600
    let expected = 2.0 * 800.0 + 1.0 * 2000.0;
    assert!(
        (cost - expected).abs() < 0.01,
        "expected weekly cost {}, got {}",
        expected,
        cost
    );

    // Verify monthly cost is 4x weekly
    let monthly = city.resource::<TrainTransitState>().total_monthly_cost();
    assert!(
        (monthly - expected * 4.0).abs() < 0.01,
        "expected monthly cost {}, got {}",
        expected * 4.0,
        monthly
    );
}

#[test]
fn test_train_station_on_water_rejected() {
    use crate::train_transit::TrainTransitState;

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(50, 50).cell_type = CellType::Water;

    let mut state = TrainTransitState::default();

    let result = state.add_station(50, 50, "Aquatic".to_string(), &grid);
    assert!(
        result.is_none(),
        "should not be able to place station on water"
    );

    let result = state.add_station(51, 50, "Dry".to_string(), &grid);
    assert!(result.is_some(), "should place station on grass");
}

#[test]
fn test_train_saveable_roundtrip_integration() {
    use crate::train_transit::TrainTransitState;
    use crate::Saveable;

    let mut state = TrainTransitState::default();
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    state.add_station(10, 10, "Alpha".to_string(), &grid);
    state.add_station(50, 50, "Beta".to_string(), &grid);
    state.add_line("Red".to_string(), vec![0, 1]);

    let bytes = state.save_to_bytes().expect("should produce bytes");
    let restored = TrainTransitState::load_from_bytes(&bytes);
    assert_eq!(restored.stations.len(), 2, "should restore 2 stations");
    assert_eq!(restored.lines.len(), 1, "should restore 1 line");
    assert_eq!(
        restored.stations[0].name, "Alpha",
        "station name should survive roundtrip"
    );
}
