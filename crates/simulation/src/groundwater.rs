use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::pollution::PollutionGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::weather::{Weather, WeatherEvent};

/// Groundwater table level per cell (0=dry, 255=saturated).
#[derive(Resource)]
pub struct GroundwaterGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for GroundwaterGrid {
    fn default() -> Self {
        Self {
            levels: vec![128; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl GroundwaterGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }

    fn add(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_add(amount);
    }

    fn sub(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_sub(amount);
    }
}

/// Water quality per cell (0=contaminated, 255=pure).
#[derive(Resource)]
pub struct WaterQualityGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for WaterQualityGrid {
    fn default() -> Self {
        Self {
            levels: vec![200; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl WaterQualityGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }

    fn add(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_add(amount);
    }

    fn sub(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_sub(amount);
    }
}

/// Aggregated groundwater statistics for the UI.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroundwaterStats {
    pub avg_level: f32,
    pub avg_quality: f32,
    pub contaminated_cells: u32,
    pub treatment_capacity: u32,
}

/// Initialize groundwater levels based on terrain elevation.
/// Low elevation = high water table, near water cells = high water table.
pub fn init_groundwater(grid: &WorldGrid) -> (GroundwaterGrid, WaterQualityGrid) {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let mut gw = GroundwaterGrid {
        levels: vec![0; total],
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
    };
    let wq = WaterQualityGrid {
        levels: vec![200; total],
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
    };

    // Phase 1: Base groundwater from elevation (inverted: low elevation = high water)
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            // Elevation is roughly 0.15-0.65 range in the Tel Aviv map.
            // Invert: lower elevation => higher groundwater.
            let inv_elevation = (1.0 - cell.elevation).clamp(0.0, 1.0);
            let base_level = (inv_elevation * 200.0) as u8;
            gw.set(x, y, base_level);
        }
    }

    // Phase 2: Boost groundwater near water cells (rivers, coast)
    // Use a simple proximity boost: for each water cell, boost neighbors in radius 5
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type != CellType::Water {
                continue;
            }
            let radius = 5i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                        continue;
                    }
                    let dist = dx.abs() + dy.abs();
                    let boost = (30i32 - dist * 5).max(0) as u8;
                    if boost > 0 {
                        gw.add(nx as usize, ny as usize, boost);
                    }
                }
            }
        }
    }

    (gw, wq)
}

/// Main groundwater update system. Runs every 100 ticks via SlowTickTimer.
///
/// - Industrial buildings contaminate nearby groundwater (reduce quality)
/// - Air pollution seeps into groundwater (PollutionGrid affects quality)
/// - Water treatment plants purify water in radius (increase quality)
/// - Groundwater level drops near heavy building density (water usage)
/// - Rain replenishes groundwater levels
pub fn update_groundwater(
    slow_timer: Res<crate::SlowTickTimer>,
    mut groundwater: ResMut<GroundwaterGrid>,
    mut quality: ResMut<WaterQualityGrid>,
    mut stats: ResMut<GroundwaterStats>,
    grid: Res<WorldGrid>,
    pollution: Res<PollutionGrid>,
    weather: Res<Weather>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Natural quality recovery (slow) ---
    for val in quality.levels.iter_mut() {
        *val = val.saturating_add(1);
    }

    // --- Phase 2: Industrial buildings contaminate nearby groundwater ---
    for building in &buildings {
        if building.zone_type != ZoneType::Industrial {
            continue;
        }
        let intensity = 8i32 + building.level as i32 * 4;
        let radius = 6i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = building.grid_x as i32 + dx;
                let ny = building.grid_y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let dist = dx.abs() + dy.abs();
                let decay = (intensity - dist).max(0) as u8;
                if decay > 0 {
                    quality.sub(nx as usize, ny as usize, decay);
                }
            }
        }
    }

    // --- Phase 3: Air pollution seeps into groundwater quality ---
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let air_poll = pollution.get(x, y);
            if air_poll > 10 {
                // Pollution above 10 reduces water quality: scale 1/10 of air pollution
                let seep = (air_poll / 10).min(5);
                quality.sub(x, y, seep);
            }
        }
    }

    // --- Phase 4: Water treatment plants purify groundwater ---
    let mut treatment_count = 0u32;
    for service in &services {
        if service.service_type != ServiceType::WaterTreatmentPlant {
            continue;
        }
        treatment_count += 1;
        let radius = 12i32;
        let purification = 20u8;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let dist = dx.abs() + dy.abs();
                let effect = purification.saturating_sub(dist as u8);
                if effect > 0 {
                    quality.add(nx as usize, ny as usize, effect);
                }
            }
        }
    }

    // --- Phase 5: Groundwater level drops near heavy building density ---
    // Each building in radius 3 draws down groundwater slightly
    for building in &buildings {
        let usage = match building.zone_type {
            ZoneType::Industrial => 3u8,
            ZoneType::CommercialHigh | ZoneType::Office => 2u8,
            ZoneType::ResidentialHigh | ZoneType::CommercialLow => 1u8,
            _ => 0u8,
        };
        if usage == 0 {
            continue;
        }
        let radius = 3i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = building.grid_x as i32 + dx;
                let ny = building.grid_y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let dist = dx.abs() + dy.abs();
                let draw = usage.saturating_sub(dist as u8);
                if draw > 0 {
                    groundwater.sub(nx as usize, ny as usize, draw);
                }
            }
        }
    }

    // --- Phase 6: Well pumps extract groundwater ---
    for service in &services {
        if service.service_type != ServiceType::WellPump {
            continue;
        }
        // Well pumps draw down groundwater in a small radius
        let radius = 4i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let dist = dx.abs() + dy.abs();
                let draw = 2u8.saturating_sub(dist as u8 / 2);
                if draw > 0 {
                    groundwater.sub(nx as usize, ny as usize, draw);
                }
            }
        }
    }

    // --- Phase 7: Rain replenishes groundwater ---
    let rain_amount: u8 = match weather.current_event {
        WeatherEvent::Rain => 3,
        WeatherEvent::Storm => 5,
        _ => 0,
    };
    if rain_amount > 0 {
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if grid.get(x, y).cell_type != CellType::Water {
                    groundwater.add(x, y, rain_amount);
                }
            }
        }
    }

    // --- Phase 8: Recharge from nearby water bodies (rivers/coast) ---
    // Cells adjacent to water slowly recharge
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type != CellType::Water {
                continue;
            }
            let neighbors: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
            for (dx, dy) in neighbors {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;
                if grid.get(ux, uy).cell_type != CellType::Water {
                    groundwater.add(ux, uy, 1);
                }
            }
        }
    }

    // --- Compute stats ---
    let total = GRID_WIDTH * GRID_HEIGHT;
    let mut level_sum: u64 = 0;
    let mut quality_sum: u64 = 0;
    let mut contaminated: u32 = 0;

    for i in 0..total {
        level_sum += groundwater.levels[i] as u64;
        quality_sum += quality.levels[i] as u64;
        if quality.levels[i] < 50 {
            contaminated += 1;
        }
    }

    stats.avg_level = level_sum as f32 / total as f32;
    stats.avg_quality = quality_sum as f32 / total as f32;
    stats.contaminated_cells = contaminated;
    stats.treatment_capacity = treatment_count;
}

