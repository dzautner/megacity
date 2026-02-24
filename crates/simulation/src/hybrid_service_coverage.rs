//! SVC-001: Hybrid Service Coverage Model
//!
//! Replaces binary Euclidean radius coverage with a hybrid model combining:
//! 1. Road-network BFS distance from each service building (not crow-flies)
//! 2. Distance decay: coverage = 1.0 at station, 0.0 at max road distance
//! 3. Quality factor from budget funding level (0.5 to 1.5)
//! 4. Capacity utilization: over-capacity degrades quality proportionally
//! 5. Effective service = proximity * capacity_effectiveness * quality
//!
//! A fire station across a river with no bridge provides zero coverage.
//! The legacy bitflag `ServiceCoverageGrid` remains available for backward
//! compatibility with the happiness system.

use std::collections::VecDeque;

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::budget::ExtendedBudget;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::service_budget::{Department, ServiceBudgetState};
use crate::service_capacity::ServiceCapacity;
use crate::services::{ServiceBuilding, ServiceType};
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Service categories for the quality grid
// ---------------------------------------------------------------------------

/// Categories tracked in the hybrid coverage grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum ServiceCategory {
    Health,
    Education,
    Police,
    Park,
    Entertainment,
    Telecom,
    Transport,
    Fire,
}

impl ServiceCategory {
    pub const ALL: [ServiceCategory; 8] = [
        ServiceCategory::Health,
        ServiceCategory::Education,
        ServiceCategory::Police,
        ServiceCategory::Park,
        ServiceCategory::Entertainment,
        ServiceCategory::Telecom,
        ServiceCategory::Transport,
        ServiceCategory::Fire,
    ];

    /// Map a ServiceType to its coverage category.
    /// Note: entertainment is checked before park because `is_park` includes
    /// Stadium/Plaza/SportsField, but the happiness system treats those as
    /// entertainment -- we must match that behavior.
    pub fn from_service_type(st: ServiceType) -> Option<ServiceCategory> {
        if ServiceBuilding::is_health(st) {
            Some(ServiceCategory::Health)
        } else if ServiceBuilding::is_education(st) {
            Some(ServiceCategory::Education)
        } else if ServiceBuilding::is_police(st) {
            Some(ServiceCategory::Police)
        } else if is_entertainment(st) {
            Some(ServiceCategory::Entertainment)
        } else if is_park_only(st) {
            Some(ServiceCategory::Park)
        } else if ServiceBuilding::is_telecom(st) {
            Some(ServiceCategory::Telecom)
        } else if ServiceBuilding::is_transport(st) {
            Some(ServiceCategory::Transport)
        } else if ServiceBuilding::is_fire(st) {
            Some(ServiceCategory::Fire)
        } else {
            None
        }
    }

    pub fn grid_index(self) -> usize {
        match self {
            ServiceCategory::Health => 0,
            ServiceCategory::Education => 1,
            ServiceCategory::Police => 2,
            ServiceCategory::Park => 3,
            ServiceCategory::Entertainment => 4,
            ServiceCategory::Telecom => 5,
            ServiceCategory::Transport => 6,
            ServiceCategory::Fire => 7,
        }
    }
}

/// Entertainment types (matches happiness coverage COVERAGE_ENTERTAINMENT).
fn is_entertainment(st: ServiceType) -> bool {
    matches!(
        st,
        ServiceType::Stadium | ServiceType::Plaza | ServiceType::SportsField
    )
}

/// Park types excluding entertainment (SmallPark, LargePark, Playground only).
fn is_park_only(st: ServiceType) -> bool {
    matches!(
        st,
        ServiceType::SmallPark | ServiceType::LargePark | ServiceType::Playground
    )
}

// ---------------------------------------------------------------------------
// HybridCoverageGrid resource
// ---------------------------------------------------------------------------

const NUM_CATEGORIES: usize = 8;
const GRID_CELLS: usize = GRID_WIDTH * GRID_HEIGHT;

/// Per-cell, per-service-category coverage quality (f32, 0.0 to 1.0+).
#[derive(Resource)]
pub struct HybridCoverageGrid {
    /// Flat array: `data[category_index * GRID_CELLS + cell_index]`
    pub data: Vec<f32>,
    /// Dirty flag -- set when service buildings or roads change.
    pub dirty: bool,
}

impl Default for HybridCoverageGrid {
    fn default() -> Self {
        Self {
            data: vec![0.0; NUM_CATEGORIES * GRID_CELLS],
            dirty: true,
        }
    }
}

