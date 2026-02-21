use crate::grid::{CellType, RoadType};

// ---------------------------------------------------------------------------
// Road upgrade integration tests (UX-022)
// ---------------------------------------------------------------------------

#[test]
fn test_road_upgrade_local_to_avenue() {
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(50000.0)
            .with_road(128, 128, 132, 128, RoadType::Local);

    assert_eq!(city.segment_count(), 1);
    assert_eq!(city.segment_road_type(0), Some(RoadType::Local));

    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RoadType::Avenue);
    assert_eq!(city.segment_road_type(0), Some(RoadType::Avenue));
}

#[test]
fn test_road_upgrade_full_path_to_boulevard() {
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(100000.0)
            .with_road(128, 128, 132, 128, RoadType::Path);

    // Path -> Local -> Avenue -> Boulevard
    let result = city.upgrade_segment_by_index(0);
    assert_eq!(result.unwrap(), RoadType::Local);

    let result = city.upgrade_segment_by_index(0);
    assert_eq!(result.unwrap(), RoadType::Avenue);

    let result = city.upgrade_segment_by_index(0);
    assert_eq!(result.unwrap(), RoadType::Boulevard);

    // Boulevard has no further upgrade
    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_err());
}

#[test]
fn test_road_upgrade_deducts_cost() {
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(50000.0)
            .with_road(128, 128, 132, 128, RoadType::Local);

    let budget_before = city.budget().treasury;
    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_ok());
    let budget_after = city.budget().treasury;

    // Cost should have been deducted
    assert!(budget_after < budget_before);
    // The deduction should be positive
    assert!(budget_before - budget_after > 0.0);
}

#[test]
fn test_road_upgrade_insufficient_funds() {
    use crate::test_harness::TestCity;

    let mut city = TestCity::new()
        .with_budget(0.0)
        .with_road(128, 128, 132, 128, RoadType::Local);

    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Not enough money");

    // Road type should remain unchanged
    assert_eq!(city.segment_road_type(0), Some(RoadType::Local));
}

#[test]
fn test_road_upgrade_preserves_connections() {
    use crate::test_harness::TestCity;

    // Create two connected road segments
    let mut city = TestCity::new()
        .with_budget(100000.0)
        .with_road(128, 128, 132, 128, RoadType::Local)
        .with_road(132, 128, 136, 128, RoadType::Local);

    assert_eq!(city.segment_count(), 2);

    // Upgrade first segment
    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RoadType::Avenue);

    // Second segment should be unchanged
    assert_eq!(city.segment_road_type(1), Some(RoadType::Local));

    // Both segments should still exist
    assert_eq!(city.segment_count(), 2);
}

#[test]
fn test_road_upgrade_updates_grid_cells() {
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(50000.0)
            .with_road(128, 128, 132, 128, RoadType::Local);

    // Before upgrade: cells should be Local type
    let grid = city.grid();
    for x in 128..=132 {
        let cell = grid.get(x, 128);
        if cell.cell_type == CellType::Road {
            assert_eq!(cell.road_type, RoadType::Local);
        }
    }

    city.upgrade_segment_by_index(0).unwrap();

    // After upgrade: road cells should be Avenue type
    let grid = city.grid();
    for x in 128..=132 {
        let cell = grid.get(x, 128);
        if cell.cell_type == CellType::Road {
            assert_eq!(cell.road_type, RoadType::Avenue);
        }
    }
}

#[test]
fn test_road_upgrade_highway_at_max() {
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(100000.0)
            .with_road(128, 128, 132, 128, RoadType::Highway);

    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Already at maximum road tier");
}
