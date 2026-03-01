//! TEST-001: Unit Tests for Economy/Tax Formulas
//!
//! Tests all economy formulas with known inputs/outputs: tax collection, expense
//! calculation, treasury updates, loan interest. Verifies no NaN/Inf in treasury.

use crate::budget::{ExtendedBudget, Loan, ZoneTaxRates};
use crate::buildings::types::Building;
use crate::economy::{property_tax_for_building, CityBudget};
use crate::grid::{RoadType, ZoneType};
use crate::loans::{LoanBook, LoanTier};
use crate::services::ServiceType;
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
// 1. Tax collection formula with known population and rate
// ====================================================================

/// Verify property_tax_for_building computes land_value * level * rate exactly.
#[test]
fn test_formula_property_tax_known_inputs() {
    // 100 * 2 * 0.10 = 20.0
    let tax = property_tax_for_building(100.0, 2, 0.10);
    assert!((tax - 20.0).abs() < 1e-6, "Expected 20.0, got {tax}");
}

/// Verify tax formula linearity: doubling land value doubles tax.
#[test]
fn test_formula_property_tax_linear_in_land_value() {
    let tax_a = property_tax_for_building(200.0, 1, 0.05);
    let tax_b = property_tax_for_building(400.0, 1, 0.05);
    assert!(
        (tax_b - 2.0 * tax_a).abs() < 1e-6,
        "Tax should double when land value doubles: {tax_a} vs {tax_b}"
    );
}

/// Verify tax formula linearity in building level.
#[test]
fn test_formula_property_tax_linear_in_level() {
    let tax_l1 = property_tax_for_building(50.0, 1, 0.08);
    let tax_l4 = property_tax_for_building(50.0, 4, 0.08);
    assert!(
        (tax_l4 - 4.0 * tax_l1).abs() < 1e-6,
        "Level 4 tax should be 4x level 1: l1={tax_l1}, l4={tax_l4}"
    );
}

/// Verify tax formula linearity in rate.
#[test]
fn test_formula_property_tax_linear_in_rate() {
    let tax_5pct = property_tax_for_building(100.0, 1, 0.05);
    let tax_15pct = property_tax_for_building(100.0, 1, 0.15);
    assert!(
        (tax_15pct - 3.0 * tax_5pct).abs() < 1e-6,
        "15% rate should be 3x 5%: 5%={tax_5pct}, 15%={tax_15pct}"
    );
}

/// Verify residential buildings generate tax in the residential income breakdown.
#[test]
fn test_formula_residential_tax_collection_with_known_rate() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    // Set a known tax rate
    {
        let world = city.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .zone_taxes
            .residential = 0.12;
    }

    // Advance past collection interval
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>();
    assert!(
        extended.income_breakdown.residential_tax > 0.0,
        "Residential tax should be positive with a residential building"
    );
}

/// Verify commercial buildings generate tax in the commercial income breakdown.
#[test]
fn test_formula_commercial_tax_with_known_rate() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialLow, 3)
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    {
        let world = city.world_mut();
        world.resource_mut::<ExtendedBudget>().zone_taxes.commercial = 0.15;
    }

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>();
    assert!(
        extended.income_breakdown.commercial_tax > 0.0,
        "Commercial tax should be positive"
    );
}

/// Verify industrial buildings generate tax in the industrial income breakdown.
#[test]
fn test_formula_industrial_tax_with_known_rate() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 2)
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    {
        let world = city.world_mut();
        world.resource_mut::<ExtendedBudget>().zone_taxes.industrial = 0.08;
    }

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>();
    assert!(
        extended.income_breakdown.industrial_tax > 0.0,
        "Industrial tax should be positive"
    );
}

/// Verify office buildings generate tax in the office income breakdown.
#[test]
fn test_formula_office_tax_with_known_rate() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Office, 2)
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    {
        let world = city.world_mut();
        world.resource_mut::<ExtendedBudget>().zone_taxes.office = 0.10;
    }

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>();
    assert!(
        extended.income_breakdown.office_tax > 0.0,
        "Office tax should be positive"
    );
}

