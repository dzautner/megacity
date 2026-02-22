use crate::grid::ZoneType;
use crate::policies::{Policies, Policy};
use crate::pollution::PollutionGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::wind::WindState;

// ====================================================================
// PollutionGrid resource tests
// ====================================================================

#[test]
fn test_pollution_grid_exists_in_new_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<PollutionGrid>();
}

#[test]
fn test_pollution_grid_starts_at_zero() {
    let city = TestCity::new();
    let grid = city.resource::<PollutionGrid>();
    let total: u64 = grid.levels.iter().map(|&v| v as u64).sum();
    assert_eq!(total, 0, "new city should have zero total pollution");
}

#[test]
fn test_pollution_grid_dimensions_match_config() {
    let city = TestCity::new();
    let grid = city.resource::<PollutionGrid>();
    assert_eq!(grid.width, crate::config::GRID_WIDTH);
    assert_eq!(grid.height, crate::config::GRID_HEIGHT);
    assert_eq!(grid.levels.len(), grid.width * grid.height);
}

// ====================================================================
// Industrial building generates pollution
// ====================================================================

#[test]
fn test_pollution_industrial_building_generates_pollution() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 1);

    // Run a slow cycle so update_pollution fires
    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let at_building = grid.get(128, 128);
    assert!(
        at_building > 0,
        "industrial building should generate pollution at its location, got {}",
        at_building
    );
}

#[test]
fn test_pollution_industrial_building_radiates_to_nearby_cells() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 1);

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    // Check a nearby cell (within radius 8)
    let nearby = grid.get(130, 130);
    assert!(
        nearby > 0,
        "pollution should radiate to nearby cells, got {} at (130,130)",
        nearby
    );
}

#[test]
fn test_pollution_industrial_decays_with_distance() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 1);

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let close = grid.get(129, 128); // distance 1
    let far = grid.get(135, 128); // distance 7
    assert!(
        close > far,
        "pollution should decay with distance: close={} should be > far={}",
        close,
        far
    );
}

#[test]
fn test_pollution_industrial_zero_beyond_radius() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 1);

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    // The radius is 8 Manhattan distance, and intensity for level 1 is (5+3)*1.0 = 8.
    // Beyond Manhattan distance 8, decay = (8 - dist).max(0) = 0.
    let far = grid.get(128, 140); // distance 12, well beyond radius
    assert_eq!(
        far, 0,
        "pollution should be 0 well beyond industrial radius, got {}",
        far
    );
}

#[test]
fn test_pollution_higher_level_industrial_produces_more() {
    let mut city_low = TestCity::new().with_building(128, 128, ZoneType::Industrial, 1);
    let mut city_high = TestCity::new().with_building(128, 128, ZoneType::Industrial, 3);

    city_low.tick_slow_cycle();
    city_high.tick_slow_cycle();

    let level1 = city_low.resource::<PollutionGrid>().get(128, 128);
    let level3 = city_high.resource::<PollutionGrid>().get(128, 128);
    assert!(
        level3 > level1,
        "higher level industrial should produce more pollution: level3={} should be > level1={}",
        level3,
        level1
    );
}

// ====================================================================
// Pollution levels clamped to [0, 255]
// ====================================================================

#[test]
fn test_pollution_saturates_without_wrapping() {
    // Place several industrial buildings near each other for high pollution.
    // With saturating_add the values should cap at 255 rather than wrapping.
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::Industrial, 3)
        .with_building(130, 128, ZoneType::Industrial, 3)
        .with_building(128, 130, ZoneType::Industrial, 3)
        .with_building(130, 130, ZoneType::Industrial, 3);

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    // Find the maximum pollution value -- it should be high due to stacking,
    // but not have wrapped around to a small value.
    let max_level = grid.levels.iter().copied().max().unwrap_or(0);
    assert!(
        max_level > 20,
        "stacked industrial buildings should produce significant pollution, max={}",
        max_level
    );
    // Verify that no cells have suspiciously low values near the buildings
    // (which would indicate u8 overflow wrapping).
    let at_center = grid.get(129, 129);
    assert!(
        at_center > 10,
        "center of industrial cluster should have high pollution (no wrap), got {}",
        at_center
    );
}

// ====================================================================
// Road pollution
// ====================================================================

#[test]
fn test_pollution_roads_add_base_pollution() {
    let mut city = TestCity::new().with_road(100, 128, 120, 128, crate::grid::RoadType::Local);

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    // Roads add +2 pollution. Check a cell along the road.
    let road_pollution = grid.get(110, 128);
    assert!(
        road_pollution >= 2,
        "road cells should have at least 2 pollution, got {}",
        road_pollution
    );
}

// ====================================================================
// Park reduces pollution
// ====================================================================

#[test]
fn test_pollution_park_reduces_nearby_pollution() {
    // Industrial at (128,128), park at (128,132) -- within park radius (6)
    let mut city_no_park = TestCity::new().with_building(128, 128, ZoneType::Industrial, 1);

    let mut city_with_park = TestCity::new()
        .with_building(128, 128, ZoneType::Industrial, 1)
        .with_service(128, 132, ServiceType::SmallPark);

    city_no_park.tick_slow_cycle();
    city_with_park.tick_slow_cycle();

    // Check pollution near the park location
    let no_park_level = city_no_park.resource::<PollutionGrid>().get(128, 132);
    let with_park_level = city_with_park.resource::<PollutionGrid>().get(128, 132);

    assert!(
        with_park_level < no_park_level,
        "park should reduce pollution: with_park={} should be < no_park={}",
        with_park_level,
        no_park_level
    );
}

