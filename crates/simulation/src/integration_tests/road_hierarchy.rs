use crate::grid::RoadType;
use crate::test_harness::TestCity;

// ====================================================================
// Road Hierarchy (TRAF-003)
// ====================================================================

#[test]
fn test_road_hierarchy_local_to_highway_creates_violation() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Build a Local road and a Highway that share a node (they meet at the
    // endpoint of the first segment and the start of the second).
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_road(100, 110, 100, 120, RoadType::Highway);

    // Tick enough for the hierarchy check to run (interval = 200)
    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        !state.violations.is_empty(),
        "Local-to-Highway connection should produce a hierarchy violation"
    );

    // The violation should report 2 levels skipped (diff=3, skipped=2)
    let v = &state.violations[0];
    assert_eq!(v.levels_skipped, 2, "Local(1) to Highway(4) skips 2 levels");
}

#[test]
fn test_road_hierarchy_proper_chain_no_violations() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Build a proper hierarchy: Local -> Avenue -> Boulevard -> Highway
    // Each pair differs by at most 1 level.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_road(100, 110, 100, 120, RoadType::Avenue)
        .with_road(100, 120, 100, 130, RoadType::Boulevard)
        .with_road(100, 130, 100, 140, RoadType::Highway);

    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        state.violations.is_empty(),
        "Proper hierarchy chain should produce no violations, got {} violations",
        state.violations.len()
    );
}

#[test]
fn test_road_hierarchy_same_type_no_violations() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Two local roads sharing a node — no violation
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_road(100, 110, 100, 120, RoadType::Local);

    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        state.violations.is_empty(),
        "Same road type connections should produce no violations"
    );
}

#[test]
fn test_road_hierarchy_violation_generates_advisor_message() {
    use crate::advisors::{AdvisorPanel, TipId};
    use crate::road_hierarchy::RoadHierarchyState;

    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_road(100, 110, 100, 120, RoadType::Highway);

    city.tick(200);

    // Verify violation was detected
    let state = city.resource::<RoadHierarchyState>();
    assert!(!state.violations.is_empty());

    // Verify advisor message was generated
    let panel = city.resource::<AdvisorPanel>();
    let hierarchy_msgs: Vec<_> = panel
        .messages
        .iter()
        .filter(|m| m.tip_id == TipId::RoadHierarchyViolation)
        .collect();
    assert!(
        !hierarchy_msgs.is_empty(),
        "Advisor should generate a road hierarchy violation message"
    );
    assert!(
        hierarchy_msgs[0].location.is_some(),
        "Hierarchy advisor message should include a location"
    );
}

#[test]
fn test_road_hierarchy_avenue_to_boulevard_no_violation() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Avenue (level 2) -> Boulevard (level 3) = diff 1, no violation
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Avenue)
        .with_road(100, 110, 100, 120, RoadType::Boulevard);

    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        state.violations.is_empty(),
        "Avenue to Boulevard should not produce a violation (adjacent levels)"
    );
}

#[test]
fn test_road_hierarchy_path_to_avenue_is_violation() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Path (level 0) -> Avenue (level 2) = diff 2, violation
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Path)
        .with_road(100, 110, 100, 120, RoadType::Avenue);

    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        !state.violations.is_empty(),
        "Path to Avenue should produce a hierarchy violation"
    );
    assert_eq!(state.violations[0].levels_skipped, 1);
}
