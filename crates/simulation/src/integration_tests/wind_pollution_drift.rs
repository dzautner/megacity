use crate::grid::ZoneType;
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;
use crate::wind::WindState;

// ====================================================================
// 8-direction wind drift integration tests
// ====================================================================

#[test]
fn test_wind_drift_diagonal_ne_shifts_industrial_pollution() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = std::f32::consts::FRAC_PI_4; // NE
        wind.speed = 0.8;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    // NE (downwind) should have more pollution than SW (upwind) at same distance
    let ne = grid.get(133, 133); // 5 cells NE
    let sw = grid.get(123, 123); // 5 cells SW

    assert!(
        ne > sw,
        "NE wind: downwind NE={} should be > upwind SW={}",
        ne,
        sw
    );
}

#[test]
fn test_wind_drift_diagonal_sw_shifts_pollution() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 5.0 * std::f32::consts::FRAC_PI_4; // SW
        wind.speed = 0.8;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let sw = grid.get(123, 123);
    let ne = grid.get(133, 133);

    assert!(
        sw > ne,
        "SW wind: downwind SW={} should be > upwind NE={}",
        sw,
        ne
    );
}

#[test]
fn test_wind_drift_diagonal_se_shifts_pollution() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 7.0 * std::f32::consts::FRAC_PI_4; // SE
        wind.speed = 0.8;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let se = grid.get(133, 123);
    let nw = grid.get(123, 133);

    assert!(
        se > nw,
        "SE wind: downwind SE={} should be > upwind NW={}",
        se,
        nw
    );
}

#[test]
fn test_wind_drift_diagonal_nw_shifts_pollution() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 3.0 * std::f32::consts::FRAC_PI_4; // NW
        wind.speed = 0.8;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let nw = grid.get(123, 133);
    let se = grid.get(133, 123);

    assert!(
        nw > se,
        "NW wind: downwind NW={} should be > upwind SE={}",
        nw,
        se
    );
}

// ====================================================================
// Fractional drift / speed scaling
// ====================================================================

#[test]
fn test_wind_drift_speed_scales_shift_magnitude() {
    // Faster wind should shift pollution further downwind
    let mut city_slow = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city_slow.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0; // east
        wind.speed = 0.2;
    }

    let mut city_fast = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city_fast.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0; // east
        wind.speed = 0.9;
    }

    city_slow.tick_slow_cycle();
    city_fast.tick_slow_cycle();

    // Further downwind cells should have more pollution with faster wind
    let slow_far = city_slow.resource::<PollutionGrid>().get(135, 128);
    let fast_far = city_fast.resource::<PollutionGrid>().get(135, 128);

    assert!(
        fast_far >= slow_far,
        "faster wind should shift more pollution further: fast_far={}, slow_far={}",
        fast_far,
        slow_far
    );
}

// ====================================================================
// Calm wind threshold (speed < 0.1)
// ====================================================================

#[test]
fn test_wind_drift_calm_threshold_no_drift() {
    // Speed just below threshold: pollution should be symmetric
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0;
        wind.speed = 0.09; // below 0.1 threshold
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let east = grid.get(133, 128);
    let west = grid.get(123, 128);

    assert_eq!(
        east, west,
        "below calm threshold: pollution should be symmetric, east={}, west={}",
        east, west
    );
}

// ====================================================================
// Boundary drain
// ====================================================================

#[test]
fn test_wind_drift_boundary_drain_loses_total_pollution() {
    // When strong wind pushes pollution toward the edge, some should drain
    // off, reducing total pollution compared to the same setup in the center.
    // We compare total grid-wide pollution between edge and center buildings.
    let mut city_edge = TestCity::new().with_building(252, 128, ZoneType::Industrial, 2);
    {
        let world = city_edge.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0; // east, pushing toward edge
        wind.speed = 0.9;
    }

    let mut city_center = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city_center.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0; // east
        wind.speed = 0.9;
    }

    city_edge.tick_slow_cycle();
    city_center.tick_slow_cycle();

    let edge_total: u64 = city_edge
        .resource::<PollutionGrid>()
        .levels
        .iter()
        .map(|&v| v as u64)
        .sum();

    let center_total: u64 = city_center
        .resource::<PollutionGrid>()
        .levels
        .iter()
        .map(|&v| v as u64)
        .sum();

    assert!(
        edge_total < center_total,
        "boundary drain: edge total={} should be < center total={} (pollution lost off edge)",
        edge_total,
        center_total
    );
}