/// Higher building levels produce proportionally more tax.
#[test]
fn test_formula_higher_level_produces_proportional_tax() {
    // Level 1 city
    let mut city_l1 = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_budget(10_000.0);
    fill_buildings(&mut city_l1);
    force_clock_to_day(&mut city_l1, 32);
    city_l1.tick(10);
    let tax_l1 = city_l1
        .resource::<ExtendedBudget>()
        .income_breakdown
        .residential_tax;

    // Level 3 city (same location, same land value baseline)
    let mut city_l3 = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_budget(10_000.0);
    fill_buildings(&mut city_l3);
    force_clock_to_day(&mut city_l3, 32);
    city_l3.tick(10);
    let tax_l3 = city_l3
        .resource::<ExtendedBudget>()
        .income_breakdown
        .residential_tax;

    // Level 3 should yield exactly 3x tax (same land value, same rate, level factor)
    assert!(
        (tax_l3 - 3.0 * tax_l1).abs() < 0.01,
        "Level 3 tax should be 3x level 1: l1={tax_l1}, l3={tax_l3}"
    );
}

// ====================================================================
// 2. Expense deduction for each service type
// ====================================================================

/// Each service type has a specific monthly maintenance cost.
/// Verify that placing services adds the correct amount to expense breakdown.
#[test]
fn test_formula_service_expense_fire_station() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::FireStation)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let expected = crate::services::ServiceBuilding::monthly_maintenance(ServiceType::FireStation);
    let actual = city
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .service_costs;
    assert!(
        (actual - expected).abs() < 0.01,
        "Fire station expense: expected {expected}, got {actual}"
    );
}

#[test]
fn test_formula_service_expense_hospital() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::Hospital)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let expected = crate::services::ServiceBuilding::monthly_maintenance(ServiceType::Hospital);
    let actual = city
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .service_costs;
    assert!(
        (actual - expected).abs() < 0.01,
        "Hospital expense: expected {expected}, got {actual}"
    );
}

#[test]
fn test_formula_service_expense_police_station() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::PoliceStation)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let expected =
        crate::services::ServiceBuilding::monthly_maintenance(ServiceType::PoliceStation);
    let actual = city
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .service_costs;
    assert!(
        (actual - expected).abs() < 0.01,
        "Police station expense: expected {expected}, got {actual}"
    );
}

#[test]
fn test_formula_multiple_services_expense_sum() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::Hospital)
        .with_service(60, 60, ServiceType::FireStation)
        .with_service(90, 90, ServiceType::University)
        .with_budget(100_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let expected = crate::services::ServiceBuilding::monthly_maintenance(ServiceType::Hospital)
        + crate::services::ServiceBuilding::monthly_maintenance(ServiceType::FireStation)
        + crate::services::ServiceBuilding::monthly_maintenance(ServiceType::University);
    let actual = city
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .service_costs;
    assert!(
        (actual - expected).abs() < 0.01,
        "Multiple services expense: expected {expected}, got {actual}"
    );
}

/// Road maintenance expense scales with road type cost.
#[test]
fn test_formula_road_maintenance_expense_by_type() {
    // Place a short road of each type and verify maintenance appears
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Highway)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let road_maint = city
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .road_maintenance;
    assert!(
        road_maint > 0.0,
        "Road maintenance should be positive with highway roads"
    );
}

/// More road cells should produce higher maintenance cost.
#[test]
fn test_formula_more_road_cells_higher_maintenance() {
    // Short road
    let mut city_short = TestCity::new()
        .with_road(10, 10, 15, 10, RoadType::Local)
        .with_budget(50_000.0);
    force_clock_to_day(&mut city_short, 32);
    city_short.tick(10);
    let maint_short = city_short
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .road_maintenance;

    // Longer road
    let mut city_long = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_budget(50_000.0);
    force_clock_to_day(&mut city_long, 32);
    city_long.tick(10);
    let maint_long = city_long
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .road_maintenance;

    assert!(
        maint_long > maint_short,
        "Longer road should cost more to maintain: short={maint_short}, long={maint_long}"
    );
}

