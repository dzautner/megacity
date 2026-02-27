use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::{Building, MixedUseBuilding};
use crate::game_params::GameParams;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::services::ServiceBuilding;
use crate::time_of_day::GameClock;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CityBudget {
    pub treasury: f64,
    pub tax_rate: f32, // 0.0..1.0
    pub monthly_income: f64,
    pub monthly_expenses: f64,
    pub last_collection_day: u32,
}

impl Default for CityBudget {
    fn default() -> Self {
        Self {
            treasury: 50_000.0,
            tax_rate: 0.1,
            monthly_income: 0.0,
            monthly_expenses: 0.0,
            last_collection_day: 0,
        }
    }
}

/// Compute property tax for a single building.
/// Formula: land_value * building_level * tax_rate
pub fn property_tax_for_building(land_value: f64, building_level: u8, tax_rate: f32) -> f64 {
    land_value * building_level as f64 * tax_rate as f64
}

/// Collect total fuel cost from all power plant state resources.
fn total_power_plant_fuel_cost(
    coal: &crate::coal_power::CoalPowerState,
    gas: &crate::gas_power::GasPowerState,
    nuclear: &crate::nuclear_power::NuclearPowerState,
    oil: &crate::oil_power::OilPowerState,
    biomass: &crate::biomass_power::BiomassPowerState,
) -> f64 {
    coal.total_fuel_cost as f64
        + gas.total_fuel_cost as f64
        + nuclear.total_fuel_cost as f64
        + oil.total_fuel_cost as f64
        + biomass.total_fuel_cost as f64
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn collect_taxes(
    clock: Res<GameClock>,
    mut budget: ResMut<CityBudget>,
    buildings: Query<&Building>,
    services_q: Query<&ServiceBuilding>,
    grid_res: Res<WorldGrid>,
    land_value: Res<crate::land_value::LandValueGrid>,
    policies: Res<crate::policies::Policies>,
    tourism: Res<crate::tourism::Tourism>,
    mut extended: ResMut<crate::budget::ExtendedBudget>,
    params: (
        Res<GameParams>,
        Res<crate::coal_power::CoalPowerState>,
        Res<crate::gas_power::GasPowerState>,
        Res<crate::nuclear_power::NuclearPowerState>,
        Res<crate::oil_power::OilPowerState>,
        Res<crate::biomass_power::BiomassPowerState>,
    ),
) {
    let (
        game_params,
        coal_state,
        gas_state,
        nuclear_state,
        oil_state,
        biomass_state,
    ) = params;

    // Collect every N days (configurable via GameParams)
    let interval = game_params.economy.tax_collection_interval_days;
    if clock.day <= budget.last_collection_day + interval {
        return;
    }
    budget.last_collection_day = clock.day;

    // Property tax: sum of (land_value * building_level * zone_tax_rate) per building
    let zone_rates = &extended.zone_taxes;
    let industrial_tax_mult = policies.industrial_tax_multiplier();

    let mut residential_tax = 0.0_f64;
    let mut commercial_tax = 0.0_f64;
    let mut industrial_tax = 0.0_f64;
    let mut office_tax = 0.0_f64;

    for b in &buildings {
        // Look up land value at the building's grid cell
        let lv = if grid_res.in_bounds(b.grid_x, b.grid_y) {
            land_value.get(b.grid_x, b.grid_y) as f64
        } else {
            50.0 // fallback baseline
        };

        if b.zone_type.is_mixed_use() {
            // MixedUse buildings generate both residential and commercial tax,
            // split proportionally based on static capacity ratios for the level.
            let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(b.level);
            let total_cap = comm_cap + res_cap;
            if total_cap > 0 {
                let res_fraction = res_cap as f64 / total_cap as f64;
                let comm_fraction = comm_cap as f64 / total_cap as f64;
                let res_tax =
                    property_tax_for_building(lv * res_fraction, b.level, zone_rates.residential);
                let comm_tax =
                    property_tax_for_building(lv * comm_fraction, b.level, zone_rates.commercial);
                residential_tax += res_tax;
                commercial_tax += comm_tax;
            }
            continue;
        }

        let rate = if b.zone_type.is_residential() {
            zone_rates.residential
        } else if b.zone_type.is_commercial() {
            zone_rates.commercial
        } else if b.zone_type == ZoneType::Industrial {
            zone_rates.industrial * industrial_tax_mult
        } else if b.zone_type == ZoneType::Office {
            zone_rates.office
        } else {
            0.0
        };

        let tax = property_tax_for_building(lv, b.level, rate);

        if b.zone_type.is_residential() {
            residential_tax += tax;
        } else if b.zone_type.is_commercial() {
            commercial_tax += tax;
        } else if b.zone_type == ZoneType::Industrial {
            industrial_tax += tax;
        } else if b.zone_type == ZoneType::Office {
            office_tax += tax;
        }
    }

    let property_income = residential_tax + commercial_tax + industrial_tax + office_tax;
    let mut income = property_income;

    // Tourism income
    income += tourism.monthly_tourism_income;

    // Expenses: road maintenance (scaled by road type)
    let road_expense: f64 = grid_res
        .cells
        .iter()
        .filter(|c| c.cell_type == CellType::Road)
        .map(|c| c.road_type.maintenance_cost())
        .sum();

    // Service maintenance costs — scaled by service budget slider
    let service_budgets = &extended.service_budgets;
    let service_expense: f64 = services_q
        .iter()
        .map(|s| {
            let base = ServiceBuilding::monthly_maintenance(s.service_type);
            let budget_level = service_budgets.for_service(s.service_type);
            base * budget_level as f64
        })
        .sum();

    // Policy costs
    let policy_expense = policies.total_monthly_cost();

    // Power plant fuel costs
    let fuel_expense = total_power_plant_fuel_cost(
        &coal_state,
        &gas_state,
        &nuclear_state,
        &oil_state,
        &biomass_state,
    );

    // Loan payments
    let loan_payments = extended.process_loan_payments(&mut budget.treasury);

    // Track breakdowns
    extended.income_breakdown.residential_tax = residential_tax;
    extended.income_breakdown.commercial_tax = commercial_tax;
    extended.income_breakdown.industrial_tax = industrial_tax;
    extended.income_breakdown.office_tax = office_tax;
    extended.income_breakdown.trade_income = tourism.monthly_tourism_income;
    extended.expense_breakdown.road_maintenance = road_expense;
    extended.expense_breakdown.service_costs = service_expense;
    extended.expense_breakdown.policy_costs = policy_expense;
    extended.expense_breakdown.loan_payments = loan_payments;
    extended.expense_breakdown.fuel_costs = fuel_expense;

    budget.monthly_income = income;
    budget.monthly_expenses =
        road_expense + service_expense + policy_expense + fuel_expense;
    budget.treasury += budget.monthly_income - budget.monthly_expenses;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_tax_basic() {
        // Building at land_value=100, level 3, rate 5% => 100 * 3 * 0.05 = 15.0
        let tax = property_tax_for_building(100.0, 3, 0.05);
        assert!((tax - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_property_tax_doubling_land_value() {
        // Doubling land value should double the property tax
        let tax1 = property_tax_for_building(50.0, 2, 0.10);
        let tax2 = property_tax_for_building(100.0, 2, 0.10);
        assert!((tax2 - 2.0 * tax1).abs() < 0.001);
    }

    #[test]
    fn test_property_tax_zero_buildings() {
        // Zero land value => zero tax
        let tax = property_tax_for_building(0.0, 3, 0.10);
        assert!((tax - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_property_tax_level_scaling() {
        // Level 1 vs level 5 at same land value and rate
        let tax_l1 = property_tax_for_building(80.0, 1, 0.10);
        let tax_l5 = property_tax_for_building(80.0, 5, 0.10);
        assert!((tax_l5 - 5.0 * tax_l1).abs() < 0.001);
    }

    #[test]
    fn test_property_tax_rate_range() {
        // 1% rate
        let tax_low = property_tax_for_building(100.0, 1, 0.01);
        assert!((tax_low - 1.0).abs() < 0.001);

        // 10% rate
        let tax_high = property_tax_for_building(100.0, 1, 0.10);
        assert!((tax_high - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_initial_treasury() {
        let budget = CityBudget::default();
        assert_eq!(budget.treasury, 50_000.0);
    }

    #[test]
    fn test_total_power_plant_fuel_cost() {
        let coal = crate::coal_power::CoalPowerState {
            plant_count: 1,
            total_output_mw: 66.0,
            total_fuel_cost: 1980.0,
            total_co2_tons: 66.0,
        };
        let gas = crate::gas_power::GasPowerState {
            plant_count: 1,
            total_output_mw: 225.0,
            total_fuel_cost: 9000.0,
            total_co2_tons: 90.0,
        };
        let nuclear = crate::nuclear_power::NuclearPowerState::default();
        let oil = crate::oil_power::OilPowerState::default();
        let biomass = crate::biomass_power::BiomassPowerState::default();

        let total = total_power_plant_fuel_cost(&coal, &gas, &nuclear, &oil, &biomass);
        assert!((total - 10980.0).abs() < 0.01);
    }
}

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityBudget>().add_systems(
            FixedUpdate,
            collect_taxes
                .after(crate::happiness::update_happiness)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
