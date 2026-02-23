// =============================================================================
// World generation: Tel Aviv terrain, roads, zoning, buildings, utilities,
// services, and initial citizens.
// =============================================================================

mod roads;
mod spawning;
mod zoning;

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::{CellType, WorldGrid};
use crate::groundwater;
use crate::natural_resources;
use crate::natural_resources::ResourceGrid;
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;

pub use self::roads::build_tel_aviv_roads;
pub use self::spawning::{
    spawn_tel_aviv_buildings, spawn_tel_aviv_citizens, spawn_tel_aviv_services,
    spawn_tel_aviv_utilities,
};
pub use self::zoning::apply_zones;

/// Marker resource that, when present, causes `init_world` to skip the
/// Tel Aviv map generation. Used by the test harness to start with a blank grid.
#[derive(Resource)]
pub struct SkipWorldInit;

pub fn init_world(
    mut commands: Commands,
    mut segments: ResMut<RoadSegmentStore>,
    skip: Option<Res<SkipWorldInit>>,
) {
    if skip.is_some() {
        return;
    }
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    // --- Tel Aviv terrain: Mediterranean coast on west, Yarkon River in north ---
    generate_tel_aviv_terrain(&mut grid);

    // Natural resources
    let mut resource_grid = ResourceGrid::default();
    let elevations: Vec<f32> = grid.cells.iter().map(|c| c.elevation).collect();
    natural_resources::generate_resources(&mut resource_grid, &elevations, 42);
    commands.insert_resource(resource_grid);

    let mut roads = RoadNetwork::default();

    // --- Road network using Bezier segments ---
    build_tel_aviv_roads(&mut segments, &mut grid, &mut roads);

    // --- Zoning ---
    apply_zones(&mut grid);

    // --- Buildings ---
    let building_entities = spawn_tel_aviv_buildings(&mut commands, &mut grid);

    // --- Utilities ---
    spawn_tel_aviv_utilities(&mut commands, &mut grid);

    // --- Services ---
    spawn_tel_aviv_services(&mut commands, &mut grid);

    // --- Citizens ---
    spawn_tel_aviv_citizens(&mut commands, &grid, &building_entities);

    // --- Groundwater ---
    let (gw_grid, wq_grid) = groundwater::init_groundwater(&grid);
    commands.insert_resource(gw_grid);
    commands.insert_resource(wq_grid);

    let budget = CityBudget {
        treasury: 100_000.0,
        ..CityBudget::default()
    };
    commands.insert_resource(budget);
    commands.insert_resource(grid);
    commands.insert_resource(roads);
}

// =============================================================================
// Tel Aviv terrain
// =============================================================================

fn generate_tel_aviv_terrain(grid: &mut WorldGrid) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            let xf = x as f32;
            let yf = y as f32;

            let coast = coastline_x(yf);

            // Yarkon River (east-west around y~185, meandering)
            let yarkon_cy = 185.0 + 1.5 * (xf * 0.04).sin();
            let yarkon_hw = 2.0;
            let is_yarkon = (yf - yarkon_cy).abs() < yarkon_hw && xf > coast - 3.0 && xf < 195.0;

            let noise =
                ((x.wrapping_mul(7919).wrapping_add(y.wrapping_mul(6271))) % 100) as f32 / 100.0;

            let cell = grid.get_mut(x, y);

            if xf < coast || is_yarkon {
                cell.cell_type = CellType::Water;
                cell.elevation = 0.15 + noise * 0.1;
            } else {
                cell.cell_type = CellType::Grass;
                let dist_from_coast = (xf - coast).max(0.0);
                cell.elevation = 0.35 + (dist_from_coast * 0.002).min(0.3) + noise * 0.05;
            }
        }
    }
}

/// Coastline x-position as a function of y (north-south).
/// Models Tel Aviv's coast: Jaffa headland in the south, gentle curves northward.
pub(super) fn coastline_x(y: f32) -> f32 {
    let base = 55.0;

    // Jaffa headland pushes west around y=40-60
    let jaffa = if y > 25.0 && y < 75.0 {
        let t = (y - 50.0) / 25.0;
        -7.0 * (1.0 - t * t).max(0.0)
    } else {
        0.0
    };

    // Gentle coastal waves
    let wave = 2.5 * (y * 0.03).sin() + 1.5 * (y * 0.08).cos();

    base + jaffa + wave
}

// =============================================================================
// Helpers
// =============================================================================

pub(super) fn find_free_grass_cell(
    grid: &WorldGrid,
    cx: usize,
    cy: usize,
    search_radius: usize,
) -> Option<(usize, usize)> {
    for r in 0..=search_radius {
        let min_x = cx.saturating_sub(r);
        let max_x = (cx + r).min(GRID_WIDTH - 1);
        let min_y = cy.saturating_sub(r);
        let max_y = (cy + r).min(GRID_HEIGHT - 1);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if r > 0 {
                    let dx = x.abs_diff(cx);
                    let dy = y.abs_diff(cy);
                    if dx != r && dy != r {
                        continue;
                    }
                }
                let cell = grid.get(x, y);
                if cell.cell_type == CellType::Grass && cell.building_id.is_none() {
                    return Some((x, y));
                }
            }
        }
    }
    None
}
