//! Barcelona Superblock District Policy (TRAF-009).
//!
//! Extends the basic superblock system (TRAF-008) with a full district policy:
//! - Interior roads are converted to `RoadType::Path` (pedestrian, zero vehicles)
//! - Original road types are stored for reversion
//! - Happiness bonus (+8-12) for residential zones inside superblocks
//! - Land value bonus (+15-25) inside superblocks
//! - Noise reduction (-10-20) inside superblocks
//! - Pollution reduction (-5-10) inside superblocks
//! - Perimeter congestion penalty (+20-40% occupancy increase)
//!
//! The `SuperblockPolicyState` resource tracks which superblocks have had their
//! policy activated (roads converted) vs just being designated.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::land_value::LandValueGrid;
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::superblock::SuperblockState;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Happiness bonus range for residential cells inside an active superblock.
const HAPPINESS_BONUS_MIN: f32 = 8.0;
const HAPPINESS_BONUS_MAX: f32 = 12.0;

/// Land value bonus range (additive, u8 units) for cells inside an active superblock.
const LAND_VALUE_BONUS_MIN: u8 = 15;
const LAND_VALUE_BONUS_MAX: u8 = 25;

/// Noise reduction range applied to interior superblock cells.
const NOISE_REDUCTION_MIN: u8 = 10;
const NOISE_REDUCTION_MAX: u8 = 20;

/// Pollution reduction range applied to interior superblock cells.
const POLLUTION_REDUCTION_MIN: u8 = 5;
const POLLUTION_REDUCTION_MAX: u8 = 10;

/// Congestion multiplier applied to perimeter road traffic volume.
/// 1.3 = +30% traffic on perimeter roads.
const PERIMETER_CONGESTION_MULTIPLIER: f32 = 1.3;

/// Monthly cost per superblock cell for maintaining pedestrian infrastructure.
const MONTHLY_COST_PER_CELL: f64 = 0.2;

// =============================================================================
// Types
// =============================================================================

/// Encode a `RoadType` as a u8 discriminant for bitcode serialization.
/// `RoadType` only derives serde, not bitcode, so we store as u8.
fn road_type_to_u8(rt: RoadType) -> u8 {
    match rt {
        RoadType::Local => 0,
        RoadType::Avenue => 1,
        RoadType::Boulevard => 2,
        RoadType::Highway => 3,
        RoadType::OneWay => 4,
        RoadType::Path => 5,
    }
}

/// Decode a u8 discriminant back to `RoadType`.
fn u8_to_road_type(v: u8) -> RoadType {
    match v {
        0 => RoadType::Local,
        1 => RoadType::Avenue,
        2 => RoadType::Boulevard,
        3 => RoadType::Highway,
        4 => RoadType::OneWay,
        _ => RoadType::Path,
    }
}

/// Record of a road cell's original road type before superblock conversion.
/// Stores road type as u8 since `RoadType` lacks bitcode derives.
#[derive(Debug, Clone, Encode, Decode)]
pub struct OriginalRoad {
    pub x: u16,
    pub y: u16,
    /// Road type stored as u8 discriminant (see `road_type_to_u8`).
    pub road_type_id: u8,
}

impl OriginalRoad {
    /// Create a new record from grid coordinates and a road type.
    pub fn new(x: usize, y: usize, road_type: RoadType) -> Self {
        Self {
            x: x as u16,
            y: y as u16,
            road_type_id: road_type_to_u8(road_type),
        }
    }

    /// Get the stored road type.
    pub fn road_type(&self) -> RoadType {
        u8_to_road_type(self.road_type_id)
    }
}

/// Per-superblock activation state.
#[derive(Debug, Clone, Encode, Decode)]
pub struct SuperblockPolicyEntry {
    /// Index into `SuperblockState::superblocks`.
    pub superblock_index: usize,
    /// Original road types that were converted when the policy was activated.
    pub original_roads: Vec<OriginalRoad>,
    /// Whether this superblock policy is currently active (roads converted).
    pub active: bool,
}

// =============================================================================
// Resource
// =============================================================================

/// Tracks which superblocks have active district policies and stores
/// original road types for reversion.
#[derive(Resource, Clone, Encode, Decode)]
pub struct SuperblockPolicyState {
    /// Active policy entries keyed by superblock index.
    pub entries: Vec<SuperblockPolicyEntry>,
    /// Cached happiness bonus (computed from number of active superblocks).
    pub happiness_bonus: f32,
    /// Cached monthly upkeep cost across all active superblocks.
    pub monthly_cost: f64,
    /// Perimeter congestion multiplier applied to traffic on perimeter roads.
    pub perimeter_congestion: f32,
}

