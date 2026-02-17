use bevy::prelude::*;
use crate::config::{GRID_WIDTH, GRID_HEIGHT};
use crate::grid::{CellType, WorldGrid};
use crate::services::ServiceBuilding;
use std::collections::VecDeque;

#[derive(Resource)]
pub struct EducationGrid {
    pub levels: Vec<u8>, // 0=None, 1=Elementary, 2=HighSchool, 3=University
    pub width: usize,
    pub height: usize,
}

impl Default for EducationGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl EducationGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }
}

// System: propagate education from school buildings via BFS through roads.
// Only runs every 100 ticks — education availability changes very slowly.
pub fn propagate_education(
    slow_tick: Res<crate::SlowTickTimer>,
    mut edu_grid: ResMut<EducationGrid>,
    grid: Res<WorldGrid>,
    services: Query<&ServiceBuilding>,
    mut visited_buf: Local<Vec<bool>>,
) {
    if !slow_tick.should_run() {
        return;
    }
    edu_grid.levels.fill(0);

    // Lazily initialize the reusable visited buffer
    let grid_len = GRID_WIDTH * GRID_HEIGHT;
    if visited_buf.len() != grid_len {
        *visited_buf = vec![false; grid_len];
    }

    // Collect education sources sorted by level (highest first so they override)
    let mut sources: Vec<(usize, usize, u8, u32)> = Vec::new();
    for service in &services {
        let level = ServiceBuilding::education_level(service.service_type);
        if level > 0 {
            let range = (service.radius / 16.0) as u32; // Convert pixel radius to grid cells
            sources.push((service.grid_x, service.grid_y, level, range));
        }
    }
    // Sort highest level first
    sources.sort_by(|a, b| b.2.cmp(&a.2));

    for (sx, sy, level, range) in sources {
        bfs_education(&mut edu_grid, &grid, &mut visited_buf, sx, sy, level, range);
    }
}

fn bfs_education(
    edu_grid: &mut EducationGrid,
    grid: &WorldGrid,
    visited: &mut [bool],
    sx: usize,
    sy: usize,
    level: u8,
    range: u32,
) {
    visited.fill(false);
    let mut queue = VecDeque::new();
    queue.push_back(((sx, sy), 0u32));
    visited[sy * GRID_WIDTH + sx] = true;

    // Mark source
    if edu_grid.get(sx, sy) < level {
        edu_grid.set(sx, sy, level);
    }

    while let Some(((x, y), dist)) = queue.pop_front() {
        if dist >= range { continue; }
        let (neighbors, ncount) = grid.neighbors4(x, y);
        for &(nx, ny) in &neighbors[..ncount] {
            let idx = ny * GRID_WIDTH + nx;
            if visited[idx] { continue; }
            let cell_type = grid.get(nx, ny).cell_type;
            if cell_type == CellType::Road || cell_type == CellType::Grass {
                visited[idx] = true;
                if edu_grid.get(nx, ny) < level {
                    edu_grid.set(nx, ny, level);
                }
                if cell_type == CellType::Road {
                    queue.push_back(((nx, ny), dist + 1));
                }
            }
        }
    }
}
