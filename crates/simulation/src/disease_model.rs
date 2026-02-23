//! SERV-006: Health and Disease Model
//!
//! Three disease types with distinct spread mechanics:
//! - **Flu**: seasonal, density-dependent, mild severity
//! - **Food poisoning**: sanitation/water-pollution-linked
//! - **Respiratory illness**: pollution-dependent
//!
//! Hospital treatment reduces severity and recovery time.
//! Untreated disease increases mortality risk.
//! City-wide health stats track infection rate, hospital utilization, and mortality.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenDetails};
use crate::pollution::PollutionGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::stats::CityStats;
use crate::time_of_day::GameClock;
use crate::weather::Weather;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Disease types
// ---------------------------------------------------------------------------

/// The three disease categories in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum DiseaseType {
    /// Seasonal flu: spreads faster in winter, density-dependent.
    Flu,
    /// Food poisoning: linked to poor sanitation (water pollution).
    FoodPoisoning,
    /// Respiratory illness: linked to air pollution levels.
    Respiratory,
}

impl DiseaseType {
    /// Base infection rate per slow tick (probability 0.0-1.0).
    pub fn base_infection_rate(self) -> f32 {
        match self {
            DiseaseType::Flu => 0.03,
            DiseaseType::FoodPoisoning => 0.02,
            DiseaseType::Respiratory => 0.015,
        }
    }

    /// Base recovery time in slow ticks (each slow tick ~ 100 fixed updates).
    pub fn base_recovery_ticks(self) -> u32 {
        match self {
            DiseaseType::Flu => 5,
            DiseaseType::FoodPoisoning => 3,
            DiseaseType::Respiratory => 8,
        }
    }

    /// Severity rating (0.0-1.0). Higher = more impact on health and mortality.
    pub fn severity(self) -> f32 {
        match self {
            DiseaseType::Flu => 0.2,
            DiseaseType::FoodPoisoning => 0.3,
            DiseaseType::Respiratory => 0.5,
        }
    }

