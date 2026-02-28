//! Computes projected income/expenses every tick so the LLM agent never
//! sees stale zeros before the first tax collection event (issue #1937).
//!
//! The `IncomeProjection` resource is updated each slow-tick by reading
//! the same data sources as `collect_taxes` in `economy.rs`, but without
//! waiting for the 30-day collection interval.

use bevy::prelude::*;

use crate::budget::ExtendedBudget;
use crate::buildings::{Building, MixedUseBuilding};
use crate::economy::property_tax_for_building;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::land_value::LandValueGrid;
use crate::policies::Policies;
use crate::services::ServiceBuilding;
use crate::tourism::Tourism;
use crate::SlowTickTimer;

/// Projected income and expenses based on current city state.
///
/// Updated every slow-tick (not every tick, to avoid per-frame query cost).
/// Mirrors the formula in `collect_taxes` but is always up-to-date.
#[derive(Resource, Debug, Clone, Default)]
pub struct IncomeProjection {
    /// Projected monthly income from property taxes + tourism.
    pub projected_income: f64,
    /// Projected monthly expenses from roads + services + policies + fuel.
    pub projected_expenses: f64,
}

/// Recompute projected income/expenses each slow-tick.
#[allow(clippy::too_many_arguments)]
pub fn update_income_projection(
    slow_tick: Res<SlowTickTimer>,
    buildings: Query<&Building>,
    services_q: Query<&ServiceBuilding>,
    grid_res: Res<WorldGrid>,
    land_value: Res<LandValueGrid>,
    extended: Res<ExtendedBudget>,
    policies: Res<Policies>,
    tourism: Res<Tourism>,
    params: (
        Res<crate::coal_power::CoalPowerState>,
        Res<crate::gas_power::GasPowerState>,
        Res<crate::nuclear_power::NuclearPowerState>,
        Res<crate::oil_power::OilPowerState>,
        Res<crate::biomass_power::BiomassPowerState>,
    ),
    mut projection: ResMut<IncomeProjection>,
) {
    if !slow_tick.should_run() {
        return;
    }

    let (coal_state, gas_state, nuclear_state, oil_state, biomass_state) = params;
    let zone_rates = &extended.zone_taxes;
    let industrial_tax_mult = policies.industrial_tax_multiplier();

    // ── Income: property taxes ────────────────────────────────────────
    let mut total_tax = 0.0_f64;

    for b in &buildings {
        let lv = if grid_res.in_bounds(b.grid_x, b.grid_y) {
            land_value.get(b.grid_x, b.grid_y) as f64
        } else {
            50.0
        };

        if b.zone_type.is_mixed_use() {
            let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(b.level);
            let total_cap = comm_cap + res_cap;
            if total_cap > 0 {
                let res_frac = res_cap as f64 / total_cap as f64;
                let comm_frac = comm_cap as f64 / total_cap as f64;
                total_tax +=
                    property_tax_for_building(lv * res_frac, b.level, zone_rates.residential);
                total_tax +=
                    property_tax_for_building(lv * comm_frac, b.level, zone_rates.commercial);
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

        total_tax += property_tax_for_building(lv, b.level, rate);
    }

    let income = total_tax + tourism.monthly_tourism_income;

    // ── Expenses ──────────────────────────────────────────────────────
    let road_expense: f64 = grid_res
        .cells
        .iter()
        .filter(|c| c.cell_type == CellType::Road)
        .map(|c| c.road_type.maintenance_cost())
        .sum();

    let service_budgets = &extended.service_budgets;
    let service_expense: f64 = services_q
        .iter()
        .map(|s| {
            let base = ServiceBuilding::monthly_maintenance(s.service_type);
            let budget_level = service_budgets.for_service(s.service_type);
            base * budget_level as f64
        })
        .sum();

    let policy_expense = policies.total_monthly_cost();

    let fuel_expense = coal_state.total_fuel_cost as f64
        + gas_state.total_fuel_cost as f64
        + nuclear_state.total_fuel_cost as f64
        + oil_state.total_fuel_cost as f64
        + biomass_state.total_fuel_cost as f64;

    let expenses = road_expense + service_expense + policy_expense + fuel_expense;

    projection.projected_income = income;
    projection.projected_expenses = expenses;
}

pub struct IncomeProjectionPlugin;

impl Plugin for IncomeProjectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IncomeProjection>();
        app.add_systems(
            FixedUpdate,
            update_income_projection
                .after(crate::economy::collect_taxes)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
