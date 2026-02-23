//! ECS systems, Saveable implementation, and plugin for district policies.

use bevy::prelude::*;

use crate::districts::DistrictMap;
use crate::SlowTickTimer;

use super::lookup::*;
use super::types::*;

// =============================================================================
// System
// =============================================================================

/// System: update district policy lookup tables every slow tick.
///
/// Reads `DistrictPolicyState` and city-wide `ExtendedBudget` to compute
/// effective per-district values, writing them to `DistrictPolicyLookup`.
pub fn update_district_policies(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<DistrictPolicyState>,
    budget: Res<crate::budget::ExtendedBudget>,
    district_map: Res<DistrictMap>,
    mut lookup: ResMut<DistrictPolicyLookup>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let city_wide_taxes = &budget.zone_taxes;
    let num_districts = district_map.districts.len();

    // Clear previous lookup data
    lookup.effective_taxes.clear();
    lookup.max_building_level.clear();
    lookup.heavy_industry_banned.clear();
    lookup.commercial_demand_bonus.clear();
    lookup.noise_multiplier.clear();
    lookup.park_multiplier.clear();
    lookup.service_budget_multiplier.clear();

    // Compute effective values for each district that has overrides
    for (&di, overrides) in &state.overrides {
        if di >= num_districts {
            continue;
        }

        // Tax rates
        let effective = compute_effective_taxes(overrides, city_wide_taxes);
        lookup.effective_taxes.insert(di, effective);

        // Building level
        let max_level = compute_max_building_level(overrides);
        if max_level != NORMAL_MAX_LEVEL {
            lookup.max_building_level.insert(di, max_level);
        }

        // Heavy industry ban
        if overrides.heavy_industry_ban {
            lookup.heavy_industry_banned.insert(di, true);
        }

        // Commercial demand bonus
        let bonus = compute_commercial_bonus(overrides);
        if bonus > 0.0 {
            lookup.commercial_demand_bonus.insert(di, bonus);
        }

        // Noise multiplier
        let noise_mult = compute_noise_multiplier(overrides);
        if (noise_mult - 1.0).abs() > f32::EPSILON {
            lookup.noise_multiplier.insert(di, noise_mult);
        }

        // Park multiplier
        let park_mult = compute_park_multiplier(overrides);
        if (park_mult - 1.0).abs() > f32::EPSILON {
            lookup.park_multiplier.insert(di, park_mult);
        }

        // Service budget multiplier
        let service_mult = compute_service_multiplier(overrides);
        if (service_mult - DEFAULT_SERVICE_BUDGET_MULTIPLIER).abs() > f32::EPSILON {
            lookup.service_budget_multiplier.insert(di, service_mult);
        }
    }

    // Update aggregate stats
    state.total_monthly_cost = compute_total_monthly_cost(&state.overrides);
    state.total_active_policies = compute_total_active_policies(&state.overrides);
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for DistrictPolicyState {
    const SAVE_KEY: &'static str = "district_policies";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no districts have any overrides
        if self.overrides.is_empty() {
            return None;
        }
        // Also skip if all overrides are at defaults
        if self.overrides.values().all(|o| o.is_default()) {
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

pub struct DistrictPoliciesPlugin;

impl Plugin for DistrictPoliciesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DistrictPolicyState>()
            .init_resource::<DistrictPolicyLookup>()
            .add_systems(
                FixedUpdate,
                update_district_policies
                    .after(crate::districts::district_stats)
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<DistrictPolicyState>();
    }
}
