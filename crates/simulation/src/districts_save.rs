//! Saveable implementation and save-registration plugin for `DistrictMap`.
//!
//! Persists player-defined district boundaries, names, policies, and cell
//! assignments across save/load cycles.

use bevy::prelude::*;

use crate::districts::DistrictMap;
use crate::Saveable;

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl Saveable for DistrictMap {
    const SAVE_KEY: &'static str = "district_map";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no cells are assigned to any district.
        if self.cell_map.iter().all(|c| c.is_none()) {
            // Also check if any district has been renamed or had policies
            // changed from defaults.
            let all_default = self.districts.iter().enumerate().all(|(i, d)| {
                let default_map = DistrictMap::default();
                if i >= default_map.districts.len() {
                    return false; // Extra districts added
                }
                let dd = &default_map.districts[i];
                d.name == dd.name
                    && d.color == dd.color
                    && d.cells.is_empty()
                    && is_policies_default(&d.policies)
            });
            if all_default && self.districts.len() == DistrictMap::default().districts.len() {
                return None;
            }
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

/// Check if district policies are at their default values.
fn is_policies_default(p: &crate::districts::DistrictPolicies) -> bool {
    p.tax_rate.is_none()
        && p.speed_limit.is_none()
        && !p.noise_ordinance
        && !p.heavy_industry_ban
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct DistrictSavePlugin;

impl Plugin for DistrictSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<DistrictMap>();
    }
}
