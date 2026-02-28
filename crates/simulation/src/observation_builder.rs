//! Builds a `CityObservation` snapshot from ECS resources each tick.
//!
//! The `build_observation` system runs in `FixedUpdate` / `SimulationSet::PostSim`
//! so that all simulation writes have settled before we capture the snapshot.

use bevy::prelude::*;

use crate::ascii_map;
use crate::city_observation::{
    ActionResultEntry, AttractivenessSnapshot, CityObservation, CityWarning, HappinessSnapshot,
    PopulationSnapshot, ServiceCoverageSnapshot, ZoneDemandSnapshot,
};
use crate::citizen::{Citizen, WorkLocation};
use crate::coverage_metrics::CoverageMetrics;
use crate::crime::CrimeGrid;
use crate::economy::CityBudget;
use crate::game_actions::{ActionResult, ActionResultLog};
use crate::grid::WorldGrid;
use crate::immigration::CityAttractiveness;
use crate::homelessness::HomelessnessStats;
use crate::pollution::PollutionGrid;
use crate::stats::CityStats;
use crate::time_of_day::GameClock;
use crate::traffic_congestion::TrafficCongestion;
use crate::virtual_population::VirtualPopulation;
use crate::zones::ZoneDemand;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Resource: holds the latest observation
// ---------------------------------------------------------------------------

/// The most recent city observation, updated every tick in PostSim.
#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentObservation {
    pub observation: CityObservation,
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Snapshot the city state into `CurrentObservation`.
///
/// Reads from existing ECS resources and populates a `CityObservation`.
/// Fields that don't have a clear data source yet use sensible defaults.
#[allow(clippy::too_many_arguments)]
pub fn build_observation(
    tick_counter: Res<TickCounter>,
    clock: Res<GameClock>,
    budget: Res<CityBudget>,
    stats: Res<CityStats>,
    zone_demand: Res<ZoneDemand>,
    coverage: Res<CoverageMetrics>,
    homelessness: Res<HomelessnessStats>,
    virtual_pop: Res<VirtualPopulation>,
    traffic_congestion: Res<TrafficCongestion>,
    pollution_grid: Res<PollutionGrid>,
    crime_grid: Res<CrimeGrid>,
    action_log: Res<ActionResultLog>,
    grid: Res<WorldGrid>,
    attract: Res<CityAttractiveness>,
    employed_citizens: Query<(), (With<Citizen>, With<WorkLocation>)>,
    mut current: ResMut<CurrentObservation>,
) {
    let real_employed = employed_citizens.iter().count() as u32;
    let total_employed = real_employed + virtual_pop.virtual_employed;

    let population_total = stats.population;
    let unemployed = population_total.saturating_sub(total_employed);

    // Average happiness from real citizens (virtual citizens don't have
    // individual happiness in the ECS).
    let avg_happiness = stats.average_happiness;

    // Compute warning thresholds
    let warnings = compute_warnings(
        &budget,
        &coverage,
        &homelessness,
        &traffic_congestion,
        &pollution_grid,
        &crime_grid,
        population_total,
        unemployed,
    );

    // Populate recent action results from the ActionResultLog.
    let recent_action_results: Vec<ActionResultEntry> = action_log
        .last_n(10)
        .iter()
        .map(|(action, result)| {
            let mut summary = format!("{:?}", action);
            summary.truncate(100);
            ActionResultEntry {
                action_summary: summary,
                success: matches!(result, ActionResult::Success),
            }
        })
        .collect();

    // Build overview map from the world grid
    let overview_map = ascii_map::build_overview_map(&grid);

    current.observation = CityObservation {
        tick: tick_counter.0,
        day: clock.day,
        hour: clock.hour,
        speed: clock.speed,
        paused: clock.paused,

        treasury: budget.treasury,
        monthly_income: budget.monthly_income,
        monthly_expenses: budget.monthly_expenses,
        net_income: budget.monthly_income - budget.monthly_expenses,

        population: PopulationSnapshot {
            total: population_total,
            employed: total_employed,
            unemployed,
            homeless: homelessness.total_homeless,
        },

        zone_demand: ZoneDemandSnapshot {
            residential: zone_demand.residential,
            commercial: zone_demand.commercial,
            industrial: zone_demand.industrial,
            office: zone_demand.office,
        },

        power_coverage: coverage.power,
        water_coverage: coverage.water,

        services: ServiceCoverageSnapshot {
            fire: coverage.fire,
            police: coverage.police,
            health: coverage.health,
            education: coverage.education,
        },

        happiness: HappinessSnapshot {
            overall: avg_happiness,
            // TODO: Expose per-factor happiness breakdown once a HappinessBreakdown
            // resource is added. For now, we only report the aggregate.
            components: Vec::new(),
        },

        attractiveness_score: attract.overall_score,
        attractiveness: AttractivenessSnapshot {
            employment: attract.employment_factor,
            happiness: attract.happiness_factor,
            services: attract.services_factor,
            housing: attract.housing_factor,
            tax: attract.tax_factor,
        },

        building_count: stats.residential_buildings
            + stats.commercial_buildings
            + stats.industrial_buildings
            + stats.office_buildings
            + stats.mixed_use_buildings,

        warnings,

        recent_action_results,

        overview_map,
    };
}

