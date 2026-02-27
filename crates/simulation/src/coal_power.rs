//! POWER-002: Coal Power Plant Generator
//!
//! Implements coal power plants as placeable power generator buildings that
//! contribute to `EnergyGrid.total_supply_mwh`. Each coal plant has:
//!
//! - 200 MW capacity, 0.33 capacity factor (dispatchable)
//! - Fuel cost: $30/MWh
//! - Air pollution source: Q=100.0
//! - CO2 emissions: 1.0 tons/MWh
//! - 3×3 building footprint

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::energy_demand::EnergyGrid;
use crate::utilities::{UtilitySource, UtilityType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Maximum generation capacity in MW.
pub const COAL_CAPACITY_MW: f32 = 200.0;

/// Capacity factor (fraction of capacity actually dispatched on average).
pub const COAL_CAPACITY_FACTOR: f32 = 0.33;

/// Fuel cost in dollars per MWh generated.
pub const COAL_FUEL_COST_PER_MWH: f32 = 30.0;

/// CO2 emission rate in tons per MWh.
pub const COAL_CO2_TONS_PER_MWH: f32 = 1.0;

/// Building footprint in grid cells (width, height).
pub const COAL_FOOTPRINT: (usize, usize) = (3, 3);

// =============================================================================
// PowerPlantType enum
// =============================================================================

/// The type of power plant (coal, natural gas, wind turbine, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum PowerPlantType {
    Coal,
    NaturalGas,
    WindTurbine,
    WasteToEnergy,
    HydroDam,
    Geothermal,
    Biomass,
    Oil,
    Nuclear,
}

// =============================================================================
// PowerPlant component
// =============================================================================

/// Component attached to power plant entities. Tracks the plant's type,
/// capacity, current output, and fuel cost.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PowerPlant {
    /// The kind of power plant.
    pub plant_type: PowerPlantType,
    /// Maximum generation capacity in MW.
    pub capacity_mw: f32,
    /// Current instantaneous output in MW (capacity × capacity_factor).
    pub current_output_mw: f32,
    /// Fuel cost per MWh of generation (dollars).
    pub fuel_cost: f32,
    /// Grid x position of the plant.
    pub grid_x: usize,
    /// Grid y position of the plant.
    pub grid_y: usize,
}

impl PowerPlant {
    /// Create a new coal power plant at the given grid position.
    pub fn new_coal(grid_x: usize, grid_y: usize) -> Self {
        Self {
            plant_type: PowerPlantType::Coal,
            capacity_mw: COAL_CAPACITY_MW,
            current_output_mw: COAL_CAPACITY_MW * COAL_CAPACITY_FACTOR,
            fuel_cost: COAL_FUEL_COST_PER_MWH,
            grid_x,
            grid_y,
        }
    }
}

// =============================================================================
// CoalPowerState resource (city-wide coal power stats)
// =============================================================================

/// Aggregated city-wide state for coal power generation.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct CoalPowerState {
    /// Number of active coal plants in the city.
    pub plant_count: u32,
    /// Total generation from all coal plants (MW).
    pub total_output_mw: f32,
    /// Total fuel cost across all coal plants ($/tick cycle).
    pub total_fuel_cost: f32,
    /// Total CO2 emitted this cycle (tons).
    pub total_co2_tons: f32,
}

impl Default for CoalPowerState {
    fn default() -> Self {
        Self {
            plant_count: 0,
            total_output_mw: 0.0,
            total_fuel_cost: 0.0,
            total_co2_tons: 0.0,
        }
    }
}

