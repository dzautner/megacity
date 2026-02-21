use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::test_harness::TestCity;

// ====================================================================
// Blueprint system integration tests
// ====================================================================

#[test]
fn test_blueprint_capture_empty_area_produces_empty_blueprint() {
    let city = TestCity::new();
    let grid = city.grid();
    let segments = city.road_segments();

    let bp =
        crate::blueprints::Blueprint::capture(grid, segments, 50, 50, 10, 10, "Empty".to_string());
    assert_eq!(bp.name, "Empty");
    assert_eq!(bp.width, 10);
    assert_eq!(bp.height, 10);
    assert!(bp.segments.is_empty(), "empty area should have no segments");
    assert!(
        bp.zone_cells.is_empty(),
        "empty area should have no zone cells"
    );
}

#[test]
fn test_blueprint_capture_and_place_zones() {
    let city = TestCity::new().with_zone_rect(20, 20, 24, 24, ZoneType::ResidentialLow);

    let grid = city.grid();
    let segments = city.road_segments();

    // Capture the zoned region
    let bp =
        crate::blueprints::Blueprint::capture(grid, segments, 20, 20, 5, 5, "ResBlock".to_string());
    assert_eq!(
        bp.zone_cells.len(),
        25,
        "5x5 region should capture 25 zone cells"
    );

    // Place it at a different location
    let mut city = city;
    let world = city.world_mut();
    world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
        world.resource_scope(|world, mut segs: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                let result = bp.place(&mut grid, &mut segs, &mut roads, 100, 100);
                assert_eq!(result.zones_placed, 25, "should place 25 zone cells");
            });
        });
    });

    // Verify zones were placed at the new location
    let grid = city.grid();
    for y in 100..105 {
        for x in 100..105 {
            assert_eq!(
                grid.get(x, y).zone,
                ZoneType::ResidentialLow,
                "cell ({},{}) should be ResidentialLow",
                x,
                y
            );
        }
    }
    // Verify original zone is still there
    assert_eq!(grid.get(20, 20).zone, ZoneType::ResidentialLow);
}

#[test]
fn test_blueprint_capture_and_place_road_segments() {
    let city = TestCity::new().with_road(30, 30, 30, 40, RoadType::Avenue);

    let initial_seg_count = city.road_segments().segments.len();
    assert!(initial_seg_count > 0, "should have at least one segment");

    let grid = city.grid();
    let segments = city.road_segments();

    // Capture region containing the road
    let bp =
        crate::blueprints::Blueprint::capture(grid, segments, 25, 25, 20, 20, "Road".to_string());
    assert!(!bp.segments.is_empty(), "should capture road segments");
    assert_eq!(
        bp.segments[0].road_type,
        crate::blueprints::BlueprintRoadType::Avenue,
        "captured segment should be Avenue type"
    );

    // Place at a new location
    let mut city = city;
    let world = city.world_mut();
    world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
        world.resource_scope(|world, mut segs: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                let result = bp.place(&mut grid, &mut segs, &mut roads, 100, 100);
                assert!(
                    result.segments_placed > 0,
                    "should place at least one segment"
                );
            });
        });
    });

    // Verify new segments were added
    let final_seg_count = city.road_segments().segments.len();
    assert!(
        final_seg_count > initial_seg_count,
        "segment count should increase after placing blueprint"
    );
}
