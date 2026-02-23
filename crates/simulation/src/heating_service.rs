//! SVC-010: Heating Service and Weather Integration
//!
//! Bridges the heating infrastructure (HeatingGrid, HeatingPlant) with the
//! weather system, energy grid, and citizen health/happiness. Key features:
//!
//! - Automatically attaches `HeatingPlant` components to service buildings
//!   of type `DistrictHeatingPlant` or `GeothermalPlant`.
//! - Tracks energy consumed by heating operations (both district and individual).
//! - Individual (per-building) heating for buildings without district coverage:
//!   higher cost, lower efficiency.
//! - Health risk penalties for citizens in unheated buildings during cold weather.
//! - Aggregate `HeatingServiceState` resource with Saveable persistence.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::citizen::{CitizenDetails, HomeLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::ZoneType;
use crate::heating::{heating_demand, HeatingGrid, HeatingPlant, HeatingPlantType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::weather::Weather;
use crate::Saveable;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Temperature below which buildings need heating (Celsius).
pub const COMFORT_THRESHOLD_C: f32 = 10.0;

/// Health penalty per slow tick for citizens in unheated buildings during cold weather.
/// Scaled by heating demand intensity.
pub const COLD_HEALTH_PENALTY_PER_TICK: f32 = 0.15;

/// Individual heating efficiency (vs. district heating which is 0.80+).
pub const INDIVIDUAL_HEATING_EFFICIENCY: f32 = 0.50;

/// Individual heating cost per building per unit of demand (monthly equivalent).
pub const INDIVIDUAL_HEATING_COST_PER_BUILDING: f64 = 0.12;

/// Energy consumed per unit of heating demand per building (MW).
/// District plants are more efficient; individual heating uses this higher rate.
pub const INDIVIDUAL_HEATING_ENERGY_MW: f32 = 0.002;

/// Energy consumed per unit of heating demand for district-heated buildings (MW).
pub const DISTRICT_HEATING_ENERGY_MW: f32 = 0.001;

// =============================================================================
// HeatingServiceState resource (Saveable)
// =============================================================================

/// City-wide heating service statistics and tracking.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct HeatingServiceState {
    /// Number of buildings using individual (non-district) heating.
    pub individual_heating_count: u32,
    /// Number of buildings covered by district heating plants.
    pub district_heating_count: u32,
    /// Number of buildings with no heating at all (cold weather only).
    pub unheated_count: u32,
    /// Total energy consumed by heating this cycle (MW).
    pub heating_energy_mw: f32,
    /// Total monthly cost of individual heating.
    pub individual_heating_cost: f64,
    /// Current heating demand factor (0.0 = no demand, 1.0+ = high demand).
    pub current_demand: f32,
    /// Number of citizens suffering cold-related health effects.
    pub cold_affected_citizens: u32,
}

impl Default for HeatingServiceState {
    fn default() -> Self {
        Self {
            individual_heating_count: 0,
            district_heating_count: 0,
            unheated_count: 0,
            heating_energy_mw: 0.0,
            individual_heating_cost: 0.0,
            current_demand: 0.0,
            cold_affected_citizens: 0,
        }
    }
}

