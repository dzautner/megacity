use bevy::prelude::*;

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};

/// Penalty applied to happiness for cells with unprocessed deaths nearby.
pub const DEATH_CARE_PENALTY: f32 = 8.0;

/// Radius (in cells) around unprocessed deaths that incurs a happiness penalty.
pub const UNPROCESSED_DEATH_RADIUS: i32 = 3;

/// Per-cell accumulation of unburied/unprocessed dead bodies.
#[derive(Resource)]
pub struct DeathCareGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for DeathCareGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl DeathCareGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }

    /// Increment the death counter on a cell, saturating at 255.
    pub fn record_death(&mut self, x: usize, y: usize) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_add(1);
    }

    /// Check whether any cell in a small radius has unprocessed deaths.
    pub fn has_nearby_unprocessed(&self, cx: usize, cy: usize) -> bool {
        for dy in -UNPROCESSED_DEATH_RADIUS..=UNPROCESSED_DEATH_RADIUS {
            for dx in -UNPROCESSED_DEATH_RADIUS..=UNPROCESSED_DEATH_RADIUS {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx >= 0
                    && ny >= 0
                    && (nx as usize) < GRID_WIDTH
                    && (ny as usize) < GRID_HEIGHT
                    && self.levels[ny as usize * self.width + nx as usize] > 0
                {
                    return true;
                }
            }
        }
        false
    }
}

/// City-wide death care statistics.
#[derive(Resource, Debug, Clone, Default)]
pub struct DeathCareStats {
    pub total_deaths_this_month: u32,
    pub processed_this_month: u32,
    pub unprocessed: u32,
}

/// Death care processing system.
///
/// Runs every 20 ticks. Cemeteries and crematoriums collect deaths within
/// their coverage radius, decrementing the `DeathCareGrid`.
pub fn death_care_processing(
    tick: Res<crate::TickCounter>,
    mut death_grid: ResMut<DeathCareGrid>,
    mut stats: ResMut<DeathCareStats>,
    services: Query<&ServiceBuilding>,
) {
    if !tick.0.is_multiple_of(20) {
        return;
    }

    let mut processed_this_tick: u32 = 0;

    // Cemeteries and crematoriums collect deaths within their radius
    for service in &services {
        let collection_rate = match service.service_type {
            ServiceType::Cemetery => 2u8,    // collects up to 2 per cell per tick
            ServiceType::Crematorium => 3u8, // crematoriums process faster
            _ => continue,
        };

        let radius = (service.radius / CELL_SIZE) as i32;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = sx + dx;
                let ny = sy + dy;
                if nx < 0 || ny < 0 || nx >= GRID_WIDTH as i32 || ny >= GRID_HEIGHT as i32 {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;
                let current = death_grid.get(ux, uy);
                if current > 0 {
                    let collected = current.min(collection_rate);
                    death_grid.set(ux, uy, current - collected);
                    processed_this_tick += collected as u32;
                }
            }
        }
    }

    stats.processed_this_month += processed_this_tick;

    // Count total unprocessed across the grid
    stats.unprocessed = death_grid.levels.iter().map(|&v| v as u32).sum();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_death_care_grid_default_is_all_zeros() {
        let grid = DeathCareGrid::default();
        assert_eq!(grid.levels.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(grid.levels.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_death_care_stats_tracks_deaths() {
        let mut stats = DeathCareStats::default();
        assert_eq!(stats.total_deaths_this_month, 0);
        assert_eq!(stats.processed_this_month, 0);
        assert_eq!(stats.unprocessed, 0);

        stats.total_deaths_this_month += 5;
        stats.processed_this_month += 3;
        stats.unprocessed = 2;

        assert_eq!(stats.total_deaths_this_month, 5);
        assert_eq!(stats.processed_this_month, 3);
        assert_eq!(stats.unprocessed, 2);
    }

    #[test]
    fn test_cemetery_service_type_name() {
        assert_eq!(ServiceType::Cemetery.name(), "Cemetery");
    }

    #[test]
    fn test_crematorium_service_type_name() {
        assert_eq!(ServiceType::Crematorium.name(), "Crematorium");
    }

    #[test]
    fn test_record_death_increments() {
        let mut grid = DeathCareGrid::default();
        assert_eq!(grid.get(10, 10), 0);
        grid.record_death(10, 10);
        assert_eq!(grid.get(10, 10), 1);
        grid.record_death(10, 10);
        assert_eq!(grid.get(10, 10), 2);
    }

    #[test]
    fn test_record_death_saturates_at_255() {
        let mut grid = DeathCareGrid::default();
        grid.set(10, 10, 254);
        grid.record_death(10, 10);
        assert_eq!(grid.get(10, 10), 255);
        grid.record_death(10, 10);
        assert_eq!(grid.get(10, 10), 255); // saturates
    }

    #[test]
    fn test_has_nearby_unprocessed() {
        let mut grid = DeathCareGrid::default();
        assert!(!grid.has_nearby_unprocessed(10, 10));
        grid.record_death(12, 10); // within radius 3
        assert!(grid.has_nearby_unprocessed(10, 10));
    }

    #[test]
    fn test_has_nearby_unprocessed_out_of_range() {
        let mut grid = DeathCareGrid::default();
        grid.record_death(20, 20); // far away
        assert!(!grid.has_nearby_unprocessed(10, 10));
    }
}

pub struct DeathCarePlugin;

impl Plugin for DeathCarePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeathCareGrid>()
            .init_resource::<DeathCareStats>()
            .add_systems(
                FixedUpdate,
                death_care_processing.after(crate::imports_exports::process_trade),
            );
    }
}
