//! POWER-014: Waste-to-Energy Power Plant
//!
//! Implements waste-to-energy (WTE) plants that incinerate municipal waste to
//! generate electricity. WTE reduces landfill volume by 90% but produces air
//! emissions requiring scrubbers.
//!
//! Key specs:
//! - 200-1000 tons/day waste input, generates 0.5-1.0 MWh/ton electricity
//! - Energy output: waste_tons * BTU_per_lb * 2000 * boiler_eff * generator_eff / 3412 / 1000 / 24
//!   (converts BTU/day -> kWh/day -> MWh/day -> MW average)
//! - Default: 500 tons/day = ~17.5 MW average output
//! - Construction cost: $50M, build time: 10 game-days (1000 ticks at 10Hz)
//! - Operating cost: $40-60/ton, revenue from tipping fees $50-80/ton
//! - Air pollution: Q=45.0 (with scrubbers: Q=20.0)
//! - Ash residue: 10% of input mass
//! - 4x4 building footprint

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::EnergyGrid;
use crate::garbage::WasteSystem;
use crate::waste_composition::WasteComposition;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Building footprint in grid cells (width, height).
pub const WTE_FOOTPRINT: (usize, usize) = (4, 4);

/// Minimum waste input in tons/day.
pub const WTE_MIN_WASTE_TONS: f32 = 200.0;

/// Maximum waste input in tons/day.
pub const WTE_MAX_WASTE_TONS: f32 = 1000.0;

/// Default waste input in tons/day.
pub const WTE_DEFAULT_WASTE_TONS: f32 = 500.0;

/// Boiler efficiency (fraction of heat energy captured).
pub const WTE_BOILER_EFFICIENCY: f32 = 0.80;

/// Generator (turbine) efficiency (fraction of steam energy converted to electricity).
pub const WTE_GENERATOR_EFFICIENCY: f32 = 0.33;

/// BTU per kWh (conversion factor).
pub const BTU_PER_KWH: f32 = 3_412.0;

/// Pounds per ton (short ton).
pub const LBS_PER_TON: f32 = 2000.0;

/// Hours per day (for converting daily energy to average power).
pub const HOURS_PER_DAY: f32 = 24.0;

/// kW per MW conversion.
pub const KW_PER_MW: f32 = 1000.0;

/// Construction cost in dollars.
pub const WTE_CONSTRUCTION_COST: f64 = 50_000_000.0;

/// Build time in game ticks (10 game-days at 100 ticks/day).
pub const WTE_BUILD_TICKS: u32 = 1000;

/// Operating cost per ton of waste processed (dollars).
pub const WTE_OPERATING_COST_PER_TON: f32 = 50.0;

/// Revenue from tipping fees per ton of waste received (dollars).
pub const WTE_TIPPING_FEE_PER_TON: f32 = 65.0;

/// Air pollution emission rate Q (without scrubbers).
pub const WTE_POLLUTION_Q_RAW: f32 = 45.0;

/// Air pollution emission rate Q (with scrubbers installed).
pub const WTE_POLLUTION_Q_SCRUBBED: f32 = 20.0;

/// Ash residue fraction (10% of input mass remains as ash).
pub const WTE_ASH_FRACTION: f32 = 0.10;

/// Fuel cost used for merit-order dispatch ($/MWh).
/// WTE has negative effective fuel cost because tipping fees offset costs,
/// but we use a small positive value so it dispatches after renewables.
pub const WTE_FUEL_COST_PER_MWH: f32 = 5.0;

// =============================================================================
// Energy output calculation
// =============================================================================

/// Calculates the average power output in MW for a given waste input rate.
///
/// Formula: waste_tons * BTU_per_lb * 2000 * boiler_eff * generator_eff / 3412 / 1000 / 24
///
/// Steps:
/// 1. waste_tons * BTU_per_lb * 2000 = total BTU/day
/// 2. * boiler_eff * generator_eff = useful BTU/day
/// 3. / 3412 = kWh/day
/// 4. / 1000 = MWh/day
/// 5. / 24 = MW (average power)
///
/// Using default MSW composition (~5,443 BTU/lb):
///   500 * 5443 * 2000 * 0.80 * 0.33 / 3412 / 1000 / 24 = ~17.5 MW
pub fn calculate_wte_output_mw(waste_tons_per_day: f32, btu_per_lb: f32) -> f32 {
    let total_btu_per_day = waste_tons_per_day * btu_per_lb * LBS_PER_TON;
    let useful_btu_per_day = total_btu_per_day * WTE_BOILER_EFFICIENCY * WTE_GENERATOR_EFFICIENCY;
    let kwh_per_day = useful_btu_per_day / BTU_PER_KWH;
    let mwh_per_day = kwh_per_day / KW_PER_MW;
    mwh_per_day / HOURS_PER_DAY
}

