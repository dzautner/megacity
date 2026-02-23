//! POWER-006: Wind Turbine Farm Power Plant
//!
//! Implements wind turbine farms as power generators that contribute to
//! `EnergyGrid.total_supply_mwh`. Power output follows a cubic wind curve
//! based on the global `WindState.speed` (0.0–1.0 normalized):
//!
//! - Below cut-in speed (0.1): no output
//! - Above cut-out speed (0.95): shutdown (no output)
//! - Otherwise: `output = nameplate * wind_speed^3`
//!
//! Each wind farm has:
//! - 100 MW nameplate capacity
//! - Fuel cost: $0/MWh
//! - Air pollution: Q=0.0 (clean energy)
//! - Noise: 55 dB source level
//! - 3×3 building footprint

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::energy_demand::EnergyGrid;
use crate::noise::NoisePollutionGrid;
use crate::utilities::{UtilitySource, UtilityType};
use crate::wind::WindState;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Nameplate capacity of a single wind turbine farm (MW).
pub const WIND_FARM_NAMEPLATE_MW: f32 = 100.0;

/// Below this normalized wind speed, turbines do not generate power.
pub const CUT_IN_SPEED: f32 = 0.1;

/// Above this normalized wind speed, turbines shut down for safety.
pub const CUT_OUT_SPEED: f32 = 0.95;

/// Fuel cost per MWh (zero for wind).
pub const WIND_FUEL_COST_PER_MWH: f32 = 0.0;

/// Source noise level for a wind turbine farm (dB).
const WIND_TURBINE_NOISE_DB: u8 = 55;

/// Noise radiation radius in grid cells.
const NOISE_RADIUS: i32 = 4;

/// Noise decay per cell of distance (dB).
const NOISE_DECAY_PER_CELL: u8 = 10;

/// Building footprint in grid cells (width, height).
pub const WIND_FOOTPRINT: (usize, usize) = (3, 3);

// =============================================================================
// WindPowerState resource (city-wide wind power stats)
// =============================================================================

/// Aggregated city-wide state for wind power generation.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct WindPowerState {
    /// Number of active wind farms in the city.
    pub farm_count: u32,
    /// Total generation from all wind farms (MW).
    pub total_output_mw: f32,
    /// Current wind speed used for generation (0.0–1.0).
    pub current_wind_speed: f32,
}

impl Default for WindPowerState {
    fn default() -> Self {
        Self {
            farm_count: 0,
            total_output_mw: 0.0,
            current_wind_speed: 0.0,
        }
    }
}

