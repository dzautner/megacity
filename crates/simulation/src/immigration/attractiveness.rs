use bevy::prelude::*;

use crate::buildings::{Building, UnderConstruction};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::education_jobs::EmploymentStats;
use crate::happiness::{
    ServiceCoverageGrid, COVERAGE_EDUCATION, COVERAGE_FIRE, COVERAGE_HEALTH, COVERAGE_POLICE,
};
use crate::stats::CityStats;
use crate::TickCounter;

use super::types::{
    CityAttractiveness, ATTRACTIVENESS_INTERVAL, WEIGHT_EMPLOYMENT, WEIGHT_HAPPINESS,
    WEIGHT_HOUSING, WEIGHT_SERVICES, WEIGHT_TAX,
};

// ---------------------------------------------------------------------------
// System: compute_attractiveness
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn compute_attractiveness(
    tick: Res<TickCounter>,
    employment_stats: Res<EmploymentStats>,
    city_stats: Res<CityStats>,
    budget: Res<CityBudget>,
    coverage: Res<ServiceCoverageGrid>,
    buildings: Query<&Building, Without<UnderConstruction>>,
    mut attractiveness: ResMut<CityAttractiveness>,
) {
    if !tick.0.is_multiple_of(ATTRACTIVENESS_INTERVAL) {
        return;
    }

    // --- Employment factor ---
    // Low unemployment = attractive. 0% unemployment -> 1.0, 20%+ -> 0.0
    let unemployment = employment_stats.unemployment_rate;
    let employment_factor = (1.0 - unemployment * 5.0).clamp(0.0, 1.0);

    // --- Happiness factor ---
    // Average happiness mapped from 0-100 to 0.0-1.0.
    // When no citizens exist yet, treat happiness as a pleasant baseline
    // (60/100) rather than 0 — an empty city isn't "unhappy", it just has
    // no one to survey yet.  This prevents a bootstrapping deadlock where
    // the city can never attract its first residents.
    let raw_happiness = if city_stats.population == 0 {
        65.0
    } else {
        city_stats.average_happiness
    };
    let happiness_factor = (raw_happiness / 100.0).clamp(0.0, 1.0);

    // --- Services factor ---
    // Fraction of populated cells covered by health, education, police, fire
    // Sample every 4th cell for performance
    let total_cells = GRID_WIDTH * GRID_HEIGHT;
    let mut health_count = 0u32;
    let mut edu_count = 0u32;
    let mut police_count = 0u32;
    let mut fire_count = 0u32;
    let mut sampled = 0u32;

    for idx in (0..total_cells).step_by(4) {
        sampled += 1;
        let flags = coverage.flags[idx];
        if flags & COVERAGE_HEALTH != 0 {
            health_count += 1;
        }
        if flags & COVERAGE_EDUCATION != 0 {
            edu_count += 1;
        }
        if flags & COVERAGE_POLICE != 0 {
            police_count += 1;
        }
        if flags & COVERAGE_FIRE != 0 {
            fire_count += 1;
        }
    }

    let services_factor = if sampled > 0 {
        let health_cov = health_count as f32 / sampled as f32;
        let edu_cov = edu_count as f32 / sampled as f32;
        let police_cov = police_count as f32 / sampled as f32;
        let fire_cov = fire_count as f32 / sampled as f32;
        ((health_cov + edu_cov + police_cov + fire_cov) / 4.0).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // --- Housing factor ---
    // Available residential capacity vs current occupancy
    // MixedUse buildings also provide residential capacity
    let mut total_res_capacity = 0u32;
    let mut total_res_occupants = 0u32;
    for b in &buildings {
        if b.zone_type.is_residential() || b.zone_type.is_mixed_use() {
            total_res_capacity += b.capacity;
            total_res_occupants += b.occupants;
        }
    }

    let housing_factor = if total_res_capacity > 0 {
        let vacancy_rate = (total_res_capacity.saturating_sub(total_res_occupants)) as f32
            / total_res_capacity as f32;
        // Ideal vacancy is 5-15%. Too low = no room, too high = ghost town.
        // Peak attractiveness at ~10% vacancy.
        // Exception: brand-new cities with 0 occupants are attractive (plenty of
        // room for pioneers), not penalised as abandoned.
        if total_res_occupants == 0 {
            0.8 // Brand-new empty housing — very attractive to pioneers
        } else if vacancy_rate < 0.02 {
            0.1 // Almost no housing available
        } else if vacancy_rate < 0.05 {
            0.3 + (vacancy_rate - 0.02) * 10.0 // 0.3-0.6
        } else if vacancy_rate <= 0.20 {
            0.6 + (0.20 - (vacancy_rate - 0.10).abs()) * 2.0 // peak around 10%
        } else {
            // Too much vacancy suggests abandonment
            (1.0 - (vacancy_rate - 0.20) * 2.0).max(0.2)
        }
        .clamp(0.0, 1.0)
    } else {
        0.0 // No residential buildings at all
    };

    // --- Tax factor ---
    // 10% baseline is neutral (0.5). Lower = more attractive, higher = less.
    let baseline_tax = 0.10;
    let tax_diff = budget.tax_rate - baseline_tax;
    // Each 1% deviation from baseline shifts factor by 0.05
    let tax_factor = (0.5 - tax_diff * 5.0).clamp(0.0, 1.0);

    // --- Overall score ---
    let overall = employment_factor * WEIGHT_EMPLOYMENT
        + happiness_factor * WEIGHT_HAPPINESS
        + services_factor * WEIGHT_SERVICES
        + housing_factor * WEIGHT_HOUSING
        + tax_factor * WEIGHT_TAX;

    attractiveness.overall_score = overall.clamp(0.0, 100.0);
    attractiveness.employment_factor = employment_factor;
    attractiveness.happiness_factor = happiness_factor;
    attractiveness.services_factor = services_factor;
    attractiveness.housing_factor = housing_factor;
    attractiveness.tax_factor = tax_factor;
}
