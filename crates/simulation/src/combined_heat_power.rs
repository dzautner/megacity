//! POWER-021: Combined Heat and Power (CHP) from Power Plants
//!
//! Implements CHP upgrades for thermal power plants (coal, gas, biomass) and
//! waste-to-energy (WTE) plants. CHP-upgraded plants provide district heating
//! as a co-product of electricity generation.
//!
//! Key specs:
//! - Eligible plant types: Coal, NaturalGas, Biomass, WasteToEnergy
//! - CHP upgrade cost: $20M per plant
//! - +15% overall efficiency bonus
//! - Heating coverage radius: 20 grid cells (BFS propagation via HeatingPlant)
//! - Heat output = 0.5x electricity output (kWh heat per kWh electricity)
//! - Saveable for persistence

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::heating::{HeatingPlant, HeatingPlantType};
use crate::weather::Weather;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Cost of upgrading a plant to CHP (dollars).
pub const CHP_UPGRADE_COST: f64 = 20_000_000.0;

/// Efficiency bonus applied to CHP-upgraded plants (fraction: 0.15 = +15%).
pub const CHP_EFFICIENCY_BONUS: f32 = 0.15;

/// BFS heating propagation radius for CHP plants (grid cells).
pub const CHP_HEATING_RADIUS: u32 = 20;

/// Heat output multiplier relative to electricity output.
/// 0.5 means 0.5 kWh heat per 1 kWh electricity.
pub const CHP_HEAT_OUTPUT_RATIO: f32 = 0.5;

/// Maximum heat level at the source for CHP plants.
/// Scaled by heat output ratio to differentiate from dedicated heating plants.
pub const CHP_SOURCE_HEAT_CAPACITY: u8 = 200;

// =============================================================================
// ChpState resource
// =============================================================================

/// City-wide state tracking which power plants have CHP upgrades.
/// Plants are identified by their grid position (x, y).
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct ChpState {
    /// Set of grid positions (x, y) where CHP-upgraded plants are located.
    pub upgraded_positions: Vec<(usize, usize)>,
    /// Number of CHP-upgraded plants.
    pub upgrade_count: u32,
    /// Total heat output across all CHP plants (MW thermal).
    pub total_heat_output_mw: f32,
    /// Total electricity efficiency bonus applied (MW saved).
    pub total_efficiency_bonus_mw: f32,
}

impl Default for ChpState {
    fn default() -> Self {
        Self {
            upgraded_positions: Vec::new(),
            upgrade_count: 0,
            total_heat_output_mw: 0.0,
            total_efficiency_bonus_mw: 0.0,
        }
    }
}

