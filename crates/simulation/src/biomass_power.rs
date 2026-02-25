//! POWER-017: Biomass Power Plant
//!
//! Implements biomass power plants that burn organic waste and agricultural
//! output to generate electricity. Links to waste management and agriculture
//! systems. Each biomass plant has:
//!
//! - 25 MW capacity, 0.80 capacity factor
//! - Fuel cost: $30/MWh (uses waste as fuel)
//! - Construction cost: $40M, build time: 8 game-days
//! - Air pollution: Q=25.0 (moderate, lower than coal)
//! - CO2 emissions: 0.23 tons/MWh (considered carbon-neutral lifecycle)
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
pub const BIOMASS_CAPACITY_MW: f32 = 25.0;

/// Capacity factor (fraction of capacity actually dispatched on average).
pub const BIOMASS_CAPACITY_FACTOR: f32 = 0.80;

/// Fuel cost in dollars per MWh generated.
pub const BIOMASS_FUEL_COST_PER_MWH: f32 = 30.0;

/// CO2 emission rate in tons per MWh (biomass is relatively low-emission).
pub const BIOMASS_CO2_TONS_PER_MWH: f32 = 0.23;

/// Construction cost in dollars.
pub const BIOMASS_CONSTRUCTION_COST: f64 = 40_000_000.0;

/// Build time in game ticks (8 game-days at 100 ticks/day).
pub const BIOMASS_BUILD_TICKS: u32 = 800;

/// Building footprint in grid cells (width, height).
pub const BIOMASS_FOOTPRINT: (usize, usize) = (3, 3);

/// Air pollution emission rate Q (moderate, lower than coal).
pub const BIOMASS_POLLUTION_Q: f32 = 25.0;

// =============================================================================
// PowerPlant constructor for Biomass
// =============================================================================

impl PowerPlant {
    /// Create a new biomass power plant at the given grid position.
    pub fn new_biomass(grid_x: usize, grid_y: usize) -> Self {
        Self {
            plant_type: PowerPlantType::Biomass,
            capacity_mw: BIOMASS_CAPACITY_MW,
            current_output_mw: BIOMASS_CAPACITY_MW * BIOMASS_CAPACITY_FACTOR,
            fuel_cost: BIOMASS_FUEL_COST_PER_MWH,
            grid_x,
            grid_y,
        }
    }
}

// =============================================================================
// BiomassPowerState resource (city-wide biomass power stats)
// =============================================================================

/// Aggregated city-wide state for biomass power generation.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BiomassPowerState {
    /// Number of active biomass plants in the city.
    pub plant_count: u32,
    /// Total generation from all biomass plants (MW).
    pub total_output_mw: f32,
    /// Total fuel cost across all biomass plants ($/tick cycle).
    pub total_fuel_cost: f32,
    /// Total CO2 emitted this cycle (tons).
    pub total_co2_tons: f32,
}

impl Default for BiomassPowerState {
    fn default() -> Self {
        Self {
            plant_count: 0,
            total_output_mw: 0.0,
            total_fuel_cost: 0.0,
            total_co2_tons: 0.0,
        }
    }
}

