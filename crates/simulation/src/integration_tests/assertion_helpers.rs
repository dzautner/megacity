use crate::grid::{CellType, RoadType, ZoneType};
use crate::test_harness::TestCity;

#[test]
fn assert_citizen_count_between_passes() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(110, 110, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (110, 110));

    city.assert_citizen_count_between(0, 10);
    city.assert_citizen_count_between(1, 1);
}

#[test]
#[should_panic(expected = "Expected citizen count")]
fn assert_citizen_count_between_fails() {
    let mut city = TestCity::new();
    city.assert_citizen_count_between(1, 10);
}

#[test]
fn assert_budget_above_passes() {
    let city = TestCity::new().with_budget(50_000.0);
    city.assert_budget_above(49_000.0);
}

#[test]
#[should_panic(expected = "Expected treasury")]
fn assert_budget_above_fails() {
    let city = TestCity::new().with_budget(1_000.0);
    city.assert_budget_above(5_000.0);
}

#[test]
fn assert_budget_below_passes() {
    let city = TestCity::new().with_budget(1_000.0);
    city.assert_budget_below(5_000.0);
}

#[test]
#[should_panic(expected = "Expected treasury")]
fn assert_budget_below_fails() {
    let city = TestCity::new().with_budget(50_000.0);
    city.assert_budget_below(1_000.0);
}

#[test]
fn assert_has_road_passes() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let grid = city.grid();
    let mut found_road = false;
    for y in 100..=110 {
        if grid.get(100, y).cell_type == CellType::Road {
            city.assert_has_road(100, y);
            found_road = true;
            break;
        }
    }
    assert!(found_road, "should find at least one road cell");
}

#[test]
#[should_panic(expected = "Expected road")]
fn assert_has_road_fails() {
    let city = TestCity::new();
    city.assert_has_road(100, 100);
}

#[test]
fn assert_has_building_passes() {
    let city = TestCity::new().with_building(100, 100, ZoneType::ResidentialLow, 1);
    city.assert_has_building(100, 100);
}

#[test]
#[should_panic(expected = "Expected building")]
fn assert_has_building_fails() {
    let city = TestCity::new();
    city.assert_has_building(100, 100);
}