impl crate::Saveable for WindPowerState {
    const SAVE_KEY: &'static str = "wind_power";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.farm_count == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Pure calculation
// =============================================================================

/// Calculate wind turbine output using the cubic wind power curve.
///
/// Returns 0.0 if wind speed is below cut-in or above cut-out thresholds.
pub fn wind_power_output(nameplate_mw: f32, wind_speed: f32) -> f32 {
    if wind_speed < CUT_IN_SPEED || wind_speed > CUT_OUT_SPEED {
        return 0.0;
    }
    nameplate_mw * wind_speed * wind_speed * wind_speed
}

// =============================================================================
// Systems
// =============================================================================

/// Attaches `PowerPlant` components to `UtilitySource` entities of type
/// `WindTurbine` that don't already have one.
pub fn attach_wind_power_plants(
    timer: Res<SlowTickTimer>,
    mut commands: Commands,
    sources: Query<(Entity, &UtilitySource), Without<PowerPlant>>,
) {
    if !timer.should_run() {
        return;
    }

    for (entity, source) in &sources {
        if source.utility_type == UtilityType::WindTurbine {
            commands.entity(entity).insert(PowerPlant {
                plant_type: PowerPlantType::WindTurbine,
                capacity_mw: WIND_FARM_NAMEPLATE_MW,
                current_output_mw: 0.0,
                fuel_cost: WIND_FUEL_COST_PER_MWH,
                grid_x: source.grid_x,
                grid_y: source.grid_y,
            });
        }
    }
}

/// Aggregates wind power plant output into `EnergyGrid.total_supply_mwh`
/// and updates `WindPowerState`. Runs every slow tick.
pub fn aggregate_wind_power(
    timer: Res<SlowTickTimer>,
    wind: Res<WindState>,
    mut plants: Query<&mut PowerPlant>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut wind_state: ResMut<WindPowerState>,
) {
    if !timer.should_run() {
        return;
    }

    let mut count = 0u32;
    let mut total_output = 0.0f32;

    for mut plant in &mut plants {
        if plant.plant_type != PowerPlantType::WindTurbine {
            continue;
        }
        let output = wind_power_output(plant.capacity_mw, wind.speed);
        plant.current_output_mw = output;
        total_output += output;
        count += 1;
    }

    wind_state.farm_count = count;
    wind_state.total_output_mw = total_output;
    wind_state.current_wind_speed = wind.speed;

    // Add wind generation to the energy grid supply
    energy_grid.total_supply_mwh += total_output;
}

/// Adds noise pollution around each wind turbine farm. Runs every slow tick.
///
/// Wind turbines emit 55 dB at the source, decaying with distance.
/// Only turbines currently producing power generate noise.
pub fn wind_turbine_noise(
    timer: Res<SlowTickTimer>,
    mut noise: ResMut<NoisePollutionGrid>,
    plants: Query<&PowerPlant>,
) {
    if !timer.should_run() {
        return;
    }

    for plant in &plants {
        if plant.plant_type != PowerPlantType::WindTurbine {
            continue;
        }

        // Only generate noise when the turbine is actually producing power
        if plant.current_output_mw <= 0.0 {
            continue;
        }

        let cx = plant.grid_x as i32;
        let cy = plant.grid_y as i32;

        for dy in -NOISE_RADIUS..=NOISE_RADIUS {
            for dx in -NOISE_RADIUS..=NOISE_RADIUS {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let dist = (dx.unsigned_abs() + dy.unsigned_abs()) as u8;
                let decayed = WIND_TURBINE_NOISE_DB.saturating_sub(dist * NOISE_DECAY_PER_CELL);
                if decayed > 0 {
                    let idx = ny as usize * GRID_WIDTH + nx as usize;
                    noise.levels[idx] = noise.levels[idx].saturating_add(decayed).min(100);
                }
            }
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers wind power plant resources and systems.
pub struct WindPowerPlugin;

impl Plugin for WindPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindPowerState>().add_systems(
            FixedUpdate,
            (
                attach_wind_power_plants,
                aggregate_wind_power.after(attach_wind_power_plants),
                wind_turbine_noise.after(aggregate_wind_power),
            )
                .after(crate::wind::update_wind)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WindPowerState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_output_at_half_speed() {
        let output = wind_power_output(100.0, 0.5);
        let expected = 100.0 * 0.5_f32.powi(3);
        assert!(
            (output - expected).abs() < 0.001,
            "expected {expected}, got {output}"
        );
    }

    #[test]
    fn test_zero_output_below_cut_in() {
        assert_eq!(wind_power_output(100.0, 0.0), 0.0);
        assert_eq!(wind_power_output(100.0, 0.05), 0.0);
        assert_eq!(wind_power_output(100.0, 0.09), 0.0);
    }

    #[test]
    fn test_zero_output_above_cut_out() {
        assert_eq!(wind_power_output(100.0, 0.96), 0.0);
        assert_eq!(wind_power_output(100.0, 1.0), 0.0);
    }

    #[test]
    fn test_output_at_cut_in_boundary() {
        let output = wind_power_output(100.0, 0.1);
        let expected = 100.0 * 0.1_f32.powi(3);
        assert!(
            (output - expected).abs() < 0.001,
            "at cut-in boundary: expected {expected}, got {output}"
        );
    }

    #[test]
    fn test_output_at_cut_out_boundary() {
        let output = wind_power_output(100.0, 0.95);
        let expected = 100.0 * 0.95_f32.powi(3);
        assert!(
            (output - expected).abs() < 0.001,
            "at cut-out boundary: expected {expected}, got {output}"
        );
    }

    #[test]
    fn test_output_scales_with_nameplate() {
        let output_100 = wind_power_output(100.0, 0.5);
        let output_200 = wind_power_output(200.0, 0.5);
        assert!(
            (output_200 - output_100 * 2.0).abs() < 0.001,
            "output should scale linearly with nameplate capacity"
        );
    }

    #[test]
    fn test_wind_power_state_default() {
        let state = WindPowerState::default();
        assert_eq!(state.farm_count, 0);
        assert_eq!(state.total_output_mw, 0.0);
        assert_eq!(state.current_wind_speed, 0.0);
    }

    #[test]
    fn test_wind_power_state_save_skip_empty() {
        use crate::Saveable;
        let state = WindPowerState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should not produce save bytes"
        );
    }

    #[test]
    fn test_wind_power_state_roundtrip() {
        use crate::Saveable;
        let state = WindPowerState {
            farm_count: 2,
            total_output_mw: 25.0,
            current_wind_speed: 0.5,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = WindPowerState::load_from_bytes(&bytes);
        assert_eq!(loaded.farm_count, 2);
        assert!((loaded.total_output_mw - 25.0).abs() < f32::EPSILON);
        assert!((loaded.current_wind_speed - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_wind_footprint() {
        assert_eq!(WIND_FOOTPRINT, (3, 3));
    }
}
