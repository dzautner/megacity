use crate::grid::RoadType;
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;

#[test]
fn test_bulldoze_road_refunds_half_cost() {
    let initial_budget = 5000.0;
    let mut city =
        TestCity::new()
            .with_budget(initial_budget)
            .with_road(100, 100, 105, 100, RoadType::Avenue);

    // Verify road is placed
    city.assert_has_road(102, 100);

    let budget_before = city.budget().treasury;

    // Bulldoze one Avenue road cell -- should refund 50% of Avenue cost (20 * 0.5 = 10)
    city.bulldoze_road_at(102, 100);

    let budget_after = city.budget().treasury;
    let refund = budget_after - budget_before;
    let expected = RoadType::Avenue.cost() * 0.5;
    assert!(
        (refund - expected).abs() < 0.01,
        "Expected refund {expected}, got {refund}"
    );
}

#[test]
fn test_bulldoze_service_building_refunds_half_cost() {
    let initial_budget = 10000.0;
    let mut city =
        TestCity::new()
            .with_budget(initial_budget)
            .with_service(50, 50, ServiceType::Hospital);

    let budget_before = city.budget().treasury;

    // Bulldoze the hospital -- should refund 50% of 1000 = 500
    city.bulldoze_service_at(50, 50);

    let budget_after = city.budget().treasury;
    let refund = budget_after - budget_before;
    let expected = ServiceBuilding::cost(ServiceType::Hospital) * 0.5;
    assert!(
        (refund - expected).abs() < 0.01,
        "Expected refund {expected}, got {refund}"
    );
}

#[test]
fn test_bulldoze_multiple_roads_accumulates_refunds() {
    let initial_budget = 5000.0;
    let mut city = TestCity::new().with_budget(initial_budget).with_road(
        100,
        100,
        105,
        100,
        RoadType::Highway,
    );

    let budget_before = city.budget().treasury;

    // Bulldoze 3 Highway road cells
    city.bulldoze_road_at(101, 100);
    city.bulldoze_road_at(102, 100);
    city.bulldoze_road_at(103, 100);

    let budget_after = city.budget().treasury;
    let total_refund = budget_after - budget_before;
    let expected = RoadType::Highway.cost() * 0.5 * 3.0;
    assert!(
        (total_refund - expected).abs() < 0.01,
        "Expected total refund {expected}, got {total_refund}"
    );
}

#[test]
fn test_bulldoze_refund_allows_bankruptcy_recovery() {
    // Start with very low budget but expensive roads already placed
    let mut city = TestCity::new()
        .with_budget(0.0) // bankrupt!
        .with_road(100, 100, 110, 100, RoadType::Boulevard);

    assert!(city.budget().treasury < 1.0, "Should start near-bankrupt");

    // Bulldoze 5 Boulevard cells to recover money
    for x in 100..105 {
        city.bulldoze_road_at(x, 100);
    }

    let expected_refund = RoadType::Boulevard.cost() * 0.5 * 5.0;
    assert!(
        city.budget().treasury >= expected_refund - 0.01,
        "Treasury {} should be >= expected refund {}",
        city.budget().treasury,
        expected_refund
    );
    assert!(
        city.budget().treasury > 0.0,
        "Player should have recovered from bankruptcy via bulldoze refunds"
    );
}
