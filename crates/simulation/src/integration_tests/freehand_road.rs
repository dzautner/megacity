use crate::freehand_road::{
    filter_short_segments, simplify_rdp, FreehandDrawState, FREEHAND_MIN_SAMPLE_DIST,
    FREEHAND_MIN_SEGMENT_LEN, FREEHAND_SIMPLIFY_TOLERANCE,
};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::test_harness::TestCity;
use bevy::math::Vec2;

// ====================================================================
// Freehand road drawing tests (UX-020)
// ====================================================================

#[test]
fn test_freehand_state_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<FreehandDrawState>();
}

#[test]
fn test_freehand_state_default_disabled() {
    let city = TestCity::new();
    let state = city.resource::<FreehandDrawState>();
    assert!(!state.enabled);
    assert!(!state.drawing);
    assert!(state.raw_points.is_empty());
}

#[test]
fn test_freehand_simplify_straight_path_creates_single_segment() {
    // A straight line of points should simplify to 2 points (= 1 segment)
    let points: Vec<Vec2> = (0..20)
        .map(|i| Vec2::new(i as f32 * FREEHAND_MIN_SAMPLE_DIST, 0.0))
        .collect();
    let simplified = simplify_rdp(&points, FREEHAND_SIMPLIFY_TOLERANCE);
    assert_eq!(
        simplified.len(),
        2,
        "straight line should simplify to 2 endpoints"
    );
}

#[test]
fn test_freehand_simplify_l_shaped_path_keeps_corner() {
    // L-shaped path: go right then go down
    let mut points = Vec::new();
    for i in 0..10 {
        points.push(Vec2::new(i as f32 * FREEHAND_MIN_SAMPLE_DIST, 0.0));
    }
    for i in 1..10 {
        points.push(Vec2::new(
            9.0 * FREEHAND_MIN_SAMPLE_DIST,
            i as f32 * FREEHAND_MIN_SAMPLE_DIST,
        ));
    }
    let simplified = simplify_rdp(&points, FREEHAND_SIMPLIFY_TOLERANCE);
    // Should be 3 points: start, corner, end
    assert!(
        simplified.len() >= 3,
        "L-shape should have at least 3 points, got {}",
        simplified.len()
    );
}

#[test]
fn test_blueprint_place_skips_water_cells() {
    let mut city = TestCity::new();

    // Set some cells to water
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(80, 80).cell_type = CellType::Water;
        grid.get_mut(81, 80).cell_type = CellType::Water;
    }

    let bp = crate::blueprints::Blueprint {
        name: "Test".to_string(),
        width: 3,
        height: 1,
        segments: vec![],
        zone_cells: vec![
            crate::blueprints::BlueprintZoneCell {
                dx: 0,
                dy: 0,
                zone_type: crate::blueprints::BlueprintZoneType::ResidentialLow,
            },
            crate::blueprints::BlueprintZoneCell {
                dx: 1,
                dy: 0,
                zone_type: crate::blueprints::BlueprintZoneType::ResidentialLow,
            },
            crate::blueprints::BlueprintZoneCell {
                dx: 2,
                dy: 0,
                zone_type: crate::blueprints::BlueprintZoneType::ResidentialLow,
            },
        ],
    };

    let world = city.world_mut();
    world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
        world.resource_scope(|world, mut segs: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                let result = bp.place(&mut grid, &mut segs, &mut roads, 80, 80);
                // Two cells are water, only one should be placed
                assert_eq!(result.zones_placed, 1, "should skip water cells");
            });
        });
    });
}

#[test]
fn test_blueprint_library_saveable_persistence() {
    let mut lib = crate::blueprints::BlueprintLibrary::default();
    lib.add(crate::blueprints::Blueprint {
        name: "Saved Layout".to_string(),
        width: 15,
        height: 15,
        segments: vec![crate::blueprints::BlueprintSegment {
            p0: [0.0, 0.0],
            p1: [80.0, 0.0],
            p2: [160.0, 0.0],
            p3: [240.0, 0.0],
            road_type: crate::blueprints::BlueprintRoadType::Boulevard,
        }],
        zone_cells: vec![crate::blueprints::BlueprintZoneCell {
            dx: 1,
            dy: 0,
            zone_type: crate::blueprints::BlueprintZoneType::CommercialHigh,
        }],
    });

    // Save to bytes and restore
    use crate::Saveable;
    let bytes = lib.save_to_bytes().expect("non-empty library should save");
    let restored = crate::blueprints::BlueprintLibrary::load_from_bytes(&bytes);

    assert_eq!(restored.count(), 1);
    let bp = restored.get(0).unwrap();
    assert_eq!(bp.name, "Saved Layout");
    assert_eq!(bp.segments.len(), 1);
    assert_eq!(bp.zone_cells.len(), 1);
}