// =============================================================================
// WteState resource
// =============================================================================

/// City-wide aggregated state for waste-to-energy plants.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct WteState {
    /// Number of active WTE plants.
    pub plant_count: u32,
    /// Total waste consumed per day across all plants (tons).
    pub total_waste_consumed_tons: f32,
    /// Total energy output across all plants (MW).
    pub total_output_mw: f32,
    /// Total ash residue produced per day (tons).
    pub total_ash_tons: f32,
    /// Total operating cost per day (dollars).
    pub total_operating_cost: f32,
    /// Total tipping fee revenue per day (dollars).
    pub total_tipping_revenue: f32,
    /// Whether scrubbers are installed (reduces pollution).
    pub scrubbers_installed: bool,
    /// Current pollution Q value (depends on scrubber status).
    pub current_pollution_q: f32,
}

impl Default for WteState {
    fn default() -> Self {
        Self {
            plant_count: 0,
            total_waste_consumed_tons: 0.0,
            total_output_mw: 0.0,
            total_ash_tons: 0.0,
            total_operating_cost: 0.0,
            total_tipping_revenue: 0.0,
            scrubbers_installed: true,
            current_pollution_q: WTE_POLLUTION_Q_SCRUBBED,
        }
    }
}

impl crate::Saveable for WteState {
    const SAVE_KEY: &'static str = "waste_to_energy";

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
// WtePlant component
// =============================================================================

/// Component attached to WTE plant entities for per-plant tracking.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WtePlant {
    /// Waste input capacity in tons/day for this plant.
    pub waste_capacity_tons: f32,
    /// Current waste being processed in tons/day.
    pub current_waste_tons: f32,
    /// Grid x position.
    pub grid_x: usize,
    /// Grid y position.
    pub grid_y: usize,
}

impl WtePlant {
    /// Create a new WTE plant at the given grid position with default capacity.
    pub fn new(grid_x: usize, grid_y: usize) -> Self {
        Self {
            waste_capacity_tons: WTE_DEFAULT_WASTE_TONS,
            current_waste_tons: 0.0,
            grid_x,
            grid_y,
        }
    }

    /// Create a new WTE plant with a specific waste capacity.
    pub fn with_capacity(grid_x: usize, grid_y: usize, capacity_tons: f32) -> Self {
        Self {
            waste_capacity_tons: capacity_tons.clamp(WTE_MIN_WASTE_TONS, WTE_MAX_WASTE_TONS),
            current_waste_tons: 0.0,
            grid_x,
            grid_y,
        }
    }
}

