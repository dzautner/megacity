//! POWER-007: Hydroelectric Dam Power Plant
//!
//! Implements hydroelectric dams as renewable, dispatchable power sources.
//! Output depends on water flow which varies seasonally:
//!
//! - Spring: 0.55 capacity factor (snowmelt runoff)
//! - Summer: 0.30 capacity factor (lower rainfall)
//! - Autumn: 0.35 capacity factor (moderate flow)
//! - Winter: 0.50 capacity factor (rain season)
//!
//! Average capacity factor: ~0.40
//!
//! Each hydroelectric dam has:
//! - 200 MW nameplate capacity
//! - Fuel cost: $0/MWh (water is free)
//! - Air pollution: Q=0.0 (clean energy)
//! - Construction cost: $500M
//! - Maintenance: $5M/year
//! - 4×4 building footprint
//! - Must be placed on water cells

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::utilities::{UtilitySource, UtilityType};
use crate::weather::{Season, Weather};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Nameplate capacity of a single hydroelectric dam in MW.
pub const HYDRO_NAMEPLATE_MW: f32 = 200.0;

/// Fuel cost per MWh (zero — water flow is free).
pub const HYDRO_FUEL_COST_PER_MWH: f32 = 0.0;

/// Air pollution emission factor (zero for hydro).
pub const HYDRO_AIR_POLLUTION_Q: f32 = 0.0;

/// Construction cost in dollars.
pub const HYDRO_CONSTRUCTION_COST: f64 = 500_000_000.0;

/// Annual maintenance cost in dollars.
pub const HYDRO_ANNUAL_MAINTENANCE: f64 = 5_000_000.0;

/// Grid footprint of a hydroelectric dam (4x4 cells).
pub const HYDRO_DAM_FOOTPRINT: (usize, usize) = (4, 4);

// =============================================================================
// Seasonal capacity factor
// =============================================================================

/// Returns the seasonal capacity factor for hydroelectric dams.
///
/// Water flow varies by season:
/// - Spring: high flow from snowmelt (0.55)
/// - Summer: low flow, reduced rainfall (0.30)
/// - Autumn: moderate flow (0.35)
/// - Winter: increased precipitation (0.50)
///
/// Average across seasons: (0.55 + 0.30 + 0.35 + 0.50) / 4 = 0.425 ≈ 0.40
pub fn seasonal_capacity_factor(season: Season) -> f32 {
    match season {
        Season::Spring => 0.55,
        Season::Summer => 0.30,
        Season::Autumn => 0.35,
        Season::Winter => 0.50,
    }
}

// =============================================================================
// Resource
// =============================================================================

/// City-wide hydroelectric power generation state.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct HydroPowerState {
    /// Number of hydroelectric dams in the city.
    pub dam_count: u32,
    /// Total hydro output across all dams in MW.
    pub total_output_mw: f32,
    /// Current seasonal capacity factor (for UI display).
    pub current_capacity_factor: f32,
}

impl Default for HydroPowerState {
    fn default() -> Self {
        Self {
            dam_count: 0,
            total_output_mw: 0.0,
            current_capacity_factor: 0.0,
        }
    }
}

