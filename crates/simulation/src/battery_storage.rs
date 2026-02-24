//! POWER-008: Battery Energy Storage System
//!
//! Implements battery storage facilities that store excess electricity during
//! off-peak periods and discharge during peak demand. Features:
//!
//! - Two tiers: Small (10 MWh, 5 MW rate, $5M) and Large (100 MWh, 50 MW rate, $40M)
//! - Charges when supply > demand; discharges when demand > supply
//! - Round-trip efficiency: 85% (15% energy loss on discharge)
//! - State of charge (SOC) tracked: 0–100%
//! - Reserve threshold: 20% minimum stored energy

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::energy_demand::EnergyGrid;
use crate::energy_dispatch::EnergyDispatchState;
use crate::{decode_or_warn, Saveable, SimulationSet, TickCounter};

// =============================================================================
// Constants
// =============================================================================

/// How often (in ticks) the battery system runs.
const BATTERY_INTERVAL: u64 = 4;

/// Round-trip efficiency: 85% — for every 1 MWh stored, 0.85 MWh is recovered.
const ROUND_TRIP_EFFICIENCY: f32 = 0.85;

/// Minimum state-of-charge reserve threshold (20%).
/// Batteries will not discharge below this level under normal operation.
const RESERVE_THRESHOLD: f32 = 0.20;

// =============================================================================
// BatteryTier
// =============================================================================

/// Tier of battery storage facility.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode, Default,
)]
pub enum BatteryTier {
    /// Small battery: 10 MWh capacity, 5 MW charge/discharge rate, $5M cost.
    #[default]
    Small,
    /// Large battery: 100 MWh capacity, 50 MW charge/discharge rate, $40M cost.
    Large,
}

impl BatteryTier {
    /// Energy capacity in MWh.
    pub fn capacity_mwh(self) -> f32 {
        match self {
            BatteryTier::Small => 10.0,
            BatteryTier::Large => 100.0,
        }
    }

    /// Maximum charge/discharge rate in MW.
    pub fn max_rate_mw(self) -> f32 {
        match self {
            BatteryTier::Small => 5.0,
            BatteryTier::Large => 50.0,
        }
    }

    /// Construction cost in dollars.
    pub fn cost(self) -> f64 {
        match self {
            BatteryTier::Small => 5_000_000.0,
            BatteryTier::Large => 40_000_000.0,
        }
    }
}

// =============================================================================
// BatteryUnit — individual battery instance
// =============================================================================

/// State of a single battery storage unit.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BatteryUnit {
    /// Tier of this battery (Small or Large).
    pub tier: BatteryTier,
    /// Current stored energy in MWh.
    pub stored_mwh: f32,
    /// Grid position X.
    pub grid_x: usize,
    /// Grid position Y.
    pub grid_y: usize,
}

impl BatteryUnit {
    /// Create a new battery unit at the given position, starting empty.
    pub fn new(tier: BatteryTier, grid_x: usize, grid_y: usize) -> Self {
        Self {
            tier,
            stored_mwh: 0.0,
            grid_x,
            grid_y,
        }
    }

    /// Capacity of this unit in MWh.
    pub fn capacity(&self) -> f32 {
        self.tier.capacity_mwh()
    }

    /// Maximum charge/discharge rate in MW.
    pub fn max_rate(&self) -> f32 {
        self.tier.max_rate_mw()
    }

    /// State of charge as a fraction (0.0–1.0).
    pub fn soc(&self) -> f32 {
        if self.capacity() == 0.0 {
            return 0.0;
        }
        (self.stored_mwh / self.capacity()).clamp(0.0, 1.0)
    }

    /// Available energy for discharge (above reserve threshold), in MWh.
    pub fn available_discharge_mwh(&self) -> f32 {
        let reserve = self.capacity() * RESERVE_THRESHOLD;
        (self.stored_mwh - reserve).max(0.0)
    }

    /// Available capacity for charging, in MWh.
    pub fn available_charge_mwh(&self) -> f32 {
        (self.capacity() - self.stored_mwh).max(0.0)
    }

    /// Charge this battery by the given amount (clamped to capacity and rate).
    /// Returns the actual amount charged in MWh.
    pub fn charge(&mut self, mwh: f32) -> f32 {
        let max_charge = self.available_charge_mwh().min(self.max_rate());
        let actual = mwh.min(max_charge).max(0.0);
        self.stored_mwh = (self.stored_mwh + actual).min(self.capacity());
        actual
    }

