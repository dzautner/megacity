use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::services::{ServiceBuilding, ServiceType};

/// Noise pollution grid -- higher values = louder area.
/// Values are capped at 100.
#[derive(Resource, bitcode::Encode, bitcode::Decode)]
pub struct NoisePollutionGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for NoisePollutionGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl NoisePollutionGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }

    /// Add noise at a position, capping at 100.
    fn add(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_add(amount).min(100);
    }

    /// Subtract noise at a position (floor at 0).
    fn sub(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_sub(amount);
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation — persists noise pollution grid across save / load
// ---------------------------------------------------------------------------

impl crate::Saveable for NoisePollutionGrid {
    const SAVE_KEY: &'static str = "noise_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

pub fn update_noise_pollution(
    slow_timer: Res<crate::SlowTickTimer>,
    mut noise: ResMut<NoisePollutionGrid>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Clear the grid each update
    noise.levels.fill(0);

    // --- Roads generate noise based on road type ---
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road {
                let road_noise = match cell.road_type {
                    RoadType::Highway => 25u8,
                    RoadType::Boulevard => 15,
                    RoadType::Avenue => 10,
                    RoadType::Local => 5,
                    RoadType::OneWay => 5,
                    RoadType::Path => 0,
                };
                if road_noise > 0 {
                    noise.add(x, y, road_noise);
                }
            }
        }
    }

    // --- Industrial buildings generate noise=20 in 3-cell radius ---
    for building in &buildings {
        if building.zone_type == ZoneType::Industrial {
            let radius = 3i32;
            let intensity = 20u8;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = building.grid_x as i32 + dx;
                    let ny = building.grid_y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        let dist = dx.abs() + dy.abs();
                        let decay = intensity.saturating_sub(dist as u8 * 3);
                        if decay > 0 {
                            noise.add(nx as usize, ny as usize, decay);
                        }
                    }
                }
            }
        }
    }

    // --- Airports generate noise=30 in 5-cell radius ---
    // --- Stadiums generate noise=15 in 3-cell radius ---
    for service in &services {
        match service.service_type {
            ServiceType::SmallAirstrip
            | ServiceType::RegionalAirport
            | ServiceType::InternationalAirport => {
                let (radius, intensity) = match service.service_type {
                    ServiceType::SmallAirstrip => (5i32, 25u8),
                    ServiceType::RegionalAirport => (7i32, 35u8),
                    ServiceType::InternationalAirport => (10i32, 45u8),
                    _ => (5i32, 30u8),
                };
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let nx = service.grid_x as i32 + dx;
                        let ny = service.grid_y as i32 + dy;
                        if nx >= 0
                            && ny >= 0
                            && (nx as usize) < GRID_WIDTH
                            && (ny as usize) < GRID_HEIGHT
                        {
                            let dist = dx.abs() + dy.abs();
                            let decay = intensity.saturating_sub(dist as u8 * 3);
                            if decay > 0 {
                                noise.add(nx as usize, ny as usize, decay);
                            }
                        }
                    }
                }
            }
            ServiceType::Stadium => {
                let radius = 3i32;
                let intensity = 15u8;
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let nx = service.grid_x as i32 + dx;
                        let ny = service.grid_y as i32 + dy;
                        if nx >= 0
                            && ny >= 0
                            && (nx as usize) < GRID_WIDTH
                            && (ny as usize) < GRID_HEIGHT
                        {
                            let dist = dx.abs() + dy.abs();
                            let decay = intensity.saturating_sub(dist as u8 * 3);
                            if decay > 0 {
                                noise.add(nx as usize, ny as usize, decay);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // --- Trees reduce noise: grass cells without buildings reduce noise by 2 in 1-cell radius ---
    // Collect reductions first to avoid borrow conflicts
    let mut reductions: Vec<(usize, usize)> = Vec::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Grass && cell.building_id.is_none() {
                reductions.push((x, y));
            }
        }
    }
    for (cx, cy) in reductions {
        let radius = 1i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    noise.sub(nx as usize, ny as usize, 2);
                }
            }
        }
    }
}

pub struct NoisePlugin;

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoisePollutionGrid>().add_systems(
            FixedUpdate,
            update_noise_pollution
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<NoisePollutionGrid>();
    }
}
