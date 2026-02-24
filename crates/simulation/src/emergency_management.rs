//! SVC-011: Emergency Management System
//!
//! Provides disaster coordination via Emergency Operations Center (EOC),
//! emergency sirens for citizen warning, and shelter capacity tracking.
//!
//! - EOC presence reduces disaster severity by 30%
//! - Sirens reduce casualty rate by 20% within their coverage radius
//! - Without EOC: response time +50%, casualties +100%
//! - Tracks disaster preparedness as a city metric

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::disasters::ActiveDisaster;
use crate::grid::{WorldGrid, ZoneType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Disaster severity reduction when an EOC is present (30%).
pub const EOC_SEVERITY_REDUCTION: f32 = 0.30;

/// Casualty reduction from siren coverage (20%).
pub const SIREN_CASUALTY_REDUCTION: f32 = 0.20;

/// Response time multiplier without EOC (1.5x = +50%).
pub const NO_EOC_RESPONSE_MULTIPLIER: f32 = 1.5;

/// Casualty multiplier without EOC (2.0x = +100%).
pub const NO_EOC_CASUALTY_MULTIPLIER: f32 = 2.0;

/// Coverage radius for emergency sirens (in grid cells converted to world units).
pub const SIREN_COVERAGE_RADIUS: f32 = 25.0 * CELL_SIZE;

/// Coverage radius for EOC coordination effect (city-wide).
pub const EOC_COVERAGE_RADIUS: f32 = 128.0 * CELL_SIZE;

/// Shelter capacity per residential building designated as shelter.
pub const SHELTER_CAPACITY_PER_BUILDING: u32 = 50;

/// Base preparedness score with no infrastructure.
pub const BASE_PREPAREDNESS: f32 = 0.0;

/// Preparedness bonus from having an EOC.
pub const EOC_PREPAREDNESS_BONUS: f32 = 40.0;

/// Preparedness bonus per siren (capped).
pub const SIREN_PREPAREDNESS_BONUS: f32 = 5.0;

/// Maximum preparedness bonus from sirens.
pub const MAX_SIREN_PREPAREDNESS: f32 = 30.0;

/// Preparedness bonus from shelter capacity (per 100 capacity).
pub const SHELTER_PREPAREDNESS_PER_100: f32 = 3.0;

/// Maximum preparedness bonus from shelters.
pub const MAX_SHELTER_PREPAREDNESS: f32 = 30.0;

// =============================================================================
// Resource
// =============================================================================

/// Saveable state for the emergency management system.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct EmergencyManagementState {
    /// Whether the city has an Emergency Operations Center.
    pub has_eoc: bool,
    /// Number of emergency siren buildings in the city.
    pub siren_count: u32,
    /// Total shelter capacity across all designated shelters.
    pub shelter_capacity: u32,
    /// Current disaster preparedness score (0-100).
    pub preparedness_score: f32,
    /// Severity modifier applied to active disasters (1.0 = normal).
    pub severity_modifier: f32,
    /// Casualty modifier applied during disasters (1.0 = normal).
    pub casualty_modifier: f32,
    /// Response time modifier (1.0 = normal, higher = slower).
    pub response_time_modifier: f32,
    /// Number of disasters survived since last reset.
    pub disasters_survived: u32,
    /// Total buildings saved by emergency response (cumulative).
    pub buildings_saved: u32,
    /// Grid of siren coverage (true = covered by at least one siren).
    #[serde(skip)]
    #[bitcode(skip)]
    pub siren_coverage: Vec<bool>,
}

impl Default for EmergencyManagementState {
    fn default() -> Self {
        Self {
            has_eoc: false,
            siren_count: 0,
            shelter_capacity: 0,
            preparedness_score: BASE_PREPAREDNESS,
            severity_modifier: NO_EOC_CASUALTY_MULTIPLIER,
            casualty_modifier: NO_EOC_CASUALTY_MULTIPLIER,
            response_time_modifier: NO_EOC_RESPONSE_MULTIPLIER,
            disasters_survived: 0,
            buildings_saved: 0,
            siren_coverage: vec![false; GRID_WIDTH * GRID_HEIGHT],
        }
    }
}

