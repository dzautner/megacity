use crate::grid::{CellType, RoadType};

// ---------------------------------------------------------------------------
// Parallel Road Drawing (UX-021)
// ---------------------------------------------------------------------------

#[test]
fn test_parallel_road_creates_two_segments() {
    use crate::test_harness::TestCity;

    // Simulate the parallel drawing behavior: place two parallel roads
    // at an offset to mimic what the parallel_draw system would do.
    let city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_road(100, 131, 110, 131, RoadType::Local);

    assert_eq!(city.segment_count(), 2);
    assert_eq!(city.segment_road_type(0), Some(RoadType::Local));
    assert_eq!(city.segment_road_type(1), Some(RoadType::Local));
}

#[test]
fn test_parallel_road_both_segments_rasterize() {
    use crate::test_harness::TestCity;

    let city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_road(100, 131, 110, 131, RoadType::Local);

    // Both roads should have rasterized cells
    city.assert_has_road(105, 128);
    city.assert_has_road(105, 131);

    // Check that the cells between the parallel roads are NOT roads
    let cell = city.cell(105, 130);
    assert_ne!(cell.cell_type, CellType::Road);
}

#[test]
fn test_parallel_road_highway_wider_offset() {
    use crate::test_harness::TestCity;

    // Highway roads should be placed further apart due to wider road width
    let city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 120, 120, 120, RoadType::Highway)
        .with_road(100, 126, 120, 126, RoadType::Highway);

    assert_eq!(city.segment_count(), 2);
    // Both segments should be highways
    assert_eq!(city.segment_road_type(0), Some(RoadType::Highway));
    assert_eq!(city.segment_road_type(1), Some(RoadType::Highway));
}

#[test]
fn test_parallel_road_more_cells_than_single() {
    use crate::test_harness::TestCity;

    let city_single =
        TestCity::new()
            .with_budget(100_000.0)
            .with_road(100, 128, 110, 128, RoadType::Local);
    let single_road_cells = city_single.road_cell_count();

    let city_double = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_road(100, 131, 110, 131, RoadType::Local);
    let double_road_cells = city_double.road_cell_count();

    // Two parallel roads should produce more road cells than one
    assert!(
        double_road_cells > single_road_cells,
        "Expected more road cells with two roads ({}) than one ({})",
        double_road_cells,
        single_road_cells
    );
}

#[test]
fn test_parallel_road_oneway_pair() {
    use crate::test_harness::TestCity;

    // One-way roads are a common use case for parallel drawing
    let city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 128, 115, 128, RoadType::OneWay)
        .with_road(100, 130, 115, 130, RoadType::OneWay);

    assert_eq!(city.segment_count(), 2);
    assert_eq!(city.segment_road_type(0), Some(RoadType::OneWay));
    assert_eq!(city.segment_road_type(1), Some(RoadType::OneWay));
}