impl crate::Saveable for CoalPowerState {
    const SAVE_KEY: &'static str = "coal_power";

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
/// `PowerPlant` (coal) that don't already have one.
pub fn attach_coal_power_plants(
    timer: Res<SlowTickTimer>,
    mut commands: Commands,
    sources: Query<(Entity, &UtilitySource), Without<PowerPlant>>,
) {
    if !timer.should_run() {
        return;
    }

    for (entity, source) in &sources {
        if source.utility_type == UtilityType::PowerPlant {
            commands
                .entity(entity)
                .insert(PowerPlant::new_coal(source.grid_x, source.grid_y));
        }
    }
}

/// Aggregates coal power plant output into `EnergyGrid.total_supply_mwh` and
/// updates `CoalPowerState`. Runs every slow tick.
pub fn aggregate_coal_power(
    timer: Res<SlowTickTimer>,
    plants: Query<&PowerPlant>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut coal_state: ResMut<CoalPowerState>,
) {
    if !timer.should_run() {
        return;
    }

    let mut count = 0u32;
    let mut total_output = 0.0f32;
    let mut total_fuel = 0.0f32;
    let mut total_co2 = 0.0f32;

    for plant in &plants {
        if plant.plant_type != PowerPlantType::Coal {
            continue;
        }
        count += 1;
        total_output += plant.current_output_mw;
        // Fuel cost = output_mw * fuel_cost_per_mwh
        total_fuel += plant.current_output_mw * plant.fuel_cost;
        // CO2 = output_mw * emission_rate
        total_co2 += plant.current_output_mw * COAL_CO2_TONS_PER_MWH;
    }

    coal_state.plant_count = count;
    coal_state.total_output_mw = total_output;
    coal_state.total_fuel_cost = total_fuel;
    coal_state.total_co2_tons = total_co2;

    // Add coal generation to the energy grid supply
    energy_grid.total_supply_mwh += total_output;
}

/// Plugin that registers coal power plant resources and systems.
pub struct CoalPowerPlugin;

impl Plugin for CoalPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoalPowerState>().add_systems(
            FixedUpdate,
            (
                attach_coal_power_plants,
                // Writes EnergyGrid (supply) and CoalPowerState; must run after
                // dispatch_energy which allocates load to plants (sets current_output_mw).
                aggregate_coal_power
                    .after(attach_coal_power_plants)
                    .after(crate::wind_pollution::update_pollution_gaussian_plume)
                    .after(crate::energy_dispatch::dispatch_energy),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<CoalPowerState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coal_plant_new_coal() {
        let plant = PowerPlant::new_coal(10, 20);
        assert_eq!(plant.plant_type, PowerPlantType::Coal);
        assert!((plant.capacity_mw - COAL_CAPACITY_MW).abs() < f32::EPSILON);
        assert!(
            (plant.current_output_mw - COAL_CAPACITY_MW * COAL_CAPACITY_FACTOR).abs()
                < f32::EPSILON
        );
        assert!((plant.fuel_cost - COAL_FUEL_COST_PER_MWH).abs() < f32::EPSILON);
        assert_eq!(plant.grid_x, 10);
        assert_eq!(plant.grid_y, 20);
    }

    #[test]
    fn test_coal_power_state_default() {
        let state = CoalPowerState::default();
        assert_eq!(state.plant_count, 0);
        assert!((state.total_output_mw).abs() < f32::EPSILON);
        assert!((state.total_fuel_cost).abs() < f32::EPSILON);
        assert!((state.total_co2_tons).abs() < f32::EPSILON);
    }

    #[test]
    fn test_coal_power_state_save_skip_empty() {
        use crate::Saveable;
        let state = CoalPowerState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should not produce save bytes"
        );
    }

    #[test]
    fn test_coal_power_state_roundtrip() {
        use crate::Saveable;
        let state = CoalPowerState {
            plant_count: 3,
            total_output_mw: 198.0,
            total_fuel_cost: 5940.0,
            total_co2_tons: 198.0,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = CoalPowerState::load_from_bytes(&bytes);
        assert_eq!(loaded.plant_count, 3);
        assert!((loaded.total_output_mw - 198.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_coal_footprint() {
        assert_eq!(COAL_FOOTPRINT, (3, 3));
    }
}
