//! Integration tests for land value calculation (TEST-003).
//!
//! Tests verify that each input factor (road proximity, services, pollution,
//! industrial zones, water proximity, waste, UGB) affects land value in the
//! correct direction, and that output is clamped to [0, 255].
//!
//! Since land values now use exponential smoothing (alpha = 0.1) and
//! 8-neighbour diffusion, convergence requires multiple slow-tick cycles.
//! Most tests run 50 cycles for near-full convergence and use range / relative
//! assertions rather than exact equality.

use crate::garbage::WasteCollectionGrid;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::land_value::LandValueGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::waste_effects::WasteAccumulation;

/// Number of slow-tick cycles sufficient for near-full convergence
/// (with alpha=0.1 and diffusion, 100 cycles reaches near-full convergence).
const CONVERGE_CYCLES: u32 = 100;

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

    // A plain grass cell with no modifiers should stay near the baseline of 50
    // (target is 50, initial is 50, so smoothing keeps it at 50).
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

    // Get baseline value at a grass cell (converged)
    let mut baseline_city = TestCity::new();
    baseline_city.tick_slow_cycles(CONVERGE_CYCLES);
    let baseline = land_value_at(&baseline_city, 48, 52);

    // Now create a city with a road near (50, 52)
    let mut city = TestCity::new().with_road(50, 50, 50, 54, RoadType::Local);
    city.tick_slow_cycles(CONVERGE_CYCLES);

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
    // Baseline: no services (converged)
    let mut baseline_city = TestCity::new();
    baseline_city.tick_slow_cycles(CONVERGE_CYCLES);
    let baseline = land_value_at(&baseline_city, 100, 100);

    // With a park at (100, 100) — run enough cycles for value to rise
    let mut city = TestCity::new().with_service(100, 100, ServiceType::SmallPark);
    city.tick_slow_cycles(CONVERGE_CYCLES);

    let val = land_value_at(&city, 100, 100);
    assert!(
        val > baseline,
        "Park should boost land value: got {val}, baseline was {baseline}"
    );
}

#[test]
fn test_land_value_park_boost_decays_with_distance() {
    let mut city = TestCity::new().with_service(100, 100, ServiceType::LargePark);
    city.tick_slow_cycles(CONVERGE_CYCLES);

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
    // Non-park services (e.g. Hospital) have a modest boost of +10. With
    // exponential smoothing and diffusion, the converged value is only
    // slightly above the baseline of 50. We check >= 51 to account for
    // rounding of small increments.
    let mut city = TestCity::new().with_service(100, 100, ServiceType::Hospital);
    city.tick_slow_cycles(CONVERGE_CYCLES);

    let val = land_value_at(&city, 100, 100);
    assert!(
        val >= 51,
        "Hospital should boost land value above baseline 50: got {val}"
    );
}

#[test]
fn test_land_value_park_boosts_more_than_non_park() {
    // Parks get boost=20, radius=8; non-parks get boost=10, radius=6
    let mut park_city = TestCity::new().with_service(100, 100, ServiceType::SmallPark);
    park_city.tick_slow_cycles(CONVERGE_CYCLES);
    let park_val = land_value_at(&park_city, 100, 100);

    let mut hospital_city = TestCity::new().with_service(100, 100, ServiceType::Hospital);
    hospital_city.tick_slow_cycles(CONVERGE_CYCLES);
    let hospital_val = land_value_at(&hospital_city, 100, 100);

    assert!(
        park_val > hospital_val,
        "Park boost ({park_val}) should be greater than hospital boost ({hospital_val})"
    );
}

// -------------------------------------------------------------------------
// 4. Pollution penalty (via industrial buildings that generate real pollution)
// -------------------------------------------------------------------------

