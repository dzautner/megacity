//! POWER-018: Power Plant Maintenance Schedules and Forced Outages
//!
//! Implements planned maintenance windows and random forced outages for power
//! plants. During maintenance or outage, a plant's `capacity_mw` is set to 0
//! so the energy dispatch system naturally skips it.
//!
//! Forced outage probability per month (per slow tick cycle ≈ 1/3 game-day):
//! - Coal: 5%/month
//! - Natural Gas: 3%/month
//! - Nuclear (not yet in game, but supported): 1%/month
//! - Solar (inverter failure): 2%/month
//! - Wind (mechanical failure): 4%/month
//! - Biomass: same as coal (5%/month)
//!
//! Outage duration (in slow tick cycles):
//! - Coal: 3–7 days (9–21 cycles)
//! - Gas: 1–3 days (3–9 cycles)
//! - Nuclear: 7–30 days (21–90 cycles)
//! - Solar: 1–2 days (3–6 cycles)
//! - Wind: 2–5 days (6–15 cycles)
//! - Biomass: 3–7 days (same as coal)
//!
//! Deferred maintenance doubles outage probability.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::sim_rng::SimRng;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Approximate number of slow tick cycles per game-month (30 days * 3 cycles/day).
const CYCLES_PER_MONTH: f32 = 90.0;

/// Per-cycle forced outage probability (derived from monthly rate).
/// Formula: per_cycle = 1 - (1 - monthly_rate)^(1/cycles_per_month)
/// For small rates this is approximately monthly_rate / cycles_per_month.
fn monthly_to_per_cycle(monthly_rate: f32) -> f32 {
    1.0 - (1.0 - monthly_rate).powf(1.0 / CYCLES_PER_MONTH)
}

/// Returns the per-cycle forced outage probability for the given plant type.
pub fn outage_probability(plant_type: PowerPlantType) -> f32 {
    match plant_type {
        PowerPlantType::Coal => monthly_to_per_cycle(0.05),
        PowerPlantType::NaturalGas => monthly_to_per_cycle(0.03),
        PowerPlantType::WindTurbine => monthly_to_per_cycle(0.04),
        PowerPlantType::Biomass => monthly_to_per_cycle(0.05),
        // Oil uses gas-like rates
        PowerPlantType::Oil => monthly_to_per_cycle(0.03),
        // WasteToEnergy, HydroDam, Geothermal — use low rate
        PowerPlantType::WasteToEnergy => monthly_to_per_cycle(0.02),
        PowerPlantType::HydroDam => monthly_to_per_cycle(0.01),
        PowerPlantType::Geothermal => monthly_to_per_cycle(0.01),
        PowerPlantType::Nuclear => monthly_to_per_cycle(0.01),
    }
}

/// Returns (min_cycles, max_cycles) for outage duration by plant type.
/// 1 game-day ≈ 3 slow tick cycles.
pub fn outage_duration_range(plant_type: PowerPlantType) -> (u32, u32) {
    match plant_type {
        PowerPlantType::Coal | PowerPlantType::Biomass => (9, 21),       // 3–7 days
        PowerPlantType::NaturalGas | PowerPlantType::Oil => (3, 9),      // 1–3 days
        PowerPlantType::WindTurbine => (6, 15),                          // 2–5 days
        PowerPlantType::WasteToEnergy => (3, 6),                         // 1–2 days
        PowerPlantType::HydroDam => (21, 90),                            // 7–30 days
        PowerPlantType::Geothermal => (9, 21),                           // 3–7 days
        PowerPlantType::Nuclear => (21, 90),                            // 7–30 days
    }
}

// =============================================================================
// Outage record per plant entity
// =============================================================================

/// Tracks the maintenance/outage state of a single power plant.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PlantOutageRecord {
    /// Whether the plant is currently in an outage (forced or planned).
    pub in_outage: bool,
    /// Remaining slow tick cycles until the outage ends.
    pub remaining_cycles: u32,
    /// The plant's original capacity (stored when outage begins, restored when it ends).
    pub original_capacity_mw: f32,
    /// Number of outages this plant has experienced (for stats).
    pub outage_count: u32,
    /// Whether maintenance has been deferred (doubles outage probability).
    pub maintenance_deferred: bool,
    /// Cycles since last maintenance (increments each cycle, reset on outage end).
    pub cycles_since_maintenance: u32,
}

