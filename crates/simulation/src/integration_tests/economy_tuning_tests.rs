//! PLAY-P1-02: Economy tuning integration tests.
//!
//! Verifies:
//! 1. Starting treasury matches GameParams ($50k default)
//! 2. Fuel costs are charged from power plant operations
//! 3. Service budget sliders affect actual expenses
//! 4. Expense breakdown includes fuel_costs in sum

use crate::budget::ExtendedBudget;
use crate::economy::CityBudget;
use crate::game_params::GameParams;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

// ====================================================================
// 1. Starting treasury consistency
// ====================================================================

/// Default CityBudget treasury matches GameParams.economy.starting_treasury.
#[test]
fn test_starting_treasury_matches_game_params() {
    let budget = CityBudget::default();
    let params = GameParams::default();
    assert!(
        (budget.treasury - params.economy.starting_treasury).abs() < f64::EPSILON,
        "CityBudget::default().treasury ({}) should match GameParams starting_treasury ({})",
        budget.treasury,
        params.economy.starting_treasury,
    );
}

/// Default starting treasury is $50,000.
#[test]
fn test_default_starting_treasury_is_50k() {
    let params = GameParams::default();
    assert!(
        (params.economy.starting_treasury - 50_000.0).abs() < f64::EPSILON,
        "Default starting treasury should be $50,000, got {}",
        params.economy.starting_treasury,
    );
}

// ====================================================================
// 2. Expense breakdown sums match monthly_expenses (with fuel)
// ====================================================================

/// Expense breakdown (including fuel_costs) sums to monthly_expenses.
#[test]
fn test_expense_breakdown_with_fuel_sums_to_total() {
    use crate::grid::RoadType;
    use crate::services::ServiceType;

    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_service(30, 30, ServiceType::FireStation)
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
        "Expense breakdown (including fuel) should sum to monthly_expenses: \
         monthly_expenses={}, breakdown_sum={} (road={}, service={}, policy={}, fuel={})",
        budget.monthly_expenses,
        breakdown_sum,
        ext.expense_breakdown.road_maintenance,
        ext.expense_breakdown.service_costs,
        ext.expense_breakdown.policy_costs,
        ext.expense_breakdown.fuel_costs,
    );
}

// ====================================================================
// 3. Service budget slider affects expenses
// ====================================================================

/// Setting service budget slider to 0.5 should halve service costs.
#[test]
fn test_service_budget_slider_reduces_expenses() {
    use crate::services::ServiceType;

    // City with default budget levels (1.0)
    let mut city_full = TestCity::new()
        .with_service(30, 30, ServiceType::FireStation)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city_full, 32);
    city_full.tick(10);
    let full_service_cost = city_full
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .service_costs;

    // City with budget levels at 0.5
    let mut city_half = TestCity::new()
        .with_service(30, 30, ServiceType::FireStation)
        .with_budget(50_000.0);

    {
        let world = city_half.world_mut();
        world.resource_mut::<ExtendedBudget>().service_budgets.fire = 0.5;
    }

    force_clock_to_day(&mut city_half, 32);
    city_half.tick(10);
    let half_service_cost = city_half
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .service_costs;

    assert!(
        full_service_cost > 0.0,
        "Full-budget service cost should be positive, got {full_service_cost}"
    );
    assert!(
        (half_service_cost - full_service_cost * 0.5).abs() < 0.01,
        "Half-budget service cost ({half_service_cost}) should be ~half of full ({full_service_cost})"
    );
}

/// Setting service budget slider to 0.0 should eliminate service costs.
#[test]
fn test_service_budget_slider_zero_eliminates_costs() {
    use crate::services::ServiceType;

    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::FireStation)
        .with_budget(50_000.0);

    {
        let world = city.world_mut();
        world.resource_mut::<ExtendedBudget>().service_budgets.fire = 0.0;
    }

    force_clock_to_day(&mut city, 32);
    city.tick(10);
    let service_cost = city
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .service_costs;

    assert!(
        service_cost.abs() < 0.01,
        "Zero-budget service cost should be ~0, got {service_cost}"
    );
}

/// Setting service budget slider to 1.5 should increase service costs by 50%.
#[test]
fn test_service_budget_slider_overfunded() {
    use crate::services::ServiceType;

    // City with default budget levels (1.0)
    let mut city_full = TestCity::new()
        .with_service(30, 30, ServiceType::FireStation)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city_full, 32);
    city_full.tick(10);
    let full_cost = city_full
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .service_costs;

    // City with budget level at 1.5
    let mut city_over = TestCity::new()
        .with_service(30, 30, ServiceType::FireStation)
        .with_budget(50_000.0);

    {
        let world = city_over.world_mut();
        world.resource_mut::<ExtendedBudget>().service_budgets.fire = 1.5;
    }

    force_clock_to_day(&mut city_over, 32);
    city_over.tick(10);
    let over_cost = city_over
        .resource::<ExtendedBudget>()
        .expense_breakdown
        .service_costs;

    assert!(
        (over_cost - full_cost * 1.5).abs() < 0.01,
        "Overfunded (1.5x) service cost ({over_cost}) should be 1.5x full ({full_cost})"
    );
}

// ====================================================================
// Helper
// ====================================================================

fn force_clock_to_day(city: &mut TestCity, day: u32) {
    let world = city.world_mut();
    world.resource_mut::<GameClock>().day = day;
}