impl crate::Saveable for ChpState {
    const SAVE_KEY: &'static str = "combined_heat_power";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.upgraded_positions.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Returns true if the given power plant type is eligible for CHP upgrade.
pub fn is_chp_eligible(plant_type: PowerPlantType) -> bool {
    matches!(
        plant_type,
        PowerPlantType::Coal
            | PowerPlantType::NaturalGas
            | PowerPlantType::Biomass
            | PowerPlantType::WasteToEnergy
    )
}

// =============================================================================
// Systems
// =============================================================================

/// Attach `HeatingPlant` components to CHP-upgraded power plants that don't
/// already have one. This makes the existing `update_heating` system
/// propagate heat from CHP plants via BFS automatically.
pub fn attach_chp_heating_plants(
    mut commands: Commands,
    chp_state: Res<ChpState>,
    plants: Query<(Entity, &PowerPlant), Without<HeatingPlant>>,
) {
    let upgraded_set: HashSet<(usize, usize)> = chp_state
        .upgraded_positions
        .iter()
        .copied()
        .collect();

    for (entity, plant) in &plants {
        if !is_chp_eligible(plant.plant_type) {
            continue;
        }
        let pos = (plant.grid_x, plant.grid_y);
        if !upgraded_set.contains(&pos) {
            continue;
        }

        // Heat capacity scales with electricity output via heat ratio
        let heat_capacity = (plant.current_output_mw * CHP_HEAT_OUTPUT_RATIO)
            .min(CHP_SOURCE_HEAT_CAPACITY as f32) as u8;
        let heat_capacity = heat_capacity.max(1);

        commands.entity(entity).insert(HeatingPlant {
            plant_type: HeatingPlantType::SmallBoiler,
            grid_x: plant.grid_x,
            grid_y: plant.grid_y,
            capacity: heat_capacity,
            efficiency: HeatingPlantType::DistrictHeating.efficiency()
                + CHP_EFFICIENCY_BONUS,
        });
    }
}

/// Update CHP statistics: total heat output and efficiency bonuses.
/// Runs on slow tick.
pub fn update_chp_stats(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    plants: Query<&PowerPlant>,
    mut chp_state: ResMut<ChpState>,
) {
    if !timer.should_run() {
        return;
    }

    let upgraded_set: HashSet<(usize, usize)> = chp_state
        .upgraded_positions
        .iter()
        .copied()
        .collect();

    let demand = crate::heating::heating_demand(&weather);

    let mut total_heat = 0.0f32;
    let mut total_efficiency = 0.0f32;
    let mut count = 0u32;

    for plant in &plants {
        if !is_chp_eligible(plant.plant_type) {
            continue;
        }
        let pos = (plant.grid_x, plant.grid_y);
        if !upgraded_set.contains(&pos) {
            continue;
        }

        count += 1;
        // Heat output = electricity output * heat ratio, scaled by demand
        let heat_mw = plant.current_output_mw * CHP_HEAT_OUTPUT_RATIO * demand;
        total_heat += heat_mw;
        // Efficiency bonus = electricity output * bonus percentage
        total_efficiency += plant.current_output_mw * CHP_EFFICIENCY_BONUS;
    }

    chp_state.upgrade_count = count;
    chp_state.total_heat_output_mw = total_heat;
    chp_state.total_efficiency_bonus_mw = total_efficiency;
}

/// Synchronize the HeatingPlant capacity on CHP plants to track changing
/// electricity output (e.g., after energy dispatch adjusts current_output_mw).
pub fn sync_chp_heating_capacity(
    timer: Res<SlowTickTimer>,
    chp_state: Res<ChpState>,
    mut plants: Query<(&PowerPlant, &mut HeatingPlant)>,
) {
    if !timer.should_run() {
        return;
    }

    let upgraded_set: HashSet<(usize, usize)> = chp_state
        .upgraded_positions
        .iter()
        .copied()
        .collect();

    for (power, mut heating) in &mut plants {
        if !is_chp_eligible(power.plant_type) {
            continue;
        }
        let pos = (power.grid_x, power.grid_y);
        if !upgraded_set.contains(&pos) {
            continue;
        }

        // Update heating capacity based on current electricity output
        let heat_capacity = (power.current_output_mw * CHP_HEAT_OUTPUT_RATIO)
            .min(CHP_SOURCE_HEAT_CAPACITY as f32) as u8;
        heating.capacity = heat_capacity.max(1);
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct CombinedHeatPowerPlugin;

impl Plugin for CombinedHeatPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChpState>().add_systems(
            FixedUpdate,
            (
                attach_chp_heating_plants,
                sync_chp_heating_capacity
                    .after(attach_chp_heating_plants)
                    .after(crate::energy_dispatch::dispatch_energy),
                update_chp_stats
                    .after(sync_chp_heating_capacity),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ChpState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chp_eligible_plant_types() {
        assert!(is_chp_eligible(PowerPlantType::Coal));
        assert!(is_chp_eligible(PowerPlantType::NaturalGas));
        assert!(is_chp_eligible(PowerPlantType::Biomass));
        assert!(is_chp_eligible(PowerPlantType::WasteToEnergy));
        assert!(!is_chp_eligible(PowerPlantType::WindTurbine));
        assert!(!is_chp_eligible(PowerPlantType::HydroDam));
        assert!(!is_chp_eligible(PowerPlantType::Geothermal));
        assert!(!is_chp_eligible(PowerPlantType::Nuclear));
        assert!(!is_chp_eligible(PowerPlantType::Oil));
    }

    #[test]
    fn test_chp_state_default() {
        let state = ChpState::default();
        assert!(state.upgraded_positions.is_empty());
        assert_eq!(state.upgrade_count, 0);
        assert!(state.total_heat_output_mw.abs() < f32::EPSILON);
        assert!(state.total_efficiency_bonus_mw.abs() < f32::EPSILON);
    }

    #[test]
    fn test_chp_state_save_skip_empty() {
        use crate::Saveable;
        let state = ChpState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty CHP state should not produce save bytes"
        );
    }

    #[test]
    fn test_chp_state_roundtrip() {
        use crate::Saveable;
        let state = ChpState {
            upgraded_positions: vec![(10, 20), (30, 40)],
            upgrade_count: 2,
            total_heat_output_mw: 33.0,
            total_efficiency_bonus_mw: 9.9,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = ChpState::load_from_bytes(&bytes);
        assert_eq!(loaded.upgraded_positions.len(), 2);
        assert_eq!(loaded.upgrade_count, 2);
        assert!((loaded.total_heat_output_mw - 33.0).abs() < f32::EPSILON);
        assert!((loaded.total_efficiency_bonus_mw - 9.9).abs() < 0.01);
    }

    #[test]
    fn test_chp_constants() {
        assert!((CHP_UPGRADE_COST - 20_000_000.0).abs() < f64::EPSILON);
        assert!((CHP_EFFICIENCY_BONUS - 0.15).abs() < f32::EPSILON);
        assert_eq!(CHP_HEATING_RADIUS, 20);
        assert!((CHP_HEAT_OUTPUT_RATIO - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_chp_heat_output_ratio() {
        let electricity_mw = 100.0f32;
        let heat_mw = electricity_mw * CHP_HEAT_OUTPUT_RATIO;
        assert!((heat_mw - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_chp_efficiency_bonus_value() {
        let coal_output = 200.0f32;
        let bonus = coal_output * CHP_EFFICIENCY_BONUS;
        assert!((bonus - 30.0).abs() < f32::EPSILON);
    }
}
