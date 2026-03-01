//! Per-factor happiness breakdown aggregated across all citizens.
//!
//! Provides a `HappinessBreakdown` resource that the observation builder can
//! read to populate `HappinessSnapshot.components`, giving the LLM agent
//! visibility into *what* is driving citizen happiness up or down.

use bevy::prelude::*;

use crate::citizen::{Citizen, CitizenDetails, HomeLocation, Needs, WorkLocation};
use crate::crime::CrimeGrid;
use crate::economy::CityBudget;
use crate::grid::WorldGrid;
use crate::happiness::*;
use crate::heating;
use crate::homelessness::Homeless;
use crate::policies::Policies;
use crate::traffic::TrafficGrid;
use crate::weather::Weather;
use crate::TickCounter;

/// Aggregated per-factor happiness contributions averaged across all citizens.
/// Tracks the 22 most impactful happiness factors from the simulation.
#[derive(Resource, Debug, Clone, Default)]
pub struct HappinessBreakdown {
    /// Each entry is (factor_name, average_contribution) across all citizens.
    pub factors: Vec<(String, f32)>,
}

/// Bundled resources for the breakdown system to stay under the 16-param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct BreakdownResources<'w> {
    pub road_condition: Res<'w, crate::road_maintenance::RoadConditionGrid>,
    pub waste_collection: Res<'w, crate::garbage::WasteCollectionGrid>,
    pub waste_accumulation: Res<'w, crate::waste_effects::WasteAccumulation>,
    pub garbage_grid: Res<'w, crate::garbage::GarbageGrid>,
    pub land_value_grid: Res<'w, crate::land_value::LandValueGrid>,
    pub noise_grid: Res<'w, crate::noise::NoisePollutionGrid>,
    pub pollution_grid: Res<'w, crate::pollution::PollutionGrid>,
}

/// Number of tracked happiness factors.
const NUM_FACTORS: usize = 22;

const F_EMPLOYMENT: usize = 0;
const F_COMMUTE: usize = 1;
const F_POWER: usize = 2;
const F_WATER: usize = 3;
const F_HEALTH_SVC: usize = 4;
const F_EDUCATION_SVC: usize = 5;
const F_POLICE_SVC: usize = 6;
const F_PARKS: usize = 7;
const F_ENTERTAINMENT: usize = 8;
const F_TELECOM: usize = 9;
const F_TRANSPORT: usize = 10;
const F_POLLUTION: usize = 11;
const F_GARBAGE: usize = 12;
const F_CRIME: usize = 13;
const F_NOISE: usize = 14;
const F_LAND_VALUE: usize = 15;
const F_TRAFFIC: usize = 16;
const F_TAX: usize = 17;
const F_POLICY: usize = 18;
const F_WEATHER: usize = 19;
const F_NEEDS: usize = 20;
const F_HEALTH: usize = 21;

const FACTOR_NAMES: [&str; NUM_FACTORS] = [
    "employment",
    "commute",
    "power",
    "water",
    "health_services",
    "education_services",
    "police_services",
    "parks",
    "entertainment",
    "telecom",
    "transport",
    "pollution",
    "garbage",
    "crime",
    "noise",
    "land_value",
    "traffic",
    "tax",
    "policy",
    "weather",
    "needs",
    "health",
];

const DIMINISHED_MIDPOINT: f32 = 0.7769;

