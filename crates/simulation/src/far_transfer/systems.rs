//! ECS systems for computing FAR bonuses and transferring development rights.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::districts::{DistrictMap, DISTRICTS_X, DISTRICTS_Y, DISTRICT_SIZE};
use crate::historic_preservation::HistoricPreservationState;
use crate::services::ServiceBuilding;
use crate::SlowTickTimer;

use super::types::{
    calculate_far_bonus, districts_within_transfer_radius, eligible_bonuses, is_park_service,
    FarTransferState, HISTORIC_UNUSED_FAR_PER_CELL, MAX_TRANSFER_FAR_PER_CELL,
    PARK_UNUSED_FAR_PER_CELL,
};

// =============================================================================
// Systems
// =============================================================================

/// Update FAR bonuses for all cells with buildings.
/// Determines which bonus types each building qualifies for based on
/// zone type and level, then calculates the FAR bonus.
pub fn update_far_bonuses(
    timer: Res<SlowTickTimer>,
    buildings: Query<&Building>,
    mut state: ResMut<FarTransferState>,
) {
    if !timer.should_run() {
        return;
    }

    // Clear bonus data
    state.bonus_far.fill(0.0);
    state.bonus_flags.fill(0);
    state.total_bonus_far = 0.0;

    for building in &buildings {
        let x = building.grid_x;
        let y = building.grid_y;
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            continue;
        }

        let flags = eligible_bonuses(building.zone_type, building.level);
        if flags == 0 {
            continue;
        }

        let base_far = building.zone_type.default_far();
        let bonus = calculate_far_bonus(base_far, flags);

        let idx = y * GRID_WIDTH + x;
        state.bonus_far[idx] = bonus;
        state.bonus_flags[idx] = flags;
        state.total_bonus_far += bonus;
    }
}

/// Calculate available TDR FAR from source parcels (historic districts and parks).
/// Then distribute transferred FAR to eligible receiving cells.
pub fn update_far_transfers(
    timer: Res<SlowTickTimer>,
    preservation: Res<HistoricPreservationState>,
    district_map: Res<DistrictMap>,
    services: Query<&ServiceBuilding>,
    buildings: Query<&Building>,
    mut state: ResMut<FarTransferState>,
) {
    if !timer.should_run() {
        return;
    }

    let num_stat_districts = DISTRICTS_X * DISTRICTS_Y;

    // Reset transfer tracking
    state.transferred_far.fill(0.0);
    state.district_available_far.fill(0.0);
    state.district_transferred_far.fill(0.0);
    state.total_transferred_far = 0.0;

    // Ensure vectors are correctly sized
    state.district_available_far.resize(num_stat_districts, 0.0);
    state
        .district_transferred_far
        .resize(num_stat_districts, 0.0);

    // --- Step 1: Calculate available FAR from historic districts ---
    for &di in &preservation.preserved_districts {
        if di >= district_map.districts.len() {
            continue;
        }
        for &(cx, cy) in &district_map.districts[di].cells {
            if cx >= GRID_WIDTH || cy >= GRID_HEIGHT {
                continue;
            }
            // Each historic cell contributes unused FAR to its statistical district
            let (sdx, sdy) = stat_district_for_grid(cx, cy);
            let stat_idx = sdy * DISTRICTS_X + sdx;
            if stat_idx < num_stat_districts {
                state.district_available_far[stat_idx] += HISTORIC_UNUSED_FAR_PER_CELL;
            }
        }
    }

    // --- Step 2: Calculate available FAR from park service buildings ---
    for service in &services {
        if !is_park_service(service.service_type) {
            continue;
        }
        let x = service.grid_x;
        let y = service.grid_y;
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            continue;
        }
        let (sdx, sdy) = stat_district_for_grid(x, y);
        let stat_idx = sdy * DISTRICTS_X + sdx;
        if stat_idx < num_stat_districts {
            state.district_available_far[stat_idx] += PARK_UNUSED_FAR_PER_CELL;
        }
    }

    // --- Step 3: Distribute transferred FAR to eligible receiving cells ---
    // Receiving cells are buildings at level 2+ that are within transfer radius
    // of source districts.
    for building in &buildings {
        if building.level < 2 {
            continue;
        }
        let x = building.grid_x;
        let y = building.grid_y;
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            continue;
        }

        let (dst_dx, dst_dy) = stat_district_for_grid(x, y);

        // Find total available FAR from nearby source districts
        let mut available = 0.0_f32;
        let mut source_districts: Vec<usize> = Vec::new();

        for sdy in 0..DISTRICTS_Y {
            for sdx in 0..DISTRICTS_X {
                if !districts_within_transfer_radius(sdx, sdy, dst_dx, dst_dy) {
                    continue;
                }
                let stat_idx = sdy * DISTRICTS_X + sdx;
                let remaining = state.district_available_far[stat_idx]
                    - state.district_transferred_far[stat_idx];
                if remaining > 0.0 {
                    available += remaining;
                    source_districts.push(stat_idx);
                }
            }
        }

        if available <= 0.0 || source_districts.is_empty() {
            continue;
        }

        // Transfer up to MAX_TRANSFER_FAR_PER_CELL from available sources
        let transfer_amount = available.min(MAX_TRANSFER_FAR_PER_CELL);

        let idx = y * GRID_WIDTH + x;
        state.transferred_far[idx] = transfer_amount;
        state.total_transferred_far += transfer_amount;

        // Debit the transferred FAR from source districts proportionally
        let mut remaining_to_debit = transfer_amount;
        for &stat_idx in &source_districts {
            if remaining_to_debit <= 0.0 {
                break;
            }
            let source_remaining =
                state.district_available_far[stat_idx] - state.district_transferred_far[stat_idx];
            if source_remaining <= 0.0 {
                continue;
            }
            let debit = source_remaining.min(remaining_to_debit);
            state.district_transferred_far[stat_idx] += debit;
            remaining_to_debit -= debit;
        }
    }
}

/// Helper: convert grid coordinates to statistical district coordinates.
pub(crate) fn stat_district_for_grid(gx: usize, gy: usize) -> (usize, usize) {
    (
        (gx / DISTRICT_SIZE).min(DISTRICTS_X - 1),
        (gy / DISTRICT_SIZE).min(DISTRICTS_Y - 1),
    )
}