/// Citizens living in areas with low groundwater quality suffer health penalties.
/// Quality < 50 = health penalty proportional to how bad it is.
pub fn groundwater_health_penalty(
    slow_timer: Res<crate::SlowTickTimer>,
    quality: Res<WaterQualityGrid>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for (mut details, home) in &mut citizens {
        let hx = home.grid_x;
        let hy = home.grid_y;
        if hx >= GRID_WIDTH || hy >= GRID_HEIGHT {
            continue;
        }

        let q = quality.get(hx, hy);
        if q < 50 {
            // Scale penalty: quality 0 => 1.5 health/tick, quality 49 => ~0.03
            let deficit = (50 - q) as f32;
            let penalty = deficit * 0.03;
            details.health = (details.health - penalty).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::WorldGrid;

    #[test]
    fn test_groundwater_grid_default() {
        let gw = GroundwaterGrid::default();
        assert_eq!(gw.levels.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(gw.get(0, 0), 128);
    }

    #[test]
    fn test_water_quality_grid_default() {
        let wq = WaterQualityGrid::default();
        assert_eq!(wq.levels.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(wq.get(0, 0), 200);
    }

    #[test]
    fn test_groundwater_add_sub() {
        let mut gw = GroundwaterGrid::default();
        gw.set(5, 5, 100);
        gw.add(5, 5, 50);
        assert_eq!(gw.get(5, 5), 150);
        gw.sub(5, 5, 200);
        assert_eq!(gw.get(5, 5), 0); // saturating sub
        gw.set(5, 5, 250);
        gw.add(5, 5, 10);
        assert_eq!(gw.get(5, 5), 255); // saturating add
    }

    #[test]
    fn test_water_quality_add_sub() {
        let mut wq = WaterQualityGrid::default();
        wq.set(3, 3, 100);
        wq.sub(3, 3, 30);
        assert_eq!(wq.get(3, 3), 70);
        wq.add(3, 3, 200);
        assert_eq!(wq.get(3, 3), 255); // saturating add
        wq.sub(3, 3, 255);
        assert_eq!(wq.get(3, 3), 0); // saturating sub
    }

    #[test]
    fn test_init_groundwater_elevation_correlation() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Set up: low elevation cell and high elevation cell
        grid.get_mut(10, 10).elevation = 0.2; // low => high water
        grid.get_mut(10, 10).cell_type = CellType::Grass;
        grid.get_mut(20, 20).elevation = 0.8; // high => low water
        grid.get_mut(20, 20).cell_type = CellType::Grass;

        let (gw, _wq) = init_groundwater(&grid);

        // Low elevation should have more groundwater than high elevation
        assert!(
            gw.get(10, 10) > gw.get(20, 20),
            "low elevation ({}) should have more groundwater than high ({})",
            gw.get(10, 10),
            gw.get(20, 20)
        );
    }

    #[test]
    fn test_init_groundwater_near_water_boost() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Set equal elevations
        grid.get_mut(50, 50).elevation = 0.5;
        grid.get_mut(50, 50).cell_type = CellType::Grass;
        grid.get_mut(100, 100).elevation = 0.5;
        grid.get_mut(100, 100).cell_type = CellType::Grass;

        // Put a water cell adjacent to (50,50)
        grid.get_mut(51, 50).cell_type = CellType::Water;
        grid.get_mut(51, 50).elevation = 0.2;

        let (gw, _wq) = init_groundwater(&grid);

        // Cell near water should have higher groundwater
        assert!(
            gw.get(50, 50) > gw.get(100, 100),
            "near-water cell ({}) should have more groundwater than distant cell ({})",
            gw.get(50, 50),
            gw.get(100, 100)
        );
    }

    #[test]
    fn test_groundwater_stats_default() {
        let stats = GroundwaterStats::default();
        assert_eq!(stats.avg_level, 0.0);
        assert_eq!(stats.avg_quality, 0.0);
        assert_eq!(stats.contaminated_cells, 0);
        assert_eq!(stats.treatment_capacity, 0);
    }
}