impl Saveable for HeatingServiceState {
    const SAVE_KEY: &'static str = "heating_service";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// System: Attach HeatingPlant to service buildings
// =============================================================================

/// Automatically attaches `HeatingPlant` components to service buildings
/// of type `DistrictHeatingPlant` or `GeothermalPlant` that lack them.
pub fn attach_heating_plants(
    mut commands: Commands,
    services: Query<(Entity, &ServiceBuilding), Without<HeatingPlant>>,
) {
    for (entity, service) in &services {
        let plant_type = match service.service_type {
            ServiceType::DistrictHeatingPlant => HeatingPlantType::DistrictHeating,
            ServiceType::GeothermalPlant => HeatingPlantType::Geothermal,
            _ => continue,
        };
        commands.entity(entity).insert(HeatingPlant {
            plant_type,
            grid_x: service.grid_x,
            grid_y: service.grid_y,
            capacity: plant_type.capacity(),
            efficiency: plant_type.efficiency(),
        });
    }
}

// =============================================================================
// System: Update heating service state
// =============================================================================

/// Updates heating service statistics: tracks which buildings have district
/// coverage vs. individual heating, computes energy consumption, and
/// calculates costs.
pub fn update_heating_service(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    heating_grid: Res<HeatingGrid>,
    buildings: Query<&Building>,
    mut state: ResMut<HeatingServiceState>,
) {
    if !timer.should_run() {
        return;
    }

    let demand = heating_demand(&weather);
    state.current_demand = demand;

    // No heating needed in warm weather
    if demand <= 0.0 {
        state.individual_heating_count = 0;
        state.district_heating_count = 0;
        state.unheated_count = 0;
        state.heating_energy_mw = 0.0;
        state.individual_heating_cost = 0.0;
        return;
    }

    let mut district_count = 0u32;
    let mut individual_count = 0u32;
    let mut unheated = 0u32;
    let mut total_energy = 0.0f32;
    let mut total_individual_cost = 0.0f64;

    for building in &buildings {
        // Only residential and commercial buildings need heating
        if !is_heatable_zone(building.zone_type) {
            continue;
        }

        let gx = building.grid_x.min(GRID_WIDTH - 1);
        let gy = building.grid_y.min(GRID_HEIGHT - 1);

        if heating_grid.is_heated(gx, gy) {
            // Building has district heating coverage
            district_count += 1;
            total_energy += DISTRICT_HEATING_ENERGY_MW * demand;
        } else if building.occupants > 0 {
            // Occupied building without district coverage uses individual heating
            individual_count += 1;
            total_energy += INDIVIDUAL_HEATING_ENERGY_MW * demand;
            total_individual_cost +=
                INDIVIDUAL_HEATING_COST_PER_BUILDING * demand as f64;
        } else {
            // Vacant building without heating
            unheated += 1;
        }
    }

    state.district_heating_count = district_count;
    state.individual_heating_count = individual_count;
    state.unheated_count = unheated;
    state.heating_energy_mw = total_energy;
    state.individual_heating_cost = total_individual_cost;
}

// =============================================================================
// System: Cold weather health effects
// =============================================================================

/// Applies health penalties to citizens living in unheated buildings
/// during cold weather. Buildings with no heating coverage and no occupancy
/// to pay for individual heating cause health degradation.
///
/// The penalty scales with heating demand (colder = worse).
pub fn apply_cold_health_effects(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    heating_grid: Res<HeatingGrid>,
    buildings: Query<&Building>,
    mut citizens: Query<(&HomeLocation, &mut CitizenDetails)>,
    mut state: ResMut<HeatingServiceState>,
) {
    if !timer.should_run() {
        return;
    }

    let demand = heating_demand(&weather);
    if demand <= 0.0 {
        state.cold_affected_citizens = 0;
        return;
    }

    // Build a quick lookup: for each building cell, is it heated or occupied?
    // A building is "warm" if it has district heating coverage OR has occupants
    // (who pay for individual heating).
    // An unoccupied building without district heating has NO heating.
    let mut cold_cells: Vec<bool> = vec![false; GRID_WIDTH * GRID_HEIGHT];
    for building in &buildings {
        if !is_heatable_zone(building.zone_type) {
            continue;
        }
        let gx = building.grid_x.min(GRID_WIDTH - 1);
        let gy = building.grid_y.min(GRID_HEIGHT - 1);
        let idx = gy * GRID_WIDTH + gx;
        // Cell is cold if building has no district coverage AND is vacant
        if !heating_grid.is_heated(gx, gy) && building.occupants == 0 {
            cold_cells[idx] = true;
        }
    }

    let mut affected = 0u32;
    let penalty = COLD_HEALTH_PENALTY_PER_TICK * demand;

    for (home, mut details) in &mut citizens {
        let gx = home.grid_x.min(GRID_WIDTH - 1);
        let gy = home.grid_y.min(GRID_HEIGHT - 1);
        let idx = gy * GRID_WIDTH + gx;

        // Citizens in cold cells suffer health damage
        if cold_cells[idx] {
            details.health = (details.health - penalty).max(0.0);
            affected += 1;
        }
    }

    state.cold_affected_citizens = affected;
}

// =============================================================================
// Helpers
// =============================================================================

/// Returns true if the zone type represents a building that needs heating.
fn is_heatable_zone(zone: ZoneType) -> bool {
    matches!(
        zone,
        ZoneType::ResidentialLow
            | ZoneType::ResidentialMedium
            | ZoneType::ResidentialHigh
            | ZoneType::CommercialLow
            | ZoneType::CommercialHigh
            | ZoneType::Office
            | ZoneType::MixedUse
    )
}

// =============================================================================
// Plugin
// =============================================================================

pub struct HeatingServicePlugin;

impl Plugin for HeatingServicePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeatingServiceState>().add_systems(
            FixedUpdate,
            (
                attach_heating_plants,
                update_heating_service
                    .after(crate::heating::update_heating)
                    .after(attach_heating_plants),
                apply_cold_health_effects
                    .after(update_heating_service),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<HeatingServiceState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_heatable_zone() {
        assert!(is_heatable_zone(ZoneType::ResidentialLow));
        assert!(is_heatable_zone(ZoneType::ResidentialMedium));
        assert!(is_heatable_zone(ZoneType::ResidentialHigh));
        assert!(is_heatable_zone(ZoneType::CommercialLow));
        assert!(is_heatable_zone(ZoneType::CommercialHigh));
        assert!(is_heatable_zone(ZoneType::Office));
        assert!(is_heatable_zone(ZoneType::MixedUse));
        assert!(!is_heatable_zone(ZoneType::Industrial));
        assert!(!is_heatable_zone(ZoneType::None));
    }

    #[test]
    fn test_heating_service_state_default() {
        let state = HeatingServiceState::default();
        assert_eq!(state.individual_heating_count, 0);
        assert_eq!(state.district_heating_count, 0);
        assert_eq!(state.unheated_count, 0);
        assert!((state.heating_energy_mw).abs() < f32::EPSILON);
        assert!((state.individual_heating_cost).abs() < f64::EPSILON);
        assert!((state.current_demand).abs() < f32::EPSILON);
        assert_eq!(state.cold_affected_citizens, 0);
    }

    #[test]
    fn test_comfort_threshold() {
        assert!((COMFORT_THRESHOLD_C - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_individual_heating_less_efficient() {
        assert!(
            INDIVIDUAL_HEATING_EFFICIENCY < crate::heating::HeatingPlantType::DistrictHeating.efficiency(),
            "Individual heating should be less efficient than district heating"
        );
        assert!(
            INDIVIDUAL_HEATING_COST_PER_BUILDING > crate::heating::HeatingPlantType::DistrictHeating.cost_per_unit(),
            "Individual heating should cost more than district heating"
        );
    }

    #[test]
    fn test_district_uses_less_energy_than_individual() {
        assert!(
            DISTRICT_HEATING_ENERGY_MW < INDIVIDUAL_HEATING_ENERGY_MW,
            "District heating should use less energy per building"
        );
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(HeatingServiceState::SAVE_KEY, "heating_service");
    }

    #[test]
    fn test_saveable_roundtrip() {
        let state = HeatingServiceState {
            individual_heating_count: 42,
            district_heating_count: 100,
            unheated_count: 5,
            heating_energy_mw: 12.5,
            individual_heating_cost: 1234.56,
            current_demand: 0.75,
            cold_affected_citizens: 10,
        };
        let bytes = state.save_to_bytes().unwrap();
        let restored = HeatingServiceState::load_from_bytes(&bytes);
        assert_eq!(restored.individual_heating_count, 42);
        assert_eq!(restored.district_heating_count, 100);
        assert_eq!(restored.unheated_count, 5);
        assert!((restored.heating_energy_mw - 12.5).abs() < f32::EPSILON);
        assert!((restored.individual_heating_cost - 1234.56).abs() < f64::EPSILON);
        assert!((restored.current_demand - 0.75).abs() < f32::EPSILON);
        assert_eq!(restored.cold_affected_citizens, 10);
    }
}
