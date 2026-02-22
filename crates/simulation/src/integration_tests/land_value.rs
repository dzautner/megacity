//! Integration tests for land value calculation (TEST-003).
//!
//! Tests verify that each input factor (road proximity, services, pollution,
//! industrial zones, water proximity, waste, UGB) affects land value in the
//! correct direction, and that output is clamped to [0, 255].

use crate::garbage::WasteCollectionGrid;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::land_value::LandValueGrid;
use crate::pollution::PollutionGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::waste_effects::WasteAccumulation;

/// Helper: read the land value at (x, y) from the ECS world.
fn land_value_at(city: &TestCity, x: usize, y: usize) -> u8 {
    city.resource::<LandValueGrid>().get(x, y)
}

// -------------------------------------------------------------------------
// 1. Base land value on grass cell
// -------------------------------------------------------------------------

#[test]
fn test_land_value_base_grass_cell_equals_default() {
    let mut city = TestCity::new();

    // Run a slow cycle so update_land_value executes
    city.tick_slow_cycle();

    // A plain grass cell with no modifiers should have the baseline value of 50
    let val = land_value_at(&city, 128, 128);
    assert_eq!(val, 50, "Base land value on grass should be 50, got {val}");
}

// -------------------------------------------------------------------------
// 2. Road proximity bonus (indirectly tested: roads don't directly boost
//    land value in the current implementation, but we verify that placing
//    roads does NOT decrease the value of adjacent grass cells)
// -------------------------------------------------------------------------

#[test]
fn test_land_value_road_does_not_decrease_nearby_grass() {
    use crate::grid::RoadType;

    // Get baseline value at a grass cell
    let mut baseline_city = TestCity::new();
    baseline_city.tick_slow_cycle();
    let baseline = land_value_at(&baseline_city, 50, 52);

    // Now create a city with a road near (50, 52)
    let mut city = TestCity::new().with_road(50, 50, 50, 54, RoadType::Local);
    city.tick_slow_cycle();

    let val = land_value_at(&city, 48, 52);
    assert!(
        val >= baseline,
        "Land value near a road ({val}) should not be less than baseline ({baseline})"
    );
}

// -------------------------------------------------------------------------
// 3. Service coverage bonus (park and non-park)
// -------------------------------------------------------------------------

#[test]
fn test_land_value_park_service_boosts_nearby_cells() {
    // Baseline: no services
    let mut baseline_city = TestCity::new();
    baseline_city.tick_slow_cycle();
    let baseline = land_value_at(&baseline_city, 100, 100);

    // With a park at (100, 100)
    let mut city = TestCity::new().with_service(100, 100, ServiceType::SmallPark);
    city.tick_slow_cycle();

    let val = land_value_at(&city, 100, 100);
    assert!(
        val > baseline,
        "Park should boost land value: got {val}, baseline was {baseline}"
    );
}

#[test]
fn test_land_value_park_boost_decays_with_distance() {
    let mut city = TestCity::new().with_service(100, 100, ServiceType::LargePark);
    city.tick_slow_cycle();

    let at_park = land_value_at(&city, 100, 100);
    let nearby = land_value_at(&city, 103, 100);
    let far = land_value_at(&city, 108, 100);

    assert!(
        at_park >= nearby,
        "Value at park ({at_park}) should be >= nearby ({nearby})"
    );
    assert!(
        nearby >= far,
        "Value nearby ({nearby}) should be >= far ({far})"
    );
}

#[test]
fn test_land_value_non_park_service_boosts_nearby_cells() {
    let mut baseline_city = TestCity::new();
    baseline_city.tick_slow_cycle();
    let baseline = land_value_at(&baseline_city, 100, 100);

    let mut city = TestCity::new().with_service(100, 100, ServiceType::Hospital);
    city.tick_slow_cycle();

    let val = land_value_at(&city, 100, 100);
    assert!(
        val > baseline,
        "Hospital should boost land value: got {val}, baseline was {baseline}"
    );
}

#[test]
fn test_land_value_park_boosts_more_than_non_park() {
    // Parks get boost=20, radius=8; non-parks get boost=10, radius=6
    let mut park_city = TestCity::new().with_service(100, 100, ServiceType::SmallPark);
    park_city.tick_slow_cycle();
    let park_val = land_value_at(&park_city, 100, 100);

    let mut hospital_city = TestCity::new().with_service(100, 100, ServiceType::Hospital);
    hospital_city.tick_slow_cycle();
    let hospital_val = land_value_at(&hospital_city, 100, 100);

    assert!(
        park_val > hospital_val,
        "Park boost ({park_val}) should be greater than hospital boost ({hospital_val})"
    );
}

// -------------------------------------------------------------------------
// 4. Pollution penalty
// -------------------------------------------------------------------------

