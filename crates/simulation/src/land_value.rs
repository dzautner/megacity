use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::pollution::PollutionGrid;
use crate::services::ServiceBuilding;
use crate::urban_growth_boundary::UrbanGrowthBoundary;
use bevy::prelude::*;

/// Exponential smoothing factor: how quickly land values converge to the
/// computed "target" each slow-tick.  Lower values = more momentum / slower
/// change.  At 0.1 a value reaches ~87 % of the target after 20 ticks.
const SMOOTHING_ALPHA: f32 = 0.1;

/// Weight given to each of the 8 neighbours during diffusion.
/// Total neighbour weight = 8 * DIFFUSION_WEIGHT; self weight = 1 - 8 * DIFFUSION_WEIGHT.
/// With 0.02 per neighbour, self retains 84 % of its value.
const DIFFUSION_WEIGHT: f32 = 0.02;

#[derive(Resource, bitcode::Encode, bitcode::Decode)]
pub struct LandValueGrid {
    pub values: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for LandValueGrid {
    fn default() -> Self {
        Self {
            values: vec![50; GRID_WIDTH * GRID_HEIGHT], // start at baseline 50
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl LandValueGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.values[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.values[y * self.width + x] = val;
    }
    pub fn average(&self) -> f32 {
        if self.values.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.values.iter().map(|&v| v as u64).sum();
        sum as f32 / self.values.len() as f32
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation — persists land values across save / load
// ---------------------------------------------------------------------------

impl crate::Saveable for LandValueGrid {
    const SAVE_KEY: &'static str = "land_value";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Always save — the grid is large and we can't cheaply detect
        // "still at default".
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// System: compute target values, apply exponential smoothing + diffusion
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn update_land_value(
    slow_timer: Res<crate::SlowTickTimer>,
    mut land_value: ResMut<LandValueGrid>,
    grid: Res<WorldGrid>,
    pollution: Res<PollutionGrid>,
    services: Query<&ServiceBuilding>,
    waste_collection: Res<crate::garbage::WasteCollectionGrid>,
    waste_accumulation: Res<crate::waste_effects::WasteAccumulation>,
    ugb: Res<UrbanGrowthBoundary>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let total = GRID_WIDTH * GRID_HEIGHT;

    // ---- Phase 1: compute raw "target" value per cell -----------------------
    let mut target = vec![0i32; total];

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            let mut value: i32 = 50;

            // Water proximity boost
            if cell.cell_type == CellType::Water {
                value = 30; // water itself isn't valuable
            } else {
                // Check for nearby water
                let (lv_n4, lv_n4c) = grid.neighbors4(x, y);
                for &(nx, ny) in &lv_n4[..lv_n4c] {
                    if grid.get(nx, ny).cell_type == CellType::Water {
                        value += 15;
                        break;
                    }
                }
            }

            // Industrial reduces nearby land value
            if cell.zone == ZoneType::Industrial {
                value -= 15;
            }

            // Pollution reduces value
            let poll = pollution.get(x, y) as i32;
            value -= poll / 3;

            // Uncollected waste reduces land value (WASTE-003: -10% penalty).
            let uncollected = waste_collection.uncollected(x, y);
            if uncollected > 100.0 {
                let penalty =
                    (value as f32 * crate::garbage::UNCOLLECTED_WASTE_LAND_VALUE_FACTOR) as i32;
                value -= penalty;
            }

            // Accumulated waste reduces land value (WASTE-010: -20% if nearby > 500 lbs).
            let waste_modifier =
                crate::waste_effects::waste_land_value_modifier(&waste_accumulation, x, y);
            if waste_modifier < 1.0 {
                value = (value as f32 * waste_modifier) as i32;
            }

            // Urban Growth Boundary: premium inside, penalty outside (ZONE-009).
            value += ugb.land_value_modifier(x, y);

            target[y * GRID_WIDTH + x] = value.clamp(0, 255);
        }
    }

    // Parks and services boost target values in radius
    for service in &services {
        let (boost, radius): (i32, i32) = if ServiceBuilding::is_park(service.service_type) {
            (20, 8)
        } else {
            (10, 6)
        };

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let dist = dx.abs() + dy.abs();
                    let effect = (boost - dist * 2).max(0);
                    let idx = ny as usize * GRID_WIDTH + nx as usize;
                    target[idx] = (target[idx] + effect).min(255);
                }
            }
        }
    }

    // ---- Phase 2: exponential smoothing toward targets ----------------------
    // new = alpha * target + (1 - alpha) * previous
    for i in 0..total {
        let prev = land_value.values[i] as f32;
        let tgt = target[i] as f32;
        let smoothed = SMOOTHING_ALPHA * tgt + (1.0 - SMOOTHING_ALPHA) * prev;
        land_value.values[i] = smoothed.round().clamp(0.0, 255.0) as u8;
    }

    // ---- Phase 3: neighbourhood diffusion -----------------------------------
    // Each cell blends slightly with its 8 neighbours.
    // We read from a snapshot so writes don't cascade within one tick.
    let snapshot = land_value.values.clone();
    let self_weight = 1.0 - 8.0 * DIFFUSION_WEIGHT;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let mut sum = snapshot[idx] as f32 * self_weight;

            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        sum += snapshot[ny as usize * GRID_WIDTH + nx as usize] as f32
                            * DIFFUSION_WEIGHT;
                    }
                    // Out-of-bounds neighbours contribute 0 (natural boundary).
                }
            }

            land_value.values[idx] = sum.round().clamp(0.0, 255.0) as u8;
        }
    }
}

pub struct LandValuePlugin;

impl Plugin for LandValuePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LandValueGrid>().add_systems(
            FixedUpdate,
            update_land_value
                .after(crate::pollution::update_pollution)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<LandValueGrid>();
    }
}
