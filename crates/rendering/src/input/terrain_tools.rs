use bevy::prelude::*;

use simulation::grid::WorldGrid;

use crate::terrain_render::{mark_chunk_dirty_at, ChunkDirty, TerrainChunk};

pub(crate) fn apply_terrain_raise(
    gx: usize,
    gy: usize,
    grid: &mut WorldGrid,
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
) {
    let radius = 3i32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = gx as i32 + dx;
            let ny = gy as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < grid.width && (ny as usize) < grid.height {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= radius as f32 {
                    let strength = 0.01 * (1.0 - dist / radius as f32);
                    let cell = grid.get_mut(nx as usize, ny as usize);
                    cell.elevation = (cell.elevation + strength).min(1.0);
                    if cell.elevation > 0.35 && cell.cell_type == simulation::grid::CellType::Water
                    {
                        cell.cell_type = simulation::grid::CellType::Grass;
                    }
                    mark_chunk_dirty_at(nx as usize, ny as usize, chunks, commands);
                }
            }
        }
    }
}

pub(crate) fn apply_terrain_lower(
    gx: usize,
    gy: usize,
    grid: &mut WorldGrid,
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
) {
    let radius = 3i32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = gx as i32 + dx;
            let ny = gy as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < grid.width && (ny as usize) < grid.height {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= radius as f32 {
                    let strength = 0.01 * (1.0 - dist / radius as f32);
                    let cell = grid.get_mut(nx as usize, ny as usize);
                    cell.elevation = (cell.elevation - strength).max(0.0);
                    mark_chunk_dirty_at(nx as usize, ny as usize, chunks, commands);
                }
            }
        }
    }
}

pub(crate) fn apply_terrain_level(
    gx: usize,
    gy: usize,
    grid: &mut WorldGrid,
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
) {
    let target_elev = grid.get(gx, gy).elevation;
    let radius = 3i32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = gx as i32 + dx;
            let ny = gy as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < grid.width && (ny as usize) < grid.height {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= radius as f32 {
                    let cell = grid.get_mut(nx as usize, ny as usize);
                    cell.elevation += (target_elev - cell.elevation) * 0.3;
                    mark_chunk_dirty_at(nx as usize, ny as usize, chunks, commands);
                }
            }
        }
    }
}

pub(crate) fn apply_terrain_water(
    gx: usize,
    gy: usize,
    grid: &mut WorldGrid,
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
) {
    let radius = 2i32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = gx as i32 + dx;
            let ny = gy as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < grid.width && (ny as usize) < grid.height {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= radius as f32 {
                    let cell = grid.get_mut(nx as usize, ny as usize);
                    cell.cell_type = simulation::grid::CellType::Water;
                    cell.elevation = 0.3;
                    mark_chunk_dirty_at(nx as usize, ny as usize, chunks, commands);
                }
            }
        }
    }
}