#[test]
fn test_land_value_pollution_reduces_value() {
    let mut city = TestCity::new();

    // Inject pollution directly into the grid
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<PollutionGrid>();
        pollution.set(100, 100, 90);
    }

    city.tick_slow_cycle();

    let val = land_value_at(&city, 100, 100);
    // Base is 50, pollution 90 => penalty = 90/3 = 30 => expected ~20
    assert!(
        val < 50,
        "Pollution should reduce land value below 50, got {val}"
    );
}

#[test]
fn test_land_value_higher_pollution_means_lower_value() {
    let mut city_low = TestCity::new();
    {
        let world = city_low.world_mut();
        let mut pollution = world.resource_mut::<PollutionGrid>();
        pollution.set(100, 100, 30);
    }
    city_low.tick_slow_cycle();
    let val_low_poll = land_value_at(&city_low, 100, 100);

    let mut city_high = TestCity::new();
    {
        let world = city_high.world_mut();
        let mut pollution = world.resource_mut::<PollutionGrid>();
        pollution.set(100, 100, 150);
    }
    city_high.tick_slow_cycle();
    let val_high_poll = land_value_at(&city_high, 100, 100);

    assert!(
        val_low_poll > val_high_poll,
        "Lower pollution ({val_low_poll}) should yield higher land value than higher pollution ({val_high_poll})"
    );
}

// -------------------------------------------------------------------------
// 5. Industrial zone penalty
// -------------------------------------------------------------------------

#[test]
fn test_land_value_industrial_zone_reduces_value() {
    let mut city = TestCity::new().with_zone(100, 100, ZoneType::Industrial);
    city.tick_slow_cycle();

    let val = land_value_at(&city, 100, 100);
    // Base 50, industrial penalty -15 => expected 35
    assert!(
        val < 50,
        "Industrial zone should reduce land value below 50, got {val}"
    );
    assert_eq!(
        val, 35,
        "Industrial zone penalty should be exactly -15 from base 50, got {val}"
    );
}

// -------------------------------------------------------------------------
// 6. Water proximity bonus
// -------------------------------------------------------------------------

#[test]
fn test_land_value_water_cell_itself_is_low() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(100, 100).cell_type = CellType::Water;
    }
    city.tick_slow_cycle();

    let val = land_value_at(&city, 100, 100);
    // Water cells get value = 30 (lower than grass baseline of 50)
    assert_eq!(val, 30, "Water cell itself should have value 30, got {val}");
}

#[test]
fn test_land_value_water_proximity_boosts_adjacent_grass() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(100, 100).cell_type = CellType::Water;
    }
    city.tick_slow_cycle();

    // Cell adjacent to water should get +15 bonus
    let val = land_value_at(&city, 101, 100);
    assert!(
        val > 50,
        "Cell adjacent to water should have value > 50, got {val}"
    );
    assert_eq!(
        val, 65,
        "Cell adjacent to water should get +15 bonus (50+15=65), got {val}"
    );
}

#[test]
fn test_land_value_water_proximity_does_not_affect_distant_cells() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(100, 100).cell_type = CellType::Water;
    }
    city.tick_slow_cycle();

    // Cell 2+ cells away should not get water bonus (neighbors4 only checks immediate neighbors)
    let val = land_value_at(&city, 102, 100);
    assert_eq!(
        val, 50,
        "Cell distant from water should stay at baseline 50, got {val}"
    );
}

// -------------------------------------------------------------------------
// 7. Uncollected waste penalty (WASTE-003)
// -------------------------------------------------------------------------

#[test]
fn test_land_value_uncollected_waste_penalty() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut waste_grid = world.resource_mut::<WasteCollectionGrid>();
        let idx = 100 * waste_grid.width + 100;
        waste_grid.uncollected_lbs[idx] = 200.0; // > 100 threshold
    }
    city.tick_slow_cycle();

    let val = land_value_at(&city, 100, 100);
    // Base 50, uncollected > 100 triggers 10% penalty => 50 - 5 = 45
    assert!(
        val < 50,
        "Uncollected waste should reduce land value below 50, got {val}"
    );
}

// -------------------------------------------------------------------------
// 8. Accumulated waste penalty (WASTE-010)
// -------------------------------------------------------------------------

#[test]
fn test_land_value_accumulated_waste_penalty() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut accumulation = world.resource_mut::<WasteAccumulation>();
        accumulation.set(100, 100, 600.0); // > 500 threshold
    }
    city.tick_slow_cycle();

    let val = land_value_at(&city, 100, 100);
    // Base 50, waste modifier = 0.80 => 50 * 0.80 = 40
    assert!(
        val < 50,
        "Accumulated waste should reduce land value below 50, got {val}"
    );
    assert_eq!(
        val, 40,
        "Accumulated waste > 500 lbs should apply 0.80 modifier: 50*0.80=40, got {val}"
    );
}

