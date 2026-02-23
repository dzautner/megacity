//! Waste policy state, computed effects, and pure helper functions.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::constants::*;

// =============================================================================
// Resource: waste policy toggles
// =============================================================================

/// City-wide waste reduction policy state.
///
/// Each boolean field represents an individually toggleable waste reduction
/// policy. The `WastePolicyEffects` resource is computed from these toggles
/// every slow tick and consumed by other waste/happiness systems.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct WastePolicyState {
    /// Plastic bag ban: reduces overall waste generation by 5%.
    pub plastic_bag_ban: bool,
    /// Deposit/return program: increases recycling rate by 10%.
    pub deposit_return_program: bool,
    /// Composting mandate: increases composting diversion by 15%.
    pub composting_mandate: bool,
    /// WTE mandate: diverts waste from landfill to waste-to-energy (incinerator).
    pub wte_mandate: bool,
    /// Whether the one-time infrastructure cost for deposit/return has been paid.
    pub deposit_return_built: bool,
    /// Whether the one-time enforcement cost for composting mandate has been paid.
    pub composting_mandate_setup_paid: bool,
    /// Cumulative infrastructure costs paid.
    pub total_infrastructure_cost: f64,
    /// Cumulative monthly operating costs paid.
    pub total_operating_cost: f64,
}

impl Default for WastePolicyState {
    fn default() -> Self {
        Self {
            plastic_bag_ban: false,
            deposit_return_program: false,
            composting_mandate: false,
            wte_mandate: false,
            deposit_return_built: false,
            composting_mandate_setup_paid: false,
            total_infrastructure_cost: 0.0,
            total_operating_cost: 0.0,
        }
    }
}

// =============================================================================
// Resource: computed policy effects
// =============================================================================

/// Computed effects of all active waste policies, updated each slow tick.
///
/// Other simulation systems read this resource to apply policy effects
/// without needing to know about individual policy toggles.
#[derive(Resource, Debug, Clone, Default)]
pub struct WastePolicyEffects {
    /// Multiplicative factor on waste generation (1.0 = no change, 0.95 = 5% reduction).
    pub waste_generation_multiplier: f32,
    /// Additive bonus to recycling diversion rate (0.0..1.0).
    pub recycling_rate_bonus: f32,
    /// Additive bonus to composting diversion rate (0.0..1.0).
    pub composting_diversion_bonus: f32,
    /// Tons diverted from landfill to WTE this period.
    pub wte_diversion_tons: f64,
    /// Whether WTE mandate is active and incinerators are available.
    pub wte_active: bool,
    /// Total happiness penalty from all active waste policies.
    pub happiness_penalty: f32,
    /// Total monthly cost of all active waste policies.
    pub total_monthly_cost: f64,
    /// Number of active waste policies.
    pub active_policy_count: u32,
}

// =============================================================================
// Helper functions (pure, testable)
// =============================================================================

/// Calculate the waste generation multiplier from active policies.
/// Each policy that reduces waste compounds multiplicatively.
pub fn calculate_waste_multiplier(state: &WastePolicyState) -> f32 {
    let mut multiplier = 1.0_f32;
    if state.plastic_bag_ban {
        multiplier *= 1.0 - PLASTIC_BAG_BAN_WASTE_REDUCTION;
    }
    multiplier
}

/// Calculate the total happiness penalty from active waste policies.
pub fn calculate_happiness_penalty(state: &WastePolicyState) -> f32 {
    let mut penalty = 0.0_f32;
    if state.plastic_bag_ban {
        penalty += PLASTIC_BAG_BAN_HAPPINESS_PENALTY;
    }
    if state.composting_mandate {
        penalty += COMPOSTING_MANDATE_HAPPINESS_PENALTY;
    }
    penalty
}

/// Calculate the total monthly operating cost of active waste policies.
pub fn calculate_monthly_cost(state: &WastePolicyState) -> f64 {
    let mut cost = 0.0_f64;
    if state.plastic_bag_ban {
        cost += PLASTIC_BAG_BAN_MONTHLY_COST;
    }
    if state.deposit_return_program {
        cost += DEPOSIT_RETURN_MONTHLY_COST;
    }
    if state.composting_mandate {
        cost += COMPOSTING_MANDATE_MONTHLY_COST;
    }
    if state.wte_mandate {
        cost += WTE_MANDATE_MONTHLY_COST;
    }
    cost
}

/// Calculate one-time infrastructure costs that still need to be paid.
pub fn calculate_pending_infrastructure_cost(state: &WastePolicyState) -> f64 {
    let mut cost = 0.0_f64;
    if state.deposit_return_program && !state.deposit_return_built {
        cost += DEPOSIT_RETURN_INFRASTRUCTURE_COST;
    }
    if state.composting_mandate && !state.composting_mandate_setup_paid {
        cost += COMPOSTING_MANDATE_ENFORCEMENT_COST;
    }
    cost
}

/// Count the number of active waste policies.
pub fn count_active_policies(state: &WastePolicyState) -> u32 {
    let mut count = 0u32;
    if state.plastic_bag_ban {
        count += 1;
    }
    if state.deposit_return_program {
        count += 1;
    }
    if state.composting_mandate {
        count += 1;
    }
    if state.wte_mandate {
        count += 1;
    }
    count
}

/// Calculate WTE diversion tons based on waste generated and incinerator capacity.
pub fn calculate_wte_diversion(
    wte_mandate: bool,
    period_generated_tons: f64,
    incinerator_count: u32,
) -> f64 {
    if !wte_mandate || incinerator_count == 0 {
        return 0.0;
    }
    // Each incinerator can handle 250 tons/day (from garbage.rs facility_capacity_tons)
    let incinerator_capacity = incinerator_count as f64 * 250.0;
    let divertable = period_generated_tons * WTE_MANDATE_DIVERSION_FRACTION as f64;
    divertable.min(incinerator_capacity)
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for WastePolicyState {
    const SAVE_KEY: &'static str = "waste_policies";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no policies have ever been activated
        if !self.plastic_bag_ban
            && !self.deposit_return_program
            && !self.composting_mandate
            && !self.wte_mandate
            && self.total_infrastructure_cost == 0.0
            && self.total_operating_cost == 0.0
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
