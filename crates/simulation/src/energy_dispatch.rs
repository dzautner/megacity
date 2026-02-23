//! Energy Dispatch Merit Order System (POWER-009)
//!
//! Implements the merit order dispatch algorithm that determines which generators
//! run to meet demand. Generators are dispatched in order of marginal cost
//! (`fuel_cost` field on `PowerPlant`): cheapest first, most expensive last.
//!
//! Dispatch order by fuel cost: renewables ($0) -> nuclear ($10) -> coal ($30)
//!   -> gas ($40) -> gas peaker ($80)
//!
//! The system runs every 4 ticks, sets each generator's `current_output_mw`,
//! calculates reserve margin, handles load shedding on deficit, and determines
//! the electricity price based on the marginal cost of the last dispatched unit.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::coal_power::PowerPlant;
use crate::energy_demand::EnergyGrid;
use crate::{decode_or_warn, Saveable, SimulationSet, TickCounter};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// How often (in ticks) the dispatch system runs.
const DISPATCH_INTERVAL: u64 = 4;

/// Reserve margin threshold below which scarcity pricing kicks in.
const SCARCITY_THRESHOLD: f32 = 0.1;

/// Maximum scarcity multiplier applied to the electricity price.
const MAX_SCARCITY_MULTIPLIER: f32 = 3.0;

/// Minimum demand threshold (MW) to trigger dispatch.
/// Below this, the system skips dispatch to avoid interfering with
/// plants' construction-time default output values.
const MIN_DEMAND_THRESHOLD: f32 = 0.01;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Dispatch-specific state tracking electricity price, deficit, and blackout.
///
/// Separate from `EnergyGrid` to avoid conflicts with other POWER modules
/// that contribute supply/demand data.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct EnergyDispatchState {
    /// Electricity price determined by the marginal cost of the last dispatched
    /// unit, scaled by a scarcity multiplier ($/MWh).
    pub electricity_price: f32,
    /// Whether the city is currently experiencing a power deficit.
    pub has_deficit: bool,
    /// Total available capacity across all generators (MW).
    pub total_capacity_mw: f32,
    /// Number of cells currently affected by rolling blackout.
    pub blackout_cells: u32,
    /// Rolling blackout rotation offset — incremented each dispatch tick during
    /// deficit to rotate which cells are affected.
    pub blackout_rotation: u32,
    /// Fraction of demand that is being shed (0.0 = none, 1.0 = total blackout).
    pub load_shed_fraction: f32,
    /// Number of generators that were dispatched this cycle.
    pub dispatched_count: u32,
    /// Whether the dispatch system has been activated (demand > threshold).
    pub active: bool,
}

impl Default for EnergyDispatchState {
    fn default() -> Self {
        Self {
            electricity_price: 0.0,
            has_deficit: false,
            total_capacity_mw: 0.0,
            blackout_cells: 0,
            blackout_rotation: 0,
            load_shed_fraction: 0.0,
            dispatched_count: 0,
            active: false,
        }
    }
}

