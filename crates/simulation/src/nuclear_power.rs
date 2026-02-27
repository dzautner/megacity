//! POWER-004: Nuclear Power Plant
//!
//! Implements nuclear power plants as high-capacity, zero-emission (air)
//! baseload generators. Nuclear provides massive reliable power but has very
//! high construction cost, long build time, and produces radioactive waste
//! requiring a hazardous waste facility.
//!
//! Key specs:
//! - 1000 MW capacity, 0.90 capacity factor (highest among all generators)
//! - Fuel cost: $15/MWh (low marginal cost, dispatches after renewables)
//! - Construction cost: $500M, build time: 30 game-days
//! - No air pollution emissions (zero Q source)
//! - CO2 emissions: 0.0 tons/MWh (zero carbon)
//! - Produces radioactive waste (tracked per cycle)
//! - 4×4 building footprint

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::utilities::{UtilitySource, UtilityType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Maximum generation capacity in MW.
pub const NUCLEAR_CAPACITY_MW: f32 = 1000.0;

/// Capacity factor (fraction of capacity actually dispatched on average).
/// Nuclear has the highest capacity factor of all generators (baseload).
pub const NUCLEAR_CAPACITY_FACTOR: f32 = 0.90;

/// Fuel cost in dollars per MWh generated (uranium fuel is cheap per MWh).
pub const NUCLEAR_FUEL_COST_PER_MWH: f32 = 15.0;

/// CO2 emission rate in tons per MWh (zero — nuclear is carbon-free).
pub const NUCLEAR_CO2_TONS_PER_MWH: f32 = 0.0;

/// Air pollution emission factor Q (zero — no combustion).
pub const NUCLEAR_AIR_POLLUTION_Q: f32 = 0.0;

/// Radioactive waste produced per MWh of generation (kg).
/// Typical PWR produces ~20 tonnes of spent fuel per GW-year.
/// ~20,000 kg / (1000 MW * 8760 h * 0.9) = ~0.00254 kg/MWh
/// We round up slightly for gameplay purposes.
pub const NUCLEAR_WASTE_KG_PER_MWH: f32 = 0.003;

/// Construction cost in dollars ($500 million).
pub const NUCLEAR_CONSTRUCTION_COST: f64 = 500_000_000.0;

/// Build time in game ticks (30 game-days at 100 ticks/day = 3000 ticks).
pub const NUCLEAR_BUILD_TICKS: u32 = 3000;

/// Building footprint in grid cells (width, height).
pub const NUCLEAR_FOOTPRINT: (usize, usize) = (4, 4);

// =============================================================================
// PowerPlant constructor for Nuclear
// =============================================================================

impl PowerPlant {
    /// Create a new nuclear power plant at the given grid position.
    pub fn new_nuclear(grid_x: usize, grid_y: usize) -> Self {
        Self {
            plant_type: PowerPlantType::Nuclear,
            capacity_mw: NUCLEAR_CAPACITY_MW,
            current_output_mw: NUCLEAR_CAPACITY_MW * NUCLEAR_CAPACITY_FACTOR,
            fuel_cost: NUCLEAR_FUEL_COST_PER_MWH,
            grid_x,
            grid_y,
        }
    }
}

// =============================================================================
// NuclearPowerState resource (city-wide nuclear power stats)
// =============================================================================

/// Aggregated city-wide state for nuclear power generation.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct NuclearPowerState {
    /// Number of active nuclear plants in the city.
    pub plant_count: u32,
    /// Total generation from all nuclear plants (MW).
    pub total_output_mw: f32,
    /// Total fuel cost across all nuclear plants ($/tick cycle).
    pub total_fuel_cost: f32,
    /// Total radioactive waste produced this cycle (kg).
    pub total_radioactive_waste_kg: f32,
    /// Cumulative radioactive waste produced since city start (kg).
    pub cumulative_radioactive_waste_kg: f32,
}

impl Default for NuclearPowerState {
    fn default() -> Self {
        Self {
            plant_count: 0,
            total_output_mw: 0.0,
            total_fuel_cost: 0.0,
            total_radioactive_waste_kg: 0.0,
            cumulative_radioactive_waste_kg: 0.0,
        }
    }
}

