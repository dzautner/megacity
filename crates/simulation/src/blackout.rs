//! POWER-016: Blackout and Rolling Blackout System
//!
//! Implements blackout mechanics when power demand exceeds supply:
//! - Detects deficit when `EnergyGrid.reserve_margin < 0`
//! - Sheds load by priority: Low -> Normal -> High -> Critical
//! - Rolling blackout rotates affected Standard-priority cells every 4 ticks
//! - Sets `has_power = false` on affected grid cells
//! - Tracks blackout duration for extended blackout effects
//! - Hospital without power: 5% patient mortality per game-day
//! - Extended blackouts (>3 game-days) trigger citizen exodus via happiness

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::energy_demand::{EnergyConsumer, EnergyGrid, LoadPriority};
use crate::grid::WorldGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;
use crate::{decode_or_warn, Saveable, SimulationSet, TickCounter};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// How often (in ticks) the blackout system runs.
const BLACKOUT_INTERVAL: u64 = 4;

/// Rolling blackout rotation period in ticks.
const ROLLING_ROTATION_TICKS: u64 = 4;

/// Hospital patient mortality rate per game-day without power (5%).
const HOSPITAL_MORTALITY_RATE: f32 = 0.05;

/// Extended blackout threshold in game-days before exodus effects.
const EXTENDED_BLACKOUT_DAYS: u32 = 3;

/// Happiness penalty applied to citizens in blacked-out areas.
const BLACKOUT_HAPPINESS_PENALTY: f32 = 10.0;

/// Maximum number of citizens to penalize per tick (performance bound).
const MAX_CITIZEN_PENALTIES_PER_TICK: usize = 5000;

// ---------------------------------------------------------------------------
// Serializable state (bitcode-safe subset)
// ---------------------------------------------------------------------------

/// Persisted subset of blackout state (no transient grid).
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Default)]
struct BlackoutSaveData {
    pub active: bool,
    pub affected_cell_count: u32,
    pub rotation_offset: u32,
    pub duration_days: u32,
    pub start_day: u32,
    pub load_shed_fraction: f32,
    pub shed_by_tier: [u32; 4],
    pub hospital_casualties: u32,
}

// ---------------------------------------------------------------------------
// BlackoutState resource
// ---------------------------------------------------------------------------

/// Tracks the current blackout state across the city.
#[derive(Resource, Debug, Clone)]
pub struct BlackoutState {
    /// Whether a blackout is currently active (demand > supply).
    pub active: bool,
    /// Total number of grid cells currently without power due to blackout.
    pub affected_cell_count: u32,
    /// Current rotation offset for rolling blackouts on Standard-priority cells.
    pub rotation_offset: u32,
    /// Number of game-days the current blackout has been active.
    pub duration_days: u32,
    /// The game-day when the current blackout started (0 if no blackout).
    pub start_day: u32,
    /// Fraction of total demand being shed (0.0 = none, 1.0 = total).
    pub load_shed_fraction: f32,
    /// Number of cells shed per priority tier: [Low, Normal, High, Critical].
    pub shed_by_tier: [u32; 4],
    /// Number of hospital casualties from power loss this session.
    pub hospital_casualties: u32,
    /// Per-cell blackout flag grid (true = blacked out).
    /// Recomputed each tick from EnergyGrid state; not persisted.
    pub blackout_grid: Vec<bool>,
}

impl Default for BlackoutState {
    fn default() -> Self {
        let grid_size = GRID_WIDTH * GRID_HEIGHT;
        Self {
            active: false,
            affected_cell_count: 0,
            rotation_offset: 0,
            duration_days: 0,
            start_day: 0,
            load_shed_fraction: 0.0,
            shed_by_tier: [0; 4],
            hospital_casualties: 0,
            blackout_grid: vec![false; grid_size],
        }
    }
}

impl Saveable for BlackoutState {
    const SAVE_KEY: &'static str = "blackout_state";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        let data = BlackoutSaveData {
            active: self.active,
            affected_cell_count: self.affected_cell_count,
            rotation_offset: self.rotation_offset,
            duration_days: self.duration_days,
            start_day: self.start_day,
            load_shed_fraction: self.load_shed_fraction,
            shed_by_tier: self.shed_by_tier,
            hospital_casualties: self.hospital_casualties,
        };
        Some(bitcode::encode(&data))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let data: BlackoutSaveData = decode_or_warn(Self::SAVE_KEY, bytes);
        let grid_size = GRID_WIDTH * GRID_HEIGHT;
        Self {
            active: data.active,
            affected_cell_count: data.affected_cell_count,
            rotation_offset: data.rotation_offset,
            duration_days: data.duration_days,
            start_day: data.start_day,
            load_shed_fraction: data.load_shed_fraction,
            shed_by_tier: data.shed_by_tier,
            hospital_casualties: data.hospital_casualties,
            blackout_grid: vec![false; grid_size],
        }
    }
}

// ---------------------------------------------------------------------------
// Priority mapping helpers
// ---------------------------------------------------------------------------