impl Default for SuperblockPolicyState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            happiness_bonus: 0.0,
            monthly_cost: 0.0,
            perimeter_congestion: 1.0,
        }
    }
}

impl SuperblockPolicyState {
    /// Get the happiness bonus for a cell. Returns a bonus if the cell is
    /// in the interior of an active superblock, 0.0 otherwise.
    pub fn cell_happiness_bonus(&self, x: usize, y: usize, sb_state: &SuperblockState) -> f32 {
        if !sb_state.is_interior(x, y) {
            return 0.0;
        }
        if !self.has_active_policy_at(x, y, sb_state) {
            return 0.0;
        }
        self.happiness_bonus
    }

    /// Check if any active policy covers the given cell's superblock.
    fn has_active_policy_at(&self, x: usize, y: usize, sb_state: &SuperblockState) -> bool {
        for entry in &self.entries {
            if !entry.active {
                continue;
            }
            if entry.superblock_index < sb_state.superblocks.len() {
                let sb = &sb_state.superblocks[entry.superblock_index];
                if sb.contains(x, y) {
                    return true;
                }
            }
        }
        false
    }

    /// Count total active superblocks.
    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|e| e.active).count()
    }

    /// Activate a superblock policy: convert interior roads to paths,
    /// store originals for reversion.
    pub fn activate(
        &mut self,
        superblock_index: usize,
        sb_state: &SuperblockState,
        grid: &mut WorldGrid,
    ) -> bool {
        // Check superblock exists
        if superblock_index >= sb_state.superblocks.len() {
            return false;
        }

        // Check not already activated
        if self
            .entries
            .iter()
            .any(|e| e.superblock_index == superblock_index && e.active)
        {
            return false;
        }

        let sb = &sb_state.superblocks[superblock_index];
        let mut original_roads = Vec::new();

        // Convert interior road cells to Path
        for y in sb.y0..=sb.y1.min(GRID_HEIGHT - 1) {
            for x in sb.x0..=sb.x1.min(GRID_WIDTH - 1) {
                if sb.is_interior(x, y) && grid.get(x, y).cell_type == CellType::Road {
                    let original_type = grid.get(x, y).road_type;
                    if original_type != RoadType::Path {
                        original_roads.push(OriginalRoad::new(x, y, original_type));
                        grid.get_mut(x, y).road_type = RoadType::Path;
                    }
                }
            }
        }

        self.entries.push(SuperblockPolicyEntry {
            superblock_index,
            original_roads,
            active: true,
        });

        self.recompute_cached_values(sb_state);
        true
    }

    /// Revert a superblock policy: restore original road types.
    pub fn revert(&mut self, superblock_index: usize, grid: &mut WorldGrid) -> bool {
        let entry_idx = self
            .entries
            .iter()
            .position(|e| e.superblock_index == superblock_index && e.active);

        let Some(idx) = entry_idx else {
            return false;
        };

        let entry = self.entries.remove(idx);

        // Restore original road types
        for original in &entry.original_roads {
            let x = original.x as usize;
            let y = original.y as usize;
            if grid.in_bounds(x, y) && grid.get(x, y).cell_type == CellType::Road {
                grid.get_mut(x, y).road_type = original.road_type();
            }
        }

        // Recompute will need sb_state but we can't borrow it here,
        // so the system will call recompute_cached_values separately.
        self.happiness_bonus = 0.0;
        self.monthly_cost = 0.0;
        self.perimeter_congestion = 1.0;
        true
    }

    /// Recompute cached bonus/cost values from active entries.
    pub fn recompute_cached_values(&mut self, sb_state: &SuperblockState) {
        let active_count = self.active_count();
        if active_count == 0 {
            self.happiness_bonus = 0.0;
            self.monthly_cost = 0.0;
            self.perimeter_congestion = 1.0;
            return;
        }

        // Happiness bonus scales from min to max based on coverage ratio.
        // More superblock coverage = higher bonus (up to max).
        let coverage = sb_state.coverage_ratio.clamp(0.0, 0.1) / 0.1;
        self.happiness_bonus =
            HAPPINESS_BONUS_MIN + (HAPPINESS_BONUS_MAX - HAPPINESS_BONUS_MIN) * coverage;

        // Monthly cost: per-cell cost * total coverage cells of active superblocks.
        let mut total_cells: u32 = 0;
        for entry in &self.entries {
            if entry.active && entry.superblock_index < sb_state.superblocks.len() {
                let sb = &sb_state.superblocks[entry.superblock_index];
                total_cells += sb.area() as u32;
            }
        }
        self.monthly_cost = total_cells as f64 * MONTHLY_COST_PER_CELL;

        // Perimeter congestion: increases with active superblock count.
        // Base 1.0 + 0.05 per active superblock, capped at PERIMETER_CONGESTION_MULTIPLIER.
        self.perimeter_congestion =
            (1.0 + active_count as f32 * 0.05).min(PERIMETER_CONGESTION_MULTIPLIER);
    }
}

