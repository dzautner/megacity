//! TRAF-013: Bicycle Lanes and Infrastructure
//!
//! Adds bicycle infrastructure as a transportation mode. Bike lanes can be
//! added to certain road types (Local, Avenue) or built as standalone bike
//! paths. Citizens choose biking for short-to-medium trips when bicycle
//! infrastructure exists.
//!
//! Key mechanics:
//! - Bike lane toggles on road segments (Local + bike, Avenue + bike)
//! - Standalone bike paths (reuses `RoadType::Path` segments)
//! - Citizens choose biking when: infrastructure exists, distance < 5km,
//!   age 15-65
//! - Cycling reduces car trips and traffic congestion
//! - "Encourage Biking" policy boosts cycling rate by 15%
//! - Bike lanes have a small maintenance cost
//!
//! Coverage is evaluated on the slow tick. The `BicycleLaneState` resource
//! is the source of truth for which segments have bike lanes.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::policies::{Policies, Policy};
use crate::road_segments::{RoadSegmentStore, SegmentId};
use crate::traffic::TrafficGrid;
use crate::Saveable;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Cyclist speed in km/h.
pub const CYCLIST_SPEED_KMH: f32 = 15.0;

/// Maximum practical cycling distance in km (~5 km).
pub const MAX_CYCLING_DISTANCE_KM: f32 = 5.0;

/// Maximum cycling distance in grid cells.
/// At CELL_SIZE=16m, 5km = 5000m / 16m ≈ 312 cells.
pub const MAX_CYCLING_CELLS: f32 = 312.0;

/// Monthly maintenance cost per bike-lane segment (in currency units).
pub const BIKE_LANE_MAINTENANCE_PER_SEGMENT: f64 = 0.2;

/// Congestion reduction factor per bike-lane cell.
/// Each bike-lane road cell reduces traffic density by this fraction.
pub const CONGESTION_REDUCTION_FACTOR: f32 = 0.05;

/// Cycling rate boost when "Encourage Biking" policy is active.
pub const ENCOURAGE_BIKING_BOOST: f32 = 0.15;

/// Base cycling mode share when bike infrastructure exists (fraction of
/// eligible trips that choose cycling).
pub const BASE_CYCLING_RATE: f32 = 0.10;

/// Minimum age for cycling eligibility.
pub const MIN_CYCLING_AGE: u32 = 15;

/// Maximum age for cycling eligibility.
pub const MAX_CYCLING_AGE: u32 = 65;

// =============================================================================
// Resources
// =============================================================================

/// Tracks which road segments have bicycle lanes attached.
///
/// A segment can have a bike lane if its `RoadType` is `Local`, `Avenue`,
/// or `Path` (standalone bike path). The set stores `SegmentId` values.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct BicycleLaneState {
    /// Set of segment IDs that have bike lanes enabled.
    pub segments_with_lanes: HashSet<u32>,
}

impl BicycleLaneState {
    /// Check whether a segment has a bike lane.
    pub fn has_bike_lane(&self, id: SegmentId) -> bool {
        self.segments_with_lanes.contains(&id.0)
    }

    /// Enable a bike lane on a segment. Returns `true` if newly added.
    pub fn add_bike_lane(&mut self, id: SegmentId) -> bool {
        self.segments_with_lanes.insert(id.0)
    }

    /// Remove a bike lane from a segment. Returns `true` if it was present.
    pub fn remove_bike_lane(&mut self, id: SegmentId) -> bool {
        self.segments_with_lanes.remove(&id.0)
    }

    /// Number of segments with bike lanes.
    pub fn lane_count(&self) -> usize {
        self.segments_with_lanes.len()
    }
}

impl Saveable for BicycleLaneState {
    const SAVE_KEY: &'static str = "bicycle_lanes";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.segments_with_lanes.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

/// Whether a road type supports adding a bike lane.
pub fn supports_bike_lane(road_type: RoadType) -> bool {
    matches!(
        road_type,
        RoadType::Local | RoadType::Avenue | RoadType::Path
    )
}

/// Per-cell bicycle infrastructure coverage grid.
/// Values 0-100 representing how well each cell is served by bike infrastructure.
#[derive(Resource, Clone, Debug, Encode, Decode)]
pub struct BicycleCoverageGrid {
    pub coverage: Vec<u8>,
    /// City-wide average cycling coverage (0.0-100.0).
    pub city_average: f32,
    /// City-wide cycling mode share (fraction of trips taken by bike).
    pub cycling_mode_share: f32,
    /// Total monthly maintenance cost for all bike lanes.
    pub total_maintenance_cost: f64,
}

impl Default for BicycleCoverageGrid {
    fn default() -> Self {
        Self {
            coverage: vec![0; GRID_WIDTH * GRID_HEIGHT],
            city_average: 0.0,
            cycling_mode_share: 0.0,
            total_maintenance_cost: 0.0,
        }
    }
}

impl BicycleCoverageGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.coverage[y * GRID_WIDTH + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.coverage[y * GRID_WIDTH + x] = val;
    }