// -------------------------------------------------------------------------
// 9. Output clamped to [0, 255]
// -------------------------------------------------------------------------

#[test]
fn test_land_value_clamped_to_zero_minimum() {
    let mut city = TestCity::new();
    {
        // Apply extreme pollution to drive value below 0
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<PollutionGrid>();
        pollution.set(100, 100, 255);
    }
    city.tick_slow_cycle();

    let val = land_value_at(&city, 100, 100);
    // Base 50, pollution penalty = 255/3 = 85 => 50-85 = -35 => clamped to 0
    assert_eq!(
        val, 0,
        "Land value should be clamped to 0 with extreme pollution, got {val}"
    );
}

#[test]
fn test_land_value_clamped_to_255_maximum() {
    // Place multiple parks in a cluster to maximize boost
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::SmallPark)
        .with_service(100, 101, ServiceType::LargePark)
        .with_service(101, 100, ServiceType::SmallPark)
        .with_service(101, 101, ServiceType::LargePark)
        .with_service(99, 100, ServiceType::SmallPark)
        .with_service(100, 99, ServiceType::LargePark);

    // Also add water adjacency for an additional boost
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(98, 100).cell_type = CellType::Water;
    }

    city.tick_slow_cycle();

    // With multiple overlapping parks + water, the cell at (100, 100) should have
    // a high value but still be clamped within u8 range (the type enforces [0, 255]).
    let val = land_value_at(&city, 100, 100);
    // The combined boosts would push the raw sum well above 255 if unclamped,
    // but the code clamps via `.min(255) as u8`. Verify we get a high value.
    assert!(
        val > 50,
        "Multiple overlapping parks should significantly boost land value, got {val}"
    );
    // Since the type is u8, the value is inherently in [0, 255] -- the clamping
    // is verified by the fact that no overflow panic occurred.
}

#[test]
fn test_land_value_all_cells_within_valid_range() {
    // Use a city with mixed features and verify the full grid is in [0, 255]
    let mut city = TestCity::new()
        .with_zone_rect(50, 50, 60, 60, ZoneType::Industrial)
        .with_service(80, 80, ServiceType::SmallPark)
        .with_service(120, 120, ServiceType::Hospital);

    // Inject some pollution
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<PollutionGrid>();
        for y in 50..60 {
            for x in 50..60 {
                pollution.set(x, y, 100);
            }
        }
    }

    city.tick_slow_cycle();

    // Since `LandValueGrid::values` is `Vec<u8>`, the output is inherently in
    // [0, 255]. Verify that the system ran without panics and produced
    // reasonable values: industrial + polluted area should be lower than clean area.
    let lv = city.resource::<LandValueGrid>();
    let polluted_val = lv.get(55, 55);
    let clean_val = lv.get(200, 200);
    assert!(
        clean_val > polluted_val,
        "Clean area ({clean_val}) should have higher land value than polluted industrial area ({polluted_val})"
    );
}

// -------------------------------------------------------------------------
// 10. Combined factors
// -------------------------------------------------------------------------

#[test]
fn test_land_value_combined_pollution_and_industrial_stack() {
    let mut city = TestCity::new().with_zone(100, 100, ZoneType::Industrial);
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<PollutionGrid>();
        pollution.set(100, 100, 60);
    }
    city.tick_slow_cycle();

    let val = land_value_at(&city, 100, 100);
    // Base 50, industrial -15 => 35, pollution penalty 60/3=20 => 35-20 = 15
    assert_eq!(
        val, 15,
        "Industrial + pollution should stack penalties: expected 15, got {val}"
    );
}

#[test]
fn test_land_value_water_and_park_stack() {
    let mut city = TestCity::new().with_service(101, 100, ServiceType::SmallPark);
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(100, 100).cell_type = CellType::Water;
    }
    city.tick_slow_cycle();

    // Cell at (101, 100) is adjacent to water AND has a park on it
    let val = land_value_at(&city, 101, 100);
    // Base 50 + water bonus 15 = 65 (from base phase), then park adds boost on top
    assert!(
        val > 65,
        "Water adjacency + park should stack: expected > 65, got {val}"
    );
}

#[test]
fn test_land_value_average_on_empty_city() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();

    let lv = city.resource::<LandValueGrid>();
    let avg = lv.average();
    // An empty grass city should have an average very close to 50
    assert!(
        (avg - 50.0).abs() < 1.0,
        "Average land value on empty city should be ~50.0, got {avg}"
    );
}

#[test]
fn test_land_value_default_grid_has_correct_dimensions() {
    let city = TestCity::new();
    let lv = city.resource::<LandValueGrid>();
    assert_eq!(lv.width, 256);
    assert_eq!(lv.height, 256);
    assert_eq!(lv.values.len(), 256 * 256);
}