impl PowerPlant {
    /// Create a new waste-to-energy power plant at the given grid position.
    ///
    /// Initial capacity is set based on the default waste input (500 tons/day)
    /// and average MSW energy content.
    pub fn new_wte(grid_x: usize, grid_y: usize) -> Self {
        let btu_per_lb = WasteComposition::default().energy_content_btu_per_lb();
        let capacity_mw = calculate_wte_output_mw(WTE_DEFAULT_WASTE_TONS, btu_per_lb);
        Self {
            plant_type: PowerPlantType::WasteToEnergy,
            capacity_mw,
            current_output_mw: capacity_mw,
            fuel_cost: WTE_FUEL_COST_PER_MWH,
            grid_x,
            grid_y,
        }
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Aggregates WTE plant output: determines waste available from the city's
/// waste system, calculates energy output, updates ash production, and
/// feeds output into the energy grid.
///
/// Runs every slow tick.
pub fn update_wte_plants(
    timer: Res<SlowTickTimer>,
    mut wte_plants: Query<(&mut WtePlant, &mut PowerPlant)>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut wte_state: ResMut<WteState>,
    waste_system: Res<WasteSystem>,
) {
    if !timer.should_run() {
        return;
    }

    let btu_per_lb = WasteComposition::default().energy_content_btu_per_lb();
    let pollution_q = if wte_state.scrubbers_installed {
        WTE_POLLUTION_Q_SCRUBBED
    } else {
        WTE_POLLUTION_Q_RAW
    };

    let mut total_count = 0u32;
    let mut total_waste_consumed = 0.0f32;
    let mut total_output = 0.0f32;
    let mut total_ash = 0.0f32;
    let mut total_operating_cost = 0.0f32;
    let mut total_tipping_revenue = 0.0f32;

    // Total waste available from the city (tons/day from the waste system).
    let available_waste = waste_system.period_generated_tons as f32;

    // Calculate total WTE capacity for proportional allocation.
    let total_capacity: f32 = wte_plants
        .iter()
        .filter(|(_, p)| p.plant_type == PowerPlantType::WasteToEnergy)
        .map(|(wte, _)| wte.waste_capacity_tons)
        .sum();

    for (mut wte, mut plant) in &mut wte_plants {
        if plant.plant_type != PowerPlantType::WasteToEnergy {
            continue;
        }
        total_count += 1;

        // Allocate waste proportionally to this plant's capacity.
        let waste_share = if total_capacity > 0.0 {
            (wte.waste_capacity_tons / total_capacity) * available_waste
        } else {
            0.0
        };

        // Clamp to plant capacity.
        let waste_tons = waste_share.min(wte.waste_capacity_tons);
        wte.current_waste_tons = waste_tons;

        // Calculate energy output.
        let output_mw = calculate_wte_output_mw(waste_tons, btu_per_lb);
        plant.current_output_mw = output_mw;
        plant.capacity_mw = calculate_wte_output_mw(wte.waste_capacity_tons, btu_per_lb);

        // Calculate ash residue.
        let ash_tons = waste_tons * WTE_ASH_FRACTION;

        // Calculate economics.
        let operating_cost = waste_tons * WTE_OPERATING_COST_PER_TON;
        let tipping_revenue = waste_tons * WTE_TIPPING_FEE_PER_TON;

        total_waste_consumed += waste_tons;
        total_output += output_mw;
        total_ash += ash_tons;
        total_operating_cost += operating_cost;
        total_tipping_revenue += tipping_revenue;
    }

    wte_state.plant_count = total_count;
    wte_state.total_waste_consumed_tons = total_waste_consumed;
    wte_state.total_output_mw = total_output;
    wte_state.total_ash_tons = total_ash;
    wte_state.total_operating_cost = total_operating_cost;
    wte_state.total_tipping_revenue = total_tipping_revenue;
    wte_state.current_pollution_q = pollution_q;

    // Add WTE generation to the energy grid supply.
    energy_grid.total_supply_mwh += total_output;
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers waste-to-energy plant resources and systems.
pub struct WtePlugin;

impl Plugin for WtePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WteState>().add_systems(
            FixedUpdate,
            update_wte_plants
                .after(crate::wind_pollution::update_pollution_gaussian_plume)
                .after(crate::energy_dispatch::dispatch_energy)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WteState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wte_output_formula_default() {
        let btu = WasteComposition::default().energy_content_btu_per_lb();
        let output = calculate_wte_output_mw(WTE_DEFAULT_WASTE_TONS, btu);
        // Expected: ~17.5 MW for 500 tons/day with ~5443 BTU/lb MSW
        assert!(
            output > 10.0 && output < 25.0,
            "Expected ~15-20 MW for default 500 tons/day, got {output}"
        );
    }

    #[test]
    fn test_wte_output_scales_linearly() {
        let btu = WasteComposition::default().energy_content_btu_per_lb();
        let output_500 = calculate_wte_output_mw(500.0, btu);
        let output_1000 = calculate_wte_output_mw(1000.0, btu);
        let ratio = output_1000 / output_500;
        assert!(
            (ratio - 2.0).abs() < 0.01,
            "Output should scale linearly: ratio = {ratio}"
        );
    }

    #[test]
    fn test_wte_output_zero_waste() {
        let btu = WasteComposition::default().energy_content_btu_per_lb();
        let output = calculate_wte_output_mw(0.0, btu);
        assert!(output.abs() < f32::EPSILON, "Zero waste = zero output");
    }

    #[test]
    fn test_wte_plant_new() {
        let plant = WtePlant::new(10, 20);
        assert_eq!(plant.grid_x, 10);
        assert_eq!(plant.grid_y, 20);
        assert!((plant.waste_capacity_tons - WTE_DEFAULT_WASTE_TONS).abs() < f32::EPSILON);
        assert!((plant.current_waste_tons).abs() < f32::EPSILON);
    }

    #[test]
    fn test_wte_plant_with_capacity_clamped() {
        let plant = WtePlant::with_capacity(0, 0, 50.0);
        assert!(
            (plant.waste_capacity_tons - WTE_MIN_WASTE_TONS).abs() < f32::EPSILON,
            "Should clamp to min: got {}",
            plant.waste_capacity_tons
        );

        let plant2 = WtePlant::with_capacity(0, 0, 5000.0);
        assert!(
            (plant2.waste_capacity_tons - WTE_MAX_WASTE_TONS).abs() < f32::EPSILON,
            "Should clamp to max: got {}",
            plant2.waste_capacity_tons
        );
    }

    #[test]
    fn test_power_plant_new_wte() {
        let plant = PowerPlant::new_wte(5, 5);
        assert_eq!(plant.plant_type, PowerPlantType::WasteToEnergy);
        assert!(
            plant.capacity_mw > 10.0 && plant.capacity_mw < 25.0,
            "Expected 10-25 MW capacity, got {}",
            plant.capacity_mw
        );
        assert!((plant.fuel_cost - WTE_FUEL_COST_PER_MWH).abs() < f32::EPSILON);
        assert_eq!(plant.grid_x, 5);
        assert_eq!(plant.grid_y, 5);
    }

    #[test]
    fn test_wte_state_default() {
        let state = WteState::default();
        assert_eq!(state.plant_count, 0);
        assert!(state.total_output_mw.abs() < f32::EPSILON);
        assert!(state.total_waste_consumed_tons.abs() < f32::EPSILON);
        assert!(state.total_ash_tons.abs() < f32::EPSILON);
        assert!(state.scrubbers_installed);
        assert!(
            (state.current_pollution_q - WTE_POLLUTION_Q_SCRUBBED).abs() < f32::EPSILON
        );
    }

    #[test]
    fn test_wte_state_save_skip_empty() {
        use crate::Saveable;
        let state = WteState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should not produce save bytes"
        );
    }

    #[test]
    fn test_wte_state_roundtrip() {
        use crate::Saveable;
        let state = WteState {
            plant_count: 2,
            total_waste_consumed_tons: 800.0,
            total_output_mw: 28.0,
            total_ash_tons: 80.0,
            total_operating_cost: 40000.0,
            total_tipping_revenue: 52000.0,
            scrubbers_installed: true,
            current_pollution_q: WTE_POLLUTION_Q_SCRUBBED,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = WteState::load_from_bytes(&bytes);
        assert_eq!(loaded.plant_count, 2);
        assert!((loaded.total_output_mw - 28.0).abs() < 0.1);
        assert!((loaded.total_waste_consumed_tons - 800.0).abs() < f32::EPSILON);
        assert!((loaded.total_ash_tons - 80.0).abs() < f32::EPSILON);
        assert!(loaded.scrubbers_installed);
    }

    #[test]
    fn test_wte_footprint() {
        assert_eq!(WTE_FOOTPRINT, (4, 4));
    }

    #[test]
    fn test_ash_fraction() {
        let waste_tons = 500.0f32;
        let ash = waste_tons * WTE_ASH_FRACTION;
        assert!((ash - 50.0).abs() < f32::EPSILON, "10% of 500 = 50 tons ash");
    }

    #[test]
    fn test_wte_economics() {
        let waste_tons = 500.0f32;
        let cost = waste_tons * WTE_OPERATING_COST_PER_TON;
        let revenue = waste_tons * WTE_TIPPING_FEE_PER_TON;
        assert!(
            revenue > cost,
            "Tipping fees should exceed operating costs: revenue={revenue}, cost={cost}"
        );
    }

    #[test]
    fn test_pollution_q_values() {
        assert!(WTE_POLLUTION_Q_RAW > WTE_POLLUTION_Q_SCRUBBED);
        assert!((WTE_POLLUTION_Q_RAW - 45.0).abs() < f32::EPSILON);
        assert!((WTE_POLLUTION_Q_SCRUBBED - 20.0).abs() < f32::EPSILON);
    }
}
