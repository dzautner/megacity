//! SVC-012: City Hall Administration Efficiency
//!
//! City Hall provides city-wide administration bonus. Three tiers based on population:
//! - Small (pop < 25K): 100 admin staff per 100K target
//! - Medium (25K-100K): 150 admin staff per 100K target
//! - Large (100K+): 200 admin staff per 100K target
//!
//! Administration efficiency = staff_assigned / staff_required.
//! Low efficiency: -25% construction speed, -10% tax revenue.
//! High efficiency: +5% construction speed, +5% tax revenue.
//! Central location bonus: +5 happiness city-wide (civic pride).

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};
use crate::stats::CityStats;
use crate::Saveable;

// ---------------------------------------------------------------------------
// City Hall Tier
// ---------------------------------------------------------------------------

/// The three City Hall tiers, determined by city population.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode, Serialize, Deserialize)]
pub enum CityHallTier {
    /// Population < 25,000
    #[default]
    Small,
    /// Population 25,000 - 100,000
    Medium,
    /// Population > 100,000
    Large,
}


impl CityHallTier {
    /// Determine the appropriate tier for a given population.
    pub fn for_population(population: u32) -> Self {
        if population >= 100_000 {
            Self::Large
        } else if population >= 25_000 {
            Self::Medium
        } else {
            Self::Small
        }
    }

    /// Target admin staff per 100K population for this tier.
    pub fn staff_per_100k(self) -> u32 {
        match self {
            Self::Small => 100,
            Self::Medium => 150,
            Self::Large => 200,
        }
    }

