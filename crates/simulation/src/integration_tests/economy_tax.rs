use crate::buildings::types::Building;
use crate::grid::{RoadType, ZoneType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

/// Fill all buildings to full occupancy so property tax is collected.
fn fill_buildings(city: &mut TestCity) {
    let world = city.world_mut();
    let mut q = world.query::<(bevy::prelude::Entity, &Building)>();
    let entities: Vec<(bevy::prelude::Entity, u32)> =
        q.iter(world).map(|(e, b)| (e, b.capacity)).collect();
    for (e, cap) in entities {
        if let Some(mut b) = world.get_mut::<Building>(e) {
            b.occupants = cap;
        }
    }
}

// ====================================================================
// Economy tax collection tests (issue #834 / TEST-054)
// ====================================================================

/// Helper: advance the game clock day past the tax collection interval (30 days)
/// so that the next tick triggers `collect_taxes`. This avoids running 40K+ ticks.
fn force_clock_to_day(city: &mut TestCity, day: u32) {
    let world = city.world_mut();
    world.resource_mut::<GameClock>().day = day;
}

/// Set up city with employed citizens, advance past tax collection day,
/// verify treasury increases.
#[test]
fn test_economy_tax_collection_increases_treasury() {
    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_zone_rect(12, 11, 16, 11, ZoneType::ResidentialLow)
        .with_zone_rect(20, 11, 24, 11, ZoneType::CommercialLow)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_building(14, 11, ZoneType::ResidentialLow, 1)
        .with_building(20, 11, ZoneType::CommercialLow, 2)
        .with_citizen((12, 11), (20, 11))
        .with_citizen((14, 11), (20, 11))
        .with_budget(10_000.0);

    fill_buildings(&mut city);
    let treasury_before = city.budget().treasury;

    // Advance clock past the 30-day tax collection interval
    force_clock_to_day(&mut city, 32);
    city.tick(10); // let collect_taxes system run

    let budget = city.budget().clone();
    assert!(
        budget.treasury > treasury_before,
        "Treasury should increase after tax collection: before={treasury_before}, after={}",
        budget.treasury
    );
}

/// Verify monthly_income is positive after tax collection with buildings present.
#[test]
fn test_economy_monthly_income_positive_with_buildings() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_building(14, 11, ZoneType::CommercialLow, 1)
        .with_building(16, 11, ZoneType::Industrial, 1)
        .with_citizen((12, 11), (14, 11))
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    // Advance past tax collection interval
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.monthly_income > 0.0,
        "monthly_income should be > 0 with taxable buildings, got {}",
        budget.monthly_income
    );
}

/// Verify expenses are deducted for active services (road maintenance + service costs).
#[test]
fn test_economy_expenses_deducted_for_services() {
    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_service(20, 11, ServiceType::PoliceStation)
        .with_budget(50_000.0);

    // Advance past tax collection interval
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.monthly_expenses > 0.0,
        "monthly_expenses should be > 0 with roads and services, got {}",
        budget.monthly_expenses
    );
}

/// Verify that treasury net change equals income minus expenses.
#[test]
fn test_economy_treasury_change_equals_net_income() {
    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_building(14, 11, ZoneType::CommercialLow, 2)
        .with_service(20, 11, ServiceType::FireStation)
        .with_citizen((12, 11), (14, 11))
        .with_budget(10_000.0);

    // Don't call fill_buildings here — this test has a citizen that provides
    // natural occupancy. fill_buildings creates a mismatch between the income
    // projection and actual tax collection when simulation_invariants corrects
    // the occupancy downward.
    let treasury_before = city.budget().treasury;

    // Force tax collection
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    let expected_change = budget.monthly_income - budget.monthly_expenses;
    let actual_change = budget.treasury - treasury_before;

    assert!(
        (actual_change - expected_change).abs() < 0.01,
        "Treasury change ({actual_change}) should equal income - expenses ({expected_change})"
    );
}

/// Verify tax collection only happens once per interval - not on every tick.
#[test]
fn test_economy_tax_collection_respects_interval() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_citizen((12, 11), (12, 11))
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    // First collection
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget_after_first = city.budget().clone();
    let treasury_after_first = budget_after_first.treasury;

    // last_collection_day should be updated
    assert!(
        budget_after_first.last_collection_day > 0,
        "last_collection_day should be updated after collection, got {}",
        budget_after_first.last_collection_day
    );

    // Tick again without advancing the day past the next interval
    city.tick(10);

    let treasury_after_second = city.budget().treasury;
    assert!(
        (treasury_after_second - treasury_after_first).abs() < 0.01,
        "Treasury should not change between collection intervals: first={treasury_after_first}, second={treasury_after_second}"
    );
}

