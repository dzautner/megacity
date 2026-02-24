//! Integration tests for Power Line Transmission and Service Radius (POWER-011).

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::grid::{CellType, RoadType};
use crate::power_lines::{PowerLineGrid, POWER_RANGE};
use crate::test_harness::TestCity;

/// Helper: spawn a power plant at the given grid position.
fn spawn_generator(city: &mut TestCity, x: usize, y: usize) {
    city.world_mut().spawn(PowerPlant {
        plant_type: PowerPlantType::Coal,
        capacity_mw: 200.0,
        current_output_mw: 200.0,
        fuel_cost: 30.0,
        grid_x: x,
        grid_y: y,
    });
}

/// Tick enough for the power line propagation system to run (interval = 8).
fn tick_power(city: &mut TestCity) {
    city.tick(16);
}

#[test]
fn test_power_lines_follow_roads_from_generator() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local);

    spawn_generator(&mut city, 50, 50);
    tick_power(&mut city);

    let plg = city.resource::<PowerLineGrid>();
    let w = plg.width;

    // Road cells between generator and endpoint should have power lines.
    assert!(plg.has_line[50 * w + 55], "Road cell (55,50) should have power line");
    assert!(plg.has_line[50 * w + 60], "Road cell (60,50) should have power line");
    assert!(plg.has_line[50 * w + 65], "Road cell (65,50) should have power line");

    // A disconnected cell should not have power lines.
    assert!(!plg.has_line[10 * w + 10], "Disconnected cell should not have power line");
}

#[test]
fn test_service_radius_within_power_range() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local);

    spawn_generator(&mut city, 50, 50);
    tick_power(&mut city);

    let grid = city.grid();

    // Cell adjacent to road should have power (within POWER_RANGE).
    assert!(
        grid.get(55, 51).has_power,
        "Cell 1 away from road should have power"
    );

    // Cell at exactly POWER_RANGE distance from the road should have power.
    let test_y = 50 + POWER_RANGE;
    assert!(
        grid.get(55, test_y).has_power,
        "Cell at POWER_RANGE ({}) from road should have power",
        POWER_RANGE
    );

    // Cell beyond POWER_RANGE should NOT have power.
    let beyond_y = 50 + POWER_RANGE + 1;
    assert!(
        !grid.get(55, beyond_y).has_power,
        "Cell beyond POWER_RANGE should NOT have power"
    );
}

#[test]
fn test_disconnected_road_no_power() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_road(80, 50, 90, 50, RoadType::Local);

    // Generator on the first road segment only.
    spawn_generator(&mut city, 50, 50);
    tick_power(&mut city);

    let plg = city.resource::<PowerLineGrid>();
    let w = plg.width;

    // Connected road has power lines.
    assert!(plg.has_line[50 * w + 55]);

    // Disconnected road does NOT have power lines.
    assert!(!plg.has_line[50 * w + 85], "Disconnected road should not have power lines");

    // Building near disconnected road should not have power.
    let grid = city.grid();
    assert!(
        !grid.get(85, 51).has_power,
        "Cell near disconnected road should not have power"
    );
}

#[test]
fn test_transmission_efficiency_decreases_with_distance() {
    // Build a long road to test efficiency falloff.
    let mut city = TestCity::new()
        .with_road(10, 50, 110, 50, RoadType::Local);

    spawn_generator(&mut city, 10, 50);
    tick_power(&mut city);

    let plg = city.resource::<PowerLineGrid>();
    let w = plg.width;

    let eff_near = plg.efficiency[50 * w + 15]; // 5 cells away
    let eff_far = plg.efficiency[50 * w + 60]; // 50 cells away

    assert!(
        eff_near > eff_far,
        "Efficiency should decrease with distance: near={}, far={}",
        eff_near,
        eff_far
    );

    // Near the generator, efficiency should be close to 1.0.
    assert!(
        eff_near > 0.95,
        "Efficiency at 5 cells should be > 0.95, got {}",
        eff_near
    );

    // At 50 cells, efficiency should be ~0.90 (2% per 10 cells * 5).
    assert!(
        (eff_far - 0.90).abs() < 0.05,
        "Efficiency at 50 cells should be ~0.90, got {}",
        eff_far
    );
}

#[test]
fn test_no_generators_no_power() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local);

    // No generator spawned.
    tick_power(&mut city);

    let plg = city.resource::<PowerLineGrid>();
    assert_eq!(plg.line_cell_count, 0, "No generators means no power lines");
    assert_eq!(plg.powered_cell_count, 0, "No generators means no powered cells");
}

#[test]
fn test_powered_cell_count_tracked() {
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local);

    spawn_generator(&mut city, 50, 50);
    tick_power(&mut city);

    let plg = city.resource::<PowerLineGrid>();
    assert!(plg.line_cell_count > 0, "Should have power line cells");
    assert!(plg.powered_cell_count > 0, "Should have powered cells");
    assert!(
        plg.powered_cell_count >= plg.line_cell_count,
        "Powered cells ({}) should be >= line cells ({}) due to service radius",
        plg.powered_cell_count,
        plg.line_cell_count
    );
}

#[test]
fn test_multiple_generators_best_efficiency_wins() {
    // Two generators connected by a road.
    let mut city = TestCity::new()
        .with_road(20, 50, 80, 50, RoadType::Local);

    spawn_generator(&mut city, 20, 50);
    spawn_generator(&mut city, 80, 50);
    tick_power(&mut city);

    let plg = city.resource::<PowerLineGrid>();
    let w = plg.width;

    // Midpoint at x=50 is 30 cells from each generator.
    // With one generator: eff = 1.0 - (30/10)*0.02 = 0.94
    // With two: should still get best from either side = 0.94.
    let mid_eff = plg.efficiency[50 * w + 50];
    assert!(
        mid_eff > 0.90,
        "Midpoint efficiency should benefit from closer generator, got {}",
        mid_eff
    );

    // Cell at x=25 is 5 from left generator but 55 from right.
    // Best efficiency: from left = 1.0 - (5/10)*0.02 = 0.99.
    let near_eff = plg.efficiency[50 * w + 25];
    assert!(
        near_eff > 0.98,
        "Cell near left generator should have high efficiency, got {}",
        near_eff
    );
}

#[test]
fn test_saveable_roundtrip() {
    use crate::Saveable;

    let mut plg = PowerLineGrid::default();
    plg.has_line[500] = true;
    plg.efficiency[500] = 0.92;
    plg.line_cell_count = 1;
    plg.powered_cell_count = 10;

    let bytes = plg.save_to_bytes().unwrap();
    let restored = PowerLineGrid::load_from_bytes(&bytes);

    assert!(restored.has_line[500]);
    assert!((restored.efficiency[500] - 0.92).abs() < f32::EPSILON);
    assert_eq!(restored.line_cell_count, 1);
    assert_eq!(restored.powered_cell_count, 10);
}