    /// Discharge this battery by the given amount (clamped to available and rate).
    /// Returns the actual AC output in MWh (after round-trip efficiency).
    pub fn discharge(&mut self, requested_mwh: f32) -> f32 {
        let max_discharge = self.available_discharge_mwh().min(self.max_rate());
        // We need to draw more from storage than what we deliver, due to losses
        let actual_draw = requested_mwh.min(max_discharge).max(0.0);
        self.stored_mwh = (self.stored_mwh - actual_draw).max(0.0);
        // Apply round-trip efficiency: deliver less than drawn
        actual_draw * ROUND_TRIP_EFFICIENCY
    }
}

// =============================================================================
// BatteryState resource
// =============================================================================

/// City-wide battery storage state.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BatteryState {
    /// All battery units in the city.
    pub units: Vec<BatteryUnit>,
    /// Total energy currently stored across all batteries (MWh).
    pub total_stored_mwh: f32,
    /// Total capacity across all batteries (MWh).
    pub total_capacity_mwh: f32,
    /// Aggregate state of charge (0.0–1.0).
    pub aggregate_soc: f32,
    /// Energy charged this cycle (MWh).
    pub last_charge_mwh: f32,
    /// Energy discharged this cycle (MWh, after efficiency).
    pub last_discharge_mwh: f32,
    /// Number of battery units.
    pub unit_count: u32,
}

impl Default for BatteryState {
    fn default() -> Self {
        Self {
            units: Vec::new(),
            total_stored_mwh: 0.0,
            total_capacity_mwh: 0.0,
            aggregate_soc: 0.0,
            last_charge_mwh: 0.0,
            last_discharge_mwh: 0.0,
            unit_count: 0,
        }
    }
}

impl Saveable for BatteryState {
    const SAVE_KEY: &'static str = "battery_storage";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.units.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

impl BatteryState {
    /// Recalculate aggregate fields from individual units.
    pub fn recalculate_aggregates(&mut self) {
        self.unit_count = self.units.len() as u32;
        self.total_stored_mwh = self.units.iter().map(|u| u.stored_mwh).sum();
        self.total_capacity_mwh = self.units.iter().map(|u| u.capacity()).sum();
        self.aggregate_soc = if self.total_capacity_mwh > 0.0 {
            self.total_stored_mwh / self.total_capacity_mwh
        } else {
            0.0
        };
    }