impl crate::Saveable for BiomassPowerState {
    const SAVE_KEY: &'static str = "biomass_power";

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

/// Aggregates biomass power plant output into `EnergyGrid.total_supply_mwh` and
/// updates `BiomassPowerState`. Runs every slow tick.
pub fn aggregate_biomass_power(
    timer: Res<SlowTickTimer>,
    plants: Query<&PowerPlant>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut biomass_state: ResMut<BiomassPowerState>,
) {
    if !timer.should_run() {
        return;
    }

    let mut count = 0u32;
    let mut total_output = 0.0f32;
    let mut total_fuel = 0.0f32;
    let mut total_co2 = 0.0f32;

    for plant in &plants {
        if plant.plant_type != PowerPlantType::Biomass {
            continue;
        }
        count += 1;
        total_output += plant.current_output_mw;
        // Fuel cost = output_mw * fuel_cost_per_mwh
        total_fuel += plant.current_output_mw * plant.fuel_cost;
        // CO2 = output_mw * emission_rate
        total_co2 += plant.current_output_mw * BIOMASS_CO2_TONS_PER_MWH;
    }

    biomass_state.plant_count = count;
    biomass_state.total_output_mw = total_output;
    biomass_state.total_fuel_cost = total_fuel;
    biomass_state.total_co2_tons = total_co2;

    // Add biomass generation to the energy grid supply
    energy_grid.total_supply_mwh += total_output;
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers biomass power plant resources and systems.
pub struct BiomassPowerPlugin;

impl Plugin for BiomassPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BiomassPowerState>().add_systems(
            FixedUpdate,
            aggregate_biomass_power
                .after(crate::wind_pollution::update_pollution_gaussian_plume)
                .after(crate::energy_dispatch::dispatch_energy)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<BiomassPowerState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biomass_plant_new_biomass() {
        let plant = PowerPlant::new_biomass(10, 20);
        assert_eq!(plant.plant_type, PowerPlantType::Biomass);
        assert!((plant.capacity_mw - BIOMASS_CAPACITY_MW).abs() < f32::EPSILON);
        assert!(
            (plant.current_output_mw - BIOMASS_CAPACITY_MW * BIOMASS_CAPACITY_FACTOR).abs()
                < f32::EPSILON
        );
        assert!((plant.fuel_cost - BIOMASS_FUEL_COST_PER_MWH).abs() < f32::EPSILON);
        assert_eq!(plant.grid_x, 10);
        assert_eq!(plant.grid_y, 20);
    }

    #[test]
    fn test_biomass_power_state_default() {
        let state = BiomassPowerState::default();
        assert_eq!(state.plant_count, 0);
        assert!((state.total_output_mw).abs() < f32::EPSILON);
        assert!((state.total_fuel_cost).abs() < f32::EPSILON);
        assert!((state.total_co2_tons).abs() < f32::EPSILON);
    }

    #[test]
    fn test_biomass_power_state_save_skip_empty() {
        use crate::Saveable;
        let state = BiomassPowerState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should not produce save bytes"
        );
    }

    #[test]
    fn test_biomass_power_state_roundtrip() {
        use crate::Saveable;
        let state = BiomassPowerState {
            plant_count: 2,
            total_output_mw: 40.0,
            total_fuel_cost: 1200.0,
            total_co2_tons: 9.2,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = BiomassPowerState::load_from_bytes(&bytes);
        assert_eq!(loaded.plant_count, 2);
        assert!((loaded.total_output_mw - 40.0).abs() < f32::EPSILON);
        assert!((loaded.total_fuel_cost - 1200.0).abs() < f32::EPSILON);
        assert!((loaded.total_co2_tons - 9.2).abs() < 0.01);
    }

    #[test]
    fn test_biomass_footprint() {
        assert_eq!(BIOMASS_FOOTPRINT, (3, 3));
    }

    #[test]
    fn test_biomass_construction_cost() {
        assert!((BIOMASS_CONSTRUCTION_COST - 40_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_biomass_pollution_q() {
        // Biomass should have lower pollution than coal (100.0) but higher than zero
        assert!(BIOMASS_POLLUTION_Q > 0.0);
        assert!(BIOMASS_POLLUTION_Q < crate::coal_power::COAL_CO2_TONS_PER_MWH * 100.0);
        assert!((BIOMASS_POLLUTION_Q - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_biomass_co2_lower_than_coal() {
        assert!(
            BIOMASS_CO2_TONS_PER_MWH < crate::coal_power::COAL_CO2_TONS_PER_MWH,
            "Biomass CO2 rate ({}) should be lower than coal ({})",
            BIOMASS_CO2_TONS_PER_MWH,
            crate::coal_power::COAL_CO2_TONS_PER_MWH
        );
    }
}