/// Verify no income is generated when there are no buildings (empty city).
#[test]
fn test_economy_no_income_without_buildings() {
    let mut city = TestCity::new().with_budget(10_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    // With no buildings and no roads, income and expenses should both be 0
    assert!(
        (budget.monthly_income - 0.0).abs() < 0.01,
        "monthly_income should be 0 with no buildings, got {}",
        budget.monthly_income
    );
}

/// Verify road maintenance contributes to expenses.
#[test]
fn test_economy_road_maintenance_in_expenses() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_road(10, 10, 10, 40, RoadType::Avenue)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    assert!(
        extended.expense_breakdown.road_maintenance > 0.0,
        "road maintenance expense should be > 0 with roads, got {}",
        extended.expense_breakdown.road_maintenance
    );
}

/// Verify residential buildings generate residential tax income.
#[test]
fn test_economy_residential_tax_income() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_budget(10_000.0);

    fill_buildings(&mut city);
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    assert!(
        extended.income_breakdown.residential_tax > 0.0,
        "residential tax should be > 0 for residential building, got {}",
        extended.income_breakdown.residential_tax
    );
}

/// Verify commercial buildings generate commercial tax income.
#[test]
fn test_economy_commercial_tax_income() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialLow, 2)
        .with_budget(10_000.0);

    fill_buildings(&mut city);
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    assert!(
        extended.income_breakdown.commercial_tax > 0.0,
        "commercial tax should be > 0 for commercial building, got {}",
        extended.income_breakdown.commercial_tax
    );
}

/// Verify industrial buildings generate industrial tax income.
#[test]
fn test_economy_industrial_tax_income() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 2)
        .with_budget(10_000.0);

    fill_buildings(&mut city);
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    assert!(
        extended.income_breakdown.industrial_tax > 0.0,
        "industrial tax should be > 0 for industrial building, got {}",
        extended.income_breakdown.industrial_tax
    );
}

/// Higher building level should produce more tax revenue.
#[test]
fn test_economy_higher_level_building_more_tax() {
    use crate::budget::ExtendedBudget;

    // City with level 1 building
    let mut city_low = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_budget(10_000.0);

    fill_buildings(&mut city_low);
    force_clock_to_day(&mut city_low, 32);
    city_low.tick(10);

    let tax_low = city_low
        .resource::<ExtendedBudget>()
        .income_breakdown
        .residential_tax;

    // City with level 3 building at same location
    let mut city_high = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_budget(10_000.0);

    fill_buildings(&mut city_high);
    force_clock_to_day(&mut city_high, 32);
    city_high.tick(10);

    let tax_high = city_high
        .resource::<ExtendedBudget>()
        .income_breakdown
        .residential_tax;

    assert!(
        tax_high > tax_low,
        "Higher level building should generate more tax: level1={tax_low}, level3={tax_high}"
    );
}

/// Service maintenance costs appear in the expense breakdown after tax collection.
#[test]
fn test_economy_service_maintenance_in_expense_breakdown() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::Hospital)
        .with_service(60, 60, ServiceType::PoliceStation)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    let expected_service_cost = ServiceBuilding::monthly_maintenance(ServiceType::Hospital)
        + ServiceBuilding::monthly_maintenance(ServiceType::PoliceStation);

    assert!(
        (extended.expense_breakdown.service_costs - expected_service_cost).abs() < 0.01,
        "Service costs should match sum of maintenance: got {}, expected {expected_service_cost}",
        extended.expense_breakdown.service_costs
    );
}

/// Multiple tax collections over time should each add to the treasury.
#[test]
fn test_economy_multiple_tax_collections_over_time() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_building(52, 50, ZoneType::CommercialLow, 2)
        .with_citizen((50, 50), (52, 50))
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    // First collection at day 32
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let after_first = city.budget().treasury;
    assert!(
        after_first > 10_000.0,
        "Treasury should increase after first collection: got {after_first}"
    );

    // Second collection at day 63 (32 + 31, past the next interval)
    force_clock_to_day(&mut city, 63);
    city.tick(10);

    let after_second = city.budget().treasury;
    assert!(
        after_second > after_first,
        "Treasury should increase again after second collection: first={after_first}, second={after_second}"
    );
}

/// Budget goes negative when expenses exceed income with no buildings.
#[test]
fn test_economy_treasury_decreases_with_only_expenses() {
    let mut city = TestCity::new()
        .with_road(10, 10, 80, 10, RoadType::Highway) // expensive road
        .with_service(20, 11, ServiceType::Hospital) // expensive service
        .with_budget(1_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.treasury < 1_000.0,
        "Treasury should decrease when expenses exceed income: got {}",
        budget.treasury
    );
    assert!(
        budget.monthly_expenses > budget.monthly_income,
        "Expenses ({}) should exceed income ({}) with no taxable buildings",
        budget.monthly_expenses,
        budget.monthly_income,
    );
}
