//! Waste reduction policies (WASTE-008).
//!
//! Implements waste management policies that push the waste hierarchy
//! (reduce > reuse > recycle > energy recovery > landfill). Each policy is
//! individually toggleable and has specific costs, benefits, and impacts on
//! waste generation, recycling rates, composting diversion, and citizen happiness.
//!
//! Policies:
//! - **Plastic bag ban**: -5% overall waste generation, minor happiness impact
//! - **Deposit/return program**: +10% recycling rate, $500K infrastructure cost
//! - **Composting mandate**: +15% diversion to composting, happiness -2, $1M enforcement
//! - **WTE mandate**: waste diverted from landfill to WTE (incinerator) when available
//!
//! The system reads `WasteSystem.period_generated_tons` and modifies the
//! effective waste stream via a `WastePolicyEffects` resource that other
//! systems can query.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::garbage::WasteSystem;
use crate::services::ServiceBuilding;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Waste generation reduction from plastic bag ban (5%).
pub const PLASTIC_BAG_BAN_WASTE_REDUCTION: f32 = 0.05;

/// Happiness penalty from plastic bag ban (minor citizen convenience impact).
pub const PLASTIC_BAG_BAN_HAPPINESS_PENALTY: f32 = 1.0;

/// Monthly upkeep cost for plastic bag ban enforcement.
pub const PLASTIC_BAG_BAN_MONTHLY_COST: f64 = 5_000.0;

/// Recycling rate bonus from deposit/return program (10 percentage points).
pub const DEPOSIT_RETURN_RECYCLING_BONUS: f32 = 0.10;

/// One-time infrastructure cost for deposit/return program.
pub const DEPOSIT_RETURN_INFRASTRUCTURE_COST: f64 = 500_000.0;

/// Monthly operating cost for deposit/return program.
pub const DEPOSIT_RETURN_MONTHLY_COST: f64 = 15_000.0;

/// Composting diversion bonus from composting mandate (15 percentage points).
pub const COMPOSTING_MANDATE_DIVERSION_BONUS: f32 = 0.15;

/// Happiness penalty from composting mandate (mandatory sorting is annoying).
pub const COMPOSTING_MANDATE_HAPPINESS_PENALTY: f32 = 2.0;

/// One-time enforcement setup cost for composting mandate.
pub const COMPOSTING_MANDATE_ENFORCEMENT_COST: f64 = 1_000_000.0;

/// Monthly enforcement cost for composting mandate.
pub const COMPOSTING_MANDATE_MONTHLY_COST: f64 = 25_000.0;

/// Monthly cost for WTE mandate administration.
pub const WTE_MANDATE_MONTHLY_COST: f64 = 10_000.0;

