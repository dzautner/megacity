//! Saveable implementation for `VirtualPopulation`.
//!
//! Persists the virtual population count, employment stats, and per-district
//! statistics through save/load cycles. Old saves that lack this key will
//! default to `VirtualPopulation::default()` (count = 0), preserving backward
//! compatibility.

use bevy::prelude::*;

use crate::virtual_population::VirtualPopulation;
use crate::Saveable;

impl Saveable for VirtualPopulation {
    const SAVE_KEY: &'static str = "virtual_population";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if there are no virtual citizens and cap is at default
        if self.total_virtual == 0
            && self.virtual_employed == 0
            && self.district_stats.is_empty()
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin — registers VirtualPopulation with the SaveableRegistry
// ---------------------------------------------------------------------------

pub struct VirtualPopulationSavePlugin;

impl Plugin for VirtualPopulationSavePlugin {
    fn build(&self, app: &mut App) {
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<VirtualPopulation>();
    }
}
