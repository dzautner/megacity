use bevy::prelude::*;
use bevy::render::mesh::Indices;

use simulation::config::{CELL_SIZE, CHUNKS_X, CHUNKS_Y, CHUNK_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::education::EducationGrid;
use simulation::garbage::GarbageGrid;
use simulation::grid::{CellType, RoadType, WorldGrid, ZoneType};
use simulation::groundwater::{GroundwaterGrid, WaterQualityGrid};
use simulation::land_value::LandValueGrid;
use simulation::noise::NoisePollutionGrid;
use simulation::pollution::PollutionGrid;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::snow::SnowGrid;
use simulation::traffic::TrafficGrid;
use simulation::water_pollution::WaterPollutionGrid;
use simulation::weather::{Season, Weather};

use simulation::colorblind::{ColorblindMode, ColorblindSettings};

use crate::color_ramps::{self, CIVIDIS, GROUNDWATER_LEVEL, GROUNDWATER_QUALITY, INFERNO, VIRIDIS};
use crate::colorblind_palette;
use crate::overlay::OverlayMode;

pub struct OverlayGrids<'a> {
    pub pollution: Option<&'a PollutionGrid>,
    pub land_value: Option<&'a LandValueGrid>,
    pub education: Option<&'a EducationGrid>,
    pub garbage: Option<&'a GarbageGrid>,
    pub traffic: Option<&'a TrafficGrid>,
    pub noise: Option<&'a NoisePollutionGrid>,
    pub water_pollution: Option<&'a WaterPollutionGrid>,
    pub groundwater: Option<&'a GroundwaterGrid>,
    pub water_quality: Option<&'a WaterQualityGrid>,
    pub snow: Option<&'a SnowGrid>,
}

impl<'a> OverlayGrids<'a> {
    pub fn none() -> Self {
        Self {
            pollution: None,
            land_value: None,
            education: None,
            garbage: None,
            traffic: None,
            noise: None,
            water_pollution: None,
            groundwater: None,
            water_quality: None,
            snow: None,
        }
    }
}

#[derive(Component)]
pub struct TerrainChunk {
    pub chunk_x: usize,
    pub chunk_y: usize,
}

#[derive(Component)]
pub struct ChunkDirty;

