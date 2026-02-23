//! ECS systems, Saveable implementation, and plugin for historic preservation.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::districts::DistrictMap;
use crate::land_value::LandValueGrid;
use crate::tourism::Tourism;
use crate::SlowTickTimer;

use super::{calculate_historic_tourism, historic_land_value_bonus, HistoricPreservationState};

// =============================================================================
// Systems
// =============================================================================

/// Apply land value bonuses for cells in historic preservation districts.
/// Runs on the slow tick timer to align with land value updates.
pub fn apply_historic_land_value_bonus(
    slow_timer: Res<SlowTickTimer>,
    preservation: Res<HistoricPreservationState>,
    district_map: Res<DistrictMap>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    if preservation.preserved_districts.is_empty() {
        return;
    }

    // For each preserved district, boost land values in its cells
    for &di in &preservation.preserved_districts {
        if di >= district_map.districts.len() {
            continue;
        }
        for &(cx, cy) in &district_map.districts[di].cells {
            if cx < GRID_WIDTH && cy < GRID_HEIGHT {
                let cur = land_value.get(cx, cy);
                let bonus = historic_land_value_bonus(cur);
                let new_val = (cur as i32 + bonus).min(255) as u8;
                land_value.set(cx, cy, new_val);
            }
        }
    }
}

/// Update tourism from historic districts.
/// Runs on the slow tick timer.
pub fn update_historic_tourism(
    slow_timer: Res<SlowTickTimer>,
    mut preservation: ResMut<HistoricPreservationState>,
    mut tourism: ResMut<Tourism>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let visitors = calculate_historic_tourism(preservation.preserved_districts.len());
    preservation.historic_tourism_visitors = visitors;

    // Add historic district visitors to overall tourism
    tourism.monthly_visitors += visitors;
    tourism.monthly_tourism_income += visitors as f64 * 1.5; // $1.50 per historic tourist
}

/// Decay removal penalties over time.
pub fn decay_removal_penalties(
    slow_timer: Res<SlowTickTimer>,
    mut preservation: ResMut<HistoricPreservationState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Decay each penalty and remove expired ones
    preservation
        .removal_penalties
        .retain_mut(|(_di, remaining)| {
            *remaining = remaining.saturating_sub(1);
            *remaining > 0
        });
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for HistoricPreservationState {
    const SAVE_KEY: &'static str = "historic_preservation";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no districts are preserved and no penalties are active
        if self.preserved_districts.is_empty() && self.removal_penalties.is_empty() {
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

pub struct HistoricPreservationPlugin;

impl Plugin for HistoricPreservationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HistoricPreservationState>()
            .add_systems(
                FixedUpdate,
                (
                    apply_historic_land_value_bonus.after(crate::land_value::update_land_value),
                    update_historic_tourism.after(crate::tourism::update_tourism),
                    decay_removal_penalties,
                )
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<HistoricPreservationState>();
    }
}