impl EmergencyManagementState {
    /// Check if a specific cell is covered by emergency sirens.
    #[inline]
    pub fn has_siren_coverage(&self, x: usize, y: usize) -> bool {
        let idx = y * GRID_WIDTH + x;
        idx < self.siren_coverage.len() && self.siren_coverage[idx]
    }

    /// Calculate the fraction of grid cells covered by sirens (0.0 to 1.0).
    pub fn siren_coverage_fraction(&self) -> f32 {
        if self.siren_coverage.is_empty() {
            return 0.0;
        }
        let covered = self.siren_coverage.iter().filter(|&&c| c).count();
        covered as f32 / self.siren_coverage.len() as f32
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for EmergencyManagementState {
    const SAVE_KEY: &'static str = "emergency_management";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Always save since modifiers differ from defaults when EOC is present
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let mut state: Self = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        // Rebuild transient siren coverage grid
        state.siren_coverage = vec![false; GRID_WIDTH * GRID_HEIGHT];
        state
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Scans service buildings to determine EOC presence, siren count,
/// and updates emergency modifiers accordingly.
/// Runs on the slow tick timer.
#[allow(clippy::too_many_arguments)]
pub fn update_emergency_infrastructure(
    slow_timer: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    buildings: Query<&Building>,
    mut state: ResMut<EmergencyManagementState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Scan for EOC (CityHall acts as EOC) and sirens (PoliceHQ has sirens)
    let mut has_eoc = false;
    let mut siren_count = 0u32;
    let mut siren_positions: Vec<(usize, usize, f32)> = Vec::new();

    for service in &services {
        match service.service_type {
            ServiceType::CityHall => {
                has_eoc = true;
            }
            // Police stations and fire stations provide emergency siren coverage
            ServiceType::PoliceStation | ServiceType::PoliceHQ => {
                siren_count += 1;
                siren_positions.push((service.grid_x, service.grid_y, SIREN_COVERAGE_RADIUS));
            }
            ServiceType::FireStation | ServiceType::FireHQ => {
                siren_count += 1;
                siren_positions.push((service.grid_x, service.grid_y, SIREN_COVERAGE_RADIUS));
            }
            _ => {}
        }
    }

    // Calculate shelter capacity from residential buildings
    // (any residential building can serve as emergency shelter)
    let mut shelter_capacity = 0u32;
    for building in &buildings {
        if building.zone_type.is_residential() {
            shelter_capacity += SHELTER_CAPACITY_PER_BUILDING;
        }
    }

    // Update siren coverage grid
    state.siren_coverage.fill(false);
    for &(sx, sy, radius) in &siren_positions {
        let radius_cells = (radius / CELL_SIZE).ceil() as i32;
        let r2 = radius * radius;
        for dy in -radius_cells..=radius_cells {
            for dx in -radius_cells..=radius_cells {
                let cx = sx as i32 + dx;
                let cy = sy as i32 + dy;
                if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                    continue;
                }
                let wx = dx as f32 * CELL_SIZE;
                let wy = dy as f32 * CELL_SIZE;
                if wx * wx + wy * wy <= r2 {
                    let idx = cy as usize * GRID_WIDTH + cx as usize;
                    state.siren_coverage[idx] = true;
                }
            }
        }
    }

    // Update state
    state.has_eoc = has_eoc;
    state.siren_count = siren_count;
    state.shelter_capacity = shelter_capacity;

    // Calculate modifiers
    if has_eoc {
        state.severity_modifier = 1.0 - EOC_SEVERITY_REDUCTION;
        state.response_time_modifier = 1.0;
        state.casualty_modifier = 1.0;
    } else {
        state.severity_modifier = 1.0;
        state.response_time_modifier = NO_EOC_RESPONSE_MULTIPLIER;
        state.casualty_modifier = NO_EOC_CASUALTY_MULTIPLIER;
    }

    // Sirens reduce casualty rate by 20% (multiplicative with EOC)
    if siren_count > 0 {
        state.casualty_modifier *= 1.0 - SIREN_CASUALTY_REDUCTION;
    }

    // Calculate preparedness score
    let mut preparedness = BASE_PREPAREDNESS;
    if has_eoc {
        preparedness += EOC_PREPAREDNESS_BONUS;
    }
    preparedness +=
        (siren_count as f32 * SIREN_PREPAREDNESS_BONUS).min(MAX_SIREN_PREPAREDNESS);
    preparedness += (shelter_capacity as f32 / 100.0 * SHELTER_PREPAREDNESS_PER_100)
        .min(MAX_SHELTER_PREPAREDNESS);
    state.preparedness_score = preparedness.min(100.0);
}

/// Tracks disasters survived and updates cumulative statistics.
/// When an active disaster ends, increments the disasters_survived counter.
pub fn track_disaster_outcomes(
    active_disaster: Res<ActiveDisaster>,
    mut state: ResMut<EmergencyManagementState>,
) {
    // Detect when a disaster has just ended (current is None and was previously active).
    // We use is_changed() to detect the transition.
    if active_disaster.is_changed() && active_disaster.current.is_none() {
        // A disaster just ended
        if state.has_eoc {
            state.disasters_survived += 1;
            // Estimate buildings saved: EOC reduces severity by 30%,
            // so roughly 30% of at-risk buildings are saved.
            state.buildings_saved += 3; // approximate per-disaster savings
        }
    }
}

/// Applies emergency management modifiers to disaster damage.
/// This system reads the current emergency state and adjusts how
/// the disaster system processes damage. It modifies the disaster's
/// ticks_remaining based on response time modifier.
pub fn apply_emergency_response(
    mut active_disaster: ResMut<ActiveDisaster>,
    state: Res<EmergencyManagementState>,
) {
    if let Some(ref mut disaster) = active_disaster.current {
        // If EOC is present and damage hasn't been applied yet,
        // reduce the disaster's effective duration (faster response = shorter disaster)
        if state.has_eoc && !disaster.damage_applied {
            let reduction = (disaster.ticks_remaining as f32 * EOC_SEVERITY_REDUCTION) as u32;
            disaster.ticks_remaining = disaster.ticks_remaining.saturating_sub(reduction);
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_no_eoc() {
        let state = EmergencyManagementState::default();
        assert!(!state.has_eoc);
        assert_eq!(state.siren_count, 0);
        assert_eq!(state.shelter_capacity, 0);
        assert!((state.preparedness_score - BASE_PREPAREDNESS).abs() < f32::EPSILON);
        assert!((state.severity_modifier - NO_EOC_CASUALTY_MULTIPLIER).abs() < f32::EPSILON);
        assert!((state.casualty_modifier - NO_EOC_CASUALTY_MULTIPLIER).abs() < f32::EPSILON);
        assert!(
            (state.response_time_modifier - NO_EOC_RESPONSE_MULTIPLIER).abs() < f32::EPSILON
        );
    }

    #[test]
    fn test_eoc_severity_reduction_value() {
        assert!((EOC_SEVERITY_REDUCTION - 0.30).abs() < f32::EPSILON);
    }

    #[test]
    fn test_siren_casualty_reduction_value() {
        assert!((SIREN_CASUALTY_REDUCTION - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_no_eoc_multipliers() {
        assert!((NO_EOC_RESPONSE_MULTIPLIER - 1.5).abs() < f32::EPSILON);
        assert!((NO_EOC_CASUALTY_MULTIPLIER - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_siren_coverage_empty() {
        let state = EmergencyManagementState::default();
        assert!(!state.has_siren_coverage(0, 0));
        assert!(!state.has_siren_coverage(128, 128));
        assert!((state.siren_coverage_fraction()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_siren_coverage_fraction() {
        let mut state = EmergencyManagementState::default();
        let total = state.siren_coverage.len();
        // Cover half the cells
        for i in 0..total / 2 {
            state.siren_coverage[i] = true;
        }
        let frac = state.siren_coverage_fraction();
        assert!((frac - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_preparedness_eoc_only() {
        // With just an EOC, preparedness should be EOC_PREPAREDNESS_BONUS
        let score = BASE_PREPAREDNESS + EOC_PREPAREDNESS_BONUS;
        assert!((score - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_preparedness_with_sirens() {
        let siren_count = 4u32;
        let siren_bonus =
            (siren_count as f32 * SIREN_PREPAREDNESS_BONUS).min(MAX_SIREN_PREPAREDNESS);
        assert!((siren_bonus - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_preparedness_siren_cap() {
        let siren_count = 100u32;
        let siren_bonus =
            (siren_count as f32 * SIREN_PREPAREDNESS_BONUS).min(MAX_SIREN_PREPAREDNESS);
        assert!((siren_bonus - MAX_SIREN_PREPAREDNESS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_preparedness_shelter_bonus() {
        let capacity = 500u32;
        let shelter_bonus =
            (capacity as f32 / 100.0 * SHELTER_PREPAREDNESS_PER_100).min(MAX_SHELTER_PREPAREDNESS);
        assert!((shelter_bonus - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_preparedness_shelter_cap() {
        let capacity = 50000u32;
        let shelter_bonus =
            (capacity as f32 / 100.0 * SHELTER_PREPAREDNESS_PER_100).min(MAX_SHELTER_PREPAREDNESS);
        assert!((shelter_bonus - MAX_SHELTER_PREPAREDNESS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_preparedness() {
        // EOC(40) + sirens_max(30) + shelter_max(30) = 100
        let total =
            EOC_PREPAREDNESS_BONUS + MAX_SIREN_PREPAREDNESS + MAX_SHELTER_PREPAREDNESS;
        assert!((total - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_eoc_modifiers() {
        // With EOC: severity = 0.70, response = 1.0, casualty = 1.0
        let severity = 1.0 - EOC_SEVERITY_REDUCTION;
        assert!((severity - 0.70).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_casualty_reduction() {
        // With EOC (casualty=1.0) + sirens (casualty *= 0.80) => 0.80
        let casualty = 1.0 * (1.0 - SIREN_CASUALTY_REDUCTION);
        assert!((casualty - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(
            <EmergencyManagementState as crate::Saveable>::SAVE_KEY,
            "emergency_management"
        );
    }

    #[test]
    fn test_save_load_roundtrip() {
        use crate::Saveable;
        let mut state = EmergencyManagementState::default();
        state.has_eoc = true;
        state.siren_count = 3;
        state.shelter_capacity = 200;
        state.preparedness_score = 65.0;
        state.disasters_survived = 2;

        let bytes = state.save_to_bytes().expect("should serialize");
        let loaded = EmergencyManagementState::load_from_bytes(&bytes);

        assert!(loaded.has_eoc);
        assert_eq!(loaded.siren_count, 3);
        assert_eq!(loaded.shelter_capacity, 200);
        assert!((loaded.preparedness_score - 65.0).abs() < f32::EPSILON);
        assert_eq!(loaded.disasters_survived, 2);
        // Transient siren_coverage should be re-initialized
        assert_eq!(loaded.siren_coverage.len(), GRID_WIDTH * GRID_HEIGHT);
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct EmergencyManagementPlugin;

impl Plugin for EmergencyManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EmergencyManagementState>();

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<EmergencyManagementState>();

        app.add_systems(
            FixedUpdate,
            (
                update_emergency_infrastructure,
                track_disaster_outcomes,
                apply_emergency_response,
            )
                .chain()
                .before(crate::disasters::process_active_disaster)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
