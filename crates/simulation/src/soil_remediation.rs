//! POLL-014: Soil Remediation Building and Phytoremediation
//!
//! Provides four remediation methods for cleaning up soil contamination
//! (from POLL-013):
//!
//! - **Excavation**: -10 contamination/tick, $500/cell — fast but expensive
//! - **Bioremediation**: -3/tick, $150/cell — moderate cost and speed
//! - **Phytoremediation**: -0.5/tick, $30/cell — slow but cheap
//! - **Containment**: stops lateral spread only, $80/cell — no cleanup
//!
//! Post-remediation: a cell becomes buildable again when contamination < 10.
//! Health effects: citizens on contaminated soil (>30) suffer health penalty.
//! Land value: contaminated soil reduces land value by up to -60%.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::health::HealthGrid;
use crate::land_value::LandValueGrid;
use crate::soil_contamination::SoilContaminationGrid;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Contamination level below which a cell is considered clean / buildable.
pub const BUILDABLE_THRESHOLD: f32 = 10.0;

/// Contamination level above which citizens suffer a health penalty.
pub const HEALTH_PENALTY_THRESHOLD: f32 = 30.0;

/// Maximum health penalty (applied when contamination is at its maximum).
const MAX_HEALTH_PENALTY: u8 = 40;

/// Maximum land-value penalty fraction (60%).
const MAX_LAND_VALUE_PENALTY_PTS: f32 = 30.0;

/// Contamination level at which the maximum land-value penalty is reached.
const LAND_VALUE_MAX_CONTAM: f32 = 300.0;

// ---------------------------------------------------------------------------
// Remediation method enum
// ---------------------------------------------------------------------------

/// Available soil remediation techniques. Each has a different cleanup rate
/// (contamination units removed per soil-contamination update cycle) and
/// one-time cost per cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum RemediationMethod {
    /// Fast excavation and removal of contaminated soil.
    Excavation,
    /// Microbial treatment — moderate speed.
    Bioremediation,
    /// Plant-based cleanup — slow but cheap.
    Phytoremediation,
    /// Prevents lateral spread without reducing contamination.
    Containment,
}

impl RemediationMethod {
    /// Contamination units removed per soil-contamination update cycle.
    pub fn cleanup_rate(self) -> f32 {
        match self {
            Self::Excavation => 10.0,
            Self::Bioremediation => 3.0,
            Self::Phytoremediation => 0.5,
            Self::Containment => 0.0, // does not clean — only blocks spread
        }
    }

    /// One-time cost (in dollars) to deploy this method on a single cell.
    pub fn cost(self) -> f64 {
        match self {
            Self::Excavation => 500.0,
            Self::Bioremediation => 150.0,
            Self::Phytoremediation => 30.0,
            Self::Containment => 80.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Active remediation site
// ---------------------------------------------------------------------------

/// A single active remediation site on the grid.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct RemediationSite {
    pub x: usize,
    pub y: usize,
    pub method: RemediationMethod,
}

// ---------------------------------------------------------------------------
// SoilRemediationState resource
// ---------------------------------------------------------------------------

/// Tracks all active remediation sites across the city.
#[derive(Resource, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct SoilRemediationState {
    pub sites: Vec<RemediationSite>,
}

impl SoilRemediationState {
    /// Add a remediation site. Returns false if a site already exists at (x, y).
    pub fn add_site(&mut self, x: usize, y: usize, method: RemediationMethod) -> bool {
        if self.sites.iter().any(|s| s.x == x && s.y == y) {
            return false;
        }
        self.sites.push(RemediationSite { x, y, method });
        true
    }

    /// Remove a remediation site at (x, y). Returns true if found and removed.
    pub fn remove_site(&mut self, x: usize, y: usize) -> bool {
        let before = self.sites.len();
        self.sites.retain(|s| !(s.x == x && s.y == y));
        self.sites.len() < before
    }