#[allow(clippy::too_many_arguments)]
pub fn spawn_terrain_chunks(
    mut commands: Commands,
    grid: Res<WorldGrid>,
    roads: Res<RoadNetwork>,
    segments: Res<RoadSegmentStore>,
    weather: Res<Weather>,
    snow_grid: Res<SnowGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let overlay = OverlayMode::None;
    let mut overlay_grids = OverlayGrids::none();
    overlay_grids.snow = Some(&snow_grid);
    for cy in 0..CHUNKS_Y {
        for cx in 0..CHUNKS_X {
            let mesh = build_chunk_mesh(
                &grid,
                &roads,
                &segments,
                cx,
                cy,
                &overlay,
                &overlay_grids,
                weather.season,
                ColorblindMode::Normal,
            );
            let (wx, wz) = chunk_world_pos(cx, cy);

            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.9,
                    ..default()
                })),
                Transform::from_xyz(wx, 0.0, wz),
                TerrainChunk {
                    chunk_x: cx,
                    chunk_y: cy,
                },
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn dirty_chunks_on_overlay_change(
    overlay: Res<crate::overlay::OverlayState>,
    pollution_grid: Res<PollutionGrid>,
    land_value_grid: Res<LandValueGrid>,
    education_grid: Res<EducationGrid>,
    garbage_grid: Res<GarbageGrid>,
    traffic_grid: Res<TrafficGrid>,
    noise_grid: Res<NoisePollutionGrid>,
    water_pollution_grid: Res<WaterPollutionGrid>,
    groundwater_grid: Res<GroundwaterGrid>,
    water_quality_grid: Res<WaterQualityGrid>,
    weather: Res<Weather>,
    snow_grid: Res<SnowGrid>,
    cb_settings: Res<ColorblindSettings>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    mut commands: Commands,
) {
    use crate::overlay::OverlayMode;

    if overlay.is_changed()
        || weather.is_changed()
        || snow_grid.is_changed()
        || cb_settings.is_changed()
    {
        mark_all_chunks_dirty(&chunks, &mut commands);
        return;
    }

    // When an overlay is active, dirty chunks if the underlying data changed
    let data_changed = match overlay.mode {
        OverlayMode::None => false,
        OverlayMode::Power | OverlayMode::Water => false, // power/water is per-cell in WorldGrid, handled by grid changes
        OverlayMode::Pollution => pollution_grid.is_changed(),
        OverlayMode::LandValue => land_value_grid.is_changed(),
        OverlayMode::Education => education_grid.is_changed(),
        OverlayMode::Garbage => garbage_grid.is_changed(),
        OverlayMode::Traffic => traffic_grid.is_changed(),
        OverlayMode::Noise => noise_grid.is_changed(),
        OverlayMode::WaterPollution => water_pollution_grid.is_changed(),
        OverlayMode::GroundwaterLevel => groundwater_grid.is_changed(),
        OverlayMode::GroundwaterQuality => water_quality_grid.is_changed(),
        OverlayMode::Wind => false, // Wind overlay uses gizmos, no terrain recolor
    };

    if data_changed {
        mark_all_chunks_dirty(&chunks, &mut commands);
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn rebuild_dirty_chunks(
    mut commands: Commands,
    grid: Res<WorldGrid>,
    roads: Res<RoadNetwork>,
    segments: Res<RoadSegmentStore>,
    overlay: Res<crate::overlay::OverlayState>,
    pollution_grid: Res<PollutionGrid>,
    land_value_grid: Res<LandValueGrid>,
    education_grid: Res<EducationGrid>,
    garbage_grid: Res<GarbageGrid>,
    traffic_grid: Res<TrafficGrid>,
    noise_grid: Res<NoisePollutionGrid>,
    water_pollution_grid: Res<WaterPollutionGrid>,
    groundwater_grids: (Res<GroundwaterGrid>, Res<WaterQualityGrid>),
    snow_params: (Res<SnowGrid>, Res<Weather>),
    cb_settings: Res<ColorblindSettings>,
    query: (
        Query<(Entity, &TerrainChunk, &Mesh3d), With<ChunkDirty>>,
        ResMut<Assets<Mesh>>,
    ),
) {
    let (groundwater_grid, water_quality_grid) = groundwater_grids;
    let (snow_grid, weather) = snow_params;
    let (query, mut meshes) = query;
    let cb_mode = cb_settings.mode;
    let overlay_grids = OverlayGrids {
        pollution: Some(&pollution_grid),
        land_value: Some(&land_value_grid),
        education: Some(&education_grid),
        garbage: Some(&garbage_grid),
        traffic: Some(&traffic_grid),
        noise: Some(&noise_grid),
        water_pollution: Some(&water_pollution_grid),
        groundwater: Some(&groundwater_grid),
        water_quality: Some(&water_quality_grid),
        snow: Some(&snow_grid),
    };
    for (entity, chunk, mesh_handle) in &query {
        let new_mesh = build_chunk_mesh(
            &grid,
            &roads,
            &segments,
            chunk.chunk_x,
            chunk.chunk_y,
            &overlay.mode,
            &overlay_grids,
            weather.season,
            cb_mode,
        );
        meshes.insert(&mesh_handle.0, new_mesh);
        commands.entity(entity).remove::<ChunkDirty>();
    }
}

fn chunk_world_pos(cx: usize, cy: usize) -> (f32, f32) {
    let wx = cx as f32 * CHUNK_SIZE as f32 * CELL_SIZE;
    let wz = cy as f32 * CHUNK_SIZE as f32 * CELL_SIZE;
    (wx, wz)
}

#[allow(clippy::too_many_arguments)]
pub fn build_chunk_mesh(
    grid: &WorldGrid,
    roads: &RoadNetwork,
    _segments: &RoadSegmentStore,
    cx: usize,
    cy: usize,
    overlay: &OverlayMode,
    overlay_grids: &OverlayGrids,
    season: Season,
    cb_mode: ColorblindMode,
) -> Mesh {
    let cells_in_chunk = CHUNK_SIZE * CHUNK_SIZE;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(cells_in_chunk * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(cells_in_chunk * 4);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(cells_in_chunk * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(cells_in_chunk * 6);

    let base_gx = cx * CHUNK_SIZE;
    let base_gy = cy * CHUNK_SIZE;

    for ly in 0..CHUNK_SIZE {
        for lx in 0..CHUNK_SIZE {
            let gx = base_gx + lx;
            let gy = base_gy + ly;

            if gx >= GRID_WIDTH || gy >= GRID_HEIGHT {
                continue;
            }

            let cell = grid.get(gx, gy);
            let snow_depth = overlay_grids.snow.map(|sg| sg.get(gx, gy)).unwrap_or(0.0);
            let base_color = terrain_color(cell, gx, gy, season, snow_depth, cb_mode);
            let color = apply_overlay(
                base_color,
                cell,
                gx,
                gy,
                grid,
                overlay,
                overlay_grids,
                cb_mode,
            );

            let x0 = lx as f32 * CELL_SIZE;
            let z0 = ly as f32 * CELL_SIZE;
            let x1 = (lx + 1) as f32 * CELL_SIZE;
            let z1 = (ly + 1) as f32 * CELL_SIZE;
            let y = 0.0;

            let c: [f32; 4] = color.to_srgba().to_f32_array();

            // Cheap coastline blending: tint cells adjacent to water
            let c = if cell.cell_type != CellType::Road {
                coast_tint(grid, gx, gy, c, cell.cell_type)
            } else {
                c
            };

            // 4 vertices, 2 triangles per cell
            let vi = positions.len() as u32;
            positions.push([x0, y, z0]);
            positions.push([x1, y, z0]);
            positions.push([x1, y, z1]);
            positions.push([x0, y, z1]);
            normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
            colors.extend_from_slice(&[c; 4]);

            indices.push(vi);
            indices.push(vi + 2);
            indices.push(vi + 1);
            indices.push(vi);
            indices.push(vi + 3);
            indices.push(vi + 2);

            // Road surface and markings
            if cell.cell_type == CellType::Road && *overlay == OverlayMode::None {
                add_road_markings(
                    &mut positions,
                    &mut normals,
                    &mut colors,
                    &mut indices,
                    grid,
                    roads,
                    gx,
                    gy,
                    lx,
                    ly,
                    cell.road_type,
                );
            }
        }
    }

    let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; positions.len()];
    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
            | bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn terrain_color(
    cell: &simulation::grid::Cell,
    gx: usize,
    gy: usize,
    season: Season,
    snow_depth: f32,
    cb_mode: ColorblindMode,
) -> Color {
    // Per-cell noise for variation (no two cells look identical)
    let noise = ((gx.wrapping_mul(7919).wrapping_add(gy.wrapping_mul(6271))) % 100) as f32 / 100.0;
    let v = (noise - 0.5) * 0.04; // +/- 2% color variation

    let base_color = if cell.zone != ZoneType::None && cell.cell_type != CellType::Road {
        // Urban ground: light concrete/pavement tones (must contrast with dark road asphalt)
        let zone_kind = match cell.zone {
            ZoneType::ResidentialLow => colorblind_palette::ZoneColorKind::ResidentialLow,
            ZoneType::ResidentialMedium => colorblind_palette::ZoneColorKind::ResidentialMedium,
            ZoneType::ResidentialHigh => colorblind_palette::ZoneColorKind::ResidentialHigh,
            ZoneType::CommercialLow => colorblind_palette::ZoneColorKind::CommercialLow,
            ZoneType::CommercialHigh => colorblind_palette::ZoneColorKind::CommercialHigh,
            ZoneType::Industrial => colorblind_palette::ZoneColorKind::Industrial,
            ZoneType::Office => colorblind_palette::ZoneColorKind::Office,
            ZoneType::MixedUse => colorblind_palette::ZoneColorKind::MixedUse,
            ZoneType::None => unreachable!(),
        };
        let (r, g, b) = colorblind_palette::zone_color(zone_kind, cb_mode);
        Color::srgb(
            (r + v).clamp(0.0, 1.0),
            (g + v * 0.8).clamp(0.0, 1.0),
            (b + v * 0.6).clamp(0.0, 1.0),
        )
    } else {
        match cell.cell_type {
            CellType::Water => {
                let depth = 1.0 - cell.elevation / 0.35;
                // Urban waterways: gray-green, not deep blue
                let r = 0.12 + depth * 0.04 + v * 0.5;
                let g = 0.22 + depth * 0.08 + v * 0.3;
                let b = 0.38 + depth * 0.18 + v * 0.2;
                Color::srgb(r, g, b)
            }
            CellType::Road => {
                // Road cells render as light sidewalk/pavement — the asphalt strip is drawn on top
                let (r, g, b) = if cell.road_type == RoadType::Path {
                    (0.48, 0.44, 0.36) // Dirt path
                } else {
                    (0.62, 0.60, 0.57) // Light concrete sidewalk (contrasts with dark asphalt)
                };
                Color::srgb(
                    (r + v * 0.3).clamp(0.0, 1.0),
                    (g + v * 0.3).clamp(0.0, 1.0),
                    (b + v * 0.2).clamp(0.0, 1.0),
                )
            }
            CellType::Grass => {
                // Grass color varies by season with per-cell noise variation
                let [sr, sg, sb] = season.grass_color();
                let elev = cell.elevation;
                let patch =
                    ((gx.wrapping_mul(31).wrapping_add(gy.wrapping_mul(47))) % 100) as f32 / 100.0;
                let r = sr + elev * 0.06 + patch * 0.08 + v;
                let g = sg + elev * 0.10 + patch * 0.04 + v * 0.5;
                let b = sb + elev * 0.04 + patch * 0.03 + v * 0.3;
                Color::srgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
            }
        }
    };

    // Snow overlay: blend toward white based on snow depth.
    // Water cells don't get snow overlay. Full white at 6+ inches.
    if snow_depth > 0.0 && cell.cell_type != CellType::Water {
        let snow_factor = (snow_depth / 6.0).min(1.0);
        // Snow white with slight blue tint and per-cell noise for variation
        let snow_r = 0.92 + v * 0.3;
        let snow_g = 0.94 + v * 0.2;
        let snow_b = 0.98 + v * 0.1;
        let srgba = base_color.to_srgba();
        let r = srgba.red * (1.0 - snow_factor) + snow_r * snow_factor;
        let g = srgba.green * (1.0 - snow_factor) + snow_g * snow_factor;
        let b = srgba.blue * (1.0 - snow_factor) + snow_b * snow_factor;
        Color::srgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    } else {
        base_color
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_overlay(
    base: Color,
    cell: &simulation::grid::Cell,
    gx: usize,
    gy: usize,
    _grid: &WorldGrid,
    overlay: &OverlayMode,
    grids: &OverlayGrids,
    cb_mode: ColorblindMode,
) -> Color {
    match overlay {
        OverlayMode::None => base,
        OverlayMode::Power => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            let palette = colorblind_palette::power_palette(cb_mode);
            color_ramps::overlay_binary(base, &palette, cell.has_power)
        }
        OverlayMode::Water => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            let palette = colorblind_palette::water_palette(cb_mode);
            color_ramps::overlay_binary(base, &palette, cell.has_water)
        }
        OverlayMode::Traffic => {
            if cell.cell_type == CellType::Road {
                if let Some(traffic) = grids.traffic {
                    let congestion = traffic.congestion_level(gx, gy);
                    // Inferno: black (no traffic) -> red/orange -> yellow (gridlock)
                    color_ramps::overlay_continuous(&INFERNO, congestion)
                } else {
                    base
                }
            } else {
                color_ramps::darken(base, 0.5)
            }
        }
        OverlayMode::Pollution => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(pollution) = grids.pollution {
                let intensity = (pollution.get(gx, gy) as f32 / 50.0).clamp(0.0, 1.0);
                // Inferno: dark (clean) -> bright (polluted)
                color_ramps::overlay_continuous(&INFERNO, intensity)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::LandValue => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(land_value) = grids.land_value {
                let value = land_value.get(gx, gy) as f32 / 255.0;
                // Cividis: dark navy (low) -> yellow (high) -- CVD safe
                color_ramps::overlay_continuous(&CIVIDIS, value)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::Education => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(education) = grids.education {
                let level = education.get(gx, gy) as f32 / 3.0;
                // Viridis: purple (uneducated) -> teal -> yellow (highly educated)
                color_ramps::overlay_continuous(&VIRIDIS, level)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::Garbage => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(garbage) = grids.garbage {
                let level = (garbage.get(gx, gy) as f32 / 30.0).clamp(0.0, 1.0);
                // Inferno: dark (clean) -> bright (lots of garbage)
                color_ramps::overlay_continuous(&INFERNO, level)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::Noise => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(noise) = grids.noise {
                let level = (noise.get(gx, gy) as f32 / 100.0).clamp(0.0, 1.0);
                // Inferno: black (quiet) -> red/orange -> yellow (loud)
                color_ramps::overlay_continuous(&INFERNO, level)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::WaterPollution => {
            if let Some(wp) = grids.water_pollution {
                let level = (wp.get(gx, gy) as f32 / 255.0).clamp(0.0, 1.0);
                if cell.cell_type == CellType::Water {
                    // Viridis reversed: yellow (clean) -> teal -> purple (polluted)
                    // Reverse t so clean water = bright, polluted = dark
                    color_ramps::overlay_continuous(&VIRIDIS, 1.0 - level)
                } else if level > 0.0 {
                    // Land cells near polluted water get a subtle brown tint
                    color_ramps::blend_tint(base, Color::srgba(0.5, 0.35, 0.15, level * 0.4))
                } else {
                    color_ramps::darken(base, 0.7)
                }
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::GroundwaterLevel => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(gw) = grids.groundwater {
                let level = gw.get(gx, gy);
                let t = level as f32 / 255.0;
                let color = color_ramps::overlay_continuous(&GROUNDWATER_LEVEL, t);
                // Depletion warning: cells with level < 30% (~76) get a pulsing highlight
                if level < 76 {
                    // Blend toward warning orange for depleted cells
                    let warning_intensity = (1.0 - level as f32 / 76.0) * 0.3;
                    color_ramps::blend_tint(color, Color::srgba(1.0, 0.6, 0.0, warning_intensity))
                } else {
                    color
                }
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::GroundwaterQuality => {
            if cell.cell_type == CellType::Water {
                return base;
            }
            if let Some(wq) = grids.water_quality {
                let quality = wq.get(gx, gy);
                let t = quality as f32 / 255.0;
                color_ramps::overlay_continuous(&GROUNDWATER_QUALITY, t)
            } else {
                color_ramps::darken(base, 0.8)
            }
        }
        OverlayMode::Wind => {
            // Wind overlay uses gizmo streamlines, no terrain recolor needed.
            // Slightly darken the terrain for contrast with the streamline particles.
            color_ramps::darken(base, 0.7)
        }
    }
}

/// Cheap coastline tint: if a non-water cell borders water (or vice versa),
/// blend its color slightly toward a shore tone. Only checks 4 cardinal neighbors
/// (cell type only, no color recomputation).
fn coast_tint(
    grid: &WorldGrid,
    gx: usize,
    gy: usize,
    cell_color: [f32; 4],
    cell_type: CellType,
) -> [f32; 4] {
    // Count how many cardinal neighbors are on the other side of the shore
    let mut water_neighbors = 0u32;
    if gx > 0 && grid.get(gx - 1, gy).cell_type == CellType::Water {
        water_neighbors += 1;
    }
    if gx + 1 < GRID_WIDTH && grid.get(gx + 1, gy).cell_type == CellType::Water {
        water_neighbors += 1;
    }
    if gy > 0 && grid.get(gx, gy - 1).cell_type == CellType::Water {
        water_neighbors += 1;
    }
    if gy + 1 < GRID_HEIGHT && grid.get(gx, gy + 1).cell_type == CellType::Water {
        water_neighbors += 1;
    }

    if cell_type == CellType::Water {
        // Water cell next to land: lighten toward sandy shore
        let land_neighbors = 4 - water_neighbors;
        if land_neighbors == 0 {
            return cell_color;
        }
        let blend = land_neighbors as f32 * 0.15; // up to 0.6 for corner water cells
        let shore: [f32; 4] = [0.35, 0.38, 0.32, 1.0]; // muddy shore
        return [
            cell_color[0] + (shore[0] - cell_color[0]) * blend,
            cell_color[1] + (shore[1] - cell_color[1]) * blend,
            cell_color[2] + (shore[2] - cell_color[2]) * blend,
            cell_color[3],
        ];
    }

    // Land cell next to water: darken/blue-tint slightly
    if water_neighbors == 0 {
        return cell_color;
    }
    let blend = water_neighbors as f32 * 0.12;
    let wet: [f32; 4] = [0.18, 0.28, 0.32, 1.0]; // wet ground
    [
        cell_color[0] + (wet[0] - cell_color[0]) * blend,
        cell_color[1] + (wet[1] - cell_color[1]) * blend,
        cell_color[2] + (wet[2] - cell_color[2]) * blend,
        cell_color[3],
    ]
}

fn count_road_neighbors_8(grid: &WorldGrid, gx: usize, gy: usize) -> usize {
    let mut count = 0;
    for &(dx, dy) in &[
        (-1isize, -1isize),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ] {
        let nx = gx as isize + dx;
        let ny = gy as isize + dy;
        if nx >= 0
            && ny >= 0
            && (nx as usize) < GRID_WIDTH
            && (ny as usize) < GRID_HEIGHT
            && grid.get(nx as usize, ny as usize).cell_type == CellType::Road
        {
            count += 1;
        }
    }
    count
}

#[allow(clippy::too_many_arguments)]
fn add_road_markings(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    grid: &WorldGrid,
    roads: &RoadNetwork,
    gx: usize,
    gy: usize,
    lx: usize,
    ly: usize,
    road_type: RoadType,
) {
    if road_type == RoadType::Path {
        return;
    }

    let is_intersection = roads
        .intersections
        .contains(&simulation::roads::RoadNode(gx, gy));

    let has_left = gx > 0 && grid.get(gx - 1, gy).cell_type == CellType::Road;
    let has_right = gx + 1 < GRID_WIDTH && grid.get(gx + 1, gy).cell_type == CellType::Road;
    let has_up = gy + 1 < GRID_HEIGHT && grid.get(gx, gy + 1).cell_type == CellType::Road;
    let has_down = gy > 0 && grid.get(gx, gy - 1).cell_type == CellType::Road;

    let x_base = lx as f32 * CELL_SIZE;
    let z_base = ly as f32 * CELL_SIZE;
    let cx = x_base + CELL_SIZE * 0.5;
    let cz = z_base + CELL_SIZE * 0.5;

    // Road surface width varies by type (in world units, CELL_SIZE = 16)
    // Local: 2 narrow lanes = 7m road. Avenue: 2 wide lanes = 10m. Boulevard: 4 lanes = 14m. Highway: full width.
    let road_half_w: f32 = match road_type {
        RoadType::Local | RoadType::OneWay => 3.5,
        RoadType::Avenue => 5.0,
        RoadType::Boulevard => 7.0,
        RoadType::Highway => 7.5,
        RoadType::Path => 3.0,
    };

    // Asphalt color — dark like real roads (high contrast with light pavement around buildings)
    let noise = ((gx.wrapping_mul(3571).wrapping_add(gy.wrapping_mul(2143))) % 100) as f32 / 100.0;
    let av = (noise - 0.5) * 0.02;
    let asphalt: [f32; 4] = match road_type {
        RoadType::Highway => [0.10 + av, 0.10 + av, 0.12 + av, 1.0],
        RoadType::Boulevard => [0.13 + av, 0.13 + av, 0.15 + av, 1.0],
        RoadType::Avenue => [0.16 + av, 0.16 + av, 0.18 + av, 1.0],
        _ => [0.20 + av, 0.20 + av, 0.22 + av, 1.0],
    };

    let y_road = 0.03;
    let y_mark = 0.06;
    let y_curb = 0.12;

    let is_horizontal = has_left || has_right;
    let is_vertical = has_up || has_down;

    // Global world coordinates for continuous dash patterns
    let world_x = gx as f32 * CELL_SIZE;
    let world_z = gy as f32 * CELL_SIZE;

    // Dense area detection: count all 8 neighbors
    let road_neighbors_8 = count_road_neighbors_8(grid, gx, gy);
    let is_dense = road_neighbors_8 >= 6;

    // --- Asphalt road surface ---
    if is_intersection {
        // Full asphalt at intersections
        push_quad_3d(
            positions,
            normals,
            colors,
            indices,
            x_base,
            z_base,
            x_base + CELL_SIZE,
            z_base + CELL_SIZE,
            y_road,
            asphalt,
        );

        // Crosswalks only at non-dense boundary intersections
        if !is_dense {
            let stripe_w = 0.5;
            let stripe_gap = 1.2;
            let cw_color: [f32; 4] = [0.82, 0.82, 0.80, 0.75];
            let cw_inset = 1.5;

            if has_down {
                let zz = z_base + cw_inset;
                let mut sx = x_base + 1.5;
                while sx + stripe_w < x_base + CELL_SIZE - 1.5 {
                    push_quad_3d(
                        positions,
                        normals,
                        colors,
                        indices,
                        sx,
                        zz,
                        sx + stripe_w,
                        zz + 2.0,
                        y_mark,
                        cw_color,
                    );
                    sx += stripe_w + stripe_gap;
                }
            }
            if has_up {
                let zz = z_base + CELL_SIZE - cw_inset - 2.0;
                let mut sx = x_base + 1.5;
                while sx + stripe_w < x_base + CELL_SIZE - 1.5 {
                    push_quad_3d(
                        positions,
                        normals,
                        colors,
                        indices,
                        sx,
                        zz,
                        sx + stripe_w,
                        zz + 2.0,
                        y_mark,
                        cw_color,
                    );
                    sx += stripe_w + stripe_gap;
                }
            }
            if has_left {
                let xx = x_base + cw_inset;
                let mut sz = z_base + 1.5;
                while sz + stripe_w < z_base + CELL_SIZE - 1.5 {
                    push_quad_3d(
                        positions,
                        normals,
                        colors,
                        indices,
                        xx,
                        sz,
                        xx + 2.0,
                        sz + stripe_w,
                        y_mark,
                        cw_color,
                    );
                    sz += stripe_w + stripe_gap;
                }
            }
            if has_right {
                let xx = x_base + CELL_SIZE - cw_inset - 2.0;
                let mut sz = z_base + 1.5;
                while sz + stripe_w < z_base + CELL_SIZE - 1.5 {
                    push_quad_3d(
                        positions,
                        normals,
                        colors,
                        indices,
                        xx,
                        sz,
                        xx + 2.0,
                        sz + stripe_w,
                        y_mark,
                        cw_color,
                    );
                    sz += stripe_w + stripe_gap;
                }
            }
        }
    } else {
        // Straight road: adaptive asphalt width that extends to adjacent road cells
        if is_horizontal && !is_vertical {
            // Extend asphalt vertically toward neighboring road cells
            let z_top = if has_down { z_base } else { cz - road_half_w };
            let z_bot = if has_up {
                z_base + CELL_SIZE
            } else {
                cz + road_half_w
            };
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                z_top,
                x_base + CELL_SIZE,
                z_bot,
                y_road,
                asphalt,
            );
        } else if is_vertical && !is_horizontal {
            // Extend asphalt horizontally toward neighboring road cells
            let x_left = if has_left { x_base } else { cx - road_half_w };
            let x_right = if has_right {
                x_base + CELL_SIZE
            } else {
                cx + road_half_w
            };
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_left,
                z_base,
                x_right,
                z_base + CELL_SIZE,
                y_road,
                asphalt,
            );
        } else {
            // Both horizontal and vertical (but not intersection) — draw both strips with extensions
            let z_top = if has_down { z_base } else { cz - road_half_w };
            let z_bot = if has_up {
                z_base + CELL_SIZE
            } else {
                cz + road_half_w
            };
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                z_top,
                x_base + CELL_SIZE,
                z_bot,
                y_road,
                asphalt,
            );

            let x_left = if has_left { x_base } else { cx - road_half_w };
            let x_right = if has_right {
                x_base + CELL_SIZE
            } else {
                cx + road_half_w
            };
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_left,
                z_base,
                x_right,
                z_base + CELL_SIZE,
                y_road,
                asphalt,
            );
        }

        // --- Lane markings (skip in dense areas) ---
        if !is_dense {
            let lw = 0.12; // line half-width
            let dash = 3.0;
            let gap = 4.0;
            let period = dash + gap;

            if is_horizontal {
                let x0 = x_base;
                let x1 = x_base + CELL_SIZE;

                match road_type {
                    RoadType::Local | RoadType::OneWay => {
                        let mut sx = x0 - (world_x % period);
                        while sx < x1 {
                            let d0 = sx.max(x0);
                            let d1 = (sx + dash).min(x1);
                            if d1 > d0 {
                                push_quad_3d(
                                    positions,
                                    normals,
                                    colors,
                                    indices,
                                    d0,
                                    cz - lw,
                                    d1,
                                    cz + lw,
                                    y_mark,
                                    [0.95, 0.95, 0.90, 0.85],
                                );
                            }
                            sx += period;
                        }
                    }
                    RoadType::Avenue => {
                        let s = 0.2;
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            x0,
                            cz - s - lw,
                            x1,
                            cz - s + lw,
                            y_mark,
                            [0.90, 0.80, 0.15, 0.90],
                        );
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            x0,
                            cz + s - lw,
                            x1,
                            cz + s + lw,
                            y_mark,
                            [0.90, 0.80, 0.15, 0.90],
                        );
                    }
                    RoadType::Boulevard => {
                        let lane_w = road_half_w * 0.5;
                        let median_hw = 0.8;
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            x0,
                            cz - median_hw,
                            x1,
                            cz + median_hw,
                            y_curb * 0.7,
                            [0.35, 0.45, 0.30, 1.0],
                        );
                        for &off in &[-lane_w, lane_w] {
                            let mut sx = x0 - (world_x % period);
                            while sx < x1 {
                                let d0 = sx.max(x0);
                                let d1 = (sx + dash).min(x1);
                                if d1 > d0 {
                                    push_quad_3d(
                                        positions,
                                        normals,
                                        colors,
                                        indices,
                                        d0,
                                        cz + off - lw,
                                        d1,
                                        cz + off + lw,
                                        y_mark,
                                        [1.0, 1.0, 1.0, 0.35],
                                    );
                                }
                                sx += period;
                            }
                        }
                    }
                    RoadType::Highway => {
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            x0,
                            cz - 0.2,
                            x1,
                            cz + 0.2,
                            y_mark,
                            [0.80, 0.70, 0.12, 0.75],
                        );
                        let lane_w = road_half_w * 0.5;
                        for &off in &[-lane_w, lane_w] {
                            let mut sx = x0 - (world_x % period);
                            while sx < x1 {
                                let d0 = sx.max(x0);
                                let d1 = (sx + dash).min(x1);
                                if d1 > d0 {
                                    push_quad_3d(
                                        positions,
                                        normals,
                                        colors,
                                        indices,
                                        d0,
                                        cz + off - lw,
                                        d1,
                                        cz + off + lw,
                                        y_mark,
                                        [0.95, 0.95, 0.90, 0.85],
                                    );
                                }
                                sx += period;
                            }
                        }
                        for &edge in &[-road_half_w + 0.3, road_half_w - 0.3] {
                            push_quad_3d(
                                positions,
                                normals,
                                colors,
                                indices,
                                x0,
                                cz + edge - lw,
                                x1,
                                cz + edge + lw,
                                y_mark,
                                [1.0, 1.0, 1.0, 0.55],
                            );
                        }
                    }
                    RoadType::Path => {}
                }
            }

            if is_vertical {
                let z0 = z_base;
                let z1 = z_base + CELL_SIZE;

                match road_type {
                    RoadType::Local | RoadType::OneWay => {
                        let mut sz = z0 - (world_z % period);
                        while sz < z1 {
                            let d0 = sz.max(z0);
                            let d1 = (sz + dash).min(z1);
                            if d1 > d0 {
                                push_quad_3d(
                                    positions,
                                    normals,
                                    colors,
                                    indices,
                                    cx - lw,
                                    d0,
                                    cx + lw,
                                    d1,
                                    y_mark,
                                    [0.95, 0.95, 0.90, 0.85],
                                );
                            }
                            sz += period;
                        }
                    }
                    RoadType::Avenue => {
                        let s = 0.2;
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            cx - s - lw,
                            z0,
                            cx - s + lw,
                            z1,
                            y_mark,
                            [0.90, 0.80, 0.15, 0.90],
                        );
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            cx + s - lw,
                            z0,
                            cx + s + lw,
                            z1,
                            y_mark,
                            [0.90, 0.80, 0.15, 0.90],
                        );
                    }
                    RoadType::Boulevard => {
                        let lane_w = road_half_w * 0.5;
                        let median_hw = 0.8;
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            cx - median_hw,
                            z0,
                            cx + median_hw,
                            z1,
                            y_curb * 0.7,
                            [0.35, 0.45, 0.30, 1.0],
                        );
                        for &off in &[-lane_w, lane_w] {
                            let mut sz = z0 - (world_z % period);
                            while sz < z1 {
                                let d0 = sz.max(z0);
                                let d1 = (sz + dash).min(z1);
                                if d1 > d0 {
                                    push_quad_3d(
                                        positions,
                                        normals,
                                        colors,
                                        indices,
                                        cx + off - lw,
                                        d0,
                                        cx + off + lw,
                                        d1,
                                        y_mark,
                                        [1.0, 1.0, 1.0, 0.35],
                                    );
                                }
                                sz += period;
                            }
                        }
                    }
                    RoadType::Highway => {
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            cx - 0.2,
                            z0,
                            cx + 0.2,
                            z1,
                            y_mark,
                            [0.80, 0.70, 0.12, 0.75],
                        );
                        let lane_w = road_half_w * 0.5;
                        for &off in &[-lane_w, lane_w] {
                            let mut sz = z0 - (world_z % period);
                            while sz < z1 {
                                let d0 = sz.max(z0);
                                let d1 = (sz + dash).min(z1);
                                if d1 > d0 {
                                    push_quad_3d(
                                        positions,
                                        normals,
                                        colors,
                                        indices,
                                        cx + off - lw,
                                        d0,
                                        cx + off + lw,
                                        d1,
                                        y_mark,
                                        [0.95, 0.95, 0.90, 0.85],
                                    );
                                }
                                sz += period;
                            }
                        }
                        for &edge in &[-road_half_w + 0.3, road_half_w - 0.3] {
                            push_quad_3d(
                                positions,
                                normals,
                                colors,
                                indices,
                                cx + edge - lw,
                                z0,
                                cx + edge + lw,
                                z1,
                                y_mark,
                                [1.0, 1.0, 1.0, 0.55],
                            );
                        }
                    }
                    RoadType::Path => {}
                }
            }
        }
    }

    // --- Curb edges where road meets non-road ---
    let curb_w = 0.3;
    let curb_color: [f32; 4] = [0.62, 0.60, 0.57, 1.0];

    if is_intersection {
        // Curbs only on edges facing non-road
        if !has_left {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                z_base,
                x_base + curb_w,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
        if !has_right {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base + CELL_SIZE - curb_w,
                z_base,
                x_base + CELL_SIZE,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
        if !has_down {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                z_base,
                x_base + CELL_SIZE,
                z_base + curb_w,
                y_curb,
                curb_color,
            );
        }
        if !has_up {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                z_base + CELL_SIZE - curb_w,
                x_base + CELL_SIZE,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
    } else {
        // For straight segments, curbs at the real road edge (not extended side)
        if !has_left && is_vertical {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                cx - road_half_w - curb_w,
                z_base,
                cx - road_half_w,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
        if !has_right && is_vertical {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                cx + road_half_w,
                z_base,
                cx + road_half_w + curb_w,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
        if !has_down && is_horizontal {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                cz - road_half_w - curb_w,
                x_base + CELL_SIZE,
                cz - road_half_w,
                y_curb,
                curb_color,
            );
        }
        if !has_up && is_horizontal {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                cz + road_half_w,
                x_base + CELL_SIZE,
                cz + road_half_w + curb_w,
                y_curb,
                curb_color,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn push_quad_3d(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    x0: f32,
    z0: f32,
    x1: f32,
    z1: f32,
    y: f32,
    color: [f32; 4],
) {
    let vi = positions.len() as u32;
    positions.push([x0, y, z0]);
    positions.push([x1, y, z0]);
    positions.push([x1, y, z1]);
    positions.push([x0, y, z1]);
    normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
    colors.push(color);
    colors.push(color);
    colors.push(color);
    colors.push(color);
    indices.push(vi);
    indices.push(vi + 2);
    indices.push(vi + 1);
    indices.push(vi);
    indices.push(vi + 3);
    indices.push(vi + 2);
}

pub fn mark_chunk_dirty_at(
    gx: usize,
    gy: usize,
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
) {
    let cx = gx / CHUNK_SIZE;
    let cy = gy / CHUNK_SIZE;
    for (entity, chunk) in chunks.iter() {
        if chunk.chunk_x == cx && chunk.chunk_y == cy {
            commands.entity(entity).insert(ChunkDirty);
            return;
        }
    }
}

pub fn mark_all_chunks_dirty(
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
) {
    for (entity, _) in chunks.iter() {
        commands.entity(entity).insert(ChunkDirty);
    }
}

pub fn cell_color(cell: &simulation::grid::Cell) -> Color {
    terrain_color(cell, 0, 0, Season::Spring, 0.0, ColorblindMode::Normal)
}
