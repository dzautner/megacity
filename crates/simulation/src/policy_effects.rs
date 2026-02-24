//! Policy tradeoff effects system and plugin (POL-001).
//!
//! Recomputes `PolicyTradeoffEffects` every slow tick from the active policies
//! in `Policies`. Also implements `Saveable` for the `Policies` resource so
//! active policies persist across save/load.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::policies::Policies;
use crate::policy_tradeoffs::{compute_effects, PolicyTradeoffEffects};
use crate::SlowTickTimer;

// =============================================================================
// System
// =============================================================================

/// Recompute aggregated policy effects every slow tick.
pub fn update_policy_tradeoff_effects(
    slow_timer: Res<SlowTickTimer>,
    policies: Res<Policies>,
    mut effects: ResMut<PolicyTradeoffEffects>,
) {
    if !slow_timer.should_run() {
        return;
    }
    *effects = compute_effects(&policies);
}

// =============================================================================
// Saveable wrapper for Policies (bitcode-encoded)
// =============================================================================

/// Bitcode-serializable wrapper for the active policy list.
///
/// We use a separate struct because `Policies` uses serde for backward
/// compatibility, but the Saveable extension map uses bitcode.
#[derive(Debug, Clone, Default, Encode, Decode)]
struct PolicySaveData {
    /// Indices into `Policy::all()` for each active policy.
    active_indices: Vec<u8>,
}

impl PolicySaveData {
    fn from_policies(policies: &Policies) -> Self {
        let all = crate::policies::Policy::all();
        let active_indices = policies
            .active
            .iter()
            .filter_map(|p| all.iter().position(|a| a == p).map(|i| i as u8))
            .collect();
        Self { active_indices }
    }

    fn to_policies(&self) -> Policies {
        let all = crate::policies::Policy::all();
        let active = self
            .active_indices
            .iter()
            .filter_map(|&i| all.get(i as usize).copied())
            .collect();
        Policies { active }
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for Policies {
    const SAVE_KEY: &'static str = "policy_tradeoffs";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.active.is_empty() {
            return None;
        }
        Some(bitcode::encode(&PolicySaveData::from_policies(self)))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let data: PolicySaveData = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        data.to_policies()
    }
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that registers the policy tradeoff effects system.
pub struct PolicyTradeoffsPlugin;

impl Plugin for PolicyTradeoffsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PolicyTradeoffEffects>()
            .add_systems(
                FixedUpdate,
                // Order-independent: only reads Policies and writes PolicyTradeoffEffects
                // (private resource); no shared mutable state.
                update_policy_tradeoff_effects.in_set(crate::SimulationSet::Simulation),
            );

        // Register Policies for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<Policies>();
    }
}