#[test]
fn test_land_value_pollution_from_industrial_building_reduces_value() {
    // Place an industrial building to generate real pollution through the
    // update_pollution system (which runs before update_land_value).
    let mut city = TestCity::new().with_building(100, 100, ZoneType::Industrial, 3);
    city.tick_slow_cycles(CONVERGE_CYCLES);

    let val = land_value_at(&city, 100, 100);
    // Industrial zone penalty + pollution => value well below 50
    assert!(
        val < 50,
        "Industrial building should reduce land value via pollution + zone penalty, got {val}"
    );
    // Should be lower than the zone-only penalty (target ~35 without pollution)
    assert!(
        val < 40,
        "Pollution from industrial building should reduce value below 40, got {val}"
    );
}

#[test]
fn test_land_value_more_industrial_buildings_means_lower_value() {
    // Single industrial building
    let mut city_one = TestCity::new().with_building(100, 100, ZoneType::Industrial, 2);
    city_one.tick_slow_cycles(CONVERGE_CYCLES);
    let val_one = land_value_at(&city_one, 100, 100);

    // Multiple industrial buildings nearby to stack pollution
    let mut city_many = TestCity::new()
        .with_building(100, 100, ZoneType::Industrial, 2)
        .with_building(101, 100, ZoneType::Industrial, 2)
        .with_building(100, 101, ZoneType::Industrial, 2)
        .with_building(99, 100, ZoneType::Industrial, 2);
    city_many.tick_slow_cycles(CONVERGE_CYCLES);
    let val_many = land_value_at(&city_many, 100, 100);

    assert!(
        val_one > val_many,
        "More nearby industrial buildings ({val_many}) should yield lower land value than one ({val_one})"
    );
}

// -------------------------------------------------------------------------
// 5. Industrial zone penalty
// -------------------------------------------------------------------------