/// System that computes per-factor happiness averages across all citizens.
/// Factors with absolute contribution below 0.01 are filtered out.
///
/// Runs on the same interval as `update_happiness` to stay in sync.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn compute_happiness_breakdown(
    tick: Res<TickCounter>,
    grid: Res<WorldGrid>,
    budget: Res<CityBudget>,
    traffic: Res<TrafficGrid>,
    crime_grid: Res<CrimeGrid>,
    policies: Res<Policies>,
    weather: Res<Weather>,
    coverage: Res<ServiceCoverageGrid>,
    extras: BreakdownResources,
    citizens: Query<
        (
            &CitizenDetails,
            &HomeLocation,
            Option<&WorkLocation>,
            Option<&Needs>,
            Option<&Homeless>,
        ),
        With<Citizen>,
    >,
    mut breakdown: ResMut<HappinessBreakdown>,
) {
    if !tick.0.is_multiple_of(HAPPINESS_UPDATE_INTERVAL) {
        return;
    }

    let mut sums = [0.0_f64; NUM_FACTORS];
    let mut count: u64 = 0;

    let tax_penalty = if budget.tax_rate > 0.15 {
        HIGH_TAX_PENALTY * ((budget.tax_rate - 0.15) / 0.10)
    } else {
        0.0
    };
    let policy_bonus = policies.happiness_bonus();
    let raw_weather_mod = weather.happiness_modifier();
    let weather_bonus = weather_happiness_factor(raw_weather_mod);
    let heat_demand = heating::heating_demand(&weather);
    let _ = heat_demand; // Used below only for heating factor note

    for (details, home, work, needs, _homeless) in &citizens {
        let mut factors = [0.0_f32; NUM_FACTORS];

        let weights =
            crate::wealth::WealthTier::from_education(details.education).happiness_weights();

        // Employment
        if work.is_some() {
            factors[F_EMPLOYMENT] = EMPLOYED_BONUS * weights.employment;
        }

        // Commute
        if let Some(work_loc) = work {
            let dx = (home.grid_x as i32 - work_loc.grid_x as i32).abs();
            let dy = (home.grid_y as i32 - work_loc.grid_y as i32).abs();
            if dx + dy < 20 {
                factors[F_COMMUTE] = SHORT_COMMUTE_BONUS;
            }
        }

        // Power
        let home_cell = grid.get(home.grid_x, home.grid_y);
        if home_cell.has_power {
            factors[F_POWER] = POWER_BONUS;
        } else {
            factors[F_POWER] = -(NO_POWER_PENALTY + CRITICAL_NO_POWER_PENALTY);
        }

        // Water
        if home_cell.has_water {
            factors[F_WATER] = WATER_BONUS;
        } else {
            factors[F_WATER] = -(NO_WATER_PENALTY + CRITICAL_NO_WATER_PENALTY);
        }

        // Service coverage (bitflag lookup)
        let idx = ServiceCoverageGrid::idx(home.grid_x, home.grid_y);
        let cov = coverage.flags[idx];
        if cov & COVERAGE_HEALTH != 0 {
            factors[F_HEALTH_SVC] = HEALTH_COVERAGE_BONUS * weights.services;
        }
        if cov & COVERAGE_EDUCATION != 0 {
            factors[F_EDUCATION_SVC] = EDUCATION_BONUS * weights.services;
        }
        if cov & COVERAGE_POLICE != 0 {
            factors[F_POLICE_SVC] = POLICE_BONUS * weights.services;
        }
        if cov & COVERAGE_PARK != 0 {
            factors[F_PARKS] = PARK_BONUS * weights.parks;
        }
        if cov & COVERAGE_ENTERTAINMENT != 0 {
            factors[F_ENTERTAINMENT] = ENTERTAINMENT_BONUS * weights.entertainment;
        }
        if cov & COVERAGE_TELECOM != 0 {
            factors[F_TELECOM] = TELECOM_BONUS;
        }
        if cov & COVERAGE_TRANSPORT != 0 {
            factors[F_TRANSPORT] = TRANSPORT_BONUS;
        }

        // Pollution
        let pollution = extras.pollution_grid.get(home.grid_x, home.grid_y) as f32;
        let poll_ratio = (pollution / 255.0).clamp(0.0, 1.0);
        let poll_diminished = diminishing_returns(poll_ratio, DIMINISHING_K_NEGATIVE);
        factors[F_POLLUTION] = -(poll_diminished * (255.0 / 25.0) * weights.pollution);

        // Garbage (combines garbage grid, uncollected waste, waste accumulation)
        let mut garbage_contrib = 0.0_f32;
        if extras.garbage_grid.get(home.grid_x, home.grid_y) > 10 {
            garbage_contrib -= GARBAGE_PENALTY;
        }
        let uncollected = extras.waste_collection.uncollected(home.grid_x, home.grid_y);
        if uncollected > 100.0 {
            garbage_contrib -= crate::garbage::UNCOLLECTED_WASTE_HAPPINESS_PENALTY;
        }
        let accumulated = extras.waste_accumulation.get(home.grid_x, home.grid_y);
        garbage_contrib += crate::waste_effects::waste_happiness_penalty(accumulated);
        factors[F_GARBAGE] = garbage_contrib;

        // Crime
        let crime_level = crime_grid.get(home.grid_x, home.grid_y) as f32;
        let crime_ratio = (crime_level / 255.0).clamp(0.0, 1.0);
        let crime_diminished = diminishing_returns(crime_ratio, DIMINISHING_K_NEGATIVE);
        let mut crime_contrib = -(crime_diminished * CRIME_PENALTY_MAX);
        if crime_level > CRITICAL_CRIME_THRESHOLD {
            crime_contrib -= CRITICAL_CRIME_PENALTY;
        }
        factors[F_CRIME] = crime_contrib;

        // Noise
        factors[F_NOISE] = -(extras.noise_grid.get(home.grid_x, home.grid_y) as f32) / 25.0;

        // Land value
        let land_value = extras.land_value_grid.get(home.grid_x, home.grid_y) as f32;
        let lv_ratio = (land_value / 255.0).clamp(0.0, 1.0);
        let lv_diminished = diminishing_returns(lv_ratio, DIMINISHING_K_DEFAULT);
        factors[F_LAND_VALUE] = lv_diminished * (255.0 / 50.0) * weights.land_value;

        // Traffic
        factors[F_TRAFFIC] =
            -(traffic.congestion_level(home.grid_x, home.grid_y) * CONGESTION_PENALTY);

        // Tax
        factors[F_TAX] = -tax_penalty;

        // Policy
        factors[F_POLICY] = policy_bonus;

        // Weather
        factors[F_WEATHER] = weather_bonus;

        // Needs
        if let Some(needs) = needs {
            let satisfaction = needs.overall_satisfaction();
            let needs_diminished = diminishing_returns(satisfaction, DIMINISHING_K_DEFAULT);
            let mut needs_contrib = (needs_diminished - DIMINISHED_MIDPOINT) * 35.0;
            if satisfaction < CRITICAL_NEEDS_THRESHOLD {
                needs_contrib -= CRITICAL_NEEDS_PENALTY;
            }
            factors[F_NEEDS] = needs_contrib;
        }

        // Health
        let mut health_contrib = 0.0_f32;
        if details.health < 50.0 {
            health_contrib -= (50.0 - details.health) * 0.2;
        }
        if details.health > 80.0 {
            health_contrib += 8.0;
        }
        if details.health < CRITICAL_HEALTH_THRESHOLD {
            health_contrib -= CRITICAL_HEALTH_PENALTY;
        }
        factors[F_HEALTH] = health_contrib;

        for i in 0..NUM_FACTORS {
            sums[i] += factors[i] as f64;
        }
        count += 1;
    }

    if count == 0 {
        breakdown.factors.clear();
        return;
    }

    let inv = 1.0 / count as f64;
    breakdown.factors = FACTOR_NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| ((*name).to_string(), (sums[i] * inv) as f32))
        .filter(|(_, v)| v.abs() > 0.01)
        .collect();
}

pub struct HappinessBreakdownPlugin;

impl Plugin for HappinessBreakdownPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HappinessBreakdown>();
        app.add_systems(
            FixedUpdate,
            compute_happiness_breakdown
                .after(crate::happiness::update_happiness)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