/// Fraction of landfill-bound waste diverted to WTE when mandate is active
/// and incinerator capacity is available.
pub const WTE_MANDATE_DIVERSION_FRACTION: f32 = 0.80;

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
        bitcode::decode(bytes).unwrap_or_default()
    }
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
                update_waste_policies.after(crate::garbage::update_waste_generation),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WastePolicyState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Default state tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_all_policies_off() {
        let state = WastePolicyState::default();
        assert!(!state.plastic_bag_ban);
        assert!(!state.deposit_return_program);
        assert!(!state.composting_mandate);
        assert!(!state.wte_mandate);
    }

    #[test]
    fn test_default_no_costs() {
        let state = WastePolicyState::default();
        assert_eq!(state.total_infrastructure_cost, 0.0);
        assert_eq!(state.total_operating_cost, 0.0);
    }

    #[test]
    fn test_default_setup_flags_false() {
        let state = WastePolicyState::default();
        assert!(!state.deposit_return_built);
        assert!(!state.composting_mandate_setup_paid);
    }

    #[test]
    fn test_default_effects_neutral() {
        let effects = WastePolicyEffects::default();
        assert_eq!(effects.waste_generation_multiplier, 0.0);
        assert_eq!(effects.recycling_rate_bonus, 0.0);
        assert_eq!(effects.composting_diversion_bonus, 0.0);
        assert_eq!(effects.wte_diversion_tons, 0.0);
        assert!(!effects.wte_active);
        assert_eq!(effects.happiness_penalty, 0.0);
        assert_eq!(effects.total_monthly_cost, 0.0);
        assert_eq!(effects.active_policy_count, 0);
    }

    // -------------------------------------------------------------------------
    // Waste generation multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_no_policies_multiplier_is_one() {
        let state = WastePolicyState::default();
        let mult = calculate_waste_multiplier(&state);
        assert!((mult - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_plastic_bag_ban_reduces_waste_5pct() {
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        let mult = calculate_waste_multiplier(&state);
        assert!((mult - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_other_policies_dont_affect_multiplier() {
        let mut state = WastePolicyState::default();
        state.deposit_return_program = true;
        state.composting_mandate = true;
        state.wte_mandate = true;
        let mult = calculate_waste_multiplier(&state);
        assert!((mult - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Happiness penalty tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_no_policies_no_happiness_penalty() {
        let state = WastePolicyState::default();
        let penalty = calculate_happiness_penalty(&state);
        assert!(penalty.abs() < f32::EPSILON);
    }

    #[test]
    fn test_plastic_bag_ban_happiness_penalty() {
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        let penalty = calculate_happiness_penalty(&state);
        assert!((penalty - PLASTIC_BAG_BAN_HAPPINESS_PENALTY).abs() < f32::EPSILON);
    }

    #[test]
    fn test_composting_mandate_happiness_penalty() {
        let mut state = WastePolicyState::default();
        state.composting_mandate = true;
        let penalty = calculate_happiness_penalty(&state);
        assert!((penalty - COMPOSTING_MANDATE_HAPPINESS_PENALTY).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_happiness_penalty() {
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        state.composting_mandate = true;
        let penalty = calculate_happiness_penalty(&state);
        let expected = PLASTIC_BAG_BAN_HAPPINESS_PENALTY + COMPOSTING_MANDATE_HAPPINESS_PENALTY;
        assert!((penalty - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_deposit_return_no_happiness_penalty() {
        let mut state = WastePolicyState::default();
        state.deposit_return_program = true;
        let penalty = calculate_happiness_penalty(&state);
        assert!(penalty.abs() < f32::EPSILON);
    }

    #[test]
    fn test_wte_mandate_no_happiness_penalty() {
        let mut state = WastePolicyState::default();
        state.wte_mandate = true;
        let penalty = calculate_happiness_penalty(&state);
        assert!(penalty.abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Monthly cost tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_no_policies_no_monthly_cost() {
        let state = WastePolicyState::default();
        let cost = calculate_monthly_cost(&state);
        assert!(cost.abs() < f64::EPSILON);
    }

    #[test]
    fn test_plastic_bag_ban_monthly_cost() {
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        let cost = calculate_monthly_cost(&state);
        assert!((cost - PLASTIC_BAG_BAN_MONTHLY_COST).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deposit_return_monthly_cost() {
        let mut state = WastePolicyState::default();
        state.deposit_return_program = true;
        let cost = calculate_monthly_cost(&state);
        assert!((cost - DEPOSIT_RETURN_MONTHLY_COST).abs() < f64::EPSILON);
    }

    #[test]
    fn test_composting_mandate_monthly_cost() {
        let mut state = WastePolicyState::default();
        state.composting_mandate = true;
        let cost = calculate_monthly_cost(&state);
        assert!((cost - COMPOSTING_MANDATE_MONTHLY_COST).abs() < f64::EPSILON);
    }

    #[test]
    fn test_wte_mandate_monthly_cost() {
        let mut state = WastePolicyState::default();
        state.wte_mandate = true;
        let cost = calculate_monthly_cost(&state);
        assert!((cost - WTE_MANDATE_MONTHLY_COST).abs() < f64::EPSILON);
    }

    #[test]
    fn test_all_policies_monthly_cost() {
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        state.deposit_return_program = true;
        state.composting_mandate = true;
        state.wte_mandate = true;
        let cost = calculate_monthly_cost(&state);
        let expected = PLASTIC_BAG_BAN_MONTHLY_COST
            + DEPOSIT_RETURN_MONTHLY_COST
            + COMPOSTING_MANDATE_MONTHLY_COST
            + WTE_MANDATE_MONTHLY_COST;
        assert!((cost - expected).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Infrastructure cost tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_no_policies_no_infrastructure_cost() {
        let state = WastePolicyState::default();
        let cost = calculate_pending_infrastructure_cost(&state);
        assert!(cost.abs() < f64::EPSILON);
    }

    #[test]
    fn test_deposit_return_infrastructure_cost() {
        let mut state = WastePolicyState::default();
        state.deposit_return_program = true;
        let cost = calculate_pending_infrastructure_cost(&state);
        assert!((cost - DEPOSIT_RETURN_INFRASTRUCTURE_COST).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deposit_return_already_built_no_cost() {
        let mut state = WastePolicyState::default();
        state.deposit_return_program = true;
        state.deposit_return_built = true;
        let cost = calculate_pending_infrastructure_cost(&state);
        assert!(cost.abs() < f64::EPSILON);
    }

    #[test]
    fn test_composting_mandate_enforcement_cost() {
        let mut state = WastePolicyState::default();
        state.composting_mandate = true;
        let cost = calculate_pending_infrastructure_cost(&state);
        assert!((cost - COMPOSTING_MANDATE_ENFORCEMENT_COST).abs() < f64::EPSILON);
    }

    #[test]
    fn test_composting_mandate_already_paid_no_cost() {
        let mut state = WastePolicyState::default();
        state.composting_mandate = true;
        state.composting_mandate_setup_paid = true;
        let cost = calculate_pending_infrastructure_cost(&state);
        assert!(cost.abs() < f64::EPSILON);
    }

    #[test]
    fn test_both_infrastructure_costs() {
        let mut state = WastePolicyState::default();
        state.deposit_return_program = true;
        state.composting_mandate = true;
        let cost = calculate_pending_infrastructure_cost(&state);
        let expected = DEPOSIT_RETURN_INFRASTRUCTURE_COST + COMPOSTING_MANDATE_ENFORCEMENT_COST;
        assert!((cost - expected).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Active policy count tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_no_active_policies() {
        let state = WastePolicyState::default();
        assert_eq!(count_active_policies(&state), 0);
    }

    #[test]
    fn test_one_active_policy() {
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        assert_eq!(count_active_policies(&state), 1);
    }

    #[test]
    fn test_all_active_policies() {
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        state.deposit_return_program = true;
        state.composting_mandate = true;
        state.wte_mandate = true;
        assert_eq!(count_active_policies(&state), 4);
    }

    // -------------------------------------------------------------------------
    // WTE diversion tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_wte_no_mandate_no_diversion() {
        let diversion = calculate_wte_diversion(false, 100.0, 2);
        assert!(diversion.abs() < f64::EPSILON);
    }

    #[test]
    fn test_wte_mandate_no_incinerators_no_diversion() {
        let diversion = calculate_wte_diversion(true, 100.0, 0);
        assert!(diversion.abs() < f64::EPSILON);
    }

    #[test]
    fn test_wte_mandate_with_incinerator() {
        // 100 tons generated, 80% divertable = 80 tons
        // 1 incinerator at 250 tons capacity
        // Diversion = min(80, 250) = 80
        let diversion = calculate_wte_diversion(true, 100.0, 1);
        let expected = 100.0 * WTE_MANDATE_DIVERSION_FRACTION as f64;
        assert!((diversion - expected).abs() < 0.01);
    }

    #[test]
    fn test_wte_diversion_capped_by_capacity() {
        // 1000 tons generated, 80% divertable = 800 tons
        // 1 incinerator at 250 tons capacity
        // Diversion = min(800, 250) = 250
        let diversion = calculate_wte_diversion(true, 1000.0, 1);
        assert!((diversion - 250.0).abs() < 0.01);
    }

    #[test]
    fn test_wte_diversion_multiple_incinerators() {
        // 1000 tons generated, 80% divertable = 800 tons
        // 2 incinerators at 250 each = 500 tons capacity
        // Diversion = min(800, 500) = 500
        let diversion = calculate_wte_diversion(true, 1000.0, 2);
        assert!((diversion - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_wte_diversion_zero_waste() {
        let diversion = calculate_wte_diversion(true, 0.0, 3);
        assert!(diversion.abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Constant value verification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constant_values() {
        assert_eq!(PLASTIC_BAG_BAN_WASTE_REDUCTION, 0.05);
        assert_eq!(PLASTIC_BAG_BAN_HAPPINESS_PENALTY, 1.0);
        assert_eq!(DEPOSIT_RETURN_RECYCLING_BONUS, 0.10);
        assert_eq!(DEPOSIT_RETURN_INFRASTRUCTURE_COST, 500_000.0);
        assert_eq!(COMPOSTING_MANDATE_DIVERSION_BONUS, 0.15);
        assert_eq!(COMPOSTING_MANDATE_HAPPINESS_PENALTY, 2.0);
        assert_eq!(COMPOSTING_MANDATE_ENFORCEMENT_COST, 1_000_000.0);
        assert_eq!(WTE_MANDATE_DIVERSION_FRACTION, 0.80);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = WastePolicyState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_active() {
        use crate::Saveable;
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        state.deposit_return_program = true;
        state.deposit_return_built = true;
        state.composting_mandate = true;
        state.composting_mandate_setup_paid = true;
        state.wte_mandate = true;
        state.total_infrastructure_cost = 1_500_000.0;
        state.total_operating_cost = 50_000.0;

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = WastePolicyState::load_from_bytes(&bytes);

        assert_eq!(restored.plastic_bag_ban, state.plastic_bag_ban);
        assert_eq!(
            restored.deposit_return_program,
            state.deposit_return_program
        );
        assert_eq!(restored.deposit_return_built, state.deposit_return_built);
        assert_eq!(restored.composting_mandate, state.composting_mandate);
        assert_eq!(
            restored.composting_mandate_setup_paid,
            state.composting_mandate_setup_paid
        );
        assert_eq!(restored.wte_mandate, state.wte_mandate);
        assert!(
            (restored.total_infrastructure_cost - state.total_infrastructure_cost).abs()
                < f64::EPSILON
        );
        assert!((restored.total_operating_cost - state.total_operating_cost).abs() < f64::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(WastePolicyState::SAVE_KEY, "waste_policies");
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_combined_effects_plastic_ban_and_composting() {
        let mut state = WastePolicyState::default();
        state.plastic_bag_ban = true;
        state.composting_mandate = true;

        let multiplier = calculate_waste_multiplier(&state);
        assert!((multiplier - 0.95).abs() < f32::EPSILON);

        let penalty = calculate_happiness_penalty(&state);
        let expected_penalty =
            PLASTIC_BAG_BAN_HAPPINESS_PENALTY + COMPOSTING_MANDATE_HAPPINESS_PENALTY;
        assert!((penalty - expected_penalty).abs() < f32::EPSILON);

        let cost = calculate_monthly_cost(&state);
        let expected_cost = PLASTIC_BAG_BAN_MONTHLY_COST + COMPOSTING_MANDATE_MONTHLY_COST;
        assert!((cost - expected_cost).abs() < f64::EPSILON);
    }

    #[test]
    fn test_infrastructure_costs_only_paid_once() {
        let mut state = WastePolicyState::default();
        state.deposit_return_program = true;
        state.composting_mandate = true;

        // First time: both costs pending
        let cost1 = calculate_pending_infrastructure_cost(&state);
        assert!(
            (cost1 - (DEPOSIT_RETURN_INFRASTRUCTURE_COST + COMPOSTING_MANDATE_ENFORCEMENT_COST))
                .abs()
                < f64::EPSILON
        );

        // Mark as paid
        state.deposit_return_built = true;
        state.composting_mandate_setup_paid = true;

        // Second time: no costs
        let cost2 = calculate_pending_infrastructure_cost(&state);
        assert!(cost2.abs() < f64::EPSILON);
    }

    #[test]
    fn test_wte_requires_both_mandate_and_incinerators() {
        // No mandate, has incinerators -> no diversion
        assert!(calculate_wte_diversion(false, 100.0, 2).abs() < f64::EPSILON);
        // Mandate, no incinerators -> no diversion
        assert!(calculate_wte_diversion(true, 100.0, 0).abs() < f64::EPSILON);
        // Mandate + incinerators -> diversion
        assert!(calculate_wte_diversion(true, 100.0, 1) > 0.0);
    }

    #[test]
    fn test_deposit_return_provides_recycling_bonus() {
        let mut state = WastePolicyState::default();
        state.deposit_return_program = true;

        // Verify that the bonus is exactly 10%
        assert_eq!(DEPOSIT_RETURN_RECYCLING_BONUS, 0.10);

        // No happiness penalty from deposit/return
        let penalty = calculate_happiness_penalty(&state);
        assert!(penalty.abs() < f32::EPSILON);
    }
}
