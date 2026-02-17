use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::citizen::Citizen;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::services::ServiceBuilding;
use crate::time_of_day::GameClock;


#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CityBudget {
    pub treasury: f64,
    pub tax_rate: f32,          // 0.0..1.0
    pub monthly_income: f64,
    pub monthly_expenses: f64,
    pub last_collection_day: u32,
}

impl Default for CityBudget {
    fn default() -> Self {
        Self {
            treasury: 10000.0,
            tax_rate: 0.1,
            monthly_income: 0.0,
            monthly_expenses: 0.0,
            last_collection_day: 0,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn collect_taxes(
    clock: Res<GameClock>,
    mut budget: ResMut<CityBudget>,
    citizens: Query<&Citizen>,
    buildings: Query<&Building>,
    services_q: Query<&ServiceBuilding>,
    grid_res: Res<WorldGrid>,
    land_value: Res<crate::land_value::LandValueGrid>,
    policies: Res<crate::policies::Policies>,
    tourism: Res<crate::tourism::Tourism>,
    mut extended: ResMut<crate::budget::ExtendedBudget>,
) {
    // Collect every 30 days
    if clock.day <= budget.last_collection_day + 30 {
        return;
    }
    budget.last_collection_day = clock.day;

    let pop = citizens.iter().count() as f64;

    // Income: tax per citizen
    let tax_per_citizen = 10.0 * budget.tax_rate as f64;
    let mut income = pop * tax_per_citizen;

    // Commercial and office building income scales with occupants
    let mut commercial_income = 0.0;
    let industrial_tax_mult = policies.industrial_tax_multiplier();
    for b in &buildings {
        let occ = b.occupants as f64;
        if b.zone_type == ZoneType::Office {
            commercial_income += occ * 1.5;
        } else if b.zone_type.is_commercial() {
            commercial_income += occ * 1.0;
        } else if b.zone_type == ZoneType::Industrial {
            commercial_income += occ * 0.6 * industrial_tax_mult as f64;
        }
    }
    income += commercial_income;

    // Tourism income
    income += tourism.monthly_tourism_income;

    // Land value tax boost: average land value across all cells increases income
    let total_cells = (grid_res.width * grid_res.height) as f64;
    let mut land_value_sum: f64 = 0.0;
    for y in 0..grid_res.height {
        for x in 0..grid_res.width {
            land_value_sum += land_value.get(x, y) as f64;
        }
    }
    let avg_land_value = if total_cells > 0.0 {
        land_value_sum / total_cells
    } else {
        0.0
    };
    income *= 1.0 + (avg_land_value / 500.0);

    // Expenses: road maintenance ($0.5 per road cell per month)
    let road_cells = grid_res
        .cells
        .iter()
        .filter(|c| c.cell_type == CellType::Road)
        .count() as f64;
    let road_expense = road_cells * 0.5;

    // Service maintenance costs
    let service_expense: f64 = services_q
        .iter()
        .map(|s| ServiceBuilding::monthly_maintenance(s.service_type))
        .sum();

    // Policy costs
    let policy_expense = policies.total_monthly_cost();

    // Loan payments
    let loan_payments = extended.process_loan_payments(&mut budget.treasury);

    // Track breakdowns
    extended.income_breakdown.residential_tax = pop * tax_per_citizen;
    extended.income_breakdown.commercial_tax = commercial_income;
    extended.income_breakdown.office_tax = 0.0; // included in commercial_income
    extended.income_breakdown.trade_income = tourism.monthly_tourism_income;
    extended.expense_breakdown.road_maintenance = road_expense;
    extended.expense_breakdown.service_costs = service_expense;
    extended.expense_breakdown.policy_costs = policy_expense;
    extended.expense_breakdown.loan_payments = loan_payments;

    budget.monthly_income = income;
    budget.monthly_expenses = road_expense + service_expense + policy_expense;
    budget.treasury += budget.monthly_income - budget.monthly_expenses;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tax_formula() {
        let budget = CityBudget {
            tax_rate: 0.1,
            ..Default::default()
        };
        let tax_per_citizen = 10.0 * budget.tax_rate as f64;
        assert!((tax_per_citizen - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_initial_treasury() {
        let budget = CityBudget::default();
        assert_eq!(budget.treasury, 10000.0);
    }
}