// ---------------------------------------------------------------------------
// Warning detection helpers
// ---------------------------------------------------------------------------

/// Threshold-based warning detection from current city resources.
#[allow(clippy::too_many_arguments)]
fn compute_warnings(
    budget: &CityBudget,
    coverage: &CoverageMetrics,
    homelessness: &HomelessnessStats,
    traffic_congestion: &TrafficCongestion,
    pollution_grid: &PollutionGrid,
    crime_grid: &CrimeGrid,
    population: u32,
    unemployed: u32,
) -> Vec<CityWarning> {
    let mut warnings = Vec::new();

    // Negative budget
    if budget.monthly_income < budget.monthly_expenses && budget.treasury < 0.0 {
        warnings.push(CityWarning::NegativeBudget);
    }

    // Power shortage (coverage below 80%)
    if coverage.power < 0.8 {
        warnings.push(CityWarning::PowerShortage);
    }

    // Water shortage (coverage below 80%)
    if coverage.water < 0.8 {
        warnings.push(CityWarning::WaterShortage);
    }

    // High unemployment (> 30% of population)
    if population > 0 && (unemployed as f32 / population as f32) > 0.3 {
        warnings.push(CityWarning::HighUnemployment);
    }

    // High homelessness (> 5% of population)
    if population > 0 && (homelessness.total_homeless as f32 / population as f32) > 0.05 {
        warnings.push(CityWarning::HighHomelessness);
    }

    // Traffic congestion: average speed multiplier below 0.5 across occupied cells
    let avg_speed = average_traffic_speed(traffic_congestion);
    if avg_speed < 0.5 {
        warnings.push(CityWarning::TrafficCongestion);
    }

    // High pollution: average pollution level above 128 (out of 255)
    let avg_pollution = average_grid_level(&pollution_grid.levels);
    if avg_pollution > 128.0 {
        warnings.push(CityWarning::HighPollution);
    }

    // High crime: average crime level above 128
    let avg_crime = average_grid_level(&crime_grid.levels);
    if avg_crime > 128.0 {
        warnings.push(CityWarning::HighCrime);
    }

    warnings
}

/// Average speed multiplier across all cells (1.0 = free flow).
fn average_traffic_speed(congestion: &TrafficCongestion) -> f32 {
    if congestion.speed_multipliers.is_empty() {
        return 1.0;
    }
    let sum: f32 = congestion.speed_multipliers.iter().sum();
    sum / congestion.speed_multipliers.len() as f32
}

/// Average u8 grid level (pollution, crime, etc.).
fn average_grid_level(levels: &[u8]) -> f32 {
    if levels.is_empty() {
        return 0.0;
    }
    let sum: f64 = levels.iter().map(|&v| v as f64).sum();
    (sum / levels.len() as f64) as f32
}