// ====================================================================
// 3. Treasury update: income - expenses = correct delta
// ====================================================================

/// Treasury delta equals monthly_income - monthly_expenses after collection.
#[test]
fn test_formula_treasury_delta_equals_net_income() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_building(14, 11, ZoneType::CommercialLow, 2)
        .with_service(20, 11, ServiceType::PoliceStation)
        .with_citizen((12, 11), (14, 11))
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    let treasury_before = city.budget().treasury;

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    let expected_delta = budget.monthly_income - budget.monthly_expenses;
    let actual_delta = budget.treasury - treasury_before;

    // Allow small floating point tolerance
    assert!(
        (actual_delta - expected_delta).abs() < 0.01,
        "Treasury delta ({actual_delta}) should equal income - expenses ({expected_delta})"
    );
}

/// When income exceeds expenses, treasury should increase.
#[test]
fn test_formula_treasury_increases_when_income_exceeds_expenses() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_building(52, 50, ZoneType::CommercialLow, 3)
        .with_building(54, 50, ZoneType::Industrial, 3)
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    let before = city.budget().treasury;

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let after = city.budget().treasury;
    let income = city.budget().monthly_income;
    let expenses = city.budget().monthly_expenses;

    // With many high-level buildings and no services/roads, income should exceed expenses
    assert!(
        income > expenses,
        "Income ({income}) should exceed expenses ({expenses}) with multiple high-level buildings and no services"
    );
    assert!(
        after > before,
        "Treasury should increase: before={before}, after={after}"
    );
}

/// When expenses exceed income, treasury should decrease.
#[test]
fn test_formula_treasury_decreases_when_expenses_exceed_income() {
    let mut city = TestCity::new()
        .with_road(10, 10, 80, 10, RoadType::Highway) // expensive road
        .with_service(20, 20, ServiceType::Hospital) // expensive service
        .with_service(40, 40, ServiceType::MedicalCenter) // another expensive one
        .with_budget(100_000.0);

    let before = city.budget().treasury;

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let after = city.budget().treasury;
    assert!(
        after < before,
        "Treasury should decrease with high expenses and no taxable buildings: before={before}, after={after}"
    );
}

/// Income breakdown sums should match monthly_income.
#[test]
fn test_formula_income_breakdown_sums_to_monthly_income() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_building(52, 50, ZoneType::CommercialLow, 2)
        .with_building(54, 50, ZoneType::Industrial, 2)
        .with_building(56, 50, ZoneType::Office, 2)
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    let ext = city.resource::<ExtendedBudget>();

    let breakdown_sum = ext.income_breakdown.residential_tax
        + ext.income_breakdown.commercial_tax
        + ext.income_breakdown.industrial_tax
        + ext.income_breakdown.office_tax
        + ext.income_breakdown.trade_income;

    assert!(
        (budget.monthly_income - breakdown_sum).abs() < 0.01,
        "Income breakdown sum ({breakdown_sum}) should match monthly_income ({})",
        budget.monthly_income
    );
}

/// Expense breakdown sums should match monthly_expenses.
#[test]
fn test_formula_expense_breakdown_sums_to_monthly_expenses() {
    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_service(30, 30, ServiceType::Hospital)
        .with_service(60, 60, ServiceType::PoliceStation)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    let ext = city.resource::<ExtendedBudget>();

    let breakdown_sum = ext.expense_breakdown.road_maintenance
        + ext.expense_breakdown.service_costs
        + ext.expense_breakdown.policy_costs
        + ext.expense_breakdown.fuel_costs;

    assert!(
        (budget.monthly_expenses - breakdown_sum).abs() < 0.01,
        "Expense breakdown sum ({breakdown_sum}) should match monthly_expenses ({})",
        budget.monthly_expenses
    );
}

