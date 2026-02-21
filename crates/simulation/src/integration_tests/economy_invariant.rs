use crate::grid::{RoadType, ZoneType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Economy balance invariant
// ---------------------------------------------------------------------------

/// After tax collection, verify income breakdown sums to monthly_income
/// and expense breakdown sums to monthly_expenses.
#[test]
fn test_economy_breakdown_sums_match_totals_after_tax_collection() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_road(10, 10, 10, 40, RoadType::Avenue)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_building(14, 11, ZoneType::CommercialLow, 1)
        .with_building(16, 11, ZoneType::Industrial, 1)
        .with_service(20, 11, ServiceType::PoliceStation)
        .with_budget(50_000.0);

    city.tick_slow_cycles(50);

    let budget = city.budget().clone();
    let extended = city.resource::<ExtendedBudget>().clone();

    let income_sum = extended.income_breakdown.residential_tax
        + extended.income_breakdown.commercial_tax
        + extended.income_breakdown.industrial_tax
        + extended.income_breakdown.office_tax
        + extended.income_breakdown.trade_income;

    let expense_sum = extended.expense_breakdown.road_maintenance
        + extended.expense_breakdown.service_costs
        + extended.expense_breakdown.policy_costs;

    assert!(
        (budget.monthly_income - income_sum).abs() < 0.01,
        "Income mismatch: monthly_income={} but breakdown sums to {}",
        budget.monthly_income,
        income_sum,
    );

    assert!(
        (budget.monthly_expenses - expense_sum).abs() < 0.01,
        "Expense mismatch: monthly_expenses={} but breakdown sums to {}",
        budget.monthly_expenses,
        expense_sum,
    );
}
