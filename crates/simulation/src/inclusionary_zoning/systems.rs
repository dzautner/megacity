//! ECS systems and Saveable implementation for inclusionary zoning.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::districts::DistrictMap;
use crate::SlowTickTimer;

use super::config::InclusionaryZoningState;
use super::helpers::{calculate_affordable_units, calculate_monthly_admin_cost};

/// System: update inclusionary zoning computed effects every slow tick.
///
/// Iterates all residential buildings, checks if they are in a district
/// with inclusionary zoning enabled, and aggregates total affordable/affected
/// unit counts and admin costs.
pub fn update_inclusionary_zoning(
    timer: Res<SlowTickTimer>,
    mut state: ResMut<InclusionaryZoningState>,
    district_map: Res<DistrictMap>,
    buildings: Query<&Building>,
) {
    if !timer.should_run() {
        return;
    }

    let mut total_affordable = 0u32;
    let mut total_affected = 0u32;

    for building in &buildings {
        if !building.zone_type.is_residential() && !building.zone_type.is_mixed_use() {
            continue;
        }

        let di = district_map.get_district_index_at(building.grid_x, building.grid_y);
        let affordable_pct = di
            .map(|idx| state.affordable_percentage(idx))
            .unwrap_or(0.0);

        if affordable_pct > 0.0 {
            let res_capacity = if building.zone_type.is_mixed_use() {
                let (_, res_cap) =
                    crate::buildings::MixedUseBuilding::capacities_for_level(building.level);
                res_cap
            } else {
                building.capacity
            };
            total_affected += res_capacity;
            total_affordable += calculate_affordable_units(res_capacity, affordable_pct);
        }
    }

    state.total_affordable_units = total_affordable;
    state.total_affected_units = total_affected;
    state.total_monthly_cost = calculate_monthly_admin_cost(state.enabled_district_count());
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for InclusionaryZoningState {
    const SAVE_KEY: &'static str = "inclusionary_zoning";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no districts have ever been configured
        if self.district_configs.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