/// Map a `LoadPriority` to a tier index (0=Low, 1=Normal, 2=High, 3=Critical).
fn priority_tier(priority: LoadPriority) -> usize {
    match priority {
        LoadPriority::Low => 0,
        LoadPriority::Normal => 1,
        LoadPriority::High => 2,
        LoadPriority::Critical => 3,
    }
}

/// Determine the effective load priority for a grid cell based on its zone.
fn cell_priority(grid: &WorldGrid, x: usize, y: usize) -> LoadPriority {
    let cell = grid.get(x, y);
    EnergyConsumer::priority_for_zone(cell.zone)
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Collects cells with power grouped by priority tier, then determines which
/// cells to shed based on the current deficit.
///
/// Runs every `BLACKOUT_INTERVAL` ticks.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_blackout(
    tick: Res<TickCounter>,
    energy_grid: Res<EnergyGrid>,
    clock: Res<GameClock>,
    mut blackout: ResMut<BlackoutState>,
    mut grid: ResMut<WorldGrid>,
    service_buildings: Query<&ServiceBuilding>,
) {
    if !tick.0.is_multiple_of(BLACKOUT_INTERVAL) {
        return;
    }

    let demand = energy_grid.total_demand_mwh;
    let supply = energy_grid.total_supply_mwh;
    let deficit = demand - supply;

    // Clear the per-cell blackout grid each evaluation.
    for v in blackout.blackout_grid.iter_mut() {
        *v = false;
    }
    blackout.shed_by_tier = [0; 4];
    blackout.affected_cell_count = 0;

    // No deficit — clear blackout state and return.
    if deficit <= 0.0 || demand <= 0.0 {
        if blackout.active {
            blackout.active = false;
            blackout.duration_days = 0;
            blackout.start_day = 0;
            blackout.load_shed_fraction = 0.0;
        }
        return;
    }

    // Blackout is active.
    if !blackout.active {
        blackout.active = true;
        blackout.start_day = clock.day;
    }
    blackout.duration_days = clock.day.saturating_sub(blackout.start_day);
    blackout.load_shed_fraction = (deficit / demand).clamp(0.0, 1.0);

    // Build a set of grid positions occupied by Critical service buildings
    // so we can promote those cells to Critical priority.
    let mut critical_cells = std::collections::HashSet::new();
    for service in &service_buildings {
        let prio = EnergyConsumer::priority_for_service(service.service_type);
        if prio == LoadPriority::Critical {
            critical_cells.insert((service.grid_x, service.grid_y));
        }
    }

    // Collect powered cells grouped by tier.
    let mut cells_by_tier: [Vec<usize>; 4] = [vec![], vec![], vec![], vec![]];

    let width = grid.width;
    for y in 0..grid.height {
        for x in 0..width {
            let idx = y * width + x;
            let cell = &grid.cells[idx];
            if !cell.has_power {
                continue;
            }
            let prio = if critical_cells.contains(&(x, y)) {
                LoadPriority::Critical
            } else {
                cell_priority(&grid, x, y)
            };
            let tier = priority_tier(prio);
            cells_by_tier[tier].push(idx);
        }
    }

    // Calculate how many cells we need to shed.
    let total_powered: usize = cells_by_tier.iter().map(|v| v.len()).sum();
    if total_powered == 0 {
        return;
    }
    let cells_to_shed = (total_powered as f32 * blackout.load_shed_fraction).ceil() as usize;
    let mut remaining_to_shed = cells_to_shed;

    // Shed in reverse priority order: Low (0) first, then Normal (1),
    // then High (2), then Critical (3).
    for tier in 0..4 {
        if remaining_to_shed == 0 {
            break;
        }
        let tier_cells = &cells_by_tier[tier];
        if tier_cells.is_empty() {
            continue;
        }

        // For Normal (tier=1) cells, apply rolling blackout rotation.
        if tier == 1 {
            let rotation = blackout.rotation_offset as usize;
            let count = tier_cells.len().min(remaining_to_shed);
            for i in 0..count {
                let rotated_idx = (i + rotation) % tier_cells.len();
                let grid_idx = tier_cells[rotated_idx];
                blackout.blackout_grid[grid_idx] = true;
            }
            blackout.shed_by_tier[tier] = count as u32;
            remaining_to_shed -= count;
        } else {
            let count = tier_cells.len().min(remaining_to_shed);
            for &grid_idx in tier_cells.iter().take(count) {
                blackout.blackout_grid[grid_idx] = true;
            }
            blackout.shed_by_tier[tier] = count as u32;
            remaining_to_shed -= count;
        }
    }

    // Apply blackout to the world grid: set has_power = false for affected cells.
    let total_affected = cells_to_shed - remaining_to_shed;
    blackout.affected_cell_count = total_affected as u32;
    for (idx, &blacked_out) in blackout.blackout_grid.iter().enumerate() {
        if blacked_out {
            grid.cells[idx].has_power = false;
        }
    }

    // Advance rolling blackout rotation.
    if tick.0.is_multiple_of(ROLLING_ROTATION_TICKS) {
        blackout.rotation_offset = blackout.rotation_offset.wrapping_add(1);
    }
}