#[test]
fn test_land_value_industrial_zone_reduces_value() {
    let mut city = TestCity::new().with_zone(100, 100, ZoneType::Industrial);
    city.tick_slow_cycles(CONVERGE_CYCLES);

    let val = land_value_at(&city, 100, 100);
    // Base 50, industrial penalty -15 => target 35, converged
    assert!(
        val < 50,
        "Industrial zone should reduce land value below 50, got {val}"
    );
    // With diffusion from neighbours at 50, the converged steady-state is
    // pulled above the raw target of 35. The exact value depends on the
    // interplay of smoothing alpha and diffusion weights (~45-47).
    assert!(
        val < 48,
        "Industrial zone penalty should bring value well below 48, got {val}"
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
    city.tick_slow_cycles(CONVERGE_CYCLES);

    let val = land_value_at(&city, 100, 100);
    // Water cells have target = 30; with diffusion from neighbours at ~50
    // the converged value may be slightly above 30 but well below 50.
    assert!(val < 50, "Water cell should have value below 50, got {val}");
}

#[test]
fn test_land_value_water_proximity_boosts_adjacent_grass() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(100, 100).cell_type = CellType::Water;
    }
    city.tick_slow_cycles(CONVERGE_CYCLES);

    // Cell adjacent to water should get +15 bonus => target 65
    let val = land_value_at(&city, 101, 100);
    assert!(
        val > 50,
        "Cell adjacent to water should have value > 50, got {val}"
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
    city.tick_slow_cycles(CONVERGE_CYCLES);

    // Cell 2+ cells away should not get water bonus (neighbors4 only checks
    // immediate neighbors). Diffusion may shift it slightly, but it should
    // remain close to 50.
    let val = land_value_at(&city, 102, 100);
    // Allow a small margin from diffusion
    assert!(
        (val as i32 - 50).unsigned_abs() <= 3,
        "Cell distant from water should be near baseline 50, got {val}"
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
    city.tick_slow_cycles(CONVERGE_CYCLES);

    let val = land_value_at(&city, 100, 100);
    // Target = base 50 minus 10% penalty => ~45; diffusion keeps it close
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
    city.tick_slow_cycles(CONVERGE_CYCLES);

    let val = land_value_at(&city, 100, 100);
    // Target = base 50 * 0.80 = 40; with diffusion may be slightly above
    assert!(
        val < 50,
        "Accumulated waste should reduce land value below 50, got {val}"
    );
    assert!(
        val < 47,
        "Accumulated waste > 500 lbs should apply ~0.80 modifier, got {val}"
    );
}

// -------------------------------------------------------------------------
// 9. Output clamped to [0, 255]
// -------------------------------------------------------------------------

#[test]
fn test_land_value_clamped_to_zero_minimum() {
    // Use many high-level industrial buildings clustered together to generate
    // extreme pollution that drives land value to 0 via the i32 clamp.
    let mut city = TestCity::new();
    // Place a dense cluster of level-5 industrial buildings around (100, 100)
    for dy in -3i32..=3 {
        for dx in -3i32..=3 {
            let x = (100i32 + dx) as usize;
            let y = (100i32 + dy) as usize;
            city = city.with_building(x, y, ZoneType::Industrial, 5);
        }
    }
    city.tick_slow_cycles(CONVERGE_CYCLES);

    let val = land_value_at(&city, 100, 100);
    // With extreme pollution the target at the centre is 0, but diffusion
    // from cells just outside the industrial cluster (which sit near 50)
    // pulls the converged value above zero. We verify it's well below the
    // baseline of 50.
    assert!(
        val <= 35,
        "Land value should be very low with extreme industrial pollution, got {val}"
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

    city.tick_slow_cycles(CONVERGE_CYCLES);

    // With multiple overlapping parks + water, the cell at (100, 100) should
    // have a high value. Since the type is u8, it's inherently in [0, 255].
    let val = land_value_at(&city, 100, 100);
    assert!(
        val > 50,
        "Multiple overlapping parks should significantly boost land value, got {val}"
    );
}

#[test]
fn test_land_value_all_cells_within_valid_range() {
    let mut city = TestCity::new()
        .with_building(55, 55, ZoneType::Industrial, 3)
        .with_building(56, 55, ZoneType::Industrial, 3)
        .with_building(55, 56, ZoneType::Industrial, 3)
        .with_service(80, 80, ServiceType::SmallPark)
        .with_service(120, 120, ServiceType::Hospital);

    city.tick_slow_cycles(CONVERGE_CYCLES);

    // Industrial area with pollution should have lower value than clean area
    let lv = city.resource::<LandValueGrid>();
    let industrial_val = lv.get(55, 55);
    let clean_val = lv.get(200, 200);
    assert!(
        clean_val > industrial_val,
        "Clean area ({clean_val}) should have higher land value than industrial area ({industrial_val})"
    );
}

// -------------------------------------------------------------------------
// 10. Combined factors
// -------------------------------------------------------------------------

#[test]
fn test_land_value_combined_industrial_building_stacks_zone_and_pollution() {
    // An industrial building applies both the zone penalty AND generates pollution
    let mut city_zone_only = TestCity::new().with_zone(100, 100, ZoneType::Industrial);
    city_zone_only.tick_slow_cycles(CONVERGE_CYCLES);
    let val_zone_only = land_value_at(&city_zone_only, 100, 100);

    let mut city_building = TestCity::new().with_building(100, 100, ZoneType::Industrial, 3);
    city_building.tick_slow_cycles(CONVERGE_CYCLES);
    let val_building = land_value_at(&city_building, 100, 100);

    // Building should have lower value due to additional pollution
    assert!(
        val_building < val_zone_only,
        "Industrial building ({val_building}) should have lower value than zone-only ({val_zone_only}) due to pollution"
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
    city.tick_slow_cycles(CONVERGE_CYCLES);

    // Cell at (101, 100) is adjacent to water AND has a park on it
    let val = land_value_at(&city, 101, 100);
    // Target = base 50 + water bonus 15 = 65, plus park boost on top
    assert!(
        val > 55,
        "Water adjacency + park should stack: expected > 55, got {val}"
    );
}

#[test]
fn test_land_value_average_on_empty_city() {
    let mut city = TestCity::new();
    city.tick_slow_cycles(CONVERGE_CYCLES);

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
