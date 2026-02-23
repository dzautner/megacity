//! Parking policy state, computed effects, pure computation functions,
//! the ECS system, saveable implementation, and plugin registration.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::grid::{WorldGrid, ZoneType};
use crate::SlowTickTimer;

use super::constants::{parking_cost_per_space, parking_ratio, PARKING_MAXIMUM_FRACTION};

// =============================================================================
// Resource: parking policy state
// =============================================================================

/// City-wide parking policy configuration.
///
/// Controls whether parking minimums are enforced and whether a parking
/// maximum cap is in effect. These policies affect building construction
/// costs and transit dependency.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct ParkingPolicyState {
    /// When true, parking minimums are eliminated (no required parking).
    /// Reduces construction costs but increases transit dependency.
    pub eliminate_minimums: bool,
    /// When true, a parking maximum is enforced (caps parking at a fraction
    /// of the minimum ratio). Encourages transit use.
    pub parking_maximum: bool,
}

// =============================================================================
// Resource: computed parking effects
// =============================================================================

/// Computed effects of parking policies, updated each slow tick.
///
/// Other simulation systems read this resource to determine parking-related
/// cost adjustments and transit dependency impacts.
#[derive(Resource, Debug, Clone, Default)]
pub struct ParkingEffects {
    /// Total required parking spaces across all buildings.
    pub total_required_spaces: u32,
    /// Total additional construction cost from parking requirements.
    pub total_parking_cost: f64,
    /// Average effective parking ratio (accounting for policy modifications).
    pub effective_ratio_multiplier: f32,
    /// Whether parking minimums are currently eliminated.
    pub minimums_eliminated: bool,
    /// Whether parking maximum is currently active.
    pub maximum_active: bool,
    /// Number of buildings affected by parking requirements.
    pub buildings_affected: u32,
}

// =============================================================================
// Pure computation functions
// =============================================================================

/// Calculate the effective parking ratio for a zone type given current policy.
///
/// - Default: returns the full parking_ratio for the zone.
/// - Eliminate minimums: returns 0 (no required parking).
/// - Parking maximum: caps ratio at `PARKING_MAXIMUM_FRACTION` of the minimum.
/// - Both: eliminate minimums takes precedence (ratio = 0).
pub fn effective_parking_ratio(zone: ZoneType, state: &ParkingPolicyState) -> f32 {
    if state.eliminate_minimums {
        return 0.0;
    }
    let base = parking_ratio(zone);
    if state.parking_maximum {
        base * PARKING_MAXIMUM_FRACTION
    } else {
        base
    }
}

/// Calculate the number of required parking spaces for a building.
///
/// For residential zones, the ratio is spaces-per-unit (capacity = units).
/// For commercial/industrial/office, we approximate units from capacity
/// (each capacity unit ~ 1 person ~ some sqft equivalent).
pub fn required_parking_spaces(zone: ZoneType, capacity: u32, state: &ParkingPolicyState) -> u32 {
    let ratio = effective_parking_ratio(zone, state);
    if ratio <= 0.0 {
        return 0;
    }
    // For all zone types, required spaces = capacity * ratio (rounded up)
    (capacity as f32 * ratio).ceil() as u32
}

/// Calculate the additional construction cost from parking requirements
/// for a single building.
pub fn parking_construction_cost(zone: ZoneType, capacity: u32, state: &ParkingPolicyState) -> f64 {
    let spaces = required_parking_spaces(zone, capacity, state);
    spaces as f64 * parking_cost_per_space(zone)
}

/// Calculate the ratio multiplier for the current policy state.
/// Returns 0.0 if minimums eliminated, 0.5 if maximum active, 1.0 otherwise.
pub fn ratio_multiplier(state: &ParkingPolicyState) -> f32 {
    if state.eliminate_minimums {
        0.0
    } else if state.parking_maximum {
        PARKING_MAXIMUM_FRACTION
    } else {
        1.0
    }
}

// =============================================================================
// System
// =============================================================================

/// System: update parking effects every slow tick.
///
/// Iterates all buildings and computes total required parking spaces and
/// aggregate construction cost impact from parking requirements.
pub fn update_parking_effects(
    timer: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    state: Res<ParkingPolicyState>,
    mut effects: ResMut<ParkingEffects>,
) {
    if !timer.should_run() {
        return;
    }

    let mut total_spaces = 0u32;
    let mut total_cost = 0.0f64;
    let mut buildings_affected = 0u32;

    for cell in &grid.cells {
        if let Some(entity) = cell.building_id {
            if let Ok(building) = buildings.get(entity) {
                let spaces = required_parking_spaces(building.zone_type, building.capacity, &state);
                let cost = parking_construction_cost(building.zone_type, building.capacity, &state);

                if spaces > 0 {
                    buildings_affected += 1;
                }

                total_spaces += spaces;
                total_cost += cost;
            }
        }
    }

    effects.total_required_spaces = total_spaces;
    effects.total_parking_cost = total_cost;
    effects.effective_ratio_multiplier = ratio_multiplier(&state);
    effects.minimums_eliminated = state.eliminate_minimums;
    effects.maximum_active = state.parking_maximum;
    effects.buildings_affected = buildings_affected;
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for ParkingPolicyState {
    const SAVE_KEY: &'static str = "parking_policy";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if at default state (no policies active)
        if !self.eliminate_minimums && !self.parking_maximum {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct ParkingPlugin;

impl Plugin for ParkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParkingPolicyState>()
            .init_resource::<ParkingEffects>()
            .add_systems(
                FixedUpdate,
                update_parking_effects.in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ParkingPolicyState>();
    }
}
