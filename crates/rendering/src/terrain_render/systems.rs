use bevy::prelude::*;

use simulation::colorblind::ColorblindSettings;
use simulation::config::{CHUNKS_X, CHUNKS_Y};
use simulation::education::EducationGrid;
use simulation::garbage::GarbageGrid;
use simulation::groundwater::{GroundwaterGrid, WaterQualityGrid};
use simulation::land_value::LandValueGrid;
use simulation::noise::NoisePollutionGrid;
use simulation::pollution::PollutionGrid;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::snow::SnowGrid;
use simulation::traffic::TrafficGrid;
use simulation::water_pollution::WaterPollutionGrid;
use simulation::weather::Weather;

use simulation::colorblind::ColorblindMode;
use simulation::grid::WorldGrid;
use simulation::network_viz::NetworkVizData;

use crate::overlay::OverlayMode;

use super::mesh::{build_chunk_mesh, chunk_world_pos};
use super::types::{ChunkDirty, DualOverlayInfo, OverlayGrids, TerrainChunk};

#[allow(clippy::too_many_arguments)]
pub fn spawn_terrain_chunks(
    mut commands: Commands,
    grid: Res<WorldGrid>,
    roads: Res<RoadNetwork>,
    segments: Res<RoadSegmentStore>,
    weather: Res<Weather>,
    snow_grid: Res<SnowGrid>,
    network_viz: Res<NetworkVizData>,
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
                &network_viz,
                &DualOverlayInfo::default(),
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
    overlay_params: (
        Res<crate::overlay::OverlayState>,
        Res<crate::overlay::DualOverlayState>,
    ),
    pollution_grid: Res<PollutionGrid>,
    land_value_grid: Res<LandValueGrid>,
    education_grid: Res<EducationGrid>,
    garbage_grid: Res<GarbageGrid>,
    traffic_grid: Res<TrafficGrid>,
    noise_grid: Res<NoisePollutionGrid>,
    water_pollution_grid: Res<WaterPollutionGrid>,
    groundwater_grids: (Res<GroundwaterGrid>, Res<WaterQualityGrid>),
    weather: Res<Weather>,
    snow_grid: Res<SnowGrid>,
    network_viz: Res<NetworkVizData>,
    cb_settings: Res<ColorblindSettings>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    mut commands: Commands,
) {
    use crate::overlay::OverlayMode;

    let (overlay, dual_overlay) = overlay_params;
    let (groundwater_grid, water_quality_grid) = groundwater_grids;

    if overlay.is_changed()
        || dual_overlay.is_changed()
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
        OverlayMode::Power | OverlayMode::Water => network_viz.is_changed(), // re-color by source when viz data updates
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
        return;
    }

    // When dual overlay is active, also dirty chunks if the secondary overlay's data changed
    if dual_overlay.secondary != OverlayMode::None && overlay.mode != OverlayMode::None {
        let secondary_changed = match dual_overlay.secondary {
            OverlayMode::None => false,
            OverlayMode::Power | OverlayMode::Water => network_viz.is_changed(),
            OverlayMode::Pollution => pollution_grid.is_changed(),
            OverlayMode::LandValue => land_value_grid.is_changed(),
            OverlayMode::Education => education_grid.is_changed(),
            OverlayMode::Garbage => garbage_grid.is_changed(),
            OverlayMode::Traffic => traffic_grid.is_changed(),
            OverlayMode::Noise => noise_grid.is_changed(),
            OverlayMode::WaterPollution => water_pollution_grid.is_changed(),
            OverlayMode::GroundwaterLevel => groundwater_grid.is_changed(),
            OverlayMode::GroundwaterQuality => water_quality_grid.is_changed(),
            OverlayMode::Wind => false,
        };
        if secondary_changed {
            mark_all_chunks_dirty(&chunks, &mut commands);
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn rebuild_dirty_chunks(
    mut commands: Commands,
    grid: Res<WorldGrid>,
    roads: Res<RoadNetwork>,
    segments: Res<RoadSegmentStore>,
    overlay_params: (
        Res<crate::overlay::OverlayState>,
        Res<NetworkVizData>,
        Res<crate::overlay::DualOverlayState>,
    ),
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
    let (overlay, network_viz, dual_overlay) = overlay_params;
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
        let dual_info = DualOverlayInfo {
            secondary: dual_overlay.secondary,
            mode: dual_overlay.mode,
            blend_factor: dual_overlay.blend_factor,
        };
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
            &network_viz,
            &dual_info,
        );
        meshes.insert(&mesh_handle.0, new_mesh);
        commands.entity(entity).remove::<ChunkDirty>();
    }
}

pub fn mark_chunk_dirty_at(
    gx: usize,
    gy: usize,
    chunks: &Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    commands: &mut Commands,
) {
    let cx = gx / simulation::config::CHUNK_SIZE;
    let cy = gy / simulation::config::CHUNK_SIZE;
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