// ====================================================================
// 4. Treasury never produces NaN or Infinity
// ====================================================================

/// Treasury should remain finite after normal tax collection.
#[test]
fn test_formula_treasury_finite_after_collection() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_service(30, 30, ServiceType::FireStation)
        .with_budget(10_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.treasury.is_finite(),
        "Treasury should be finite, got {}",
        budget.treasury
    );
    assert!(
        budget.monthly_income.is_finite(),
        "Monthly income should be finite"
    );
    assert!(
        budget.monthly_expenses.is_finite(),
        "Monthly expenses should be finite"
    );
}

/// Treasury should remain finite with zero tax rate.
#[test]
fn test_formula_treasury_finite_with_zero_tax_rate() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_budget(10_000.0);

    {
        let world = city.world_mut();
        let mut ext = world.resource_mut::<ExtendedBudget>();
        ext.zone_taxes = ZoneTaxRates {
            residential: 0.0,
            commercial: 0.0,
            industrial: 0.0,
            office: 0.0,
        };
    }

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.treasury.is_finite(),
        "Treasury should be finite with zero rates"
    );
    assert!(!budget.treasury.is_nan(), "Treasury should not be NaN");
}

/// Treasury should remain finite with very large values.
#[test]
fn test_formula_treasury_finite_with_large_values() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 5)
        .with_budget(f64::MAX / 2.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.treasury.is_finite(),
        "Treasury should remain finite even with large starting value"
    );
}

/// Treasury should remain finite with deeply negative values.
#[test]
fn test_formula_treasury_finite_with_negative_values() {
    let mut city = TestCity::new()
        .with_road(10, 10, 80, 10, RoadType::Highway)
        .with_service(30, 30, ServiceType::Hospital)
        .with_budget(-1_000_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.treasury.is_finite(),
        "Treasury should remain finite when deeply negative"
    );
    assert!(
        !budget.treasury.is_nan(),
        "Treasury should not be NaN when negative"
    );
}

/// Monthly income and expenses should never be NaN after multiple cycles.
#[test]
fn test_formula_no_nan_after_multiple_cycles() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_building(52, 50, ZoneType::CommercialLow, 1)
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_service(40, 40, ServiceType::FireStation)
        .with_budget(10_000.0);

    // Run multiple collection cycles
    for day in (32..200).step_by(31) {
        force_clock_to_day(&mut city, day as u32);
        city.tick(10);

        let budget = city.budget();
        assert!(
            budget.treasury.is_finite(),
            "Treasury not finite on day {day}: {}",
            budget.treasury
        );
        assert!(
            budget.monthly_income.is_finite(),
            "Income not finite on day {day}"
        );
        assert!(
            budget.monthly_expenses.is_finite(),
            "Expenses not finite on day {day}"
        );
    }
}

// ====================================================================
// 5. Zero-population edge case
// ====================================================================

/// An empty city with no buildings generates zero income.
#[test]
fn test_formula_zero_population_zero_income() {
    let mut city = TestCity::new().with_budget(10_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.monthly_income.abs() < 0.01,
        "Income should be ~0 with no buildings or population, got {}",
        budget.monthly_income
    );
}

