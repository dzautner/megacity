//! POWER-003: Natural Gas Combined-Cycle Power Plant
//!
//! Implements natural gas combined-cycle power plants as placeable power
//! generator buildings that contribute to `EnergyGrid.total_supply_mwh`.
//! Each gas plant has:
//!
//! - 500 MW capacity, 0.45 capacity factor (dispatchable)
//! - Fuel cost: $40/MWh
//! - Air pollution source: Q=35.0 (65% less than coal)
//! - CO2 emissions: 0.4 tons/MWh
//! - 2×3 building footprint

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
pub const GAS_CAPACITY_MW: f32 = 500.0;

/// Capacity factor (fraction of capacity actually dispatched on average).
pub const GAS_CAPACITY_FACTOR: f32 = 0.45;

/// Fuel cost in dollars per MWh generated.
pub const GAS_FUEL_COST_PER_MWH: f32 = 40.0;

/// CO2 emission rate in tons per MWh.
pub const GAS_CO2_TONS_PER_MWH: f32 = 0.4;

/// Building footprint in grid cells (width, height).
pub const GAS_FOOTPRINT: (usize, usize) = (2, 3);

// =============================================================================
// PowerPlant constructor for NaturalGas
// =============================================================================

impl PowerPlant {
    /// Create a new natural gas combined-cycle power plant at the given grid
    /// position.
    pub fn new_gas(grid_x: usize, grid_y: usize) -> Self {
        Self {
            plant_type: PowerPlantType::NaturalGas,
            capacity_mw: GAS_CAPACITY_MW,
            current_output_mw: GAS_CAPACITY_MW * GAS_CAPACITY_FACTOR,
            fuel_cost: GAS_FUEL_COST_PER_MWH,
            grid_x,
            grid_y,
        }
    }
}

// =============================================================================
// GasPowerState resource (city-wide gas power stats)
// =============================================================================

/// Aggregated city-wide state for natural gas power generation.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct GasPowerState {
    /// Number of active gas plants in the city.
    pub plant_count: u32,
    /// Total generation from all gas plants (MW).
    pub total_output_mw: f32,
    /// Total fuel cost across all gas plants ($/tick cycle).
    pub total_fuel_cost: f32,
    /// Total CO2 emitted this cycle (tons).
    pub total_co2_tons: f32,
}

impl Default for GasPowerState {
    fn default() -> Self {
        Self {
            plant_count: 0,
            total_output_mw: 0.0,
            total_fuel_cost: 0.0,
            total_co2_tons: 0.0,
        }
    }
}

impl crate::Saveable for GasPowerState {
    const SAVE_KEY: &'static str = "gas_power";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.plant_count == 0 {
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
/// `GasPlant` that don't already have one.
pub fn attach_gas_power_plants(
    timer: Res<SlowTickTimer>,
    mut commands: Commands,
    sources: Query<(Entity, &UtilitySource), Without<PowerPlant>>,
) {
    if !timer.should_run() {
        return;
    }

    for (entity, source) in &sources {
        if source.utility_type == UtilityType::GasPlant {
            commands
                .entity(entity)
                .insert(PowerPlant::new_gas(source.grid_x, source.grid_y));
        }
    }
}

/// Aggregates gas power plant output into `EnergyGrid.total_supply_mwh` and
/// updates `GasPowerState`. Runs every slow tick.
pub fn aggregate_gas_power(
    timer: Res<SlowTickTimer>,
    plants: Query<&PowerPlant>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut gas_state: ResMut<GasPowerState>,
) {
    if !timer.should_run() {
        return;
    }

    let mut count = 0u32;
    let mut total_output = 0.0f32;
    let mut total_fuel = 0.0f32;
    let mut total_co2 = 0.0f32;

    for plant in &plants {
        if plant.plant_type != PowerPlantType::NaturalGas {
            continue;
        }
        count += 1;
        total_output += plant.current_output_mw;
        total_fuel += plant.current_output_mw * plant.fuel_cost;
        total_co2 += plant.current_output_mw * GAS_CO2_TONS_PER_MWH;
    }

    gas_state.plant_count = count;
    gas_state.total_output_mw = total_output;
    gas_state.total_fuel_cost = total_fuel;
    gas_state.total_co2_tons = total_co2;

    // Add gas generation to the energy grid supply
    energy_grid.total_supply_mwh += total_output;
}
/// Plugin that registers natural gas power plant resources and systems.
pub struct GasPowerPlugin;

impl Plugin for GasPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GasPowerState>().add_systems(
            FixedUpdate,
            (
                attach_gas_power_plants,
                // Writes EnergyGrid (supply) and GasPowerState; must run after
                // dispatch_energy which allocates load to plants (sets current_output_mw).
                aggregate_gas_power
                    .after(attach_gas_power_plants)
                    .after(crate::wind_pollution::update_pollution_gaussian_plume)
                    .after(crate::energy_dispatch::dispatch_energy),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<GasPowerState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_plant_new_gas() {
        let plant = PowerPlant::new_gas(10, 20);
        assert_eq!(plant.plant_type, PowerPlantType::NaturalGas);
        assert!((plant.capacity_mw - GAS_CAPACITY_MW).abs() < f32::EPSILON);
        assert!(
            (plant.current_output_mw - GAS_CAPACITY_MW * GAS_CAPACITY_FACTOR).abs()
                < f32::EPSILON
        );
        assert!((plant.fuel_cost - GAS_FUEL_COST_PER_MWH).abs() < f32::EPSILON);
        assert_eq!(plant.grid_x, 10);
        assert_eq!(plant.grid_y, 20);
    }

    #[test]
    fn test_gas_power_state_default() {
        let state = GasPowerState::default();
        assert_eq!(state.plant_count, 0);
        assert!((state.total_output_mw).abs() < f32::EPSILON);
        assert!((state.total_fuel_cost).abs() < f32::EPSILON);
        assert!((state.total_co2_tons).abs() < f32::EPSILON);
    }

    #[test]
    fn test_gas_power_state_save_skip_empty() {
        use crate::Saveable;
        let state = GasPowerState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should not produce save bytes"
        );
    }

    #[test]
    fn test_gas_power_state_roundtrip() {
        use crate::Saveable;
        let state = GasPowerState {
            plant_count: 2,
            total_output_mw: 450.0,
            total_fuel_cost: 18000.0,
            total_co2_tons: 180.0,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = GasPowerState::load_from_bytes(&bytes);
        assert_eq!(loaded.plant_count, 2);
        assert!((loaded.total_output_mw - 450.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_gas_footprint() {
        assert_eq!(GAS_FOOTPRINT, (2, 3));
    }
}