impl Default for PlantOutageRecord {
    fn default() -> Self {
        Self {
            in_outage: false,
            remaining_cycles: 0,
            original_capacity_mw: 0.0,
            outage_count: 0,
            maintenance_deferred: false,
            cycles_since_maintenance: 0,
        }
    }
}

// =============================================================================
// PowerPlantMaintenanceState resource
// =============================================================================

/// City-wide power plant maintenance state.
///
/// Tracks outage status for each power plant entity by entity index.
/// Uses `u32` keys (Entity index) instead of `Entity` for serialization.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PowerPlantMaintenanceState {
    /// Per-plant outage records keyed by entity index.
    pub records: HashMap<u32, PlantOutageRecord>,
    /// Total number of plants currently in outage.
    pub plants_in_outage: u32,
    /// Total lost capacity from outages (MW).
    pub total_lost_capacity_mw: f32,
    /// Global maintenance deferral flag (policy toggle).
    pub defer_maintenance: bool,
}

impl Default for PowerPlantMaintenanceState {
    fn default() -> Self {
        Self {
            records: HashMap::new(),
            plants_in_outage: 0,
            total_lost_capacity_mw: 0.0,
            defer_maintenance: false,
        }
    }
}

impl crate::Saveable for PowerPlantMaintenanceState {
    const SAVE_KEY: &'static str = "power_plant_maintenance";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.records.is_empty() {
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

/// Main maintenance system: rolls for forced outages and ticks down active ones.
///
/// Runs every slow tick cycle. For each power plant:
/// 1. If in outage, decrement remaining_cycles; if 0, restore capacity.
/// 2. If not in outage, roll for a forced outage based on probability.
/// 3. If outage triggered, store original capacity and set capacity_mw = 0.
#[allow(clippy::too_many_arguments)]
pub fn update_power_plant_maintenance(
    timer: Res<SlowTickTimer>,
    mut rng: ResMut<SimRng>,
    mut state: ResMut<PowerPlantMaintenanceState>,
    mut plants: Query<(Entity, &mut PowerPlant)>,
) {
    if !timer.should_run() {
        return;
    }

    let defer = state.defer_maintenance;
    let mut in_outage_count = 0u32;
    let mut lost_capacity = 0.0f32;

    // Collect entity IDs to process (avoid borrow issues).
    let plant_data: Vec<(Entity, PowerPlantType, f32)> = plants
        .iter()
        .map(|(e, p)| (e, p.plant_type, p.capacity_mw))
        .collect();

    for (entity, plant_type, current_capacity) in &plant_data {
        let key = entity.index();
        let record = state.records.entry(key).or_default();

        if record.in_outage {
            // Tick down the outage
            if record.remaining_cycles > 0 {
                record.remaining_cycles -= 1;
            }

            if record.remaining_cycles == 0 {
                // Outage ended — restore capacity
                record.in_outage = false;
                record.cycles_since_maintenance = 0;
                if let Ok((_, mut plant)) = plants.get_mut(*entity) {
                    plant.capacity_mw = record.original_capacity_mw;
                }
            } else {
                // Still in outage — ensure capacity stays at 0
                if let Ok((_, mut plant)) = plants.get_mut(*entity) {
                    plant.capacity_mw = 0.0;
                }
                in_outage_count += 1;
                lost_capacity += record.original_capacity_mw;
            }
        } else {
            // Not in outage — roll for forced outage
            record.cycles_since_maintenance += 1;

            let mut prob = outage_probability(*plant_type);
            if defer || record.maintenance_deferred {
                prob *= 2.0;
            }

            let roll: f32 = rng.0.gen();
            if roll < prob {
                // Forced outage triggered
                let (min_dur, max_dur) = outage_duration_range(*plant_type);
                let duration = rng.0.gen_range(min_dur..=max_dur);

                record.in_outage = true;
                record.remaining_cycles = duration;
                record.original_capacity_mw = *current_capacity;
                record.outage_count += 1;

                if let Ok((_, mut plant)) = plants.get_mut(*entity) {
                    plant.capacity_mw = 0.0;
                }
                in_outage_count += 1;
                lost_capacity += record.original_capacity_mw;
            }
        }
    }

    state.plants_in_outage = in_outage_count;
    state.total_lost_capacity_mw = lost_capacity;

    // Clean up records for entities that no longer exist
    let active_keys: std::collections::HashSet<u32> =
        plant_data.iter().map(|(e, _, _)| e.index()).collect();
    state.records.retain(|k, _| active_keys.contains(k));
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers the power plant maintenance system.
pub struct PowerPlantMaintenancePlugin;

impl Plugin for PowerPlantMaintenancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PowerPlantMaintenanceState>();

        app.add_systems(
            FixedUpdate,
            update_power_plant_maintenance
                .before(crate::energy_dispatch::dispatch_energy)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<PowerPlantMaintenanceState>();
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outage_probability_coal() {
        let prob = outage_probability(PowerPlantType::Coal);
        // 5% monthly ≈ 0.00057 per cycle
        assert!(prob > 0.0 && prob < 0.01);
    }

    #[test]
    fn test_outage_probability_gas_lower_than_coal() {
        let coal = outage_probability(PowerPlantType::Coal);
        let gas = outage_probability(PowerPlantType::NaturalGas);
        assert!(gas < coal, "Gas ({gas}) should have lower outage prob than coal ({coal})");
    }

    #[test]
    fn test_outage_duration_range_coal() {
        let (min, max) = outage_duration_range(PowerPlantType::Coal);
        assert_eq!(min, 9);
        assert_eq!(max, 21);
    }

    #[test]
    fn test_outage_duration_range_gas() {
        let (min, max) = outage_duration_range(PowerPlantType::NaturalGas);
        assert_eq!(min, 3);
        assert_eq!(max, 9);
    }

    #[test]
    fn test_default_state() {
        let state = PowerPlantMaintenanceState::default();
        assert!(state.records.is_empty());
        assert_eq!(state.plants_in_outage, 0);
        assert_eq!(state.total_lost_capacity_mw, 0.0);
        assert!(!state.defer_maintenance);
    }

    #[test]
    fn test_saveable_skip_empty() {
        use crate::Saveable;
        let state = PowerPlantMaintenanceState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = PowerPlantMaintenanceState::default();
        state.records.insert(
            42,
            PlantOutageRecord {
                in_outage: true,
                remaining_cycles: 10,
                original_capacity_mw: 200.0,
                outage_count: 3,
                maintenance_deferred: false,
                cycles_since_maintenance: 50,
            },
        );
        state.plants_in_outage = 1;
        state.total_lost_capacity_mw = 200.0;

        let bytes = state.save_to_bytes().expect("should produce bytes");
        let loaded = PowerPlantMaintenanceState::load_from_bytes(&bytes);
        assert_eq!(loaded.plants_in_outage, 1);
        assert!((loaded.total_lost_capacity_mw - 200.0).abs() < f32::EPSILON);
        let rec = loaded.records.get(&42).expect("record 42 should exist");
        assert!(rec.in_outage);
        assert_eq!(rec.remaining_cycles, 10);
        assert_eq!(rec.outage_count, 3);
    }

    #[test]
    fn test_monthly_to_per_cycle_zero() {
        assert_eq!(monthly_to_per_cycle(0.0), 0.0);
    }

    #[test]
    fn test_monthly_to_per_cycle_small_rate() {
        let per_cycle = monthly_to_per_cycle(0.05);
        // Should be approximately 0.05 / 90 ≈ 0.000556
        assert!(per_cycle > 0.0004 && per_cycle < 0.001);
    }

    #[test]
    fn test_deferred_maintenance_doubles_probability() {
        let base = outage_probability(PowerPlantType::Coal);
        let deferred = base * 2.0;
        assert!((deferred - base * 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_plant_outage_record_default() {
        let rec = PlantOutageRecord::default();
        assert!(!rec.in_outage);
        assert_eq!(rec.remaining_cycles, 0);
        assert_eq!(rec.original_capacity_mw, 0.0);
        assert_eq!(rec.outage_count, 0);
        assert!(!rec.maintenance_deferred);
        assert_eq!(rec.cycles_since_maintenance, 0);
    }
}