// =============================================================================
// Saveable
// =============================================================================

impl crate::Saveable for SuperblockPolicyState {
    const SAVE_KEY: &'static str = "superblock_policy";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.entries.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// System: apply environmental bonuses to interior superblock cells.
/// Runs on slow tick to reduce per-frame cost.
pub fn apply_superblock_policy_effects(
    slow_timer: Res<SlowTickTimer>,
    policy: Res<SuperblockPolicyState>,
    sb_state: Res<SuperblockState>,
    mut land_value: ResMut<LandValueGrid>,
    mut noise: ResMut<NoisePollutionGrid>,
    mut pollution: ResMut<PollutionGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    if policy.active_count() == 0 {
        return;
    }

    // Scale bonuses by coverage: more coverage means stronger effect.
    let coverage_factor = sb_state.coverage_ratio.clamp(0.0, 0.1) / 0.1;
    let lv_bonus = LAND_VALUE_BONUS_MIN
        + ((LAND_VALUE_BONUS_MAX - LAND_VALUE_BONUS_MIN) as f32 * coverage_factor) as u8;
    let noise_reduction = NOISE_REDUCTION_MIN
        + ((NOISE_REDUCTION_MAX - NOISE_REDUCTION_MIN) as f32 * coverage_factor) as u8;
    let pollution_reduction = POLLUTION_REDUCTION_MIN
        + ((POLLUTION_REDUCTION_MAX - POLLUTION_REDUCTION_MIN) as f32 * coverage_factor) as u8;

    // Apply bonuses to interior cells of active superblocks.
    for entry in &policy.entries {
        if !entry.active {
            continue;
        }
        if entry.superblock_index >= sb_state.superblocks.len() {
            continue;
        }
        let sb = &sb_state.superblocks[entry.superblock_index];

        for y in sb.y0..=sb.y1.min(GRID_HEIGHT - 1) {
            for x in sb.x0..=sb.x1.min(GRID_WIDTH - 1) {
                if !sb.is_interior(x, y) {
                    continue;
                }

                // Land value bonus
                let current_lv = land_value.get(x, y);
                land_value.set(x, y, current_lv.saturating_add(lv_bonus));

                // Noise reduction
                let current_noise = noise.get(x, y);
                noise.set(x, y, current_noise.saturating_sub(noise_reduction));

                // Pollution reduction
                let current_poll = pollution.get(x, y);
                pollution.set(x, y, current_poll.saturating_sub(pollution_reduction));
            }
        }
    }
}

/// System: apply perimeter congestion penalty to traffic grid.
/// Increases traffic volume on perimeter cells of active superblocks.
pub fn apply_perimeter_congestion(
    slow_timer: Res<SlowTickTimer>,
    policy: Res<SuperblockPolicyState>,
    sb_state: Res<SuperblockState>,
    mut traffic: ResMut<crate::traffic::TrafficGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    if policy.active_count() == 0 || policy.perimeter_congestion <= 1.0 {
        return;
    }

    let multiplier = policy.perimeter_congestion;

    for entry in &policy.entries {
        if !entry.active {
            continue;
        }
        if entry.superblock_index >= sb_state.superblocks.len() {
            continue;
        }
        let sb = &sb_state.superblocks[entry.superblock_index];

        for y in sb.y0..=sb.y1.min(GRID_HEIGHT - 1) {
            for x in sb.x0..=sb.x1.min(GRID_WIDTH - 1) {
                if sb.is_perimeter(x, y) {
                    let current = traffic.get(x, y) as f32;
                    let increased = (current * multiplier).min(u16::MAX as f32) as u16;
                    traffic.set(x, y, increased);
                }
            }
        }
    }
}

/// System: recompute cached policy values periodically.
pub fn recompute_policy_cache(
    slow_timer: Res<SlowTickTimer>,
    sb_state: Res<SuperblockState>,
    mut policy: ResMut<SuperblockPolicyState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    policy.recompute_cached_values(&sb_state);
}

// =============================================================================
// Plugin
// =============================================================================

pub struct SuperblockPolicyPlugin;

impl Plugin for SuperblockPolicyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuperblockPolicyState>();

        app.add_systems(
            FixedUpdate,
            (
                recompute_policy_cache.after(crate::superblock::update_superblock_stats),
                apply_superblock_policy_effects
                    .after(recompute_policy_cache)
                    .after(crate::noise::update_noise_pollution)
                    .after(crate::land_value::update_land_value),
                apply_perimeter_congestion.after(recompute_policy_cache),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<SuperblockPolicyState>();
    }
}