    /// Returns coverage as a 0.0-1.0 fraction.
    #[inline]
    pub fn coverage_fraction(&self, x: usize, y: usize) -> f32 {
        self.get(x, y) as f32 / 100.0
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Prunes bike lane entries for segments that no longer exist in the store.
/// This prevents stale references after road demolition.
pub fn prune_stale_bike_lanes(
    timer: Res<SlowTickTimer>,
    segments: Res<RoadSegmentStore>,
    mut bike_state: ResMut<BicycleLaneState>,
) {
    if !timer.should_run() {
        return;
    }

    let valid_ids: HashSet<u32> = segments.segments.iter().map(|s| s.id.0).collect();
    bike_state
        .segments_with_lanes
        .retain(|id| valid_ids.contains(id));
}

/// Recomputes the bicycle coverage grid based on current bike lane state.
///
/// Coverage radiates outward from bike-lane cells with linear decay:
/// - Full coverage (100) on the bike lane cell itself
/// - Linear decay to 0 over MAX_CYCLING_CELLS distance
///
/// Also computes city-wide average coverage, cycling mode share, and
/// maintenance costs.
#[allow(clippy::too_many_arguments)]
pub fn update_bicycle_coverage(
    timer: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    segments: Res<RoadSegmentStore>,
    bike_state: Res<BicycleLaneState>,
    policies: Res<Policies>,
    mut coverage: ResMut<BicycleCoverageGrid>,
) {
    if !timer.should_run() {
        return;
    }

    // Reset coverage grid
    coverage.coverage.fill(0);

    // Collect all cells that are part of bike-lane segments
    let mut bike_cells: Vec<(usize, usize)> = Vec::new();
    for seg in &segments.segments {
        if bike_state.has_bike_lane(seg.id) && supports_bike_lane(seg.road_type) {
            for &(cx, cy) in &seg.rasterized_cells {
                bike_cells.push((cx, cy));
            }
        }
    }

    // Also include Path-type road cells as implicit bike infrastructure
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road && cell.road_type == RoadType::Path {
                bike_cells.push((x, y));
            }
        }
    }

    bike_cells.sort_unstable();
    bike_cells.dedup();

    if bike_cells.is_empty() {
        coverage.city_average = 0.0;
        coverage.cycling_mode_share = 0.0;
        coverage.total_maintenance_cost = 0.0;
        return;
    }

    // Mark bike lane cells with full coverage
    for &(bx, by) in &bike_cells {
        coverage.set(bx, by, 100);
    }

    // Radiate coverage outward using a simple BFS-like spread.
    // For performance we use a limited radius (25 cells ≈ 400m) for the
    // coverage heatmap, even though cycling range is longer.
    let spread_radius: i32 = 25;
    for &(bx, by) in &bike_cells {
        let bx_i = bx as i32;
        let by_i = by as i32;
        for dy in -spread_radius..=spread_radius {
            for dx in -spread_radius..=spread_radius {
                let nx = bx_i + dx;
                let ny = by_i + dy;
                if nx < 0 || ny < 0 || nx >= GRID_WIDTH as i32 || ny >= GRID_HEIGHT as i32 {
                    continue;
                }
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > spread_radius as f32 {
                    continue;
                }
                let score = ((1.0 - dist / spread_radius as f32) * 100.0) as u8;
                let ux = nx as usize;
                let uy = ny as usize;
                let current = coverage.get(ux, uy);
                if score > current {
                    coverage.set(ux, uy, score);
                }
            }
        }
    }

    // Compute city-wide average
    let total: u64 = coverage.coverage.iter().map(|&v| v as u64).sum();
    let cell_count = (GRID_WIDTH * GRID_HEIGHT) as f64;
    coverage.city_average = (total as f64 / cell_count) as f32;

    // Compute cycling mode share
    let mut cycling_rate = if !bike_cells.is_empty() {
        BASE_CYCLING_RATE
    } else {
        0.0
    };

    // Scale by coverage density (more infrastructure = higher mode share)
    let coverage_density = (bike_cells.len() as f32 / 100.0).min(1.0);
    cycling_rate *= 0.5 + 0.5 * coverage_density;

    // Encourage Biking policy boost
    if policies.is_active(Policy::EncourageBiking) {
        cycling_rate += ENCOURAGE_BIKING_BOOST;
    }

    coverage.cycling_mode_share = cycling_rate.clamp(0.0, 0.6);

