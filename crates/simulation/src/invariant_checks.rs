//! Runtime invariant guards for budget, population, and core simulation resources.
//!
//! These systems run every slow-tick cycle (~100 ticks) and validate that
//! core simulation values haven't become corrupted (NaN, infinity, or
//! out-of-range). On violation, a warning is logged and the value is
//! clamped or reset to a safe default.

use bevy::prelude::*;

use crate::economy::CityBudget;
use crate::stats::CityStats;
use crate::SlowTickTimer;

/// Extreme floor for treasury — any value below this is treated as corrupt.
const TREASURY_FLOOR: f64 = -1_000_000_000.0;

/// Maximum reasonable citizen entity count before we warn.
const MAX_CITIZEN_ENTITIES: u32 = 100_000;

/// Tracks the number of core resource invariant violations detected during
/// the last validation pass. Used by integration tests.
#[derive(Resource, Default, Debug)]
pub struct CoreInvariantViolations {
    pub budget_treasury: u32,
    pub budget_income: u32,
    pub budget_expenses: u32,
    pub population: u32,
    pub happiness: u32,
    pub citizen_count_warning: u32,
}

// ---------------------------------------------------------------------------
// Budget / treasury checks
// ---------------------------------------------------------------------------

/// Validate that budget fields are finite and within reasonable bounds.
pub fn validate_budget(
    slow_tick: Res<SlowTickTimer>,
    mut budget: ResMut<CityBudget>,
    mut violations: ResMut<CoreInvariantViolations>,
) {
    if !slow_tick.should_run() {
        return;
    }
    violations.budget_treasury = 0;
    violations.budget_income = 0;
    violations.budget_expenses = 0;

    // Treasury: not NaN, not infinity, not below extreme floor
    if !budget.treasury.is_finite() || budget.treasury < TREASURY_FLOOR {
        warn!(
            "Invariant violation: treasury is {} (NaN/Inf/extreme). Resetting to 0.",
            budget.treasury
        );
        budget.treasury = 0.0;
        violations.budget_treasury += 1;
    }

    // Monthly income: not NaN, not infinity, not negative
    if !budget.monthly_income.is_finite() || budget.monthly_income < 0.0 {
        warn!(
            "Invariant violation: monthly_income is {}. Resetting to 0.",
            budget.monthly_income
        );
        budget.monthly_income = 0.0;
        violations.budget_income += 1;
    }

    // Monthly expenses: not NaN, not infinity, not negative
    if !budget.monthly_expenses.is_finite() || budget.monthly_expenses < 0.0 {
        warn!(
            "Invariant violation: monthly_expenses is {}. Resetting to 0.",
            budget.monthly_expenses
        );
        budget.monthly_expenses = 0.0;
        violations.budget_expenses += 1;
    }

    // Tax rate: clamp to 0.0..1.0, not NaN
    if !budget.tax_rate.is_finite() {
        warn!(
            "Invariant violation: tax_rate is {}. Resetting to 0.1.",
            budget.tax_rate
        );
        budget.tax_rate = 0.1;
        violations.budget_treasury += 1;
    } else if !(0.0..=1.0).contains(&budget.tax_rate) {
        warn!(
            "Invariant violation: tax_rate {} out of [0,1]. Clamping.",
            budget.tax_rate
        );
        budget.tax_rate = budget.tax_rate.clamp(0.0, 1.0);
        violations.budget_treasury += 1;
    }
}

// ---------------------------------------------------------------------------
// Population and happiness checks
// ---------------------------------------------------------------------------

/// Validate that CityStats population and happiness are sane.
pub fn validate_stats(
    slow_tick: Res<SlowTickTimer>,
    mut stats: ResMut<CityStats>,
    mut violations: ResMut<CoreInvariantViolations>,
) {
    if !slow_tick.should_run() {
        return;
    }
    violations.population = 0;
    violations.happiness = 0;
    violations.citizen_count_warning = 0;

    // Average happiness: clamp to [0, 100], not NaN
    if !stats.average_happiness.is_finite() {
        warn!(
            "Invariant violation: average_happiness is {}. Resetting to 0.",
            stats.average_happiness
        );
        stats.average_happiness = 0.0;
        violations.happiness += 1;
    } else if !(0.0..=100.0).contains(&stats.average_happiness) {
        warn!(
            "Invariant violation: average_happiness {} out of [0,100]. Clamping.",
            stats.average_happiness
        );
        stats.average_happiness = stats.average_happiness.clamp(0.0, 100.0);
        violations.happiness += 1;
    }

    // Citizen entity count sanity warning (not corrected, just logged)
    if stats.population > MAX_CITIZEN_ENTITIES {
        warn!(
            "Invariant warning: population {} exceeds reasonable threshold {}.",
            stats.population, MAX_CITIZEN_ENTITIES
        );
        violations.citizen_count_warning += 1;
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct InvariantChecksPlugin;

impl Plugin for InvariantChecksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoreInvariantViolations>()
            .add_systems(
                FixedUpdate,
                (validate_budget, validate_stats)
                    .in_set(crate::SimulationSet::PostSim),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_invariant_violations_default() {
        let v = CoreInvariantViolations::default();
        assert_eq!(v.budget_treasury, 0);
        assert_eq!(v.budget_income, 0);
        assert_eq!(v.budget_expenses, 0);
        assert_eq!(v.population, 0);
        assert_eq!(v.happiness, 0);
        assert_eq!(v.citizen_count_warning, 0);
    }

    #[test]
    fn test_treasury_floor_constant() {
        assert!(TREASURY_FLOOR < 0.0);
        assert!(TREASURY_FLOOR.is_finite());
    }
}
