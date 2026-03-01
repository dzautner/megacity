use bevy::prelude::*;

use crate::citizen::{Citizen, CitizenDetails, HomeLocation, Needs, WorkLocation};
use crate::crime::CrimeGrid;
use crate::death_care::{self, DeathCareGrid};
use crate::economy::CityBudget;
use crate::grid::WorldGrid;
use crate::heating::{self, HeatingGrid};
use crate::homelessness::Homeless;
use crate::policies::Policies;
use crate::postal::PostalCoverage;
use crate::traffic::TrafficGrid;
use crate::wealth::WealthTier;
use crate::weather::Weather;

use super::constants::*;
use super::coverage::ServiceCoverageGrid;

/// The diminishing returns value at satisfaction = 0.5. Used to center the
/// needs contribution so that 50% satisfaction gives ~0 happiness contribution,
/// preserving the original formula's behavior while adding diminishing returns.
const DIMINISHED_MIDPOINT: f32 = 0.7769; // 1 - exp(-3 * 0.5)

/// Bundled secondary resources for update_happiness to stay within the 16-param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct HappinessExtras<'w> {
    pub road_condition: Res<'w, crate::road_maintenance::RoadConditionGrid>,
    pub death_care_grid: Res<'w, DeathCareGrid>,
    pub heating_grid: Res<'w, HeatingGrid>,
    pub postal_coverage: Res<'w, PostalCoverage>,
    pub waste_collection: Res<'w, crate::garbage::WasteCollectionGrid>,
    pub waste_accumulation: Res<'w, crate::waste_effects::WasteAccumulation>,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_happiness(
    tick: Res<crate::TickCounter>,
    grid: Res<WorldGrid>,
    budget: Res<CityBudget>,
    traffic: Res<TrafficGrid>,
    pollution_grid: Res<crate::pollution::PollutionGrid>,
    garbage_grid: Res<crate::garbage::GarbageGrid>,
    land_value_grid: Res<crate::land_value::LandValueGrid>,
    crime_grid: Res<CrimeGrid>,
    noise_grid: Res<crate::noise::NoisePollutionGrid>,
    policies: Res<Policies>,
    weather: Res<Weather>,
    coverage: Res<ServiceCoverageGrid>,
    extras: HappinessExtras,
    mut citizens: Query<
        (
            &mut CitizenDetails,
            &HomeLocation,
            Option<&WorkLocation>,
            Option<&Needs>,
            Option<&Homeless>,
        ),
        With<Citizen>,
    >,
) {
    #[cfg(feature = "trace")]
    let _span = bevy::log::info_span!("update_happiness").entered();
    let road_condition = &extras.road_condition;
    let death_care_grid = &extras.death_care_grid;
    let heating_grid = &extras.heating_grid;
    let postal_coverage = &extras.postal_coverage;
    let waste_collection = &extras.waste_collection;
    let waste_accumulation = &extras.waste_accumulation;
    if !tick.0.is_multiple_of(HAPPINESS_UPDATE_INTERVAL) {
        return;
    }
    let tax_penalty = if budget.tax_rate > 0.15 {
        HIGH_TAX_PENALTY * ((budget.tax_rate - 0.15) / 0.10)
    } else {
        0.0
    };

    // Pre-compute shared values to avoid redundant reads per citizen
    let policy_bonus = policies.happiness_bonus();
    let raw_weather_mod = weather.happiness_modifier();
    let weather_bonus = weather_happiness_factor(raw_weather_mod);
    let heat_demand = heating::heating_demand(&weather);

    citizens
        .par_iter_mut()
        .for_each(|(mut details, home, work, needs, homeless)| {
            let happiness = compute_citizen_happiness(
                &details,
                home,
                work,
                needs,
                homeless,
                &grid,
                &coverage,
                &pollution_grid,
                &garbage_grid,
                &land_value_grid,
                &crime_grid,
                &noise_grid,
                &traffic,
                road_condition,
                death_care_grid,
                heating_grid,
                postal_coverage,
                waste_collection,
                waste_accumulation,
                tax_penalty,
                policy_bonus,
                weather_bonus,
                heat_demand,
            );
            details.happiness = happiness.clamp(0.0, 100.0);
        });
}

