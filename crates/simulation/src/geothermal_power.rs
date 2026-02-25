//! POWER-013: Geothermal Power Plant
//!
//! Implements geothermal power plants as renewable, dispatchable baseload
//! generators. Geothermal energy extracts heat from underground reservoirs,
//! producing constant output regardless of weather or time of day.
//!
//! Key specs:
//! - 30 MW capacity, 0.90 capacity factor
//! - Fuel cost: $0/MWh (renewable)
//! - Construction cost: $120M, build time: 15 game-days
//! - No air pollution emissions (Q=0)
//! - Baseload-only (constant output)
//! - 3×3 building footprint

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

/// Maximum generation capacity per geothermal plant (MW).
pub const GEOTHERMAL_CAPACITY_MW: f32 = 30.0;

/// Capacity factor — fraction of capacity actually dispatched on average.
/// Geothermal is highly reliable with 90% uptime.
pub const GEOTHERMAL_CAPACITY_FACTOR: f32 = 0.90;

/// Fuel cost per MWh (zero — renewable heat source).
pub const GEOTHERMAL_FUEL_COST_PER_MWH: f32 = 0.0;

/// Building footprint in grid cells (width, height).
pub const GEOTHERMAL_FOOTPRINT: (usize, usize) = (3, 3);

// =============================================================================
// PowerPlant constructor for Geothermal
// =============================================================================

impl PowerPlant {
    /// Create a new geothermal power plant at the given grid position.
    pub fn new_geothermal(grid_x: usize, grid_y: usize) -> Self {
        Self {
            plant_type: PowerPlantType::Geothermal,
            capacity_mw: GEOTHERMAL_CAPACITY_MW,
            current_output_mw: GEOTHERMAL_CAPACITY_MW * GEOTHERMAL_CAPACITY_FACTOR,
            fuel_cost: GEOTHERMAL_FUEL_COST_PER_MWH,
            grid_x,
            grid_y,
        }
    }
}

// =============================================================================
// GeothermalPowerState resource (city-wide geothermal stats)
// =============================================================================

/// Aggregated city-wide state for geothermal power generation.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct GeothermalPowerState {
    /// Number of active geothermal plants in the city.
    pub plant_count: u32,
    /// Total generation from all geothermal plants (MW).
    pub total_output_mw: f32,
}

impl crate::Saveable for GeothermalPowerState {
    const SAVE_KEY: &'static str = "geothermal_power";

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
/// `Geothermal` that don't already have one.
pub fn attach_geothermal_power_plants(
    timer: Res<SlowTickTimer>,
    mut commands: Commands,
    sources: Query<(Entity, &UtilitySource), Without<PowerPlant>>,
) {
    if !timer.should_run() {
        return;
    }

    for (entity, source) in &sources {
        if source.utility_type == UtilityType::Geothermal {
            commands
                .entity(entity)
                .insert(PowerPlant::new_geothermal(source.grid_x, source.grid_y));
        }
    }
}

/// Aggregates geothermal power plant output into `EnergyGrid.total_supply_mwh`
/// and updates `GeothermalPowerState`. Runs every slow tick.
///
/// Geothermal plants produce constant baseload power — output is always
/// `capacity_mw * capacity_factor`, unaffected by weather or time of day.
pub fn aggregate_geothermal_power(
    timer: Res<SlowTickTimer>,
    plants: Query<&PowerPlant>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut geo_state: ResMut<GeothermalPowerState>,
) {
    if !timer.should_run() {
        return;
    }

    let mut count = 0u32;
    let mut total_output = 0.0f32;

    for plant in &plants {
        if plant.plant_type != PowerPlantType::Geothermal {
            continue;
        }
        count += 1;
        total_output += plant.current_output_mw;
    }

    geo_state.plant_count = count;
    geo_state.total_output_mw = total_output;

    // Add geothermal generation to the energy grid supply
    energy_grid.total_supply_mwh += total_output;
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers geothermal power plant resources and systems.
pub struct GeothermalPowerPlugin;

impl Plugin for GeothermalPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GeothermalPowerState>().add_systems(
            FixedUpdate,
            (
                attach_geothermal_power_plants,
                aggregate_geothermal_power
                    .after(attach_geothermal_power_plants)
                    .after(crate::energy_dispatch::dispatch_energy),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<GeothermalPowerState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geothermal_plant_constructor() {
        let plant = PowerPlant::new_geothermal(10, 20);
        assert_eq!(plant.plant_type, PowerPlantType::Geothermal);
        assert!((plant.capacity_mw - GEOTHERMAL_CAPACITY_MW).abs() < f32::EPSILON);
        assert!(
            (plant.current_output_mw - GEOTHERMAL_CAPACITY_MW * GEOTHERMAL_CAPACITY_FACTOR).abs()
                < f32::EPSILON
        );
        assert!((plant.fuel_cost - GEOTHERMAL_FUEL_COST_PER_MWH).abs() < f32::EPSILON);
        assert_eq!(plant.grid_x, 10);
        assert_eq!(plant.grid_y, 20);
    }

    #[test]
    fn test_geothermal_zero_fuel_cost() {
        let plant = PowerPlant::new_geothermal(5, 5);
        assert_eq!(plant.fuel_cost, 0.0, "Geothermal should have zero fuel cost");
    }

    #[test]
    fn test_geothermal_baseload_output() {
        let plant = PowerPlant::new_geothermal(5, 5);
        let expected = GEOTHERMAL_CAPACITY_MW * GEOTHERMAL_CAPACITY_FACTOR;
        assert!(
            (plant.current_output_mw - expected).abs() < f32::EPSILON,
            "Geothermal baseload output should be {expected} MW, got {}",
            plant.current_output_mw
        );
    }

    #[test]
    fn test_geothermal_power_state_default() {
        let state = GeothermalPowerState::default();
        assert_eq!(state.plant_count, 0);
        assert!((state.total_output_mw).abs() < f32::EPSILON);
    }

    #[test]
    fn test_geothermal_power_state_save_skip_empty() {
        use crate::Saveable;
        let state = GeothermalPowerState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should not produce save bytes"
        );
    }

    #[test]
    fn test_geothermal_power_state_roundtrip() {
        use crate::Saveable;
        let state = GeothermalPowerState {
            plant_count: 2,
            total_output_mw: 54.0,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = GeothermalPowerState::load_from_bytes(&bytes);
        assert_eq!(loaded.plant_count, 2);
        assert!((loaded.total_output_mw - 54.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_geothermal_footprint() {
        assert_eq!(GEOTHERMAL_FOOTPRINT, (3, 3));
    }

    #[test]
    fn test_geothermal_capacity_factor_ninety_percent() {
        assert!(
            (GEOTHERMAL_CAPACITY_FACTOR - 0.90).abs() < f32::EPSILON,
            "Capacity factor should be 0.90"
        );
    }
}