    /// Hospital beds needed per infected citizen.
    pub fn beds_needed(self) -> f32 {
        match self {
            DiseaseType::Flu => 0.1,
            DiseaseType::FoodPoisoning => 0.3,
            DiseaseType::Respiratory => 0.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-citizen disease component
// ---------------------------------------------------------------------------

/// Attached to citizens who are currently infected.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseStatus {
    pub disease_type: DiseaseType,
    /// Remaining slow ticks until natural recovery.
    pub recovery_remaining: u32,
    /// Whether this citizen is receiving hospital treatment.
    pub hospitalized: bool,
}

// ---------------------------------------------------------------------------
// City-wide disease state (Saveable resource)
// ---------------------------------------------------------------------------

/// City-wide disease statistics and tracking.
#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct DiseaseState {
    pub flu_count: u32,
    pub food_poisoning_count: u32,
    pub respiratory_count: u32,
    pub total_infected: u32,
    pub infection_rate: f32,
    pub hospital_beds: u32,
    pub beds_in_use: u32,
    pub hospital_utilization: f32,
    pub mortality_this_cycle: u32,
    pub cumulative_mortality: u32,
    pub mortality_rate: f32,
}

impl crate::Saveable for DiseaseState {
    const SAVE_KEY: &'static str = "disease_state";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Hospital capacity
// ---------------------------------------------------------------------------

/// Beds provided by each health-service tier.
pub fn beds_for_service(service_type: ServiceType) -> u32 {
    match service_type {
        ServiceType::MedicalClinic => 10,
        ServiceType::Hospital => 50,
        ServiceType::MedicalCenter => 150,
        _ => 0,
    }
}

/// Compute total hospital bed capacity from all health service buildings.
pub fn compute_hospital_beds(services: &Query<&ServiceBuilding>) -> u32 {
    services
        .iter()
        .filter(|s| ServiceBuilding::is_health(s.service_type))
        .map(|s| beds_for_service(s.service_type))
        .sum()
}

// ---------------------------------------------------------------------------
// Environmental multipliers
// ---------------------------------------------------------------------------

/// Flu spreads faster in winter, slower in summer.
pub fn flu_season_multiplier(weather: &Weather) -> f32 {
    use crate::weather::types::Season;
    match weather.season {
        Season::Winter => 2.5,
        Season::Autumn => 1.5,
        Season::Spring => 1.0,
        Season::Summer => 0.4,
    }
}

/// Density multiplier: more citizens = faster flu spread.
pub fn density_multiplier(population: u32) -> f32 {
    let pop_factor = (population as f32 / 1000.0).clamp(0.0, 5.0);
    (0.5 + pop_factor * 0.3).clamp(0.5, 2.0)
}

/// Average pollution level across the grid (returned as 0.0-1.0).
pub fn average_pollution(pollution: &PollutionGrid) -> f32 {
    if pollution.levels.is_empty() {
        return 0.0;
    }
    let total: u64 = pollution.levels.iter().map(|&v| v as u64).sum();
    (total as f32 / pollution.levels.len() as f32 / 255.0).clamp(0.0, 1.0)
}

/// Sanitation risk approximated from pollution levels.
pub fn sanitation_risk(pollution: &PollutionGrid) -> f32 {
    (average_pollution(pollution) * 1.5).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Core systems
// ---------------------------------------------------------------------------

/// System: spread diseases to healthy citizens based on environmental factors.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn spread_diseases(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    pollution: Res<PollutionGrid>,
    stats: Res<CityStats>,
    disease_state: Res<DiseaseState>,
    clock: Res<GameClock>,
    citizens: Query<(Entity, &CitizenDetails), (With<Citizen>, Without<DiseaseStatus>)>,
    mut commands: Commands,
) {
    if !timer.should_run() {
        return;
    }
    let population = stats.population;
    if population == 0 {
        return;
    }

    let day_seed = clock.day;
    let flu_rate = DiseaseType::Flu.base_infection_rate()
        * flu_season_multiplier(&weather)
        * density_multiplier(population);
    let food_rate =
        DiseaseType::FoodPoisoning.base_infection_rate() * (1.0 + sanitation_risk(&pollution));
    let respiratory_rate = DiseaseType::Respiratory.base_infection_rate()
        * (1.0 + average_pollution(&pollution) * 3.0);
    let flu_pressure = 1.0 + (disease_state.flu_count as f32 / population as f32) * 2.0;

    for (entity, details) in citizens.iter() {
        let resistance = (details.health / 100.0).clamp(0.0, 1.0);
        let age_factor = match details.age {
            0..=5 => 1.5,
            6..=17 => 0.8,
            18..=54 => 1.0,
            55..=64 => 1.3,
            _ => 1.8,
        };

        let hash = ((entity.index() as u64).wrapping_mul(2654435761)
            ^ (day_seed as u64).wrapping_mul(40503))
            % 10000;
        let roll = hash as f32 / 10000.0;

        let effective_flu = flu_rate * flu_pressure * age_factor * (1.0 - resistance * 0.5);
        if roll < effective_flu {
            commands.entity(entity).insert(DiseaseStatus {
                disease_type: DiseaseType::Flu,
                recovery_remaining: DiseaseType::Flu.base_recovery_ticks(),
                hospitalized: false,
            });
            continue;
        }

        let roll2 = ((hash.wrapping_mul(31)) % 10000) as f32 / 10000.0;
        let effective_food = food_rate * age_factor * (1.0 - resistance * 0.3);
        if roll2 < effective_food {
            commands.entity(entity).insert(DiseaseStatus {
                disease_type: DiseaseType::FoodPoisoning,
                recovery_remaining: DiseaseType::FoodPoisoning.base_recovery_ticks(),
                hospitalized: false,
            });
            continue;
        }

        let roll3 = ((hash.wrapping_mul(127)) % 10000) as f32 / 10000.0;
        let effective_resp = respiratory_rate * age_factor * (1.0 - resistance * 0.4);
        if roll3 < effective_resp {
            commands.entity(entity).insert(DiseaseStatus {
                disease_type: DiseaseType::Respiratory,
                recovery_remaining: DiseaseType::Respiratory.base_recovery_ticks(),
                hospitalized: false,
            });
            continue;
        }
    }
}

/// System: progress disease recovery, apply health effects, handle mortality.
pub fn progress_disease(
    timer: Res<SlowTickTimer>,
    mut disease_state: ResMut<DiseaseState>,
    services: Query<&ServiceBuilding>,
    clock: Res<GameClock>,
    mut infected: Query<(Entity, &mut DiseaseStatus, &mut CitizenDetails), With<Citizen>>,
    mut commands: Commands,
) {
    if !timer.should_run() {
        return;
    }

    let total_beds = compute_hospital_beds(&services);
    let mut beds_used: u32 = 0;
    let mut flu_count: u32 = 0;
    let mut food_count: u32 = 0;
    let mut resp_count: u32 = 0;
    let mut deaths: u32 = 0;
    let day_seed = clock.day;

    for (entity, mut status, mut details) in infected.iter_mut() {
        if !status.hospitalized && beds_used < total_beds {
            let beds_needed = status.disease_type.beds_needed();
            if (beds_used as f32 + beds_needed) <= total_beds as f32 {
                status.hospitalized = true;
                beds_used += beds_needed.ceil() as u32;
            }
        } else if status.hospitalized {
            beds_used += status.disease_type.beds_needed().ceil() as u32;
        }

        let recovery_speed = if status.hospitalized { 2 } else { 1 };
        let severity_mult = if status.hospitalized { 0.5 } else { 1.0 };
        let damage = status.disease_type.severity() * severity_mult * 5.0;
        details.health = (details.health - damage).clamp(0.0, 100.0);

        if status.recovery_remaining <= recovery_speed {
            commands.entity(entity).remove::<DiseaseStatus>();
            details.health = (details.health + 5.0).clamp(0.0, 100.0);
            continue;
        }
        status.recovery_remaining -= recovery_speed;

        if !status.hospitalized && details.health < 20.0 {
            let mortality_severity = status.disease_type.severity();
            let mortality_roll = ((entity.index() as u64).wrapping_mul(7919)
                ^ (day_seed as u64).wrapping_mul(6271))
                % 1000;
            let mortality_chance = mortality_severity * 0.05;
            if (mortality_roll as f32 / 1000.0) < mortality_chance {
                commands.entity(entity).despawn();
                deaths += 1;
                continue;
            }
        }

        match status.disease_type {
            DiseaseType::Flu => flu_count += 1,
            DiseaseType::FoodPoisoning => food_count += 1,
            DiseaseType::Respiratory => resp_count += 1,
        }
    }

    disease_state.flu_count = flu_count;
    disease_state.food_poisoning_count = food_count;
    disease_state.respiratory_count = resp_count;
    disease_state.total_infected = flu_count + food_count + resp_count;
    disease_state.hospital_beds = total_beds;
    disease_state.beds_in_use = beds_used.min(total_beds);
    disease_state.hospital_utilization = if total_beds > 0 {
        (beds_used as f32 / total_beds as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    disease_state.mortality_this_cycle = deaths;
    disease_state.cumulative_mortality += deaths;
}

/// System: update infection rate and mortality rate from current counts.
pub fn update_disease_rates(
    timer: Res<SlowTickTimer>,
    stats: Res<CityStats>,
    mut disease_state: ResMut<DiseaseState>,
) {
    if !timer.should_run() {
        return;
    }
    let population = stats.population;
    disease_state.infection_rate = if population > 0 {
        disease_state.total_infected as f32 / population as f32
    } else {
        0.0
    };
    disease_state.mortality_rate = if population > 0 {
        disease_state.mortality_this_cycle as f32 / population as f32
    } else {
        0.0
    };
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct DiseaseModelPlugin;

impl Plugin for DiseaseModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiseaseState>();

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<DiseaseState>();

        app.add_systems(
            FixedUpdate,
            (
                spread_diseases,
                progress_disease.after(spread_diseases),
                update_disease_rates.after(progress_disease),
            )
                .after(crate::health::update_health_grid)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disease_base_infection_rates() {
        assert!(DiseaseType::Flu.base_infection_rate() > 0.0);
        assert!(DiseaseType::FoodPoisoning.base_infection_rate() > 0.0);
        assert!(DiseaseType::Respiratory.base_infection_rate() > 0.0);
    }

    #[test]
    fn test_disease_severity_ordering() {
        assert!(DiseaseType::Flu.severity() < DiseaseType::FoodPoisoning.severity());
        assert!(DiseaseType::FoodPoisoning.severity() < DiseaseType::Respiratory.severity());
    }

    #[test]
    fn test_density_multiplier_clamped() {
        assert!(density_multiplier(100_000) <= 2.0);
        assert!(density_multiplier(0) >= 0.5);
    }

    #[test]
    fn test_average_pollution_empty() {
        let grid = PollutionGrid {
            levels: vec![],
            width: 0,
            height: 0,
        };
        assert!((average_pollution(&grid)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_beds_for_service_types() {
        assert_eq!(beds_for_service(ServiceType::MedicalClinic), 10);
        assert_eq!(beds_for_service(ServiceType::Hospital), 50);
        assert_eq!(beds_for_service(ServiceType::MedicalCenter), 150);
        assert_eq!(beds_for_service(ServiceType::FireStation), 0);
    }

    #[test]
    fn test_disease_state_default() {
        let state = DiseaseState::default();
        assert_eq!(state.total_infected, 0);
        assert!((state.infection_rate).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sanitation_risk_clamped() {
        let extreme = PollutionGrid {
            levels: vec![255; 100],
            width: 10,
            height: 10,
        };
        assert!(sanitation_risk(&extreme) <= 1.0);
    }
}
