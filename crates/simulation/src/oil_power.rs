//! POWER-015: Oil-Fired Power Plant
//!
//! Implements oil-fired power plants as a dispatchable but expensive and dirty
//! power source. Each oil plant has:
//!
//! - 100 MW capacity (dispatchable)
//! - Fuel cost: $70/MWh (expensive)
//! - Construction cost: $80M, build time: 5 game-days
//! - Air pollution: Q=75.0 (high)
//! - CO2 emissions: 0.75 tons/MWh
//! - 3×3 building footprint

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Maximum generation capacity in MW.
pub const OIL_CAPACITY_MW: f32 = 100.0;

/// Capacity factor (dispatchable, high availability).
pub const OIL_CAPACITY_FACTOR: f32 = 0.87;

/// Fuel cost in dollars per MWh generated (expensive).
pub const OIL_FUEL_COST_PER_MWH: f32 = 70.0;

/// CO2 emission rate in tons per MWh.
pub const OIL_CO2_TONS_PER_MWH: f32 = 0.75;

/// Construction cost in dollars.
pub const OIL_CONSTRUCTION_COST: f64 = 80_000_000.0;

/// Build time in game ticks (5 game-days at 100 ticks/day).
pub const OIL_BUILD_TICKS: u32 = 500;

/// Building footprint in grid cells (width, height).
pub const OIL_FOOTPRINT: (usize, usize) = (3, 3);

/// Air pollution emission rate Q.
pub const OIL_POLLUTION_Q: f32 = 75.0;

// =============================================================================
// PowerPlant constructor for Oil
// =============================================================================

impl PowerPlant {
    /// Create a new oil-fired power plant at the given grid position.
    pub fn new_oil(grid_x: usize, grid_y: usize) -> Self {
        Self {
            plant_type: PowerPlantType::Oil,
            capacity_mw: OIL_CAPACITY_MW,
            current_output_mw: OIL_CAPACITY_MW * OIL_CAPACITY_FACTOR,
            fuel_cost: OIL_FUEL_COST_PER_MWH,
            grid_x,
            grid_y,
        }
    }
}

// =============================================================================
// OilPowerState resource (city-wide oil power stats)
// =============================================================================

/// Aggregated city-wide state for oil-fired power generation.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct OilPowerState {
    /// Number of active oil plants in the city.
    pub plant_count: u32,
    /// Total generation from all oil plants (MW).
    pub total_output_mw: f32,
    /// Total fuel cost across all oil plants ($/tick cycle).
    pub total_fuel_cost: f32,
    /// Total CO2 emitted this cycle (tons).
    pub total_co2_tons: f32,
}

impl Default for OilPowerState {
    fn default() -> Self {
        Self {
            plant_count: 0,
            total_output_mw: 0.0,
            total_fuel_cost: 0.0,
            total_co2_tons: 0.0,
        }
    }
}

impl crate::Saveable for OilPowerState {
    const SAVE_KEY: &'static str = "oil_power";

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

/// Aggregates oil power plant output into `EnergyGrid.total_supply_mwh` and
/// updates `OilPowerState`. Runs every slow tick.
pub fn aggregate_oil_power(
    timer: Res<SlowTickTimer>,
    plants: Query<&PowerPlant>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut oil_state: ResMut<OilPowerState>,
) {
    if !timer.should_run() {
        return;
    }

    let mut count = 0u32;
    let mut total_output = 0.0f32;
    let mut total_fuel = 0.0f32;
    let mut total_co2 = 0.0f32;

    for plant in &plants {
        if plant.plant_type != PowerPlantType::Oil {
            continue;
        }
        count += 1;
        total_output += plant.current_output_mw;
        total_fuel += plant.current_output_mw * plant.fuel_cost;
        total_co2 += plant.current_output_mw * OIL_CO2_TONS_PER_MWH;
    }

    oil_state.plant_count = count;
    oil_state.total_output_mw = total_output;
    oil_state.total_fuel_cost = total_fuel;
    oil_state.total_co2_tons = total_co2;

    // Add oil generation to the energy grid supply
    energy_grid.total_supply_mwh += total_output;
}

/// Plugin that registers oil-fired power plant resources and systems.
pub struct OilPowerPlugin;

impl Plugin for OilPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OilPowerState>().add_systems(
            FixedUpdate,
            aggregate_oil_power
                .after(crate::wind_pollution::update_pollution_gaussian_plume)
                .after(crate::energy_dispatch::dispatch_energy)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<OilPowerState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oil_plant_new_oil() {
        let plant = PowerPlant::new_oil(10, 20);
        assert_eq!(plant.plant_type, PowerPlantType::Oil);
        assert!((plant.capacity_mw - OIL_CAPACITY_MW).abs() < f32::EPSILON);
        assert!(
            (plant.current_output_mw - OIL_CAPACITY_MW * OIL_CAPACITY_FACTOR).abs()
                < f32::EPSILON
        );
        assert!((plant.fuel_cost - OIL_FUEL_COST_PER_MWH).abs() < f32::EPSILON);
        assert_eq!(plant.grid_x, 10);
        assert_eq!(plant.grid_y, 20);
    }

    #[test]
    fn test_oil_power_state_default() {
        let state = OilPowerState::default();
        assert_eq!(state.plant_count, 0);
        assert!((state.total_output_mw).abs() < f32::EPSILON);
        assert!((state.total_fuel_cost).abs() < f32::EPSILON);
        assert!((state.total_co2_tons).abs() < f32::EPSILON);
    }

    #[test]
    fn test_oil_power_state_save_skip_empty() {
        use crate::Saveable;
        let state = OilPowerState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should not produce save bytes"
        );
    }

    #[test]
    fn test_oil_power_state_roundtrip() {
        use crate::Saveable;
        let state = OilPowerState {
            plant_count: 2,
            total_output_mw: 174.0,
            total_fuel_cost: 12180.0,
            total_co2_tons: 130.5,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = OilPowerState::load_from_bytes(&bytes);
        assert_eq!(loaded.plant_count, 2);
        assert!((loaded.total_output_mw - 174.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_oil_footprint() {
        assert_eq!(OIL_FOOTPRINT, (3, 3));
    }

    #[test]
    fn test_oil_fuel_cost_higher_than_gas_and_coal() {
        assert!(
            OIL_FUEL_COST_PER_MWH > crate::gas_power::GAS_FUEL_COST_PER_MWH,
            "Oil fuel cost should be higher than gas"
        );
        assert!(
            OIL_FUEL_COST_PER_MWH > crate::coal_power::COAL_FUEL_COST_PER_MWH,
            "Oil fuel cost should be higher than coal"
        );
    }
}