/// Zero population city maintains stable treasury (no income, no service expenses).
#[test]
fn test_formula_zero_population_stable_treasury() {
    let mut city = TestCity::new().with_budget(5_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    // With no buildings, roads, or services, treasury should remain unchanged
    assert!(
        (budget.treasury - 5_000.0).abs() < 1.0,
        "Treasury should remain stable with zero population: got {}",
        budget.treasury
    );
}

/// Zero population: no NaN or Inf in any budget fields.
#[test]
fn test_formula_zero_population_no_nan() {
    let mut city = TestCity::new().with_budget(0.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(budget.treasury.is_finite(), "Treasury should be finite");
    assert!(budget.monthly_income.is_finite(), "Income should be finite");
    assert!(
        budget.monthly_expenses.is_finite(),
        "Expenses should be finite"
    );
    assert!(!budget.treasury.is_nan(), "Treasury should not be NaN");
}

// ====================================================================
// 6. Loan interest formulas
// ====================================================================

/// Loan monthly payment follows the amortization formula.
#[test]
fn test_formula_loan_amortization_payment() {
    let loan = Loan::new(10_000.0, 0.06, 12);
    // Monthly rate = 0.06 / 12 = 0.005
    // Payment = 10000 * 0.005 / (1 - 1.005^-12) = 860.66 (approx)
    let monthly_rate = 0.06_f64 / 12.0;
    let expected = 10_000.0 * monthly_rate / (1.0 - (1.0 + monthly_rate).powi(-12));
    assert!(
        (loan.monthly_payment - expected).abs() < 0.01,
        "Loan payment should match amortization formula: expected {expected}, got {}",
        loan.monthly_payment
    );
}

/// Zero interest rate loan results in simple equal-split payments.
#[test]
fn test_formula_loan_zero_interest() {
    let loan = Loan::new(12_000.0, 0.0, 12);
    let expected = 12_000.0 / 12.0; // 1000.0
    assert!(
        (loan.monthly_payment - expected).abs() < 0.01,
        "Zero-interest loan should be principal / term: expected {expected}, got {}",
        loan.monthly_payment
    );
}

/// Loan payments should never be NaN or infinite.
#[test]
fn test_formula_loan_payment_finite() {
    for &(principal, rate, term) in &[
        (1_000.0, 0.01_f32, 6_u32),
        (100_000.0, 0.15, 120),
        (500_000.0, 0.12, 240),
        (1.0, 0.50, 1),
    ] {
        let loan = Loan::new(principal, rate, term);
        assert!(
            loan.monthly_payment.is_finite(),
            "Payment should be finite for principal={principal}, rate={rate}, term={term}"
        );
        assert!(
            loan.monthly_payment > 0.0,
            "Payment should be positive for principal={principal}"
        );
    }
}

/// Total loan repayment exceeds principal when interest rate is positive.
#[test]
fn test_formula_loan_total_repayment_exceeds_principal() {
    let loan = Loan::new(10_000.0, 0.05, 24);
    let total_repayment = loan.monthly_payment * 24.0;
    assert!(
        total_repayment > 10_000.0,
        "Total repayment ({total_repayment}) should exceed principal (10000) with 5% interest"
    );
}

/// LoanBook: taking a loan increases treasury by loan amount.
#[test]
fn test_formula_loan_book_take_loan_increases_treasury() {
    let mut book = LoanBook::default();
    let mut treasury = 5_000.0;
    let success = book.take_loan(LoanTier::Small, &mut treasury);
    assert!(success, "Should be able to take a small loan");
    assert!(
        (treasury - 15_000.0).abs() < 0.01,
        "Treasury should increase by loan amount: expected 15000, got {treasury}"
    );
}

/// LoanBook: total debt matches sum of remaining balances.
#[test]
fn test_formula_loan_book_total_debt() {
    let mut book = LoanBook::default();
    let mut treasury = 0.0;
    book.take_loan(LoanTier::Small, &mut treasury);
    book.take_loan(LoanTier::Medium, &mut treasury);
    let expected_debt = LoanTier::Small.amount() + LoanTier::Medium.amount();
    assert!(
        (book.total_debt() - expected_debt).abs() < 0.01,
        "Total debt should be sum of loan amounts: expected {expected_debt}, got {}",
        book.total_debt()
    );
}

/// LoanBook: cannot exceed max loans.
#[test]
fn test_formula_loan_book_max_loans_enforced() {
    let mut book = LoanBook::default();
    let mut treasury = 0.0;
    // Default max is 3
    assert!(book.take_loan(LoanTier::Small, &mut treasury));
    assert!(book.take_loan(LoanTier::Medium, &mut treasury));
    assert!(book.take_loan(LoanTier::Large, &mut treasury));
    assert!(
        !book.take_loan(LoanTier::Emergency, &mut treasury),
        "Should not be able to exceed max loans"
    );
}

/// LoanBook: debt-to-income ratio calculation.
#[test]
fn test_formula_loan_debt_to_income_ratio() {
    let mut book = LoanBook::default();
    let mut treasury = 0.0;
    book.take_loan(LoanTier::Small, &mut treasury); // 10000 debt

    let ratio = book.debt_to_income(5_000.0);
    assert!(
        (ratio - 2.0).abs() < 0.01,
        "Debt-to-income should be 2.0 (10000/5000), got {ratio}"
    );
}

/// LoanBook: debt-to-income with zero income returns infinity.
#[test]
fn test_formula_loan_debt_to_income_zero_income() {
    let mut book = LoanBook::default();
    let mut treasury = 0.0;
    book.take_loan(LoanTier::Small, &mut treasury);

    let ratio = book.debt_to_income(0.0);
    assert!(
        ratio.is_infinite(),
        "Debt-to-income with zero income and debt should be infinite, got {ratio}"
    );
}

/// LoanBook: no debt and no income yields 0.0 ratio.
#[test]
fn test_formula_loan_debt_to_income_no_debt_no_income() {
    let book = LoanBook::default();
    let ratio = book.debt_to_income(0.0);
    assert!(
        (ratio - 0.0).abs() < 1e-6,
        "No debt + no income should yield 0.0, got {ratio}"
    );
}

// ====================================================================
// 7. Default/initial values
// ====================================================================

/// Default CityBudget starts with 10000 treasury.
#[test]
fn test_formula_default_budget_initial_values() {
    let budget = CityBudget::default();
    assert_eq!(budget.treasury, 50_000.0);
    assert_eq!(budget.tax_rate, 0.1);
    assert_eq!(budget.monthly_income, 0.0);
    assert_eq!(budget.monthly_expenses, 0.0);
    assert_eq!(budget.last_collection_day, 0);
}

/// Default zone tax rates are all 10%.
#[test]
fn test_formula_default_zone_tax_rates() {
    let rates = ZoneTaxRates::default();
    assert_eq!(rates.residential, 0.10);
    assert_eq!(rates.commercial, 0.10);
    assert_eq!(rates.industrial, 0.10);
    assert_eq!(rates.office, 0.10);
}

/// Multiple tax collections accumulate correctly over time.
#[test]
fn test_formula_multiple_collections_accumulate() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_building(52, 50, ZoneType::CommercialLow, 2)
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    // First collection
    force_clock_to_day(&mut city, 32);
    city.tick(10);
    let after_first = city.budget().treasury;
    assert!(
        after_first > 10_000.0,
        "Should gain money after first collection"
    );

    // Second collection
    force_clock_to_day(&mut city, 63);
    city.tick(10);
    let after_second = city.budget().treasury;
    assert!(
        after_second > after_first,
        "Should accumulate more after second collection: first={after_first}, second={after_second}"
    );

    // Third collection
    force_clock_to_day(&mut city, 94);
    city.tick(10);
    let after_third = city.budget().treasury;
    assert!(
        after_third > after_second,
        "Should accumulate more after third collection: second={after_second}, third={after_third}"
    );
}

/// Tax collection only triggers once per interval (idempotent within interval).
#[test]
fn test_formula_collection_idempotent_within_interval() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_budget(10_000.0);

    fill_buildings(&mut city);

    force_clock_to_day(&mut city, 32);
    city.tick(10);
    let after_first = city.budget().treasury;

    // Tick again at same day: should not change treasury
    city.tick(20);
    let after_extra_ticks = city.budget().treasury;

    assert!(
        (after_extra_ticks - after_first).abs() < 0.01,
        "Extra ticks at same day should not change treasury: first={after_first}, after_extra={after_extra_ticks}"
    );
}

// ====================================================================
// Helper
// ====================================================================

fn force_clock_to_day(city: &mut TestCity, day: u32) {
    let world = city.world_mut();
    world.resource_mut::<GameClock>().day = day;
}
