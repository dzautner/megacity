//! Integration tests for wind-aware Gaussian plume pollution dispersion (SVC-021).

use crate::grid::ZoneType;
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;
use crate::wind::WindState;
use crate::wind_pollution::WindPollutionConfig;

// ====================================================================
// Downwind concentration tests
// ====================================================================

#[test]
fn test_gaussian_plume_pollution_highest_downwind_east() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 3);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0; // east
        wind.speed = 0.8;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();

    // Downwind (east) should have more pollution than upwind (west)
    let downwind_sum: u32 = (132..=138)
        .map(|x| grid.get(x, 128) as u32)
        .sum();
    let upwind_sum: u32 = (118..=124)
        .map(|x| grid.get(x, 128) as u32)
        .sum();

    assert!(
        downwind_sum > upwind_sum,
        "east wind: downwind_sum={} should be > upwind_sum={}",
        downwind_sum,
        upwind_sum
    );
}

#[test]
fn test_gaussian_plume_pollution_highest_downwind_north() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 3);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = std::f32::consts::FRAC_PI_2; // north
        wind.speed = 0.8;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();

    // Downwind (north = +y) should have more pollution than upwind (south = -y)
    let downwind_sum: u32 = (132..=138)
        .map(|y| grid.get(128, y) as u32)
        .sum();
    let upwind_sum: u32 = (118..=124)
        .map(|y| grid.get(128, y) as u32)
        .sum();

    assert!(
        downwind_sum > upwind_sum,
        "north wind: downwind_sum={} should be > upwind_sum={}",
        downwind_sum,
        upwind_sum
    );
}

// ====================================================================
// Wind direction change shifts pollution pattern
// ====================================================================

#[test]
fn test_wind_direction_change_shifts_pollution_pattern() {
    // East wind scenario
    let mut city_east = TestCity::new().with_building(128, 128, ZoneType::Industrial, 3);
    {
        let world = city_east.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0; // east
        wind.speed = 0.8;
    }
    city_east.tick_slow_cycle();

    // West wind scenario
    let mut city_west = TestCity::new().with_building(128, 128, ZoneType::Industrial, 3);
    {
        let world = city_west.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = std::f32::consts::PI; // west
        wind.speed = 0.8;
    }
    city_west.tick_slow_cycle();

    let grid_east = city_east.resource::<PollutionGrid>();
    let grid_west = city_west.resource::<PollutionGrid>();

    // With east wind, more pollution east of source
    let east_east: u32 = (134..=140)
        .map(|x| grid_east.get(x, 128) as u32)
        .sum();
    let east_west: u32 = (116..=122)
        .map(|x| grid_east.get(x, 128) as u32)
        .sum();

    // With west wind, more pollution west of source
    let west_east: u32 = (134..=140)
        .map(|x| grid_west.get(x, 128) as u32)
        .sum();
    let west_west: u32 = (116..=122)
        .map(|x| grid_west.get(x, 128) as u32)
        .sum();

    assert!(
        east_east > east_west,
        "east wind: east_sum={} should be > west_sum={}",
        east_east,
        east_west
    );
    assert!(
        west_west > west_east,
        "west wind: west_sum={} should be > east_sum={}",
        west_west,
        west_east
    );
}

// ====================================================================
// Scrubber technology reduces source strength
// ====================================================================

#[test]
fn test_scrubber_technology_reduces_pollution() {
    // Without scrubbers
    let mut city_no_scrub = TestCity::new().with_building(128, 128, ZoneType::Industrial, 3);
    {
        let world = city_no_scrub.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0;
        wind.speed = 0.6;
        let mut config = world.resource_mut::<WindPollutionConfig>();
        config.scrubbers_enabled = false;
    }
    city_no_scrub.tick_slow_cycle();

    // With scrubbers
    let mut city_scrub = TestCity::new().with_building(128, 128, ZoneType::Industrial, 3);
    {
        let world = city_scrub.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0;
        wind.speed = 0.6;
        let mut config = world.resource_mut::<WindPollutionConfig>();
        config.scrubbers_enabled = true;
    }
    city_scrub.tick_slow_cycle();

    let grid_no = city_no_scrub.resource::<PollutionGrid>();
    let grid_yes = city_scrub.resource::<PollutionGrid>();

    // Total pollution should be lower with scrubbers
    let total_no: u64 = grid_no.levels.iter().map(|&v| v as u64).sum();
    let total_yes: u64 = grid_yes.levels.iter().map(|&v| v as u64).sum();

    assert!(
        total_yes < total_no,
        "scrubbers should reduce total pollution: without={}, with={}",
        total_no,
        total_yes
    );
}

// ====================================================================
// Calm wind isotropic fallback
// ====================================================================

#[test]
fn test_calm_wind_isotropic_fallback_symmetric() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0;
        wind.speed = 0.05; // below calm threshold
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();

    // Under calm conditions, pollution should be roughly symmetric
    let east = grid.get(132, 128);
    let west = grid.get(124, 128);

    assert_eq!(
        east, west,
        "calm wind: pollution should be symmetric, east={}, west={}",
        east, west
    );
}

// ====================================================================
// Factory pollution follows wind direction (diagonal)
// ====================================================================

#[test]
fn test_factory_pollution_follows_diagonal_wind() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 3);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = std::f32::consts::FRAC_PI_4; // NE
        wind.speed = 0.8;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();

    // NE should have more pollution than SW
    let ne_sum: u32 = (132..=136)
        .flat_map(|x| (132..=136).map(move |y| grid.get(x, y) as u32))
        .sum();
    let sw_sum: u32 = (120..=124)
        .flat_map(|x| (120..=124).map(move |y| grid.get(x, y) as u32))
        .sum();

    assert!(
        ne_sum > sw_sum,
        "NE wind: ne_sum={} should be > sw_sum={}",
        ne_sum,
        sw_sum
    );
}