impl Saveable for EnergyDispatchState {
    const SAVE_KEY: &'static str = "energy_dispatch";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Dispatch logic
// ---------------------------------------------------------------------------

/// Entry for sorting generators in the merit order.
struct DispatchEntry {
    entity: Entity,
    capacity_mw: f32,
    fuel_cost: f32,
}

/// Run the merit order dispatch algorithm.
///
/// 1. Collect all generators and sort by fuel_cost (ascending).
/// 2. Dispatch cheapest first until demand is met.
/// 3. Set each generator's `current_output_mw`.
/// 4. Calculate reserve margin, electricity price, and deficit state.
///
/// When demand is below `MIN_DEMAND_THRESHOLD`, the system skips dispatch
/// entirely to preserve plants' default output values set at construction.
pub fn dispatch_energy(
    tick: Res<TickCounter>,
    mut energy_grid: ResMut<EnergyGrid>,
    mut dispatch_state: ResMut<EnergyDispatchState>,
    mut plants: Query<(Entity, &mut PowerPlant)>,
) {
    if !tick.0.is_multiple_of(DISPATCH_INTERVAL) {
        return;
    }

    let demand = energy_grid.total_demand_mwh;

    // Skip dispatch when there is negligible demand.
    // This preserves plants' construction-time output values for the
    // aggregate systems (coal_power, gas_power) that read current_output_mw.
    if demand < MIN_DEMAND_THRESHOLD {
        if dispatch_state.active {
            dispatch_state.active = false;
            dispatch_state.has_deficit = false;
            dispatch_state.load_shed_fraction = 0.0;
            dispatch_state.electricity_price = 0.0;
            dispatch_state.blackout_cells = 0;
        }
        return;
    }
    dispatch_state.active = true;

    // Phase 1: Collect generators and sort by fuel_cost (merit order).
    let mut entries: Vec<DispatchEntry> = Vec::new();
    let mut total_capacity: f32 = 0.0;

    for (entity, plant) in &plants {
        total_capacity += plant.capacity_mw;
        entries.push(DispatchEntry {
            entity,
            capacity_mw: plant.capacity_mw,
            fuel_cost: plant.fuel_cost,
        });
    }

    // Sort by fuel cost ascending; break ties by larger capacity first.
    entries.sort_by(|a, b| {
        a.fuel_cost
            .partial_cmp(&b.fuel_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.capacity_mw
                    .partial_cmp(&a.capacity_mw)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    // Phase 2: Dispatch generators to meet demand.
    let mut remaining_demand = demand;
    let mut total_dispatched: f32 = 0.0;
    let mut last_dispatched_cost: f32 = 0.0;
    let mut dispatched_count: u32 = 0;

    // Reset all generators to zero output before redispatching.
    for (_, mut plant) in &mut plants {
        plant.current_output_mw = 0.0;
    }

    // Dispatch in merit order.
    for entry in &entries {
        if remaining_demand <= 0.0 {
            break;
        }

        let output = entry.capacity_mw.min(remaining_demand);
        remaining_demand -= output;
        total_dispatched += output;
        last_dispatched_cost = entry.fuel_cost;
        dispatched_count += 1;

        if let Ok((_, mut plant)) = plants.get_mut(entry.entity) {
            plant.current_output_mw = output;
        }
    }

    // Phase 3: Update EnergyGrid supply.
    energy_grid.total_supply_mwh = total_dispatched;

    // Phase 4: Update reserve margin.
    energy_grid.reserve_margin = (total_capacity - demand) / demand;

    // Phase 5: Handle deficit.
    let has_deficit = total_dispatched < demand;
    dispatch_state.has_deficit = has_deficit;
    dispatch_state.total_capacity_mw = total_capacity;
    dispatch_state.dispatched_count = dispatched_count;

    if has_deficit {
        let deficit = demand - total_dispatched;
        dispatch_state.load_shed_fraction = (deficit / demand).clamp(0.0, 1.0);
        dispatch_state.blackout_rotation =
            dispatch_state.blackout_rotation.wrapping_add(1);
    } else {
        dispatch_state.load_shed_fraction = 0.0;
        dispatch_state.blackout_cells = 0;
    }

    // Phase 6: Calculate electricity price.
    let base_price = last_dispatched_cost;

    let scarcity_multiplier = if energy_grid.reserve_margin < SCARCITY_THRESHOLD {
        let t = (SCARCITY_THRESHOLD - energy_grid.reserve_margin) / SCARCITY_THRESHOLD;
        1.0 + t * (MAX_SCARCITY_MULTIPLIER - 1.0)
    } else {
        1.0
    };

    dispatch_state.electricity_price =
        base_price * scarcity_multiplier.min(MAX_SCARCITY_MULTIPLIER);
}

/// Calculates rolling blackout cell count based on the current load shed
/// fraction and the number of powered cells in the grid.
pub fn apply_rolling_blackout(
    tick: Res<TickCounter>,
    mut dispatch_state: ResMut<EnergyDispatchState>,
    grid: Res<crate::grid::WorldGrid>,
) {
    if !tick.0.is_multiple_of(DISPATCH_INTERVAL) {
        return;
    }

    if !dispatch_state.has_deficit {
        dispatch_state.blackout_cells = 0;
        return;
    }

    let total_powered: u32 =
        grid.cells.iter().filter(|c| c.has_power).count() as u32;

    dispatch_state.blackout_cells =
        (total_powered as f32 * dispatch_state.load_shed_fraction) as u32;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EnergyDispatchPlugin;

impl Plugin for EnergyDispatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnergyDispatchState>();

        // Register for save/load.
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<EnergyDispatchState>();

        app.add_systems(
            FixedUpdate,
            (dispatch_energy, apply_rolling_blackout)
                .chain()
                .after(crate::energy_demand::aggregate_energy_demand)
                .in_set(SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_state_default() {
        let state = EnergyDispatchState::default();
        assert_eq!(state.electricity_price, 0.0);
        assert!(!state.has_deficit);
        assert_eq!(state.total_capacity_mw, 0.0);
        assert_eq!(state.blackout_cells, 0);
        assert_eq!(state.load_shed_fraction, 0.0);
        assert!(!state.active);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let state = EnergyDispatchState {
            electricity_price: 40.0,
            has_deficit: false,
            total_capacity_mw: 500.0,
            blackout_cells: 0,
            blackout_rotation: 7,
            load_shed_fraction: 0.0,
            dispatched_count: 3,
            active: true,
        };

        let bytes = state.save_to_bytes().unwrap();
        let restored = EnergyDispatchState::load_from_bytes(&bytes);

        assert!((restored.electricity_price - 40.0).abs() < f32::EPSILON);
        assert!(!restored.has_deficit);
        assert!((restored.total_capacity_mw - 500.0).abs() < f32::EPSILON);
        assert_eq!(restored.blackout_rotation, 7);
        assert_eq!(restored.dispatched_count, 3);
        assert!(restored.active);
    }
}