#[test]
fn test_pollution_large_park_also_reduces_pollution() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::Industrial, 2)
        .with_service(128, 132, ServiceType::LargePark);

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    // The park is at (128,132). At the park itself, pollution should be reduced.
    let at_park = grid.get(128, 132);
    // Without the park, industrial level 2 at distance 4 would be (5+6) - 4 = 7.
    // Park reduces by up to 8 at distance 0. So at_park could be 0.
    assert!(
        at_park < 7,
        "large park should reduce pollution at its location, got {}",
        at_park
    );
}

// ====================================================================
// Policy affects pollution (IndustrialAirFilters)
// ====================================================================

#[test]
fn test_pollution_air_filters_policy_reduces_industrial_pollution() {
    let mut city_no_policy = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);

    let mut city_with_policy = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city_with_policy.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::IndustrialAirFilters);
    }

    city_no_policy.tick_slow_cycle();
    city_with_policy.tick_slow_cycle();

    let no_policy_level = city_no_policy.resource::<PollutionGrid>().get(128, 128);
    let with_policy_level = city_with_policy.resource::<PollutionGrid>().get(128, 128);

    assert!(
        with_policy_level < no_policy_level,
        "IndustrialAirFilters should reduce pollution: with_policy={} should be < no_policy={}",
        with_policy_level,
        no_policy_level
    );
}

// ====================================================================
// Wind drift affects pollution spread direction
// ====================================================================

#[test]
fn test_pollution_wind_drift_shifts_pollution_downwind() {
    // Place industrial building in the center and set wind blowing east (direction=0).
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = 0.0; // blowing east
        wind.speed = 0.8; // strong wind
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    // Downwind (east) should have more pollution than upwind (west) at same distance
    let east = grid.get(133, 128); // 5 cells east (downwind)
    let west = grid.get(123, 128); // 5 cells west (upwind)

    assert!(
        east > west,
        "downwind (east={}) should have more pollution than upwind (west={})",
        east,
        west
    );
}

#[test]
fn test_pollution_no_wind_drift_when_calm() {
    // With negligible wind, pollution should be symmetric.
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.speed = 0.0; // no wind
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let east = grid.get(133, 128);
    let west = grid.get(123, 128);

    // With no wind, pollution should be equal in opposite directions at same distance
    assert_eq!(
        east, west,
        "with no wind, pollution should be symmetric: east={}, west={}",
        east, west
    );
}

#[test]
fn test_pollution_wind_northward_shifts_pollution_north() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 2);
    {
        let world = city.world_mut();
        let mut wind = world.resource_mut::<WindState>();
        wind.direction = std::f32::consts::FRAC_PI_2; // blowing north (positive y)
        wind.speed = 0.8;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    // In the grid, "north" = positive y. Downwind = higher y.
    let north = grid.get(128, 133); // 5 cells north (downwind)
    let south = grid.get(128, 123); // 5 cells south (upwind)

    assert!(
        north > south,
        "northward wind: north={} should have more pollution than south={}",
        north,
        south
    );
}

// ====================================================================
// Pollution resets each slow tick (recalculated from scratch)
// ====================================================================

#[test]
fn test_pollution_recalculated_each_slow_tick() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::Industrial, 1);

    // Run two slow cycles
    city.tick_slow_cycle();
    let first = city.resource::<PollutionGrid>().get(128, 128);

    city.tick_slow_cycle();
    let second = city.resource::<PollutionGrid>().get(128, 128);

    // Because pollution is recalculated from scratch each tick (levels.fill(0) then rebuild),
    // the value should be roughly the same (may differ slightly due to wind drift changes).
    // The key invariant is that it does NOT accumulate unboundedly.
    assert!(
        second <= first + 5,
        "pollution should not accumulate unboundedly: first={}, second={}",
        first,
        second
    );
}

// ====================================================================
// Residential/commercial buildings do NOT generate pollution
// ====================================================================

#[test]
fn test_pollution_residential_does_not_generate_pollution() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::ResidentialLow, 1);
    {
        // Set wind to zero to avoid any drift effects
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let at_building = grid.get(128, 128);
    assert_eq!(
        at_building, 0,
        "residential building should not generate pollution, got {}",
        at_building
    );
}

#[test]
fn test_pollution_commercial_does_not_generate_pollution() {
    let mut city = TestCity::new().with_building(128, 128, ZoneType::CommercialLow, 1);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let at_building = grid.get(128, 128);
    assert_eq!(
        at_building, 0,
        "commercial building should not generate pollution, got {}",
        at_building
    );
}

// ====================================================================
// Multiple industrial buildings stack pollution
// ====================================================================

#[test]
fn test_pollution_multiple_industrial_buildings_stack() {
    let mut city_single = TestCity::new().with_building(128, 128, ZoneType::Industrial, 1);
    {
        let world = city_single.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    let mut city_double = TestCity::new()
        .with_building(128, 128, ZoneType::Industrial, 1)
        .with_building(130, 128, ZoneType::Industrial, 1);
    {
        let world = city_double.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city_single.tick_slow_cycle();
    city_double.tick_slow_cycle();

    // At (129, 128) which is between both buildings, the double city
    // should have more pollution due to stacking.
    let single_level = city_single.resource::<PollutionGrid>().get(129, 128);
    let double_level = city_double.resource::<PollutionGrid>().get(129, 128);

    assert!(
        double_level > single_level,
        "two industrial buildings should stack pollution: double={} should be > single={}",
        double_level,
        single_level
    );
}

// ====================================================================
// Empty city has zero pollution
// ====================================================================

#[test]
fn test_pollution_empty_city_stays_zero() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let grid = city.resource::<PollutionGrid>();
    let total: u64 = grid.levels.iter().map(|&v| v as u64).sum();
    assert_eq!(
        total, 0,
        "empty city with no roads or industry should have zero pollution, got {}",
        total
    );
}
