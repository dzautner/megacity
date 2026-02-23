//! Bevy systems and plugin for waste reduction policies.

use bevy::prelude::*;

use super::constants::*;
use super::state::*;
use crate::garbage::WasteSystem;
use crate::services::ServiceBuilding;
use crate::SlowTickTimer;

// =============================================================================
// System
// =============================================================================

/// System: update waste policy effects every slow tick.
///
/// 1. Processes one-time infrastructure costs for newly activated policies.
/// 2. Computes the waste generation multiplier from active policies.
/// 3. Computes recycling and composting bonuses.
/// 4. Counts incinerators and computes WTE diversion.
/// 5. Computes happiness penalties and monthly costs.
/// 6. Writes all effects to `WastePolicyEffects`.
pub fn update_waste_policies(
    timer: Res<SlowTickTimer>,
    waste_system: Res<WasteSystem>,
    mut state: ResMut<WastePolicyState>,
    mut effects: ResMut<WastePolicyEffects>,
    services: Query<&ServiceBuilding>,
) {
    if !timer.should_run() {
        return;
    }

    // Process one-time infrastructure costs
    let pending_cost = calculate_pending_infrastructure_cost(&state);
    if pending_cost > 0.0 {
        state.total_infrastructure_cost += pending_cost;
        if state.deposit_return_program && !state.deposit_return_built {
            state.deposit_return_built = true;
        }
        if state.composting_mandate && !state.composting_mandate_setup_paid {
            state.composting_mandate_setup_paid = true;
        }
    }

    // Waste generation multiplier
    effects.waste_generation_multiplier = calculate_waste_multiplier(&state);

    // Recycling rate bonus
    effects.recycling_rate_bonus = if state.deposit_return_program {
        DEPOSIT_RETURN_RECYCLING_BONUS
    } else {
        0.0
    };

    // Composting diversion bonus
    effects.composting_diversion_bonus = if state.composting_mandate {
        COMPOSTING_MANDATE_DIVERSION_BONUS
    } else {
        0.0
    };

    // WTE mandate: count incinerators and compute diversion
    let incinerator_count = services
        .iter()
        .filter(|s| s.service_type == crate::services::ServiceType::Incinerator)
        .count() as u32;

    effects.wte_diversion_tons = calculate_wte_diversion(
        state.wte_mandate,
        waste_system.period_generated_tons,
        incinerator_count,
    );
    effects.wte_active = state.wte_mandate && incinerator_count > 0;

    // Happiness penalty
    effects.happiness_penalty = calculate_happiness_penalty(&state);

    // Monthly cost
    let monthly_cost = calculate_monthly_cost(&state);
    effects.total_monthly_cost = monthly_cost;
    state.total_operating_cost += monthly_cost / 30.0; // prorate to daily (slow tick ~ 1 day)

    // Active policy count
    effects.active_policy_count = count_active_policies(&state);
}

// =============================================================================
// Plugin
// =============================================================================

pub struct WastePoliciesPlugin;

impl Plugin for WastePoliciesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WastePolicyState>()
            .init_resource::<WastePolicyEffects>()
            .add_systems(
                FixedUpdate,
                update_waste_policies
                    .after(crate::garbage::update_waste_generation)
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WastePolicyState>();
    }
}
