//! Satellite map texture generation from grid, roads, and buildings.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use simulation::buildings::Building;
use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::road_segments::RoadSegmentStore;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;
use simulation::weather::Weather;

use super::colors::{
    road_satellite_color, road_satellite_width, satellite_terrain_color, zone_satellite_color,
};
use super::painting::{paint_circle, paint_grid_cell};
use super::types::TEX_SIZE;

/// Create a blank RGBA image of `TEX_SIZE` x `TEX_SIZE`.
pub(crate) fn create_blank_image() -> Image {
    let size = TEX_SIZE;
    let data = vec![0u8; size * size * 4];
    let mut image = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::linear();
    image
}

/// Generate the satellite map texture from grid, road segments, and buildings.
pub(crate) fn generate_satellite_image(
    grid: &simulation::grid::WorldGrid,
    segments: &RoadSegmentStore,
    weather: &Weather,
    buildings: &Query<&Building>,
    services: &Query<&ServiceBuilding>,
    utilities: &Query<&UtilitySource>,
) -> Image {
    let size = TEX_SIZE;
    let mut pixels = vec![[0u8; 4]; size * size];

    let scale_x = GRID_WIDTH as f32 / size as f32;
    let scale_y = GRID_HEIGHT as f32 / size as f32;

    // Pass 1: Terrain base colors
    for py in 0..size {
        for px in 0..size {
            let gx = ((px as f32 + 0.5) * scale_x) as usize;
            let gy = ((py as f32 + 0.5) * scale_y) as usize;
            let gx = gx.min(GRID_WIDTH - 1);
            let gy = gy.min(GRID_HEIGHT - 1);
            let cell = grid.get(gx, gy);

            pixels[py * size + px] = satellite_terrain_color(cell, weather);
        }
    }

    // Pass 2: Building fills (colored area on top of terrain)
    for building in buildings.iter() {
        let color = zone_satellite_color(building.zone_type, building.level);
        paint_grid_cell(
            &mut pixels,
            size,
            scale_x,
            scale_y,
            building.grid_x,
            building.grid_y,
            color,
        );
    }
    for service in services.iter() {
        paint_grid_cell(
            &mut pixels,
            size,
            scale_x,
            scale_y,
            service.grid_x,
            service.grid_y,
            [180, 180, 220, 255], // blue-gray for services
        );
    }
    for utility in utilities.iter() {
        paint_grid_cell(
            &mut pixels,
            size,
            scale_x,
            scale_y,
            utility.grid_x,
            utility.grid_y,
            [200, 200, 160, 255], // yellow-gray for utilities
        );
    }

    // Pass 3: Road lines from Bezier segments
    for segment in &segments.segments {
        let road_color = road_satellite_color(segment.road_type);
        let line_width = road_satellite_width(segment.road_type);

        let steps = (segment.arc_length / 2.0).max(8.0) as usize;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let pos = segment.evaluate(t);
            // World pos (p0..p3 are in world units / CELL_SIZE) to pixel coords
            let px = pos.x / CELL_SIZE / scale_x;
            let py = pos.y / CELL_SIZE / scale_y;

            paint_circle(&mut pixels, size, px, py, line_width / 2.0, road_color);
        }
    }

    // Convert pixel array to raw bytes
    let mut data = Vec::with_capacity(size * size * 4);
    for pixel in &pixels {
        data.extend_from_slice(pixel);
    }

    let mut image = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::linear();
    image
}