/// Apply happiness penalty to citizens living in blacked-out cells.
///
/// Runs after `evaluate_blackout`. The main happiness system already applies
/// a penalty for `has_power == false`; this adds an additional blackout-specific
/// penalty and doubles it during extended blackouts (>3 days).
pub fn apply_blackout_happiness_penalty(
    tick: Res<TickCounter>,
    blackout: Res<BlackoutState>,
    grid: Res<WorldGrid>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if !tick.0.is_multiple_of(BLACKOUT_INTERVAL) {
        return;
    }
    if !blackout.active {
        return;
    }

    let extended = blackout.duration_days >= EXTENDED_BLACKOUT_DAYS;
    let extra_penalty = if extended {
        BLACKOUT_HAPPINESS_PENALTY * 2.0
    } else {
        BLACKOUT_HAPPINESS_PENALTY
    };

    let mut count = 0;
    for (mut details, home) in &mut citizens {
        if count >= MAX_CITIZEN_PENALTIES_PER_TICK {
            break;
        }
        let cell = grid.get(home.grid_x, home.grid_y);
        if !cell.has_power {
            details.happiness = (details.happiness - extra_penalty).max(0.0);
            count += 1;
        }
    }
}

/// Check hospitals without power and apply patient mortality.
///
/// When a hospital's grid cell has no power, 5% of the building's occupants
/// are lost per game-day. This system runs every `BLACKOUT_INTERVAL` ticks
/// and applies a fractional per-tick mortality.
pub fn apply_hospital_mortality(
    tick: Res<TickCounter>,
    mut blackout: ResMut<BlackoutState>,
    grid: Res<WorldGrid>,
    service_buildings: Query<&ServiceBuilding>,
    mut buildings: Query<&mut Building>,
) {
    if !tick.0.is_multiple_of(BLACKOUT_INTERVAL) {
        return;
    }
    if !blackout.active {
        return;
    }

    // A game-day = 24h * 60min = 1440 ticks at 1 min/tick.
    // With BLACKOUT_INTERVAL=4, this runs 360 times per game-day.
    let ticks_per_day: f32 = 24.0 * 60.0;
    let runs_per_day = ticks_per_day / BLACKOUT_INTERVAL as f32;
    let per_run_mortality = HOSPITAL_MORTALITY_RATE / runs_per_day;

    for service in &service_buildings {
        let is_hospital = matches!(
            service.service_type,
            ServiceType::Hospital | ServiceType::MedicalCenter | ServiceType::MedicalClinic
        );
        if !is_hospital {
            continue;
        }

        let cell = grid.get(service.grid_x, service.grid_y);
        if cell.has_power {
            continue;
        }

        if let Some(building_entity) = cell.building_id {
            if let Ok(mut building) = buildings.get_mut(building_entity) {
                let casualties = (building.occupants as f32 * per_run_mortality).ceil() as u32;
                if casualties > 0 {
                    building.occupants = building.occupants.saturating_sub(casualties);
                    blackout.hospital_casualties += casualties;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct BlackoutPlugin;

impl Plugin for BlackoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlackoutState>();

        // Register for save/load.
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<BlackoutState>();

        app.add_systems(
            FixedUpdate,
            (
                evaluate_blackout,
                apply_blackout_happiness_penalty.after(evaluate_blackout),
                apply_hospital_mortality.after(evaluate_blackout),
            )
                .after(crate::energy_dispatch::dispatch_energy)
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
    fn test_blackout_state_default() {
        let state = BlackoutState::default();
        assert!(!state.active);
        assert_eq!(state.affected_cell_count, 0);
        assert_eq!(state.rotation_offset, 0);
        assert_eq!(state.duration_days, 0);
        assert_eq!(state.hospital_casualties, 0);
        assert_eq!(state.blackout_grid.len(), GRID_WIDTH * GRID_HEIGHT);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let state = BlackoutState {
            active: true,
            affected_cell_count: 42,
            rotation_offset: 7,
            duration_days: 2,
            start_day: 5,
            load_shed_fraction: 0.35,
            shed_by_tier: [10, 20, 12, 0],
            hospital_casualties: 3,
            blackout_grid: vec![],
        };

        let bytes = state.save_to_bytes().unwrap();
        let restored = BlackoutState::load_from_bytes(&bytes);

        assert!(restored.active);
        assert_eq!(restored.affected_cell_count, 42);
        assert_eq!(restored.rotation_offset, 7);
        assert_eq!(restored.duration_days, 2);
        assert_eq!(restored.start_day, 5);
        assert!((restored.load_shed_fraction - 0.35).abs() < f32::EPSILON);
        assert_eq!(restored.shed_by_tier, [10, 20, 12, 0]);
        assert_eq!(restored.hospital_casualties, 3);
        assert_eq!(restored.blackout_grid.len(), GRID_WIDTH * GRID_HEIGHT);
    }

    #[test]
    fn test_priority_tier_ordering() {
        assert_eq!(priority_tier(LoadPriority::Low), 0);
        assert_eq!(priority_tier(LoadPriority::Normal), 1);
        assert_eq!(priority_tier(LoadPriority::High), 2);
        assert_eq!(priority_tier(LoadPriority::Critical), 3);
    }
}
