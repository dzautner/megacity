//! SVC-008: Death Care Capacity and Cemetery Fill
//!
//! Cemeteries have finite capacity (1000 plots per cemetery). When full, new
//! cemeteries or crematoriums are needed. Crematoriums have unlimited long-term
//! capacity but a per-cremation time cost (queue-based processing).
//!
//! Unprocessed deceased that cannot be placed in a cemetery or crematorium
//! queue generate happiness and health penalties city-wide.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::death_care::{DeathCareGrid, DeathCareStats};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SimulationSet;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum burial plots per cemetery building.
pub const CEMETERY_CAPACITY: u32 = 1000;

/// Bodies a crematorium can process per slow tick cycle (batch size).
pub const CREMATORIUM_BATCH_SIZE: u32 = 5;

/// Happiness penalty per unprocessed body that overflows death care capacity.
pub const OVERFLOW_HAPPINESS_PENALTY: f32 = 0.5;

/// Maximum happiness penalty from death care overflow.
pub const MAX_OVERFLOW_HAPPINESS_PENALTY: f32 = 15.0;

/// Health penalty per unprocessed body that overflows death care capacity.
pub const OVERFLOW_HEALTH_PENALTY: f32 = 0.3;

/// Maximum health penalty from death care overflow.
pub const MAX_OVERFLOW_HEALTH_PENALTY: f32 = 10.0;

// ---------------------------------------------------------------------------
// Per-entity capacity tracking
// ---------------------------------------------------------------------------

/// Tracks how many plots are used in a single cemetery.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CemeteryRecord {
    pub plots_used: u32,
    pub total_plots: u32,
}

impl CemeteryRecord {
    pub fn new(capacity: u32) -> Self {
        Self {
            plots_used: 0,
            total_plots: capacity,
        }
    }

    pub fn remaining(&self) -> u32 {
        self.total_plots.saturating_sub(self.plots_used)
    }

    pub fn is_full(&self) -> bool {
        self.plots_used >= self.total_plots
    }

    /// Attempt to inter a body. Returns true if successful.
    pub fn inter(&mut self) -> bool {
        if self.plots_used < self.total_plots {
            self.plots_used += 1;
            true
        } else {
            false
        }
    }
}

/// Tracks the cremation queue for a single crematorium.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrematoriumRecord {
    /// Bodies waiting to be cremated.
    pub queue: u32,
    /// Total bodies cremated over the lifetime of this crematorium.
    pub total_cremated: u32,
}

impl CrematoriumRecord {
    /// Add a body to the cremation queue.
    pub fn enqueue(&mut self) {
        self.queue += 1;
    }

    /// Process up to `batch` bodies from the queue. Returns how many were processed.
    pub fn process(&mut self, batch: u32) -> u32 {
        let processed = self.queue.min(batch);
        self.queue -= processed;
        self.total_cremated += processed;
        processed
    }
}

// ---------------------------------------------------------------------------
// City-wide state resource
// ---------------------------------------------------------------------------

/// City-wide death care capacity state, keyed by entity index.
///
/// Entity IDs are not stable across save/load, so we use grid coordinates as
/// stable keys (serialized as `(usize, usize)`).
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeathCareCapacityState {
    /// Cemetery records keyed by `(grid_x, grid_y)`.
    pub cemeteries: HashMap<(usize, usize), CemeteryRecord>,
    /// Crematorium records keyed by `(grid_x, grid_y)`.
    pub crematoriums: HashMap<(usize, usize), CrematoriumRecord>,
    /// Bodies that could not be placed this cycle (overflow).
    pub overflow_bodies: u32,
    /// Running total of bodies interred across all cemeteries.
    pub total_interred: u32,
    /// Running total of bodies cremated across all crematoriums.
    pub total_cremated: u32,
}

impl DeathCareCapacityState {
    /// Happiness penalty from overflow bodies.
    pub fn happiness_penalty(&self) -> f32 {
        (self.overflow_bodies as f32 * OVERFLOW_HAPPINESS_PENALTY)
            .min(MAX_OVERFLOW_HAPPINESS_PENALTY)
    }

    /// Health penalty from overflow bodies.
    pub fn health_penalty(&self) -> f32 {
        (self.overflow_bodies as f32 * OVERFLOW_HEALTH_PENALTY).min(MAX_OVERFLOW_HEALTH_PENALTY)
    }

    /// Total remaining cemetery plots across all cemeteries.
    pub fn total_remaining_plots(&self) -> u32 {
        self.cemeteries.values().map(|c| c.remaining()).sum()
    }

    /// Total cemetery capacity across all cemeteries.
    pub fn total_cemetery_capacity(&self) -> u32 {
        self.cemeteries.values().map(|c| c.total_plots).sum()
    }

    /// Total plots used across all cemeteries.
    pub fn total_plots_used(&self) -> u32 {
        self.cemeteries.values().map(|c| c.plots_used).sum()
    }