    /// Add a new battery unit to the city.
    pub fn add_battery(&mut self, unit: BatteryUnit) {
        self.units.push(unit);
        self.recalculate_aggregates();
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Main battery charge/discharge system.
///
/// Runs every `BATTERY_INTERVAL` ticks after the energy dispatch system.
/// - If supply > demand: charge batteries with excess energy
/// - If demand > supply: discharge batteries to cover the deficit
pub fn battery_charge_discharge(
    tick: Res<TickCounter>,
    energy_grid: Res<EnergyGrid>,
    mut battery_state: ResMut<BatteryState>,
    mut dispatch_state: ResMut<EnergyDispatchState>,
) {
    if !tick.0.is_multiple_of(BATTERY_INTERVAL) {
        return;
    }

    if battery_state.units.is_empty() {
        battery_state.last_charge_mwh = 0.0;
        battery_state.last_discharge_mwh = 0.0;
        return;
    }

    let supply = energy_grid.total_supply_mwh;
    let demand = energy_grid.total_demand_mwh;

    let mut total_charged = 0.0_f32;
    let mut total_discharged = 0.0_f32;

    if supply > demand {
        // Excess energy — charge batteries
        let mut excess = supply - demand;
        for unit in &mut battery_state.units {
            if excess <= 0.0 {
                break;
            }
            let charged = unit.charge(excess);
            total_charged += charged;
            excess -= charged;
        }
    } else if demand > supply {
        // Deficit — discharge batteries to cover gap
        let mut deficit = demand - supply;
        for unit in &mut battery_state.units {
            if deficit <= 0.0 {
                break;
            }
            let discharged = unit.discharge(deficit);
            total_discharged += discharged;
            deficit -= discharged;
        }

        // If batteries covered some of the deficit, reduce load shedding
        if total_discharged > 0.0 && dispatch_state.active {
            let original_deficit = demand - supply;
            let remaining_deficit = (original_deficit - total_discharged).max(0.0);
            if demand > 0.0 {
                dispatch_state.load_shed_fraction =
                    (remaining_deficit / demand).clamp(0.0, 1.0);
            }
            if remaining_deficit < 0.01 {
                dispatch_state.has_deficit = false;
            }
        }
    }

    battery_state.last_charge_mwh = total_charged;
    battery_state.last_discharge_mwh = total_discharged;
    battery_state.recalculate_aggregates();
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin for the battery energy storage system.
pub struct BatteryStoragePlugin;

impl Plugin for BatteryStoragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BatteryState>();

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<BatteryState>();

        app.add_systems(
            FixedUpdate,
            battery_charge_discharge
                .after(crate::energy_dispatch::dispatch_energy)
                .in_set(SimulationSet::Simulation),
        );
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_tier_specs() {
        assert!((BatteryTier::Small.capacity_mwh() - 10.0).abs() < f32::EPSILON);
        assert!((BatteryTier::Small.max_rate_mw() - 5.0).abs() < f32::EPSILON);
        assert!((BatteryTier::Small.cost() - 5_000_000.0).abs() < f64::EPSILON);

        assert!((BatteryTier::Large.capacity_mwh() - 100.0).abs() < f32::EPSILON);
        assert!((BatteryTier::Large.max_rate_mw() - 50.0).abs() < f32::EPSILON);
        assert!((BatteryTier::Large.cost() - 40_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_battery_unit_soc() {
        let mut unit = BatteryUnit::new(BatteryTier::Small, 0, 0);
        assert!((unit.soc() - 0.0).abs() < f32::EPSILON);

        unit.stored_mwh = 5.0;
        assert!((unit.soc() - 0.5).abs() < f32::EPSILON);

        unit.stored_mwh = 10.0;
        assert!((unit.soc() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_battery_charge_clamps_to_capacity() {
        let mut unit = BatteryUnit::new(BatteryTier::Small, 0, 0);
        // Try to charge 20 MWh into a 10 MWh battery with 5 MW rate
        let charged = unit.charge(20.0);
        // Should be clamped to rate (5 MW)
        assert!((charged - 5.0).abs() < f32::EPSILON);
        assert!((unit.stored_mwh - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_battery_discharge_respects_reserve() {
        let mut unit = BatteryUnit::new(BatteryTier::Small, 0, 0);
        unit.stored_mwh = 3.0; // 30% SOC
        // Reserve is 20% = 2 MWh, so only 1 MWh available
        let available = unit.available_discharge_mwh();
        assert!((available - 1.0).abs() < f32::EPSILON);

        let discharged = unit.discharge(10.0);
        // Should get 1.0 * 0.85 = 0.85 MWh
        assert!((discharged - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_battery_discharge_efficiency() {
        let mut unit = BatteryUnit::new(BatteryTier::Large, 0, 0);
        unit.stored_mwh = 100.0; // Full
        // Available = 100 - 20 (reserve) = 80, rate limit = 50 MW
        let discharged = unit.discharge(50.0);
        // Should get 50 * 0.85 = 42.5 MWh
        assert!((discharged - 42.5).abs() < 0.01);
        assert!((unit.stored_mwh - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_battery_state_default() {
        let state = BatteryState::default();
        assert!(state.units.is_empty());
        assert_eq!(state.unit_count, 0);
        assert!((state.total_stored_mwh).abs() < f32::EPSILON);
    }

    #[test]
    fn test_battery_state_add_and_aggregate() {
        let mut state = BatteryState::default();
        let mut unit = BatteryUnit::new(BatteryTier::Small, 0, 0);
        unit.stored_mwh = 5.0;
        state.add_battery(unit);

        assert_eq!(state.unit_count, 1);
        assert!((state.total_capacity_mwh - 10.0).abs() < f32::EPSILON);
        assert!((state.total_stored_mwh - 5.0).abs() < f32::EPSILON);
        assert!((state.aggregate_soc - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = BatteryState::default();
        state.add_battery(BatteryUnit::new(BatteryTier::Small, 10, 20));
        state.add_battery(BatteryUnit::new(BatteryTier::Large, 30, 40));
        state.units[0].stored_mwh = 5.0;
        state.units[1].stored_mwh = 50.0;
        state.recalculate_aggregates();

        let bytes = state.save_to_bytes().unwrap();
        let restored = BatteryState::load_from_bytes(&bytes);

        assert_eq!(restored.units.len(), 2);
        assert!((restored.units[0].stored_mwh - 5.0).abs() < f32::EPSILON);
        assert!((restored.units[1].stored_mwh - 50.0).abs() < f32::EPSILON);
        assert_eq!(restored.unit_count, 2);
    }

    #[test]
    fn test_save_skip_empty() {
        let state = BatteryState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_reserve_threshold_at_boundary() {
        let mut unit = BatteryUnit::new(BatteryTier::Small, 0, 0);
        // Exactly at reserve (20% of 10 MWh = 2 MWh)
        unit.stored_mwh = 2.0;
        assert!(unit.available_discharge_mwh().abs() < f32::EPSILON);

        // Slightly above reserve
        unit.stored_mwh = 2.1;
        assert!(unit.available_discharge_mwh() > 0.0);
    }
}
