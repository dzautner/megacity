use bevy::prelude::*;
use rand::seq::IteratorRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::cumulative_zoning::{select_effective_zone, CumulativeZoningState};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::weather::ConstructionModifiers;
use crate::zones::{is_adjacent_to_road, ZoneDemand};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub zone_type: ZoneType,
    pub level: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u32,
    pub occupants: u32,
}

/// Component for mixed-use buildings that have both commercial ground floors
/// and residential upper floors. Attached alongside [`Building`] when the
/// zone is `ZoneType::MixedUse`.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MixedUseBuilding {
    pub commercial_capacity: u32,
    pub commercial_occupants: u32,
    pub residential_capacity: u32,
    pub residential_occupants: u32,
}

impl MixedUseBuilding {
    /// Returns (commercial_capacity, residential_capacity) for a given building level.
    /// L1=(5,8), L2=(15,30), L3=(20+20 office=40, 80), L4=(40+80=120, 200), L5=(80+200=280, 400)
    pub fn capacities_for_level(level: u8) -> (u32, u32) {
        match level {
            1 => (5, 8),
            2 => (15, 30),
            3 => (40, 80),
            4 => (120, 200),
            5 => (280, 400),
            _ => (0, 0),
        }
    }

    /// Total capacity (commercial + residential) for a given level.
    pub fn total_capacity_for_level(level: u8) -> u32 {
        let (c, r) = Self::capacities_for_level(level);
        c + r
    }
}

/// Marker component for buildings that are still under construction.
/// While present, the building cannot accept occupants.
/// Approximately 10 seconds at 10Hz fixed timestep (100 ticks).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct UnderConstruction {
    pub ticks_remaining: u32,
    pub total_ticks: u32,
}

impl Building {
    pub fn capacity_for_level(zone: ZoneType, level: u8) -> u32 {
        match (zone, level) {
            // Low-density residential: houses and small apartments
            (ZoneType::ResidentialLow, 1) => 10,
            (ZoneType::ResidentialLow, 2) => 30,
            (ZoneType::ResidentialLow, 3) => 80,
            // Medium-density residential: townhouses, duplexes, small apartments
            (ZoneType::ResidentialMedium, 1) => 15,
            (ZoneType::ResidentialMedium, 2) => 50,
            (ZoneType::ResidentialMedium, 3) => 120,
            (ZoneType::ResidentialMedium, 4) => 250,
            // High-density residential: apartment blocks and towers
            (ZoneType::ResidentialHigh, 1) => 50,
            (ZoneType::ResidentialHigh, 2) => 200,
            (ZoneType::ResidentialHigh, 3) => 500,
            (ZoneType::ResidentialHigh, 4) => 1000,
            (ZoneType::ResidentialHigh, 5) => 2000,
            // Low-density commercial: shops and small stores
            (ZoneType::CommercialLow, 1) => 8,
            (ZoneType::CommercialLow, 2) => 25,
            (ZoneType::CommercialLow, 3) => 60,
            // High-density commercial: malls and department stores
            (ZoneType::CommercialHigh, 1) => 30,
            (ZoneType::CommercialHigh, 2) => 100,
            (ZoneType::CommercialHigh, 3) => 300,
            (ZoneType::CommercialHigh, 4) => 600,
            (ZoneType::CommercialHigh, 5) => 1200,
            // Industrial: factories and warehouses
            (ZoneType::Industrial, 1) => 20,
            (ZoneType::Industrial, 2) => 60,
            (ZoneType::Industrial, 3) => 150,
            (ZoneType::Industrial, 4) => 300,
            (ZoneType::Industrial, 5) => 600,
            // Office: office towers
            (ZoneType::Office, 1) => 30,
            (ZoneType::Office, 2) => 100,
            (ZoneType::Office, 3) => 300,
            (ZoneType::Office, 4) => 700,
            (ZoneType::Office, 5) => 1500,
            // Mixed-use: total capacity (commercial + residential)
            (ZoneType::MixedUse, l) => MixedUseBuilding::total_capacity_for_level(l),
            _ => 0,
        }
    }
}