impl crate::Saveable for HydroPowerState {
    const SAVE_KEY: &'static str = "hydro_power";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.dam_count == 0 {
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
/// `HydroDam` that don't already have one.
pub fn attach_hydro_power_plants(
    timer: Res<SlowTickTimer>,
    mut commands: Commands,
    sources: Query<(Entity, &UtilitySource), Without<PowerPlant>>,
) {
    if !timer.should_run() {
        return;
    }

    for (entity, source) in &sources {
        if source.utility_type == UtilityType::HydroDam {
            commands.entity(entity).insert(PowerPlant {
                plant_type: PowerPlantType::HydroDam,
                capacity_mw: HYDRO_NAMEPLATE_MW,
                current_output_mw: 0.0,
                fuel_cost: HYDRO_FUEL_COST_PER_MWH,
                grid_x: source.grid_x,
                grid_y: source.grid_y,
            });
        }
    }
}

/// Aggregates hydroelectric dam output into `EnergyGrid.total_supply_mwh`
/// and updates `HydroPowerState`. Runs every slow tick.
///
/// Output = nameplate_capacity * seasonal_capacity_factor
pub fn aggregate_hydro_power(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    mut plants: Query<&mut PowerPlant>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut hydro_state: ResMut<HydroPowerState>,
) {
    if !timer.should_run() {
        return;
    }

    let capacity_factor = seasonal_capacity_factor(weather.season);
    let mut count = 0u32;
    let mut total_output = 0.0f32;

    for mut plant in &mut plants {
        if plant.plant_type != PowerPlantType::HydroDam {
            continue;
        }
        let output = plant.capacity_mw * capacity_factor;
        plant.current_output_mw = output;
        total_output += output;
        count += 1;
    }

    hydro_state.dam_count = count;
    hydro_state.total_output_mw = total_output;
    hydro_state.current_capacity_factor = capacity_factor;

    // Add hydro generation to the energy grid supply
    energy_grid.total_supply_mwh += total_output;
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers hydroelectric dam power plant resources and systems.
pub struct HydroPowerPlugin;

impl Plugin for HydroPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HydroPowerState>().add_systems(
            FixedUpdate,
            (
                attach_hydro_power_plants,
                aggregate_hydro_power
                    .after(attach_hydro_power_plants)
                    .after(crate::energy_dispatch::dispatch_energy),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<HydroPowerState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seasonal_factors() {
        assert_eq!(seasonal_capacity_factor(Season::Spring), 0.55);
        assert_eq!(seasonal_capacity_factor(Season::Summer), 0.30);
        assert_eq!(seasonal_capacity_factor(Season::Autumn), 0.35);
        assert_eq!(seasonal_capacity_factor(Season::Winter), 0.50);
    }

    #[test]
    fn test_spring_highest_output() {
        // Spring snowmelt should give highest capacity factor
        let spring = seasonal_capacity_factor(Season::Spring);
        let summer = seasonal_capacity_factor(Season::Summer);
        let autumn = seasonal_capacity_factor(Season::Autumn);
        let winter = seasonal_capacity_factor(Season::Winter);
        assert!(spring > summer);
        assert!(spring > autumn);
        assert!(spring > winter);
    }

    #[test]
    fn test_summer_lowest_output() {
        // Summer low rainfall should give lowest capacity factor
        let summer = seasonal_capacity_factor(Season::Summer);
        let spring = seasonal_capacity_factor(Season::Spring);
        let autumn = seasonal_capacity_factor(Season::Autumn);
        let winter = seasonal_capacity_factor(Season::Winter);
        assert!(summer < spring);
        assert!(summer < autumn);
        assert!(summer < winter);
    }

    #[test]
    fn test_average_capacity_factor_near_040() {
        let avg = (seasonal_capacity_factor(Season::Spring)
            + seasonal_capacity_factor(Season::Summer)
            + seasonal_capacity_factor(Season::Autumn)
            + seasonal_capacity_factor(Season::Winter))
            / 4.0;
        assert!(
            (avg - 0.425).abs() < 0.01,
            "average capacity factor should be ~0.40, got {avg}"
        );
    }

    #[test]
    fn test_full_output_formula() {
        // Spring: maximum seasonal output
        let output = HYDRO_NAMEPLATE_MW * seasonal_capacity_factor(Season::Spring);
        let expected = 200.0 * 0.55;
        assert!(
            (output - expected).abs() < 0.001,
            "spring output should be {expected} MW, got {output}"
        );
    }

    #[test]
    fn test_summer_output_formula() {
        let output = HYDRO_NAMEPLATE_MW * seasonal_capacity_factor(Season::Summer);
        let expected = 200.0 * 0.30;
        assert!(
            (output - expected).abs() < 0.001,
            "summer output should be {expected} MW, got {output}"
        );
    }

    #[test]
    fn test_hydro_power_state_default() {
        let state = HydroPowerState::default();
        assert_eq!(state.dam_count, 0);
        assert_eq!(state.total_output_mw, 0.0);
        assert_eq!(state.current_capacity_factor, 0.0);
    }

    #[test]
    fn test_saveable_skip_when_empty() {
        use crate::Saveable;
        let state = HydroPowerState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "empty state should skip save"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let state = HydroPowerState {
            dam_count: 2,
            total_output_mw: 220.0,
            current_capacity_factor: 0.55,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = HydroPowerState::load_from_bytes(&bytes);
        assert_eq!(loaded.dam_count, 2);
        assert!((loaded.total_output_mw - 220.0).abs() < f32::EPSILON);
        assert!((loaded.current_capacity_factor - 0.55).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hydro_dam_footprint() {
        assert_eq!(HYDRO_DAM_FOOTPRINT, (4, 4));
    }

    #[test]
    fn test_hydro_zero_fuel_cost() {
        assert_eq!(HYDRO_FUEL_COST_PER_MWH, 0.0);
    }

    #[test]
    fn test_hydro_zero_pollution() {
        assert_eq!(HYDRO_AIR_POLLUTION_Q, 0.0);
    }

    #[test]
    fn test_nameplate_capacity() {
        assert_eq!(HYDRO_NAMEPLATE_MW, 200.0);
    }
}