impl HybridCoverageGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize, category: ServiceCategory) -> f32 {
        let idx = category.grid_index() * GRID_CELLS + y * GRID_WIDTH + x;
        self.data[idx]
    }

    #[inline]
    pub fn get_clamped(&self, x: usize, y: usize, category: ServiceCategory) -> f32 {
        self.get(x, y, category).clamp(0.0, 1.0)
    }

    fn clear(&mut self) {
        self.data.fill(0.0);
    }

    #[inline]
    fn set_max(&mut self, x: usize, y: usize, category: ServiceCategory, value: f32) {
        let idx = category.grid_index() * GRID_CELLS + y * GRID_WIDTH + x;
        if value > self.data[idx] {
            self.data[idx] = value;
        }
    }
}

// ---------------------------------------------------------------------------
// BFS-based road-network coverage computation
// ---------------------------------------------------------------------------

/// Maximum BFS distance in grid cells for road-network coverage.
const MAX_BFS_DISTANCE: u32 = 60;

/// Compute coverage from a single service building using road-network BFS.
fn bfs_road_coverage(
    grid: &WorldGrid,
    start_x: usize,
    start_y: usize,
    max_road_dist: u32,
    category: ServiceCategory,
    effective_quality: f32,
    coverage: &mut HybridCoverageGrid,
) {
    let max_dist = max_road_dist.min(MAX_BFS_DISTANCE);
    if max_dist == 0 {
        return;
    }

    let mut dist = vec![u32::MAX; GRID_CELLS];
    let mut queue = VecDeque::with_capacity(256);

    seed_bfs(grid, start_x, start_y, &mut dist, &mut queue);

    // Coverage at the building cell itself
    coverage.set_max(start_x, start_y, category, effective_quality);

    while let Some((cx, cy)) = queue.pop_front() {
        let cell_dist = dist[cy * GRID_WIDTH + cx];
        if cell_dist >= max_dist {
            continue;
        }

        let proximity = 1.0 - (cell_dist as f32 / max_dist as f32);
        let quality = proximity * effective_quality;
        coverage.set_max(cx, cy, category, quality);

        // Cover non-road neighbors (buildings sit next to roads)
        apply_to_adjacent_non_road(
            grid, cx, cy, proximity, effective_quality, category, coverage,
        );

        // Expand to neighboring road cells
        for (nx, ny) in neighbors4(cx, cy) {
            if !grid.in_bounds(nx, ny) {
                continue;
            }
            let nidx = ny * GRID_WIDTH + nx;
            let new_dist = cell_dist + 1;
            if new_dist < dist[nidx] && grid.get(nx, ny).cell_type == CellType::Road {
                dist[nidx] = new_dist;
                queue.push_back((nx, ny));
            }
        }
    }
}

fn seed_bfs(
    grid: &WorldGrid,
    sx: usize,
    sy: usize,
    dist: &mut [u32],
    queue: &mut VecDeque<(usize, usize)>,
) {
    let sidx = sy * GRID_WIDTH + sx;

    if grid.in_bounds(sx, sy) && grid.get(sx, sy).cell_type == CellType::Road {
        dist[sidx] = 0;
        queue.push_back((sx, sy));
        return;
    }

    // Seed from adjacent road cells at distance 1
    for (nx, ny) in neighbors4(sx, sy) {
        if !grid.in_bounds(nx, ny) {
            continue;
        }
        let nidx = ny * GRID_WIDTH + nx;
        if grid.get(nx, ny).cell_type == CellType::Road && dist[nidx] == u32::MAX {
            dist[nidx] = 1;
            queue.push_back((nx, ny));
        }
    }

    // Mark building cell as visited at distance 0
    if grid.in_bounds(sx, sy) && dist[sidx] == u32::MAX {
        dist[sidx] = 0;
    }
}

fn apply_to_adjacent_non_road(
    grid: &WorldGrid,
    cx: usize,
    cy: usize,
    proximity: f32,
    effective_quality: f32,
    category: ServiceCategory,
    coverage: &mut HybridCoverageGrid,
) {
    let off_road_quality = proximity * effective_quality * 0.95;
    for (nx, ny) in neighbors4(cx, cy) {
        if !grid.in_bounds(nx, ny) {
            continue;
        }
        if grid.get(nx, ny).cell_type != CellType::Road {
            coverage.set_max(nx, ny, category, off_road_quality);
        }
    }
}

fn neighbors4(x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(4);
    if x > 0 {
        out.push((x - 1, y));
    }
    if y > 0 {
        out.push((x, y - 1));
    }
    out.push((x + 1, y));
    out.push((x, y + 1));
    out
}