/// Returns the maximum building level allowed by the Floor Area Ratio (FAR)
/// constraint for the given zone type.
///
/// For each candidate level 1..=5, the implied FAR is computed as:
///   implied_far = (capacity_for_level(level) * 20.0) / 256.0
///
/// The highest level where implied_far <= zone.default_far() is returned.
/// Always returns at least 1 (minimum building level).
pub fn max_level_for_far(zone: ZoneType) -> u32 {
    let far_limit = zone.default_far();
    let mut best = 1u32;
    for level in 1..=5u8 {
        let capacity = Building::capacity_for_level(zone, level);
        if capacity == 0 {
            break;
        }
        let implied_far = (capacity as f32 * 20.0) / 256.0;
        if implied_far <= far_limit {
            best = level as u32;
        }
    }
    best
}

/// Tick interval for building spawner (in sim ticks)
const SPAWN_INTERVAL: u32 = 2;

#[derive(Resource, Default)]
pub struct BuildingSpawnTimer(pub u32);

/// Maintained set of cells eligible for building placement per zone type.
/// Rebuilt from the grid whenever it changes (zoning, buildings, roads, infrastructure).
#[derive(Resource, Default)]
pub struct EligibleCells {
    pub cells: Vec<(ZoneType, Vec<(usize, usize)>)>,
}

/// Rebuild the eligible cell lists from the grid.
/// Runs only when the grid resource has changed (Bevy change detection).
pub fn rebuild_eligible_cells(grid: Res<WorldGrid>, mut eligible: ResMut<EligibleCells>) {
    if !grid.is_changed() {
        return;
    }

    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];

    let mut result: Vec<(ZoneType, Vec<(usize, usize)>)> = Vec::with_capacity(zones.len());

    for zone in zones {
        let mut cells = Vec::new();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.zone == zone
                    && cell.building_id.is_none()
                    && cell.cell_type == CellType::Grass
                    && cell.has_power
                    && cell.has_water
                    && is_adjacent_to_road(&grid, x, y)
                {
                    cells.push((x, y));
                }
            }
        }
        result.push((zone, cells));
    }

    eligible.cells = result;
}

pub fn building_spawner(
    mut commands: Commands,
    mut grid: ResMut<WorldGrid>,
    demand: Res<ZoneDemand>,
    mut timer: ResMut<BuildingSpawnTimer>,
    eligible: Res<EligibleCells>,
    cumulative_zoning: Res<CumulativeZoningState>,
) {
    timer.0 += 1;
    if timer.0 < SPAWN_INTERVAL {
        return;
    }
    timer.0 = 0;

    let mut rng = rand::thread_rng();

    for (zone, cells) in &eligible.cells {
        if demand.demand_for(*zone) < 0.1 || cells.is_empty() {
            continue;
        }

        let spawn_chance = demand.demand_for(*zone);
        let max_per_tick = 50;

        // Sample up to max_per_tick cells randomly from eligible list
        let selected: Vec<(usize, usize)> = cells
            .iter()
            .copied()
            .choose_multiple(&mut rng, max_per_tick.min(cells.len()));

        for (x, y) in selected {
            // Double-check cell is still eligible (could have changed since rebuild)
            let cell = grid.get(x, y);
            if cell.zone != *zone || cell.building_id.is_some() {
                continue;
            }

            if rng.gen::<f32>() > spawn_chance {
                continue;
            }

            // When cumulative zoning is enabled, select the highest-value
            // permitted use based on market demand. Otherwise use the
            // cell's own zone type (exclusive zoning).
            let effective_zone = if cumulative_zoning.enabled {
                select_effective_zone(*zone, &demand)
            } else {
                *zone
            };

            // Cap initial level by FAR constraint (initial level is 1, but
            // max_level_for_far is guaranteed >= 1, so this is a safety check)
            let far_cap = max_level_for_far(effective_zone) as u8;
            let initial_level = 1u8.min(far_cap);
            let capacity = Building::capacity_for_level(effective_zone, initial_level);
            let construction_ticks = 100; // ~10 seconds at 10Hz
            let entity = if effective_zone == ZoneType::MixedUse {
                let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(initial_level);
                commands
                    .spawn((
                        Building {
                            zone_type: effective_zone,
                            level: initial_level,
                            grid_x: x,
                            grid_y: y,
                            capacity,
                            occupants: 0,
                        },
                        MixedUseBuilding {
                            commercial_capacity: comm_cap,
                            commercial_occupants: 0,
                            residential_capacity: res_cap,
                            residential_occupants: 0,
                        },
                        UnderConstruction {
                            ticks_remaining: construction_ticks,
                            total_ticks: construction_ticks,
                        },
                    ))
                    .id()
            } else {
                commands
                    .spawn((
                        Building {
                            zone_type: effective_zone,
                            level: initial_level,
                            grid_x: x,
                            grid_y: y,
                            capacity,
                            occupants: 0,
                        },
                        UnderConstruction {
                            ticks_remaining: construction_ticks,
                            total_ticks: construction_ticks,
                        },
                    ))
                    .id()
            };

            // This grid mutation triggers Bevy change detection,
            // so rebuild_eligible_cells will re-run next tick
            grid.get_mut(x, y).building_id = Some(entity);
        }
    }
}

