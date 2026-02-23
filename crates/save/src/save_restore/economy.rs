// ---------------------------------------------------------------------------
// Restore functions for economy, policies, population, and lifecycle
// ---------------------------------------------------------------------------

use crate::save_codec::*;
use crate::save_types::*;

use simulation::budget::{ExtendedBudget, ServiceBudgets, ZoneTaxRates};
use simulation::life_simulation::LifeSimTimer;
use simulation::lifecycle::LifecycleTimer;
use simulation::loans::{self, LoanBook};
use simulation::policies::Policies;
use simulation::unlocks::UnlockState;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::virtual_population::{DistrictStats, VirtualPopulation};

/// Restore a `Policies` resource from saved data.
pub fn restore_policies(save: &SavePolicies) -> Policies {
    let active = save
        .active
        .iter()
        .filter_map(|&v| u8_to_policy(v))
        .collect();
    Policies { active }
}

/// Restore an `UnlockState` resource from saved data.
pub fn restore_unlock_state(save: &SaveUnlockState) -> UnlockState {
    let unlocked_nodes = save
        .unlocked_nodes
        .iter()
        .filter_map(|&v| u8_to_unlock_node(v))
        .collect();
    UnlockState {
        development_points: save.development_points,
        spent_points: save.spent_points,
        unlocked_nodes,
        last_milestone_pop: save.last_milestone_pop,
    }
}

/// Restore an `ExtendedBudget` resource from saved data.
pub fn restore_extended_budget(save: &SaveExtendedBudget) -> ExtendedBudget {
    ExtendedBudget {
        zone_taxes: ZoneTaxRates {
            residential: save.residential_tax,
            commercial: save.commercial_tax,
            industrial: save.industrial_tax,
            office: save.office_tax,
        },
        service_budgets: ServiceBudgets {
            fire: save.fire_budget,
            police: save.police_budget,
            healthcare: save.healthcare_budget,
            education: save.education_budget,
            sanitation: save.sanitation_budget,
            transport: save.transport_budget,
        },
        // Loans are stored separately in the LoanBook (budget.rs loans are legacy);
        // leave the ExtendedBudget.loans empty.
        loans: Vec::new(),
        income_breakdown: Default::default(),
        expense_breakdown: Default::default(),
    }
}

/// Restore a `LoanBook` resource from saved data.
pub fn restore_loan_book(save: &SaveLoanBook) -> LoanBook {
    let active_loans = save
        .loans
        .iter()
        .map(|sl| loans::Loan {
            name: sl.name.clone(),
            amount: sl.amount,
            interest_rate: sl.interest_rate,
            monthly_payment: sl.monthly_payment,
            remaining_balance: sl.remaining_balance,
            term_months: sl.term_months,
            months_paid: sl.months_paid,
        })
        .collect();
    LoanBook {
        active_loans,
        max_loans: save.max_loans as usize,
        credit_rating: save.credit_rating,
        last_payment_day: save.last_payment_day,
        consecutive_solvent_days: save.consecutive_solvent_days,
    }
}

/// Restore a `LifecycleTimer` resource from saved data.
pub fn restore_lifecycle_timer(save: &SaveLifecycleTimer) -> LifecycleTimer {
    LifecycleTimer {
        last_aging_day: save.last_aging_day,
        last_emigration_tick: save.last_emigration_tick,
    }
}

/// Restore a `LifeSimTimer` resource from saved data.
pub fn restore_life_sim_timer(save: &SaveLifeSimTimer) -> LifeSimTimer {
    LifeSimTimer {
        needs_tick: save.needs_tick,
        life_event_tick: save.life_event_tick,
        salary_tick: save.salary_tick,
        education_tick: save.education_tick,
        job_seek_tick: save.job_seek_tick,
        personality_tick: save.personality_tick,
        health_tick: save.health_tick,
    }
}

/// Restore a `VirtualPopulation` resource from saved data.
pub fn restore_virtual_population(save: &SaveVirtualPopulation) -> VirtualPopulation {
    let district_stats = save
        .district_stats
        .iter()
        .map(|ds| DistrictStats {
            population: ds.population,
            employed: ds.employed,
            avg_happiness: ds.avg_happiness,
            avg_age: ds.avg_age,
            age_brackets: ds.age_brackets,
            commuters_out: ds.commuters_out,
            tax_contribution: ds.tax_contribution,
            service_demand: ds.service_demand,
        })
        .collect();
    VirtualPopulation::from_saved(
        save.total_virtual,
        save.virtual_employed,
        district_stats,
        save.max_real_citizens,
    )
}

/// Restore an `UrbanGrowthBoundary` resource from saved data.
pub fn restore_urban_growth_boundary(state: &SaveUrbanGrowthBoundary) -> UrbanGrowthBoundary {
    let vertices: Vec<(f32, f32)> = state
        .vertices_x
        .iter()
        .zip(state.vertices_y.iter())
        .map(|(&x, &y)| (x, y))
        .collect();
    UrbanGrowthBoundary {
        enabled: state.enabled,
        vertices,
    }
}

/// Restore an `AgricultureState` resource from saved data.
pub fn restore_agriculture(
    save: &crate::save_types::SaveAgricultureState,
) -> simulation::agriculture::AgricultureState {
    simulation::agriculture::AgricultureState {
        growing_season_active: save.growing_season_active,
        crop_yield_modifier: save.crop_yield_modifier,
        rainfall_adequacy: save.rainfall_adequacy,
        temperature_suitability: save.temperature_suitability,
        soil_quality: save.soil_quality,
        fertilizer_bonus: save.fertilizer_bonus,
        frost_risk: save.frost_risk,
        frost_events_this_year: save.frost_events_this_year,
        frost_damage_total: save.frost_damage_total,
        has_irrigation: save.has_irrigation,
        farm_count: save.farm_count,
        annual_rainfall_estimate: save.annual_rainfall_estimate,
        last_frost_check_day: save.last_frost_check_day,
        last_rainfall_day: save.last_rainfall_day,
    }
}