#[test]
fn test_blueprint_multiple_placements_are_independent() {
    let city = TestCity::new().with_zone_rect(10, 10, 12, 12, ZoneType::Industrial);

    let grid = city.grid();
    let segments = city.road_segments();
    let bp =
        crate::blueprints::Blueprint::capture(grid, segments, 10, 10, 3, 3, "Factory".to_string());
    assert_eq!(bp.zone_cells.len(), 9);

    let mut city = city;

    // Place at two different locations
    let world = city.world_mut();
    world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
        world.resource_scope(|world, mut segs: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                let r1 = bp.place(&mut grid, &mut segs, &mut roads, 60, 60);
                assert_eq!(r1.zones_placed, 9);

                let r2 = bp.place(&mut grid, &mut segs, &mut roads, 80, 80);
                assert_eq!(r2.zones_placed, 9);
            });
        });
    });
}

#[test]
fn test_freehand_filter_removes_short_segments() {
    let points = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0), // too short
        Vec2::new(100.0, 0.0),
        Vec2::new(200.0, 0.0),
    ];
    let filtered = filter_short_segments(&points, FREEHAND_MIN_SEGMENT_LEN);
    // First two points are too close, so the 10.0 one gets filtered
    assert!(
        filtered.len() <= 3,
        "short segments should be filtered, got {} points",
        filtered.len()
    );
}

#[test]
fn test_freehand_sample_enforces_min_distance() {
    let mut state = FreehandDrawState::default();
    state.enabled = true;
    state.drawing = true;

    // First sample always accepted
    assert!(state.add_sample(Vec2::new(0.0, 0.0)));
    // Sample too close (< FREEHAND_MIN_SAMPLE_DIST)
    assert!(!state.add_sample(Vec2::new(1.0, 0.0)));
    // Sample far enough
    assert!(state.add_sample(Vec2::new(FREEHAND_MIN_SAMPLE_DIST + 1.0, 0.0)));
    assert_eq!(state.raw_points.len(), 2);
}

#[test]
fn test_freehand_reset_stroke_preserves_enabled() {
    let mut state = FreehandDrawState::default();
    state.enabled = true;
    state.drawing = true;
    state.raw_points.push(Vec2::ZERO);
    state.raw_points.push(Vec2::new(100.0, 0.0));

    state.reset_stroke();
    assert!(state.enabled, "reset_stroke should preserve enabled state");
    assert!(!state.drawing);
    assert!(state.raw_points.is_empty());
}

#[test]
fn test_freehand_simplify_and_create_road_segments() {
    // Simulate the full freehand workflow: collect points, simplify, create segments
    let mut city = TestCity::new().with_budget(100_000.0);

    // Generate a straight path of points in world coordinates
    let start_x = 128.0 * 16.0; // center of the grid
    let start_y = 128.0 * 16.0;
    let points: Vec<Vec2> = (0..10)
        .map(|i| Vec2::new(start_x + i as f32 * FREEHAND_MIN_SAMPLE_DIST, start_y))
        .collect();

    let simplified = simplify_rdp(&points, FREEHAND_SIMPLIFY_TOLERANCE);
    let simplified = filter_short_segments(&simplified, FREEHAND_MIN_SEGMENT_LEN);

    assert!(
        simplified.len() >= 2,
        "need at least 2 points to create a segment"
    );

    // Create road segments from the simplified path
    let world = city.world_mut();
    world.resource_scope(
        |world, mut segments: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
                world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                    for pair in simplified.windows(2) {
                        segments.add_straight_segment(
                            pair[0],
                            pair[1],
                            RoadType::Local,
                            24.0,
                            &mut grid,
                            &mut roads,
                        );
                    }
                });
            });
        },
    );

    let segment_count = city.road_segments().segments.len();
    assert!(
        segment_count >= 1,
        "should have at least 1 road segment, got {}",
        segment_count
    );

    // Verify road cells were created on the grid
    assert!(
        city.road_cell_count() > 0,
        "should have road cells on the grid"
    );
}

#[test]
fn test_freehand_curved_path_creates_multiple_segments() {
    // Simulate a curved freehand path
    let mut city = TestCity::new().with_budget(100_000.0);

    // Quarter-circle path
    let center_x = 128.0 * 16.0;
    let center_y = 128.0 * 16.0;
    let radius = 200.0;
    let n = 20;
    let points: Vec<Vec2> = (0..=n)
        .map(|i| {
            let angle = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
            Vec2::new(
                center_x + angle.cos() * radius,
                center_y + angle.sin() * radius,
            )
        })
        .collect();

    let simplified = simplify_rdp(&points, FREEHAND_SIMPLIFY_TOLERANCE);
    let simplified = filter_short_segments(&simplified, FREEHAND_MIN_SEGMENT_LEN);

    // Curved path should have more than 2 points
    assert!(
        simplified.len() > 2,
        "curved path should have >2 simplified points, got {}",
        simplified.len()
    );

    let world = city.world_mut();
    world.resource_scope(
        |world, mut segments: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
                world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                    for pair in simplified.windows(2) {
                        segments.add_straight_segment(
                            pair[0],
                            pair[1],
                            RoadType::Avenue,
                            24.0,
                            &mut grid,
                            &mut roads,
                        );
                    }
                });
            });
        },
    );

    let segment_count = city.road_segments().segments.len();
    assert!(
        segment_count > 1,
        "curved path should produce multiple segments, got {}",
        segment_count
    );
}