    /// Check whether a containment site exists at (x, y).
    pub fn is_contained(&self, x: usize, y: usize) -> bool {
        self.sites
            .iter()
            .any(|s| s.x == x && s.y == y && s.method == RemediationMethod::Containment)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Apply remediation cleanup to contaminated cells.
///
/// Runs on the same tick cadence as `SoilContaminationGrid` (every
/// `UPDATE_INTERVAL` ticks) so that cleanup rates are balanced against
/// accumulation rates.
fn apply_remediation(
    timer: Res<crate::soil_contamination::SoilContaminationTimer>,
    mut soil: ResMut<SoilContaminationGrid>,
    mut state: ResMut<SoilRemediationState>,
) {
    if !timer.should_run() {
        return;
    }

    // Apply cleanup and collect sites that are fully remediated.
    let mut finished = Vec::new();
    for site in &state.sites {
        let rate = site.method.cleanup_rate();
        if rate <= 0.0 {
            continue; // containment does not reduce contamination
        }
        let current = soil.get(site.x, site.y);
        let new_val = (current - rate).max(0.0);
        soil.set(site.x, site.y, new_val);
        if new_val < BUILDABLE_THRESHOLD {
            finished.push((site.x, site.y));
        }
    }

    // Auto-remove completed non-containment sites.
    for (x, y) in finished {
        let is_containment = state
            .sites
            .iter()
            .any(|s| s.x == x && s.y == y && s.method == RemediationMethod::Containment);
        if !is_containment {
            state.remove_site(x, y);
        }
    }
}

/// Prevent lateral spread from containment sites.
///
/// This system zeroes out any contamination increase at cells adjacent to a
/// containment site by clamping the neighbor cells to their pre-spread value.
/// We run AFTER the main soil contamination update so containment acts as a
/// barrier.
///
/// Implementation: for every containment site, we simply set the 4 cardinal
/// neighbors' contamination to the minimum of their current value and the
/// containment cell's value. This prevents the high-concentration cell from
/// raising its neighbors above its own level — effectively blocking spread.
fn apply_containment(
    timer: Res<crate::soil_contamination::SoilContaminationTimer>,
    mut soil: ResMut<SoilContaminationGrid>,
    state: Res<SoilRemediationState>,
) {
    if !timer.should_run() {
        return;
    }

    for site in &state.sites {
        if site.method != RemediationMethod::Containment {
            continue;
        }
        let cx = site.x;
        let cy = site.y;
        let contained_level = soil.get(cx, cy);

        let neighbors: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dx, dy) in neighbors {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                continue;
            }
            let ux = nx as usize;
            let uy = ny as usize;
            let neighbor_val = soil.get(ux, uy);
            // If the neighbor was raised above the contained level by spread,
            // clamp it back down. This effectively blocks outward diffusion.
            if neighbor_val > contained_level {
                soil.set(ux, uy, contained_level);
            }
        }
    }
}

/// Apply health penalty for citizens on contaminated soil (>30).
///
/// Reduces HealthGrid values proportionally to contamination level.
fn apply_soil_health_penalty(
    timer: Res<crate::SlowTickTimer>,
    soil: Res<SoilContaminationGrid>,
    mut health: ResMut<HealthGrid>,
) {
    if !timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let contamination = soil.get(x, y);
            if contamination <= HEALTH_PENALTY_THRESHOLD {
                continue;
            }
            // Scale penalty linearly from 0 at threshold to MAX_HEALTH_PENALTY at 500.
            let excess = contamination - HEALTH_PENALTY_THRESHOLD;
            let max_excess = 500.0 - HEALTH_PENALTY_THRESHOLD;
            let fraction = (excess / max_excess).min(1.0);
            let penalty = (fraction * MAX_HEALTH_PENALTY as f32) as u8;
            let current = health.get(x, y);
            health.levels[y * GRID_WIDTH + x] = current.saturating_sub(penalty);
        }
    }
}