// ---------------------------------------------------------------------------
// Quality factors
// ---------------------------------------------------------------------------

/// Quality factor from department funding ratio. Range: 0.5 to 1.5.
pub fn budget_quality_factor(budget_state: &ServiceBudgetState, st: ServiceType) -> f32 {
    let Some(dept) = Department::for_service(st) else {
        return 1.0;
    };
    let funding_ratio = budget_state.department(dept).funding_ratio;
    (0.5 + funding_ratio * 0.5).clamp(0.5, 1.5)
}

/// Effective quality combining capacity effectiveness and budget quality.
pub fn compute_effective_quality(
    capacity: Option<&ServiceCapacity>,
    budget_state: &ServiceBudgetState,
    service_type: ServiceType,
) -> f32 {
    let capacity_eff = capacity.map_or(1.0, |c| c.effectiveness());
    let budget_qual = budget_quality_factor(budget_state, service_type);
    capacity_eff * budget_qual
}

// ---------------------------------------------------------------------------
// Main update system
// ---------------------------------------------------------------------------

const HYBRID_COVERAGE_UPDATE_INTERVAL: u64 = 20;

#[allow(clippy::too_many_arguments)]
fn update_hybrid_coverage(
    tick: Res<TickCounter>,
    grid: Res<WorldGrid>,
    services: Query<(&ServiceBuilding, Option<&ServiceCapacity>)>,
    added_services: Query<Entity, Added<ServiceBuilding>>,
    budget_state: Res<ServiceBudgetState>,
    ext_budget: Res<ExtendedBudget>,
    mut coverage: ResMut<HybridCoverageGrid>,
) {
    if !added_services.is_empty() {
        coverage.dirty = true;
    }
    if ext_budget.is_changed() || budget_state.is_changed() {
        coverage.dirty = true;
    }

    if !coverage.dirty && !tick.0.is_multiple_of(HYBRID_COVERAGE_UPDATE_INTERVAL) {
        return;
    }
    coverage.dirty = false;
    coverage.clear();

    for (service, capacity) in &services {
        let Some(category) = ServiceCategory::from_service_type(service.service_type) else {
            continue;
        };

        let effective_quality =
            compute_effective_quality(capacity, &budget_state, service.service_type);

        let budget_level = ext_budget.service_budgets.for_service(service.service_type);
        let effective_radius = service.radius * budget_level;
        let max_road_cells = (effective_radius / crate::config::CELL_SIZE).ceil() as u32;

        bfs_road_coverage(
            &grid,
            service.grid_x,
            service.grid_y,
            max_road_cells,
            category,
            effective_quality,
            &mut coverage,
        );
    }
}

// ---------------------------------------------------------------------------
// Aggregate stats resource (saveable)
// ---------------------------------------------------------------------------

/// Lightweight stats for save/load (full grid is recomputed on load).
#[derive(Resource, Default, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct HybridCoverageStats {
    pub category_averages: [f32; NUM_CATEGORIES],
    pub covered_cell_counts: [u32; NUM_CATEGORIES],
}

fn update_hybrid_coverage_stats(
    tick: Res<TickCounter>,
    coverage: Res<HybridCoverageGrid>,
    mut stats: ResMut<HybridCoverageStats>,
) {
    if !tick.0.is_multiple_of(HYBRID_COVERAGE_UPDATE_INTERVAL) {
        return;
    }

    for cat in ServiceCategory::ALL {
        let ci = cat.grid_index();
        let base = ci * GRID_CELLS;
        let slice = &coverage.data[base..base + GRID_CELLS];

        let mut sum: f32 = 0.0;
        let mut count: u32 = 0;
        for &v in slice {
            if v > 0.0 {
                sum += v.min(1.0);
                count += 1;
            }
        }

        stats.category_averages[ci] = if count > 0 {
            sum / count as f32
        } else {
            0.0
        };
        stats.covered_cell_counts[ci] = count;
    }
}

impl crate::Saveable for HybridCoverageStats {
    const SAVE_KEY: &'static str = "hybrid_coverage";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.covered_cell_counts.iter().all(|&c| c == 0) {
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

pub struct HybridServiceCoveragePlugin;

impl Plugin for HybridServiceCoveragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HybridCoverageGrid>();
        app.init_resource::<HybridCoverageStats>();

        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<HybridCoverageStats>();

        app.add_systems(
            FixedUpdate,
            (update_hybrid_coverage, update_hybrid_coverage_stats)
                .chain()
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