    /// Total bodies in crematorium queues.
    pub fn total_cremation_queue(&self) -> u32 {
        self.crematoriums.values().map(|c| c.queue).sum()
    }
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl crate::Saveable for DeathCareCapacityState {
    const SAVE_KEY: &'static str = "deathcare_capacity";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Ensures every Cemetery/Crematorium entity has a corresponding record in the
/// capacity state. Removes records for buildings that no longer exist.
pub fn sync_deathcare_buildings(
    slow_timer: Res<crate::SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    mut state: ResMut<DeathCareCapacityState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Collect current building positions
    let mut active_cemeteries = std::collections::HashSet::new();
    let mut active_crematoriums = std::collections::HashSet::new();

    for service in &services {
        match service.service_type {
            ServiceType::Cemetery => {
                let key = (service.grid_x, service.grid_y);
                active_cemeteries.insert(key);
                state
                    .cemeteries
                    .entry(key)
                    .or_insert_with(|| CemeteryRecord::new(CEMETERY_CAPACITY));
            }
            ServiceType::Crematorium => {
                let key = (service.grid_x, service.grid_y);
                active_crematoriums.insert(key);
                state
                    .crematoriums
                    .entry(key)
                    .or_insert_with(CrematoriumRecord::default);
            }
            _ => {}
        }
    }

    // Prune records for demolished buildings
    state
        .cemeteries
        .retain(|k, _| active_cemeteries.contains(k));
    state
        .crematoriums
        .retain(|k, _| active_crematoriums.contains(k));
}

/// Processes unprocessed deaths from the `DeathCareGrid` into cemetery plots
/// and crematorium queues. Bodies that cannot be placed become overflow.
///
/// Priority: fill cemeteries first (permanent, no ongoing cost), then
/// queue overflow into crematoriums.
pub fn process_deathcare_capacity(
    slow_timer: Res<crate::SlowTickTimer>,
    death_stats: Res<DeathCareStats>,
    mut state: ResMut<DeathCareCapacityState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // How many unprocessed bodies need placement this cycle?
    let unprocessed = death_stats.unprocessed;
    if unprocessed == 0 {
        state.overflow_bodies = 0;
        return;
    }

    let mut remaining = unprocessed;

    // Phase 1: Inter into cemeteries with available plots
    for record in state.cemeteries.values_mut() {
        if remaining == 0 {
            break;
        }
        let can_inter = record.remaining().min(remaining);
        for _ in 0..can_inter {
            record.inter();
        }
        remaining -= can_inter;
    }
    let interred_this_cycle = unprocessed - remaining;
    state.total_interred += interred_this_cycle;

    // Phase 2: Queue remaining into crematoriums
    if remaining > 0 {
        let crematorium_count = state.crematoriums.len().max(1) as u32;
        let per_crematorium = remaining / crematorium_count;
        let mut extra = remaining % crematorium_count;

        for record in state.crematoriums.values_mut() {
            let batch = per_crematorium + if extra > 0 { 1 } else { 0 };
            if extra > 0 {
                extra -= 1;
            }
            for _ in 0..batch {
                record.enqueue();
            }
        }

        // If there are crematoriums, bodies are queued (not overflow yet)
        if !state.crematoriums.is_empty() {
            remaining = 0;
        }
    }

    state.overflow_bodies = remaining;
}

/// Crematoriums process their queues each slow tick cycle.
pub fn advance_crematorium_queues(
    slow_timer: Res<crate::SlowTickTimer>,
    mut state: ResMut<DeathCareCapacityState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut total_cremated = 0u32;
    for record in state.crematoriums.values_mut() {
        total_cremated += record.process(CREMATORIUM_BATCH_SIZE);
    }
    state.total_cremated += total_cremated;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct DeathCareCapacityPlugin;

impl Plugin for DeathCareCapacityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeathCareCapacityState>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<DeathCareCapacityState>();

        app.add_systems(
            FixedUpdate,
            (
                sync_deathcare_buildings,
                process_deathcare_capacity
                    .after(sync_deathcare_buildings)
                    .after(crate::death_care::death_care_processing),
                advance_crematorium_queues.after(process_deathcare_capacity),
            )
                .in_set(SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cemetery_record_capacity() {
        let mut rec = CemeteryRecord::new(3);
        assert_eq!(rec.remaining(), 3);
        assert!(!rec.is_full());
        assert!(rec.inter());
        assert!(rec.inter());
        assert!(rec.inter());
        assert!(rec.is_full());
        assert!(!rec.inter()); // full
        assert_eq!(rec.plots_used, 3);
    }

    #[test]
    fn test_crematorium_record_processing() {
        let mut rec = CrematoriumRecord::default();
        rec.enqueue();
        rec.enqueue();
        rec.enqueue();
        assert_eq!(rec.queue, 3);
        let processed = rec.process(2);
        assert_eq!(processed, 2);
        assert_eq!(rec.queue, 1);
        assert_eq!(rec.total_cremated, 2);
    }

    #[test]
    fn test_overflow_happiness_penalty() {
        let mut state = DeathCareCapacityState::default();
        assert_eq!(state.happiness_penalty(), 0.0);
        state.overflow_bodies = 10;
        assert!((state.happiness_penalty() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_overflow_happiness_penalty_capped() {
        let mut state = DeathCareCapacityState::default();
        state.overflow_bodies = 1000;
        assert!((state.happiness_penalty() - MAX_OVERFLOW_HAPPINESS_PENALTY).abs() < f32::EPSILON);
    }

    #[test]
    fn test_overflow_health_penalty() {
        let mut state = DeathCareCapacityState::default();
        state.overflow_bodies = 10;
        assert!((state.health_penalty() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_total_remaining_plots() {
        let mut state = DeathCareCapacityState::default();
        state
            .cemeteries
            .insert((10, 10), CemeteryRecord::new(1000));
        state
            .cemeteries
            .insert((20, 20), CemeteryRecord::new(1000));
        assert_eq!(state.total_remaining_plots(), 2000);
        state.cemeteries.get_mut(&(10, 10)).unwrap().plots_used = 600;
        assert_eq!(state.total_remaining_plots(), 1400);
    }

    #[test]
    fn test_state_default_is_clean() {
        let state = DeathCareCapacityState::default();
        assert_eq!(state.overflow_bodies, 0);
        assert_eq!(state.total_interred, 0);
        assert_eq!(state.total_cremated, 0);
        assert!(state.cemeteries.is_empty());
        assert!(state.crematoriums.is_empty());
    }
}