/// Apply land value penalty for contaminated cells (up to -60%).
///
/// Uses a fixed subtraction (not multiplicative) so the penalty does not
/// compound over many ticks and drive values to zero regardless of
/// contamination level.
fn apply_soil_land_value_penalty(
    timer: Res<crate::SlowTickTimer>,
    soil: Res<SoilContaminationGrid>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let contamination = soil.get(x, y);
            if contamination <= 0.0 {
                continue;
            }
            // Scale penalty from 0 at contamination=0 to MAX_LAND_VALUE_PENALTY_PTS
            // at LAND_VALUE_MAX_CONTAM. This is a fixed-point subtraction so it
            // does not compound multiplicatively over many slow ticks.
            let fraction = (contamination / LAND_VALUE_MAX_CONTAM).min(1.0);
            let penalty = (fraction * MAX_LAND_VALUE_PENALTY_PTS) as u8;
            let current = land_value.get(x, y);
            land_value.set(x, y, current.saturating_sub(penalty));
        }
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl crate::Saveable for SoilRemediationState {
    const SAVE_KEY: &'static str = "soil_remediation";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.sites.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SoilRemediationPlugin;

impl Plugin for SoilRemediationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoilRemediationState>().add_systems(
            FixedUpdate,
            (
                apply_remediation
                    .after(crate::soil_contamination::update_soil_contamination),
                apply_containment
                    .after(apply_remediation),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Health and land-value penalties run on the slow tick
        app.add_systems(
            FixedUpdate,
            (
                apply_soil_health_penalty
                    .after(crate::health::update_health_grid),
                apply_soil_land_value_penalty
                    .after(crate::land_value::update_land_value),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<SoilRemediationState>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Saveable;

    #[test]
    fn test_remediation_method_rates() {
        assert_eq!(RemediationMethod::Excavation.cleanup_rate(), 10.0);
        assert_eq!(RemediationMethod::Bioremediation.cleanup_rate(), 3.0);
        assert_eq!(RemediationMethod::Phytoremediation.cleanup_rate(), 0.5);
        assert_eq!(RemediationMethod::Containment.cleanup_rate(), 0.0);
    }

    #[test]
    fn test_remediation_method_costs() {
        assert_eq!(RemediationMethod::Excavation.cost(), 500.0);
        assert_eq!(RemediationMethod::Bioremediation.cost(), 150.0);
        assert_eq!(RemediationMethod::Phytoremediation.cost(), 30.0);
        assert_eq!(RemediationMethod::Containment.cost(), 80.0);
    }

    #[test]
    fn test_add_site() {
        let mut state = SoilRemediationState::default();
        assert!(state.add_site(10, 20, RemediationMethod::Excavation));
        assert_eq!(state.sites.len(), 1);
        // Duplicate should fail
        assert!(!state.add_site(10, 20, RemediationMethod::Bioremediation));
        assert_eq!(state.sites.len(), 1);
    }

    #[test]
    fn test_remove_site() {
        let mut state = SoilRemediationState::default();
        state.add_site(10, 20, RemediationMethod::Excavation);
        assert!(state.remove_site(10, 20));
        assert!(state.sites.is_empty());
        assert!(!state.remove_site(10, 20)); // already removed
    }

    #[test]
    fn test_is_contained() {
        let mut state = SoilRemediationState::default();
        state.add_site(5, 5, RemediationMethod::Containment);
        state.add_site(10, 10, RemediationMethod::Excavation);
        assert!(state.is_contained(5, 5));
        assert!(!state.is_contained(10, 10));
        assert!(!state.is_contained(0, 0));
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(SoilRemediationState::SAVE_KEY, "soil_remediation");
    }

    #[test]
    fn test_saveable_empty_returns_none() {
        let state = SoilRemediationState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = SoilRemediationState::default();
        state.add_site(50, 50, RemediationMethod::Excavation);
        state.add_site(100, 100, RemediationMethod::Phytoremediation);

        let bytes = state.save_to_bytes().expect("Should save non-empty state");
        let restored = SoilRemediationState::load_from_bytes(&bytes);

        assert_eq!(restored.sites.len(), 2);
        assert_eq!(restored.sites[0].x, 50);
        assert_eq!(restored.sites[0].method, RemediationMethod::Excavation);
        assert_eq!(restored.sites[1].method, RemediationMethod::Phytoremediation);
    }
}
