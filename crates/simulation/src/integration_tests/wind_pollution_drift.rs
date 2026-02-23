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
    // Sum over a band of cells 2-4 units in each diagonal direction.
    // Sample points at Manhattan distance <= 8 to stay within pollution radius.
    let ne_sum: u32 = (130..=132)
        .flat_map(|x| (130..=132).map(move |y| grid.get(x, y) as u32))
        .sum();
    let sw_sum: u32 = (124..=126)
        .flat_map(|x| (124..=126).map(move |y| grid.get(x, y) as u32))
        .sum();

    assert!(
        ne_sum > sw_sum,
        "NE wind: downwind NE_sum={} should be > upwind SW_sum={}",
        ne_sum,
        sw_sum
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
    let sw_sum: u32 = (124..=126)
        .flat_map(|x| (124..=126).map(move |y| grid.get(x, y) as u32))
        .sum();
    let ne_sum: u32 = (130..=132)
        .flat_map(|x| (130..=132).map(move |y| grid.get(x, y) as u32))
        .sum();

    assert!(
        sw_sum > ne_sum,
        "SW wind: downwind SW_sum={} should be > upwind NE_sum={}",
        sw_sum,
        ne_sum
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
    // SE direction: +x, -y in the wind vector. Sum over nearby cells to
    // capture the asymmetry within the pollution radius.
    let se_sum: u32 = (130..=132)
        .flat_map(|x| (124..=126).map(move |y| grid.get(x, y) as u32))
        .sum();
    let nw_sum: u32 = (124..=126)
        .flat_map(|x| (130..=132).map(move |y| grid.get(x, y) as u32))
        .sum();

    assert!(
        se_sum > nw_sum,
        "SE wind: downwind SE_sum={} should be > upwind NW_sum={}",
        se_sum,
        nw_sum
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
    let nw_sum: u32 = (124..=126)
        .flat_map(|x| (130..=132).map(move |y| grid.get(x, y) as u32))
        .sum();
    let se_sum: u32 = (130..=132)
        .flat_map(|x| (124..=126).map(move |y| grid.get(x, y) as u32))
        .sum();

    assert!(
        nw_sum > se_sum,
        "NW wind: downwind NW_sum={} should be > upwind SE_sum={}",
        nw_sum,
        se_sum
    );
}

// ====================================================================
// Fractional drift / speed scaling
// ====================================================================

#[test]
fn test_wind_drift_speed_scales_shift_magnitude() {
    // Faster wind should shift pollution further downwind.
    // We use a wider sampling range and sum over multiple far-downwind cells
    // to avoid flaky failures from u8 rounding at individual cells.
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

    // Sum pollution over far-downwind cells (x=134..140) to reduce noise from
    // integer rounding. Faster wind should push more total pollution further east.
    let slow_grid = city_slow.resource::<PollutionGrid>();
    let fast_grid = city_fast.resource::<PollutionGrid>();
    let slow_far_sum: u32 = (134..=140)
        .map(|x| slow_grid.get(x, 128) as u32)
        .sum();
    let fast_far_sum: u32 = (134..=140)
        .map(|x| fast_grid.get(x, 128) as u32)
        .sum();

    assert!(
        fast_far_sum >= slow_far_sum,
        "faster wind should shift more pollution further: fast_far_sum={}, slow_far_sum={}",
        fast_far_sum,
        slow_far_sum
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
