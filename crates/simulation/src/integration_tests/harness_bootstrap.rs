use crate::economy::CityBudget;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::land_value::LandValueGrid;
use crate::pollution::PollutionGrid;
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::weather::Weather;
use crate::SlowTickTimer;

#[test]
fn empty_city_has_no_citizens() {
    let mut city = TestCity::new();
    assert_eq!(city.citizen_count(), 0, "empty city should have 0 citizens");
}

#[test]
fn empty_city_has_no_buildings() {
    let mut city = TestCity::new();
    assert_eq!(
        city.building_count(),
        0,
        "empty city should have 0 buildings"
    );
}

#[test]
fn empty_city_has_no_roads() {
    let city = TestCity::new();
    assert_eq!(
        city.road_cell_count(),
        0,
        "empty city should have 0 road cells"
    );
}

#[test]
fn empty_city_has_default_budget() {
    let city = TestCity::new();
    let budget = city.budget();
    assert!(
        (budget.treasury - 10_000.0).abs() < f64::EPSILON,
        "default treasury should be 10000, got {}",
        budget.treasury
    );
    assert!(
        (budget.tax_rate - 0.1).abs() < f32::EPSILON,
        "default tax rate should be 0.1, got {}",
        budget.tax_rate
    );
}

#[test]
fn empty_city_grid_dimensions() {
    let city = TestCity::new();
    let grid = city.grid();
    assert_eq!(grid.width, 256);
    assert_eq!(grid.height, 256);
    assert_eq!(grid.cells.len(), 256 * 256);
}

#[test]
fn empty_city_all_cells_are_grass() {
    let city = TestCity::new();
    let grid = city.grid();
    for cell in &grid.cells {
        assert_eq!(cell.cell_type, CellType::Grass);
        assert_eq!(cell.zone, ZoneType::None);
        assert!(cell.building_id.is_none());
    }
}

#[test]
fn empty_city_core_resources_exist() {
    let city = TestCity::new();
    city.assert_resource_exists::<WorldGrid>();
    city.assert_resource_exists::<RoadNetwork>();
    city.assert_resource_exists::<CityBudget>();
    city.assert_resource_exists::<RoadSegmentStore>();
    city.assert_resource_exists::<GameClock>();
    city.assert_resource_exists::<Weather>();
    city.assert_resource_exists::<SlowTickTimer>();
    city.assert_resource_exists::<LandValueGrid>();
    city.assert_resource_exists::<PollutionGrid>();
}
