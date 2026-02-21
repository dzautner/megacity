use crate::grid::RoadType;
use crate::network_viz::NetworkVizData;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

#[test]
fn test_network_viz_power_source_assigns_cells() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_utility(10, 10, UtilityType::PowerPlant);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    // Source cell should be covered
    assert!(
        viz.power_source_color(10, 10).is_some(),
        "power source cell should be covered"
    );
    // Nearby road cell should be covered by the same source
    assert!(
        viz.power_source_color(15, 10).is_some(),
        "road cell within range should be covered"
    );
    // Far away cell should NOT be covered
    assert!(
        viz.power_source_color(200, 200).is_none(),
        "distant cell should not be covered"
    );
}

#[test]
fn test_network_viz_water_source_assigns_cells() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::WaterTower);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    assert!(
        viz.water_source_color(50, 50).is_some(),
        "water source cell should be covered"
    );
    assert!(
        viz.water_source_color(55, 50).is_some(),
        "road cell within range should have water source"
    );
}

#[test]
fn test_network_viz_multiple_power_sources_different_colors() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_road(40, 10, 50, 10, RoadType::Local)
        .with_utility(10, 10, UtilityType::PowerPlant)
        .with_utility(40, 10, UtilityType::SolarFarm);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    let color_a = viz.power_source_color(10, 10);
    let color_b = viz.power_source_color(40, 10);

    assert!(color_a.is_some(), "first source should be covered");
    assert!(color_b.is_some(), "second source should be covered");
    // Different sources should have different colors
    assert_ne!(
        color_a.unwrap(),
        color_b.unwrap(),
        "different sources should have different colors"
    );
}

#[test]
fn test_network_viz_disconnected_roads_no_coverage() {
    let mut city = TestCity::new()
        .with_road(10, 10, 15, 10, RoadType::Local)
        .with_road(20, 10, 25, 10, RoadType::Local) // disconnected segment
        .with_utility(10, 10, UtilityType::PowerPlant);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    // Connected segment should be covered
    assert!(
        viz.power_source_color(12, 10).is_some(),
        "connected road should be covered"
    );
    // Disconnected segment should NOT be covered
    assert!(
        viz.power_source_color(22, 10).is_none(),
        "disconnected road should not be covered"
    );
}

#[test]
fn test_network_viz_road_cells_tracked_for_pulse_lines() {
    let mut city = TestCity::new()
        .with_road(10, 10, 25, 10, RoadType::Local)
        .with_utility(10, 10, UtilityType::PowerPlant);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    // Should have road cells tracked for pulse animation
    assert!(
        !viz.power_road_cells.is_empty(),
        "should track road cells for pulse lines"
    );
    // Each road cell should have a distance value
    for &(_, _, dist, _) in &viz.power_road_cells {
        assert!(
            dist > 0,
            "road cells should have non-zero distance from source"
        );
    }
}

#[test]
fn test_network_viz_source_info_populated() {
    let mut city = TestCity::new()
        .with_road(10, 10, 25, 10, RoadType::Local)
        .with_utility(10, 10, UtilityType::PowerPlant);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    assert_eq!(
        viz.power_sources.len(),
        1,
        "should have exactly one power source"
    );

    let info = &viz.power_sources[0];
    assert_eq!(info.grid_x, 10);
    assert_eq!(info.grid_y, 10);
    assert!(info.cells_covered > 0, "source should cover some cells");
    assert!(
        info.effective_range > 0,
        "source should have positive range"
    );
}