/// Compute happiness for a single citizen. Extracted for testability.
#[allow(clippy::too_many_arguments)]
fn compute_citizen_happiness(
    details: &CitizenDetails,
    home: &HomeLocation,
    work: Option<&WorkLocation>,
    needs: Option<&Needs>,
    homeless: Option<&Homeless>,
    grid: &WorldGrid,
    coverage: &ServiceCoverageGrid,
    pollution_grid: &crate::pollution::PollutionGrid,
    garbage_grid: &crate::garbage::GarbageGrid,
    land_value_grid: &crate::land_value::LandValueGrid,
    crime_grid: &CrimeGrid,
    noise_grid: &crate::noise::NoisePollutionGrid,
    traffic: &TrafficGrid,
    road_condition: &crate::road_maintenance::RoadConditionGrid,
    death_care_grid: &DeathCareGrid,
    heating_grid: &HeatingGrid,
    postal_coverage: &PostalCoverage,
    waste_collection: &crate::garbage::WasteCollectionGrid,
    waste_accumulation: &crate::waste_effects::WasteAccumulation,
    tax_penalty: f32,
    policy_bonus: f32,
    weather_bonus: f32,
    heat_demand: f32,
) -> f32 {
    let mut happiness = BASE_HAPPINESS;

    // Wealth-tier weights
    let weights = WealthTier::from_education(details.education).happiness_weights();

    // --- Employment (weighted by tier) ---
    if work.is_some() {
        happiness += EMPLOYED_BONUS * weights.employment;
    }

    // --- Commute distance ---
    if let Some(work_loc) = work {
        let dx = (home.grid_x as i32 - work_loc.grid_x as i32).abs();
        let dy = (home.grid_y as i32 - work_loc.grid_y as i32).abs();
        let dist = dx + dy;
        if dist < 20 {
            happiness += SHORT_COMMUTE_BONUS;
        }
    }

    // --- Utilities with critical thresholds ---
    let home_cell = grid.get(home.grid_x, home.grid_y);
    if home_cell.has_power {
        happiness += POWER_BONUS;
    } else {
        happiness -= NO_POWER_PENALTY;
        happiness -= CRITICAL_NO_POWER_PENALTY;
    }
    if home_cell.has_water {
        happiness += WATER_BONUS;
    } else {
        happiness -= NO_WATER_PENALTY;
        happiness -= CRITICAL_NO_WATER_PENALTY;
    }

    // --- Service coverage (O(1) bitflag lookup from precomputed grid) ---
    let idx = ServiceCoverageGrid::idx(home.grid_x, home.grid_y);
    let cov = coverage.flags[idx];
    if cov & COVERAGE_HEALTH != 0 {
        happiness += HEALTH_COVERAGE_BONUS * weights.services;
    }
    if cov & COVERAGE_EDUCATION != 0 {
        happiness += EDUCATION_BONUS * weights.services;
    }
    if cov & COVERAGE_POLICE != 0 {
        happiness += POLICE_BONUS * weights.services;
    }
    if cov & COVERAGE_PARK != 0 {
        happiness += PARK_BONUS * weights.parks;
    }
    if cov & COVERAGE_ENTERTAINMENT != 0 {
        happiness += ENTERTAINMENT_BONUS * weights.entertainment;
    }
    if cov & COVERAGE_TELECOM != 0 {
        happiness += TELECOM_BONUS;
    }
    if cov & COVERAGE_TRANSPORT != 0 {
        happiness += TRANSPORT_BONUS;
    }

    // --- Pollution with diminishing returns ---
    let pollution = pollution_grid.get(home.grid_x, home.grid_y) as f32;
    let poll_ratio = (pollution / 255.0).clamp(0.0, 1.0);
    let poll_diminished = diminishing_returns(poll_ratio, DIMINISHING_K_NEGATIVE);
    happiness -= poll_diminished * (255.0 / 25.0) * weights.pollution;

    // --- Garbage penalty (scaled linearly: 0 at level 10, full at level 100) ---
    let garbage_level = garbage_grid.get(home.grid_x, home.grid_y) as f32;
    if garbage_level > 10.0 {
        let ratio = ((garbage_level - 10.0) / 90.0).clamp(0.0, 1.0);
        happiness -= GARBAGE_PENALTY * ratio;
    }

    // --- Uncollected waste penalty (WASTE-003, scaled: 0 at 100 lbs, full at 1000 lbs) ---
    let uncollected = waste_collection.uncollected(home.grid_x, home.grid_y);
    if uncollected > 100.0 {
        let ratio = ((uncollected - 100.0) / 900.0).clamp(0.0, 1.0);
        happiness -= crate::garbage::UNCOLLECTED_WASTE_HAPPINESS_PENALTY * ratio;
    }

    // --- Accumulated waste happiness penalty (WASTE-010) ---
    let accumulated = waste_accumulation.get(home.grid_x, home.grid_y);
    happiness += crate::waste_effects::waste_happiness_penalty(accumulated);

    // --- Crime with diminishing returns + critical threshold ---
    let crime_level = crime_grid.get(home.grid_x, home.grid_y) as f32;
    let crime_ratio = (crime_level / 255.0).clamp(0.0, 1.0);
    let crime_diminished = diminishing_returns(crime_ratio, DIMINISHING_K_NEGATIVE);
    happiness -= crime_diminished * CRIME_PENALTY_MAX;
    if crime_level > CRITICAL_CRIME_THRESHOLD {
        happiness -= CRITICAL_CRIME_PENALTY;
    }

    // --- Noise penalty ---
    happiness -= (noise_grid.get(home.grid_x, home.grid_y) as f32) / 25.0;

    // --- Land value with diminishing returns ---
    let land_value = land_value_grid.get(home.grid_x, home.grid_y) as f32;
    let lv_ratio = (land_value / 255.0).clamp(0.0, 1.0);
    let lv_diminished = diminishing_returns(lv_ratio, DIMINISHING_K_DEFAULT);
    happiness += lv_diminished * (255.0 / 50.0) * weights.land_value;

    // --- Traffic congestion ---
    let congestion = traffic.congestion_level(home.grid_x, home.grid_y);
    happiness -= congestion * CONGESTION_PENALTY;

    // --- Tax penalty ---
    happiness -= tax_penalty;

    // --- Policy bonus ---
    happiness += policy_bonus;

    // --- Weather happiness factor (with diminishing returns, pre-computed) ---
    happiness += weather_bonus;

    // --- Needs satisfaction with diminishing returns + critical threshold ---
    if let Some(needs) = needs {
        let satisfaction = needs.overall_satisfaction();
        let needs_diminished = diminishing_returns(satisfaction, DIMINISHING_K_DEFAULT);
        // Center at the diminished midpoint so 50% satisfaction gives ~0
        happiness += (needs_diminished - DIMINISHED_MIDPOINT) * 35.0;
        if satisfaction < CRITICAL_NEEDS_THRESHOLD {
            happiness -= CRITICAL_NEEDS_PENALTY;
        }
    }

    // --- Health with critical threshold ---
    if details.health < 50.0 {
        happiness -= (50.0 - details.health) * 0.2;
    }
    if details.health > 80.0 {
        happiness += 8.0;
    }
    if details.health < CRITICAL_HEALTH_THRESHOLD {
        happiness -= CRITICAL_HEALTH_PENALTY;
    }

    // --- Wealth satisfaction factor (diminishing returns on savings) ---
    happiness += wealth_satisfaction(details.savings);

    // --- Homelessness penalty ---
    if let Some(h) = homeless {
        if h.sheltered {
            happiness -= SHELTERED_PENALTY;
        } else {
            happiness -= HOMELESS_PENALTY;
        }
    }

    // --- Road condition penalty ---
    let hx = home.grid_x;
    let hy = home.grid_y;
    let check_radius: i32 = 3;
    let mut worst_road_condition: u8 = 255;
    for dy in -check_radius..=check_radius {
        for dx in -check_radius..=check_radius {
            let nx = hx as i32 + dx;
            let ny = hy as i32 + dy;
            if nx >= 0
                && ny >= 0
                && (nx as usize) < crate::config::GRID_WIDTH
                && (ny as usize) < crate::config::GRID_HEIGHT
            {
                let cond = road_condition.get(nx as usize, ny as usize);
                if cond > 0 && cond < worst_road_condition {
                    worst_road_condition = cond;
                }
            }
        }
    }
    if worst_road_condition < 50 {
        happiness -= POOR_ROAD_PENALTY;
    }

    // --- Postal coverage bonus (0 to +5) ---
    happiness += crate::postal::postal_happiness_bonus(postal_coverage, home.grid_x, home.grid_y);

    // --- Death care penalty ---
    if death_care_grid.has_nearby_unprocessed(home.grid_x, home.grid_y) {
        happiness -= death_care::DEATH_CARE_PENALTY;
    }

    // --- Heating ---
    if heat_demand > 0.0 {
        if heating_grid.is_heated(home.grid_x, home.grid_y) {
            happiness += heating::HEATING_WARM_BONUS;
        } else {
            happiness -= heating::HEATING_COLD_PENALTY * heat_demand;
        }
    }

    happiness
}