    /// The staff capacity provided by a city hall of this tier.
    pub fn staff_capacity(self) -> u32 {
        match self {
            Self::Small => 50,
            Self::Medium => 200,
            Self::Large => 500,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Small => "Small City Hall",
            Self::Medium => "Medium City Hall",
            Self::Large => "Large City Hall",
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum construction speed penalty from low admin efficiency.
const LOW_EFFICIENCY_CONSTRUCTION_PENALTY: f32 = 0.25;
/// Maximum tax revenue penalty from low admin efficiency.
const LOW_EFFICIENCY_TAX_PENALTY: f32 = 0.10;
/// Construction speed bonus from high efficiency.
const HIGH_EFFICIENCY_CONSTRUCTION_BONUS: f32 = 0.05;
/// Tax revenue bonus from high efficiency.
const HIGH_EFFICIENCY_TAX_BONUS: f32 = 0.05;
/// Civic pride happiness bonus for centrally-located city hall.
const CIVIC_PRIDE_MAX_BONUS: f32 = 5.0;
/// Distance threshold (in grid cells) from center for "central" bonus.
/// City halls within this distance get full civic pride bonus.
const CENTRAL_DISTANCE_THRESHOLD: f32 = 40.0;
/// Maximum distance where any civic pride bonus applies.
const CENTRAL_DISTANCE_MAX: f32 = 100.0;

// ---------------------------------------------------------------------------
// CityHallState resource
// ---------------------------------------------------------------------------

/// Tracks the city-wide administration state from all City Hall buildings.
#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct CityHallState {
    /// Number of city hall buildings placed.
    pub city_hall_count: u32,
    /// Total admin staff capacity across all city halls.
    pub total_staff_capacity: u32,
    /// Required staff based on current population and tier.
    pub required_staff: u32,
    /// Administration efficiency ratio (0.0 to 2.0+, clamped for effects).
    pub admin_efficiency: f32,
    /// Current tier based on population.
    pub current_tier: CityHallTier,
    /// Construction speed multiplier (0.75 to 1.05).
    pub construction_speed_multiplier: f32,
    /// Tax revenue multiplier (0.90 to 1.05).
    pub tax_revenue_multiplier: f32,
    /// Civic pride bonus (0.0 to 5.0 happiness).
    pub civic_pride_bonus: f32,
    /// Corruption metric (0.0 = clean, 1.0 = highly corrupt).
    pub corruption: f32,
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl Saveable for CityHallState {
    const SAVE_KEY: &'static str = "city_hall";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.city_hall_count == 0 && self.admin_efficiency == 0.0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Compute the centrality score for a city hall at (gx, gy).
/// Returns 1.0 for perfectly central, 0.0 for far from center.
fn centrality_score(gx: usize, gy: usize) -> f32 {
    let cx = GRID_WIDTH as f32 / 2.0;
    let cy = GRID_HEIGHT as f32 / 2.0;
    let dx = gx as f32 - cx;
    let dy = gy as f32 - cy;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist <= CENTRAL_DISTANCE_THRESHOLD {
        1.0
    } else if dist >= CENTRAL_DISTANCE_MAX {
        0.0
    } else {
        let range = CENTRAL_DISTANCE_MAX - CENTRAL_DISTANCE_THRESHOLD;
        1.0 - (dist - CENTRAL_DISTANCE_THRESHOLD) / range
    }
}

/// Main system: update city hall administration efficiency and derived effects.
pub fn update_city_hall_admin(
    slow_timer: Res<crate::SlowTickTimer>,
    stats: Res<CityStats>,
    services: Query<&ServiceBuilding>,
    mut state: ResMut<CityHallState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let population = stats.population;
    let tier = CityHallTier::for_population(population);
    state.current_tier = tier;

    // Count city halls and compute total staff capacity and best centrality
    let mut city_hall_count = 0u32;
    let mut total_capacity = 0u32;
    let mut best_centrality: f32 = 0.0;

    for service in &services {
        if service.service_type == ServiceType::CityHall {
            city_hall_count += 1;
            // Each city hall contributes its tier's staff capacity
            total_capacity += tier.staff_capacity();
            let score = centrality_score(service.grid_x, service.grid_y);
            best_centrality = best_centrality.max(score);
        }
    }

    state.city_hall_count = city_hall_count;
    state.total_staff_capacity = total_capacity;

    // Required staff: (population / 100_000) * staff_per_100k, minimum 1 if pop > 0
    let required = if population > 0 {
        ((population as f64 / 100_000.0) * tier.staff_per_100k() as f64).ceil() as u32
    } else {
        0
    };
    state.required_staff = required;

    // Administration efficiency
    let efficiency = if required > 0 {
        (total_capacity as f32 / required as f32).clamp(0.0, 2.0)
    } else if city_hall_count > 0 {
        // No population but we have a city hall: max efficiency
        2.0
    } else {
        0.0
    };
    state.admin_efficiency = efficiency;

    // Derive construction speed multiplier
    state.construction_speed_multiplier = if efficiency < 1.0 {
        // Linear penalty: 0 efficiency -> 0.75, 1.0 efficiency -> 1.0
        1.0 - LOW_EFFICIENCY_CONSTRUCTION_PENALTY * (1.0 - efficiency)
    } else {
        // Bonus for over-staffing, capped
        let bonus = HIGH_EFFICIENCY_CONSTRUCTION_BONUS * (efficiency - 1.0);
        (1.0 + bonus).min(1.0 + HIGH_EFFICIENCY_CONSTRUCTION_BONUS)
    };

    // Derive tax revenue multiplier
    state.tax_revenue_multiplier = if efficiency < 1.0 {
        // Linear penalty: 0 efficiency -> 0.90, 1.0 efficiency -> 1.0
        1.0 - LOW_EFFICIENCY_TAX_PENALTY * (1.0 - efficiency)
    } else {
        let bonus = HIGH_EFFICIENCY_TAX_BONUS * (efficiency - 1.0);
        (1.0 + bonus).min(1.0 + HIGH_EFFICIENCY_TAX_BONUS)
    };

    // Civic pride from centrality of best city hall
    state.civic_pride_bonus = if city_hall_count > 0 {
        CIVIC_PRIDE_MAX_BONUS * best_centrality
    } else {
        0.0
    };

    // Corruption metric: inversely related to efficiency
    state.corruption = if city_hall_count == 0 && population > 0 {
        // No city hall at all with population => high corruption
        1.0
    } else if efficiency < 1.0 {
        // Understaffed: corruption scales with inefficiency
        (1.0 - efficiency).clamp(0.0, 1.0) * 0.5
    } else {
        // Well-staffed: minimal corruption
        0.0
    };
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CityHallPlugin;

impl Plugin for CityHallPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityHallState>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<CityHallState>();

        app.add_systems(
            FixedUpdate,
            update_city_hall_admin
                .after(crate::stats::update_stats)
                .in_set(crate::SimulationSet::Simulation),
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
    fn test_tier_for_population_small() {
        assert_eq!(CityHallTier::for_population(0), CityHallTier::Small);
        assert_eq!(CityHallTier::for_population(10_000), CityHallTier::Small);
        assert_eq!(CityHallTier::for_population(24_999), CityHallTier::Small);
    }

    #[test]
    fn test_tier_for_population_medium() {
        assert_eq!(CityHallTier::for_population(25_000), CityHallTier::Medium);
        assert_eq!(CityHallTier::for_population(50_000), CityHallTier::Medium);
        assert_eq!(CityHallTier::for_population(99_999), CityHallTier::Medium);
    }

    #[test]
    fn test_tier_for_population_large() {
        assert_eq!(CityHallTier::for_population(100_000), CityHallTier::Large);
        assert_eq!(CityHallTier::for_population(500_000), CityHallTier::Large);
    }

    #[test]
    fn test_staff_per_100k_increases_with_tier() {
        assert!(CityHallTier::Small.staff_per_100k() < CityHallTier::Medium.staff_per_100k());
        assert!(CityHallTier::Medium.staff_per_100k() < CityHallTier::Large.staff_per_100k());
    }

    #[test]
    fn test_staff_capacity_increases_with_tier() {
        assert!(CityHallTier::Small.staff_capacity() < CityHallTier::Medium.staff_capacity());
        assert!(CityHallTier::Medium.staff_capacity() < CityHallTier::Large.staff_capacity());
    }

    #[test]
    fn test_centrality_at_center() {
        let score = centrality_score(GRID_WIDTH / 2, GRID_HEIGHT / 2);
        assert!((score - 1.0).abs() < f32::EPSILON, "Center should be 1.0");
    }

    #[test]
    fn test_centrality_at_corner() {
        let score = centrality_score(0, 0);
        assert!(
            score < 0.1,
            "Corner should have very low centrality, got {score}"
        );
    }

    #[test]
    fn test_centrality_within_threshold() {
        // GRID center = 128, within threshold of 40
        let score = centrality_score(128 + 20, 128);
        assert!(
            (score - 1.0).abs() < f32::EPSILON,
            "Within threshold should be 1.0, got {score}"
        );
    }

    #[test]
    fn test_centrality_midway() {
        // At distance ~70 from center, between 40 and 100 threshold
        let score = centrality_score(128 + 70, 128);
        assert!(
            score > 0.0 && score < 1.0,
            "Midway should be between 0 and 1, got {score}"
        );
    }

    #[test]
    fn test_default_state() {
        let state = CityHallState::default();
        assert_eq!(state.city_hall_count, 0);
        assert!((state.admin_efficiency).abs() < f32::EPSILON);
        assert!((state.civic_pride_bonus).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = CityHallState::default();
        state.city_hall_count = 2;
        state.admin_efficiency = 0.8;
        state.civic_pride_bonus = 3.5;
        state.corruption = 0.1;

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = CityHallState::load_from_bytes(&bytes);

        assert_eq!(restored.city_hall_count, 2);
        assert!((restored.admin_efficiency - 0.8).abs() < f32::EPSILON);
        assert!((restored.civic_pride_bonus - 3.5).abs() < f32::EPSILON);
        assert!((restored.corruption - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_skip_default() {
        let state = CityHallState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Default state should skip saving"
        );
    }
}