/// Advances construction progress each tick. When complete, removes the
/// `UnderConstruction` component so the building becomes operational.
/// While under construction, occupants are clamped to 0.
///
/// Progress is scaled by `ConstructionModifiers::speed_factor`:
/// - 0.0 = halted (storm), no progress
/// - 0.5 = half speed (rain), progress every other tick
/// - 1.0+ = normal or faster
pub fn progress_construction(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Building, &mut UnderConstruction)>,
    modifiers: Res<ConstructionModifiers>,
    tick: Res<crate::TickCounter>,
) {
    let speed = modifiers.speed_factor;

    for (entity, mut building, mut uc) in &mut query {
        // Ensure no occupants while under construction
        building.occupants = 0;

        if uc.ticks_remaining > 0 {
            // Determine whether to make progress this tick based on speed_factor.
            // speed >= 1.0: always progress (1 tick per tick)
            // 0 < speed < 1: progress on a fraction of ticks using modular arithmetic
            // speed == 0.0: halted (storm)
            let should_progress = if speed <= 0.0 {
                false
            } else if speed >= 1.0 {
                true
            } else {
                // Use tick counter to distribute progress evenly.
                // E.g., speed=0.5 -> progress every 2nd tick; speed=0.3 -> every ~3rd tick.
                let period = (1.0 / speed).round() as u64;
                period > 0 && tick.0.is_multiple_of(period)
            };

            if should_progress {
                uc.ticks_remaining -= 1;
            }
        }

        if uc.ticks_remaining == 0 {
            commands.entity(entity).remove::<UnderConstruction>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_capacity() {
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialLow, 1),
            10
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialLow, 2),
            30
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialHigh, 3),
            500
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialHigh, 5),
            2000
        );
        // Medium-density residential
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialMedium, 1),
            15
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialMedium, 2),
            50
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialMedium, 3),
            120
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialMedium, 4),
            250
        );
        assert_eq!(Building::capacity_for_level(ZoneType::CommercialLow, 1), 8);
        assert_eq!(Building::capacity_for_level(ZoneType::Industrial, 1), 20);
        assert_eq!(Building::capacity_for_level(ZoneType::Office, 1), 30);
    }

    #[test]
    fn test_mixed_use_capacity_per_level() {
        // L1=(5 comm, 8 res)
        assert_eq!(MixedUseBuilding::capacities_for_level(1), (5, 8));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(1), 13);
        // L2=(15, 30)
        assert_eq!(MixedUseBuilding::capacities_for_level(2), (15, 30));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(2), 45);
        // L3=(40, 80) — 20 commercial + 20 office = 40 commercial
        assert_eq!(MixedUseBuilding::capacities_for_level(3), (40, 80));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(3), 120);
        // L4=(120, 200) — 40 commercial + 80 office = 120 commercial
        assert_eq!(MixedUseBuilding::capacities_for_level(4), (120, 200));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(4), 320);
        // L5=(280, 400) — 80 commercial + 200 office = 280 commercial
        assert_eq!(MixedUseBuilding::capacities_for_level(5), (280, 400));
        assert_eq!(MixedUseBuilding::total_capacity_for_level(5), 680);
    }

    #[test]
    fn test_mixed_use_building_capacity_matches_total() {
        for level in 1..=5 {
            let total = Building::capacity_for_level(ZoneType::MixedUse, level);
            let (c, r) = MixedUseBuilding::capacities_for_level(level);
            assert_eq!(total, c + r, "Level {} total mismatch", level);
        }
    }

    #[test]
    fn test_building_only_in_zoned_cells() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        for cell in &grid.cells {
            assert!(cell.building_id.is_none());
            assert_eq!(cell.zone, ZoneType::None);
        }
    }

    #[test]
    fn test_eligible_cells_finds_zoned_cells() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place a road at (10, 10)
        grid.get_mut(10, 10).cell_type = CellType::Road;
        // Zone cells adjacent to road
        for x in 8..=9 {
            let cell = grid.get_mut(x, 10);
            cell.zone = ZoneType::ResidentialLow;
            cell.has_power = true;
            cell.has_water = true;
        }

        let mut res_cells = Vec::new();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.zone == ZoneType::ResidentialLow
                    && cell.building_id.is_none()
                    && cell.cell_type == CellType::Grass
                    && cell.has_power
                    && cell.has_water
                    && is_adjacent_to_road(&grid, x, y)
                {
                    res_cells.push((x, y));
                }
            }
        }

        assert_eq!(res_cells.len(), 2);
        assert!(res_cells.contains(&(8, 10)));
        assert!(res_cells.contains(&(9, 10)));
    }

    #[test]
    fn test_eligible_cells_excludes_occupied() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        let cell = grid.get_mut(9, 10);
        cell.zone = ZoneType::Industrial;
        cell.has_power = true;
        cell.has_water = true;
        // Mark as having a building
        cell.building_id = Some(Entity::from_raw(1));

        let mut eligible_count = 0;
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.zone == ZoneType::Industrial
                    && cell.building_id.is_none()
                    && cell.cell_type == CellType::Grass
                    && cell.has_power
                    && cell.has_water
                    && is_adjacent_to_road(&grid, x, y)
                {
                    eligible_count += 1;
                }
            }
        }

        assert_eq!(eligible_count, 0);
    }

    #[test]
    fn test_far_residential_low_limits_level() {
        // ResidentialLow FAR=0.5 should constrain building to low levels.
        // L1: capacity=10, implied_far = 10*20/256 = 0.78 > 0.5
        // So max_level_for_far should return 1 (the minimum).
        let max = max_level_for_far(ZoneType::ResidentialLow);
        assert!(max >= 1, "max_level_for_far must return at least 1");
        assert!(
            max <= 3,
            "ResidentialLow FAR=0.5 should limit to low levels, got {}",
            max
        );
    }

    #[test]
    fn test_far_residential_high_allows_higher_levels() {
        // ResidentialHigh FAR=3.0 should allow higher levels than ResidentialLow.
        let high = max_level_for_far(ZoneType::ResidentialHigh);
        let low = max_level_for_far(ZoneType::ResidentialLow);
        assert!(
            high >= low,
            "ResidentialHigh should allow at least as many levels as ResidentialLow"
        );
    }

    #[test]
    fn test_far_returns_at_least_one() {
        // All zone types (except None) should return at least 1.
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            let max = max_level_for_far(zone);
            assert!(
                max >= 1,
                "max_level_for_far({:?}) must be >= 1, got {}",
                zone,
                max
            );
        }
    }

    #[test]
    fn test_far_respects_zone_max_level() {
        // max_level_for_far should not exceed the zone's max_level.
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            let far_max = max_level_for_far(zone);
            let zone_max = zone.max_level() as u32;
            assert!(
                far_max <= zone_max,
                "max_level_for_far({:?})={} should not exceed max_level={}",
                zone,
                far_max,
                zone_max
            );
        }
    }

    #[test]
    fn test_default_far_values() {
        assert_eq!(ZoneType::ResidentialLow.default_far(), 0.5);
        assert_eq!(ZoneType::ResidentialMedium.default_far(), 1.5);
        assert_eq!(ZoneType::ResidentialHigh.default_far(), 3.0);
        assert_eq!(ZoneType::CommercialLow.default_far(), 1.5);
        assert_eq!(ZoneType::CommercialHigh.default_far(), 3.0);
        assert_eq!(ZoneType::Industrial.default_far(), 0.8);
        assert_eq!(ZoneType::Office.default_far(), 1.5);
        assert_eq!(ZoneType::MixedUse.default_far(), 3.0);
        assert_eq!(ZoneType::None.default_far(), 0.0);
    }
}

pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildingSpawnTimer>()
            .init_resource::<EligibleCells>()
            .add_systems(
                FixedUpdate,
                (
                    rebuild_eligible_cells,
                    building_spawner,
                    progress_construction,
                )
                    .chain()
                    .after(crate::zones::update_zone_demand)
                    .in_set(crate::SimulationSet::PreSim),
            );
    }
}