    // Maintenance cost
    coverage.total_maintenance_cost =
        bike_state.lane_count() as f64 * BIKE_LANE_MAINTENANCE_PER_SEGMENT;
}

/// Reduces traffic density on cells with bike lane coverage.
/// The idea: some car trips are replaced by cycling, lowering road congestion.
pub fn apply_bike_lane_congestion_relief(
    timer: Res<SlowTickTimer>,
    coverage: Res<BicycleCoverageGrid>,
    mut traffic: ResMut<TrafficGrid>,
) {
    if !timer.should_run() {
        return;
    }

    // Only apply if there is meaningful cycling mode share
    if coverage.cycling_mode_share < 0.01 {
        return;
    }

    let reduction = coverage.cycling_mode_share * CONGESTION_REDUCTION_FACTOR;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cov = coverage.coverage_fraction(x, y);
            if cov > 0.0 {
                let current = traffic.get(x, y);
                let reduce_by = (current as f32 * reduction * cov) as u16;
                traffic.set(x, y, current.saturating_sub(reduce_by));
            }
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct BicycleLanesPlugin;

impl Plugin for BicycleLanesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BicycleLaneState>()
            .init_resource::<BicycleCoverageGrid>()
            .add_systems(
                FixedUpdate,
                (
                    prune_stale_bike_lanes,
                    update_bicycle_coverage,
                    apply_bike_lane_congestion_relief.after(crate::traffic::update_traffic_density),
                ),
            );

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<BicycleLaneState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bike_lane_state_add_remove() {
        let mut state = BicycleLaneState::default();
        assert_eq!(state.lane_count(), 0);

        let id = SegmentId(42);
        assert!(state.add_bike_lane(id));
        assert!(state.has_bike_lane(id));
        assert_eq!(state.lane_count(), 1);

        // Adding again returns false (already present)
        assert!(!state.add_bike_lane(id));
        assert_eq!(state.lane_count(), 1);

        assert!(state.remove_bike_lane(id));
        assert!(!state.has_bike_lane(id));
        assert_eq!(state.lane_count(), 0);
    }

    #[test]
    fn test_supports_bike_lane() {
        assert!(supports_bike_lane(RoadType::Local));
        assert!(supports_bike_lane(RoadType::Avenue));
        assert!(supports_bike_lane(RoadType::Path));
        assert!(!supports_bike_lane(RoadType::Boulevard));
        assert!(!supports_bike_lane(RoadType::Highway));
        assert!(!supports_bike_lane(RoadType::OneWay));
    }

    #[test]
    fn test_coverage_grid_default() {
        let grid = BicycleCoverageGrid::default();
        assert_eq!(grid.coverage.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(grid.city_average, 0.0);
        assert_eq!(grid.cycling_mode_share, 0.0);
        assert_eq!(grid.total_maintenance_cost, 0.0);
    }

    #[test]
    fn test_coverage_grid_get_set() {
        let mut grid = BicycleCoverageGrid::default();
        grid.set(10, 20, 75);
        assert_eq!(grid.get(10, 20), 75);
        assert!((grid.coverage_fraction(10, 20) - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = BicycleLaneState::default();
        state.add_bike_lane(SegmentId(1));
        state.add_bike_lane(SegmentId(5));
        state.add_bike_lane(SegmentId(99));

        let bytes = state.save_to_bytes().expect("non-empty state should save");
        let restored = BicycleLaneState::load_from_bytes(&bytes);
        assert_eq!(restored.lane_count(), 3);
        assert!(restored.has_bike_lane(SegmentId(1)));
        assert!(restored.has_bike_lane(SegmentId(5)));
        assert!(restored.has_bike_lane(SegmentId(99)));
    }

    #[test]
    fn test_saveable_empty_returns_none() {
        let state = BicycleLaneState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "empty state should skip saving"
        );
    }

    #[test]
    fn test_constants_are_reasonable() {
        assert!(CYCLIST_SPEED_KMH > 0.0);
        assert!(MAX_CYCLING_DISTANCE_KM > 0.0);
        assert!(MAX_CYCLING_CELLS > 0.0);
        assert!(BIKE_LANE_MAINTENANCE_PER_SEGMENT > 0.0);
        assert!(CONGESTION_REDUCTION_FACTOR > 0.0 && CONGESTION_REDUCTION_FACTOR < 1.0);
        assert!(ENCOURAGE_BIKING_BOOST > 0.0 && ENCOURAGE_BIKING_BOOST < 1.0);
        assert!(BASE_CYCLING_RATE > 0.0 && BASE_CYCLING_RATE < 1.0);
        assert!(MIN_CYCLING_AGE < MAX_CYCLING_AGE);
    }
}