impl crate::Saveable for NuclearPowerState {
    const SAVE_KEY: &'static str = "nuclear_power";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.plant_count == 0 && self.cumulative_radioactive_waste_kg == 0.0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Attaches `PowerPlant` components to `UtilitySource` entities of type
/// `NuclearPlant` that don't already have one.
pub fn attach_nuclear_power_plants(
    timer: Res<SlowTickTimer>,
    mut commands: Commands,
    sources: Query<(Entity, &UtilitySource), Without<PowerPlant>>,
) {
    if !timer.should_run() {
        return;
    }

    for (entity, source) in &sources {
        if source.utility_type == UtilityType::NuclearPlant {
            commands
                .entity(entity)
                .insert(PowerPlant::new_nuclear(source.grid_x, source.grid_y));
        }
    }
}

/// Aggregates nuclear power plant output into `EnergyGrid.total_supply_mwh`
/// and updates `NuclearPowerState`. Runs every slow tick.
///
/// Nuclear plants produce zero air pollution and zero CO2, but generate
/// radioactive waste proportional to their energy output.
pub fn aggregate_nuclear_power(
    timer: Res<SlowTickTimer>,
    plants: Query<&PowerPlant>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut nuclear_state: ResMut<NuclearPowerState>,
) {
    if !timer.should_run() {
        return;
    }

    let mut count = 0u32;
    let mut total_output = 0.0f32;
    let mut total_fuel = 0.0f32;
    let mut total_waste = 0.0f32;

    for plant in &plants {
        if plant.plant_type != PowerPlantType::Nuclear {
            continue;
        }
        count += 1;
        total_output += plant.current_output_mw;
        // Fuel cost = output_mw * fuel_cost_per_mwh
        total_fuel += plant.current_output_mw * plant.fuel_cost;
        // Radioactive waste = output_mw * waste_rate
        total_waste += plant.current_output_mw * NUCLEAR_WASTE_KG_PER_MWH;
    }

    nuclear_state.plant_count = count;
    nuclear_state.total_output_mw = total_output;
    nuclear_state.total_fuel_cost = total_fuel;
    nuclear_state.total_radioactive_waste_kg = total_waste;
    nuclear_state.cumulative_radioactive_waste_kg += total_waste;

    // Add nuclear generation to the energy grid supply
    energy_grid.total_supply_mwh += total_output;
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers nuclear power plant resources and systems.
pub struct NuclearPowerPlugin;

impl Plugin for NuclearPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NuclearPowerState>().add_systems(
            FixedUpdate,
            (
                attach_nuclear_power_plants,
                aggregate_nuclear_power
                    .after(attach_nuclear_power_plants)
                    .after(crate::wind_pollution::update_pollution_gaussian_plume)
                    .after(crate::energy_dispatch::dispatch_energy),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<NuclearPowerState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nuclear_plant_new() {
        let plant = PowerPlant::new_nuclear(10, 20);
        assert_eq!(plant.plant_type, PowerPlantType::Nuclear);
        assert!((plant.capacity_mw - NUCLEAR_CAPACITY_MW).abs() < f32::EPSILON);
        assert!(
            (plant.current_output_mw - NUCLEAR_CAPACITY_MW * NUCLEAR_CAPACITY_FACTOR).abs()
                < f32::EPSILON
        );
        assert!((plant.fuel_cost - NUCLEAR_FUEL_COST_PER_MWH).abs() < f32::EPSILON);
        assert_eq!(plant.grid_x, 10);
        assert_eq!(plant.grid_y, 20);
    }

    #[test]
    fn test_nuclear_power_state_default() {
        let state = NuclearPowerState::default();
        assert_eq!(state.plant_count, 0);
        assert!(state.total_output_mw.abs() < f32::EPSILON);
        assert!(state.total_fuel_cost.abs() < f32::EPSILON);
        assert!(state.total_radioactive_waste_kg.abs() < f32::EPSILON);
        assert!(state.cumulative_radioactive_waste_kg.abs() < f32::EPSILON);
    }

    #[test]
    fn test_nuclear_power_state_save_skip_empty() {
        use crate::Saveable;
        let state = NuclearPowerState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should not produce save bytes"
        );
    }

    #[test]
    fn test_nuclear_power_state_roundtrip() {
        use crate::Saveable;
        let state = NuclearPowerState {
            plant_count: 2,
            total_output_mw: 1800.0,
            total_fuel_cost: 27000.0,
            total_radioactive_waste_kg: 5.4,
            cumulative_radioactive_waste_kg: 100.0,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = NuclearPowerState::load_from_bytes(&bytes);
        assert_eq!(loaded.plant_count, 2);
        assert!((loaded.total_output_mw - 1800.0).abs() < f32::EPSILON);
        assert!((loaded.total_fuel_cost - 27000.0).abs() < f32::EPSILON);
        assert!((loaded.total_radioactive_waste_kg - 5.4).abs() < f32::EPSILON);
        assert!((loaded.cumulative_radioactive_waste_kg - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_nuclear_zero_co2() {
        assert!(
            NUCLEAR_CO2_TONS_PER_MWH.abs() < f32::EPSILON,
            "Nuclear should have zero CO2 emissions"
        );
    }

    #[test]
    fn test_nuclear_zero_air_pollution() {
        assert!(
            NUCLEAR_AIR_POLLUTION_Q.abs() < f32::EPSILON,
            "Nuclear should have zero air pollution Q"
        );
    }

    #[test]
    fn test_nuclear_highest_capacity_factor() {
        assert!(
            NUCLEAR_CAPACITY_FACTOR > crate::coal_power::COAL_CAPACITY_FACTOR,
            "Nuclear CF ({}) should be higher than coal ({})",
            NUCLEAR_CAPACITY_FACTOR,
            crate::coal_power::COAL_CAPACITY_FACTOR,
        );
        assert!(
            NUCLEAR_CAPACITY_FACTOR > crate::gas_power::GAS_CAPACITY_FACTOR,
            "Nuclear CF ({}) should be higher than gas ({})",
            NUCLEAR_CAPACITY_FACTOR,
            crate::gas_power::GAS_CAPACITY_FACTOR,
        );
    }

    #[test]
    fn test_nuclear_waste_production() {
        let output_mw = NUCLEAR_CAPACITY_MW * NUCLEAR_CAPACITY_FACTOR;
        let waste_kg = output_mw * NUCLEAR_WASTE_KG_PER_MWH;
        assert!(
            waste_kg > 0.0,
            "Nuclear should produce radioactive waste, got {}",
            waste_kg
        );
        // With 900 MW output and 0.003 kg/MWh, expect ~2.7 kg/cycle
        assert!(
            (waste_kg - 2.7).abs() < 0.1,
            "Expected ~2.7 kg/cycle, got {}",
            waste_kg
        );
    }

    #[test]
    fn test_nuclear_footprint() {
        assert_eq!(NUCLEAR_FOOTPRINT, (4, 4));
    }

    #[test]
    fn test_nuclear_construction_cost() {
        assert!(
            (NUCLEAR_CONSTRUCTION_COST - 500_000_000.0).abs() < f64::EPSILON,
            "Construction cost should be $500M"
        );
    }

    #[test]
    fn test_nuclear_fuel_cost_lower_than_coal_and_gas() {
        assert!(
            NUCLEAR_FUEL_COST_PER_MWH < crate::coal_power::COAL_FUEL_COST_PER_MWH,
            "Nuclear fuel cost ({}) should be lower than coal ({})",
            NUCLEAR_FUEL_COST_PER_MWH,
            crate::coal_power::COAL_FUEL_COST_PER_MWH,
        );
        assert!(
            NUCLEAR_FUEL_COST_PER_MWH < crate::gas_power::GAS_FUEL_COST_PER_MWH,
            "Nuclear fuel cost ({}) should be lower than gas ({})",
            NUCLEAR_FUEL_COST_PER_MWH,
            crate::gas_power::GAS_FUEL_COST_PER_MWH,
        );
    }
}
