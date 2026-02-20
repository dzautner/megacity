//! Satellite View: 2D top-down map overlay at maximum zoom-out.
//!
//! When the camera zooms far enough out, this module renders a flat textured
//! quad covering the entire city showing terrain colors, road lines, and
//! building area fills. The overlay smoothly fades in as the camera distance
//! increases from `TRANSITION_START` to `TRANSITION_END`, and 3D objects
//! (buildings, roads, citizens, props) fade out simultaneously.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use simulation::buildings::Building;
use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH, WORLD_HEIGHT, WORLD_WIDTH};
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::road_segments::RoadSegmentStore;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;
use simulation::weather::Weather;

use crate::building_render::BuildingMesh3d;
use crate::citizen_render::CitizenSprite;
use crate::lane_markings::LaneMarkingMesh;
use crate::props::PropEntity;
use crate::road_render::{RoadIntersectionMesh, RoadSegmentMesh};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Camera distance at which the satellite overlay starts fading in.
const TRANSITION_START: f32 = 2500.0;

/// Camera distance at which the satellite overlay is fully opaque.
const TRANSITION_END: f32 = 3800.0;

/// Resolution of the satellite map texture (pixels per axis).
const TEX_SIZE: usize = 512;

/// Y position of the satellite quad (above terrain at Y=0).
const SATELLITE_Y: f32 = 5.0;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks the current satellite view blend factor and dirty state.
#[derive(Resource)]
pub struct SatelliteView {
    /// 0.0 = fully 3D, 1.0 = fully satellite.
    pub blend: f32,
    /// Whether the satellite texture needs regeneration.
    pub dirty: bool,
}

impl Default for SatelliteView {
    fn default() -> Self {
        Self {
            blend: 0.0,
            dirty: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker for the satellite overlay quad entity.
#[derive(Component)]
pub struct SatelliteQuad;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SatelliteViewPlugin;

impl Plugin for SatelliteViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SatelliteView>().add_systems(
            Update,
            (
                update_blend_factor,
                spawn_satellite_quad,
                mark_dirty_on_change,
                rebuild_satellite_texture,
                update_satellite_visibility,
                fade_3d_objects,
            )
                .chain(),
        );
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Update the blend factor based on camera distance.
fn update_blend_factor(
    orbit: Res<crate::camera::OrbitCamera>,
    mut satellite: ResMut<SatelliteView>,
) {
    let t = if orbit.distance <= TRANSITION_START {
        0.0
    } else if orbit.distance >= TRANSITION_END {
        1.0
    } else {
        (orbit.distance - TRANSITION_START) / (TRANSITION_END - TRANSITION_START)
    };
    // Smooth-step for a nicer transition curve
    let smooth = t * t * (3.0 - 2.0 * t);
    if (satellite.blend - smooth).abs() > 0.001 {
        satellite.blend = smooth;
    }
}

/// Spawn the satellite quad entity on first frame (if it doesn't exist yet).
fn spawn_satellite_quad(
    mut commands: Commands,
    existing: Query<Entity, With<SatelliteQuad>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !existing.is_empty() {
        return;
    }

    let image = create_blank_image();
    let image_handle = images.add(image);

    // Flat quad covering the entire world on the XZ plane
    let mesh = meshes.add(
        Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0],
                [WORLD_WIDTH, 0.0, 0.0],
                [WORLD_WIDTH, 0.0, WORLD_HEIGHT],
                [0.0, 0.0, WORLD_HEIGHT],
            ],
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        )
        .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 2, 1, 0, 3, 2])),
    );

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, SATELLITE_Y, 0.0),
        Visibility::Hidden,
        SatelliteQuad,
    ));
}

/// Mark the satellite texture as dirty when the grid or road segments change.
fn mark_dirty_on_change(
    grid: Res<WorldGrid>,
    segments: Res<RoadSegmentStore>,
    mut satellite: ResMut<SatelliteView>,
) {
    if grid.is_changed() || segments.is_changed() {
        satellite.dirty = true;
    }
}

/// Rebuild the satellite texture when dirty and the overlay is at least partially visible.
#[allow(clippy::too_many_arguments)]
fn rebuild_satellite_texture(
    mut satellite: ResMut<SatelliteView>,
    grid: Res<WorldGrid>,
    segments: Res<RoadSegmentStore>,
    weather: Res<Weather>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    utilities: Query<&UtilitySource>,
    quad_q: Query<&MeshMaterial3d<StandardMaterial>, With<SatelliteQuad>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !satellite.dirty {
        return;
    }
    // Only rebuild when at least slightly visible to avoid wasted work
    if satellite.blend < 0.01 {
        return;
    }

    satellite.dirty = false;

    let image = generate_satellite_image(
        &grid, &segments, &weather, &buildings, &services, &utilities,
    );

    let Ok(mat_handle) = quad_q.get_single() else {
        return;
    };
    let Some(mat) = materials.get_mut(mat_handle) else {
        return;
    };
    if let Some(ref tex_handle) = mat.base_color_texture {
        if let Some(existing_image) = images.get_mut(tex_handle) {
            *existing_image = image;
        }
    }
}

/// Update the satellite quad visibility and alpha based on blend factor.
fn update_satellite_visibility(
    satellite: Res<SatelliteView>,
    mut quad_q: Query<(&MeshMaterial3d<StandardMaterial>, &mut Visibility), With<SatelliteQuad>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mat_handle, mut vis) in &mut quad_q {
        if satellite.blend < 0.001 {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Visible;

        if let Some(mat) = materials.get_mut(mat_handle) {
            mat.base_color = Color::srgba(1.0, 1.0, 1.0, satellite.blend);
        }
    }
}

/// Fade out 3D objects as satellite view fades in. Uses `ParamSet` to avoid
/// conflicting `Visibility` queries.
fn fade_3d_objects(
    satellite: Res<SatelliteView>,
    mut set: ParamSet<(
        Query<&mut Visibility, With<BuildingMesh3d>>,
        Query<&mut Visibility, With<RoadSegmentMesh>>,
        Query<&mut Visibility, With<RoadIntersectionMesh>>,
        Query<&mut Visibility, With<LaneMarkingMesh>>,
        Query<&mut Visibility, With<PropEntity>>,
        Query<&mut Visibility, With<CitizenSprite>>,
    )>,
) {
    // Hide 3D objects when the satellite view is more than 70% blended in
    let hide_3d = satellite.blend > 0.7;
    let target = if hide_3d {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };

    for mut vis in &mut set.p0() {
        if *vis != target {
            *vis = target;
        }
    }
    for mut vis in &mut set.p1() {
        if *vis != target {
            *vis = target;
        }
    }
    for mut vis in &mut set.p2() {
        if *vis != target {
            *vis = target;
        }
    }
    for mut vis in &mut set.p3() {
        if *vis != target {
            *vis = target;
        }
    }
    for mut vis in &mut set.p4() {
        if *vis != target {
            *vis = target;
        }
    }
    for mut vis in &mut set.p5() {
        if *vis != target {
            *vis = target;
        }
    }
}

// ---------------------------------------------------------------------------
// Image generation
// ---------------------------------------------------------------------------

fn create_blank_image() -> Image {
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
fn generate_satellite_image(
    grid: &WorldGrid,
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

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

/// Terrain color for satellite view (simplified, no per-cell noise).
fn satellite_terrain_color(cell: &simulation::grid::Cell, weather: &Weather) -> [u8; 4] {
    let (r, g, b) = if cell.zone != ZoneType::None && cell.cell_type != CellType::Road {
        match cell.zone {
            ZoneType::ResidentialLow => (0.52, 0.56, 0.46),
            ZoneType::ResidentialMedium => (0.57, 0.58, 0.51),
            ZoneType::ResidentialHigh => (0.62, 0.60, 0.57),
            ZoneType::CommercialLow => (0.58, 0.57, 0.54),
            ZoneType::CommercialHigh => (0.60, 0.58, 0.55),
            ZoneType::Industrial => (0.55, 0.52, 0.47),
            ZoneType::Office => (0.64, 0.62, 0.58),
            ZoneType::MixedUse => (0.60, 0.58, 0.52),
            ZoneType::None => unreachable!(),
        }
    } else {
        match cell.cell_type {
            CellType::Water => {
                let depth = (1.0 - cell.elevation / 0.35).clamp(0.0, 1.0);
                (
                    0.12 + depth * 0.04,
                    0.22 + depth * 0.08,
                    0.38 + depth * 0.18,
                )
            }
            CellType::Road => (0.35, 0.35, 0.38),
            CellType::Grass => {
                let [sr, sg, sb] = weather.season.grass_color();
                (sr, sg, sb)
            }
        }
    };

    to_rgba8(r, g, b)
}

/// Building color for satellite view based on zone type and level.
fn zone_satellite_color(zone: ZoneType, level: u8) -> [u8; 4] {
    let level_factor = 1.0 - (level as f32 - 1.0) * 0.08;
    let (r, g, b) = match zone {
        ZoneType::ResidentialLow => (0.70, 0.75, 0.65),
        ZoneType::ResidentialMedium => (0.65, 0.68, 0.55),
        ZoneType::ResidentialHigh => (0.72, 0.70, 0.68),
        ZoneType::CommercialLow => (0.65, 0.60, 0.70),
        ZoneType::CommercialHigh => (0.60, 0.55, 0.68),
        ZoneType::Industrial => (0.68, 0.62, 0.50),
        ZoneType::Office => (0.62, 0.65, 0.72),
        ZoneType::MixedUse => (0.67, 0.62, 0.65),
        ZoneType::None => (0.5, 0.5, 0.5),
    };
    to_rgba8(r * level_factor, g * level_factor, b * level_factor)
}

/// Road line color for satellite view.
fn road_satellite_color(road_type: simulation::grid::RoadType) -> [u8; 4] {
    use simulation::grid::RoadType;
    match road_type {
        RoadType::Path => [160, 145, 120, 255],
        RoadType::OneWay => [90, 90, 100, 255],
        RoadType::Local => [80, 80, 90, 255],
        RoadType::Avenue => [70, 70, 80, 255],
        RoadType::Boulevard => [60, 60, 75, 255],
        RoadType::Highway => [55, 55, 70, 255],
    }
}

/// Road line width in texture pixels for satellite view.
fn road_satellite_width(road_type: simulation::grid::RoadType) -> f32 {
    use simulation::grid::RoadType;
    match road_type {
        RoadType::Path => 0.8,
        RoadType::OneWay => 1.0,
        RoadType::Local => 1.2,
        RoadType::Avenue => 1.8,
        RoadType::Boulevard => 2.4,
        RoadType::Highway => 3.0,
    }
}

// ---------------------------------------------------------------------------
// Pixel-painting helpers
// ---------------------------------------------------------------------------

/// Paint a filled circle into the pixel buffer.
fn paint_circle(
    pixels: &mut [[u8; 4]],
    size: usize,
    cx: f32,
    cy: f32,
    radius: f32,
    color: [u8; 4],
) {
    let r2 = radius * radius + 0.5; // slight expansion for anti-alias
    let min_x = ((cx - radius).floor() as isize).max(0) as usize;
    let max_x = ((cx + radius).ceil() as usize).min(size - 1);
    let min_y = ((cy - radius).floor() as isize).max(0) as usize;
    let max_y = ((cy + radius).ceil() as usize).min(size - 1);

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let dx = px as f32 - cx;
            let dy = py as f32 - cy;
            if dx * dx + dy * dy <= r2 {
                pixels[py * size + px] = color;
            }
        }
    }
}

/// Paint a single grid cell into the satellite texture.
fn paint_grid_cell(
    pixels: &mut [[u8; 4]],
    size: usize,
    scale_x: f32,
    scale_y: f32,
    grid_x: usize,
    grid_y: usize,
    color: [u8; 4],
) {
    let px_start = (grid_x as f32 / scale_x).floor() as usize;
    let py_start = (grid_y as f32 / scale_y).floor() as usize;
    let px_end = (((grid_x + 1) as f32) / scale_x).ceil() as usize;
    let py_end = (((grid_y + 1) as f32) / scale_y).ceil() as usize;

    let px_end = px_end.min(size);
    let py_end = py_end.min(size);

    for py in py_start..py_end {
        for px in px_start..px_end {
            let idx = py * size + px;
            if idx < pixels.len() {
                pixels[idx] = color;
            }
        }
    }
}

/// Convert floating-point RGB (0.0-1.0) to `[u8; 4]` RGBA with full alpha.
fn to_rgba8(r: f32, g: f32, b: f32) -> [u8; 4] {
    [
        (r * 255.0).clamp(0.0, 255.0) as u8,
        (g * 255.0).clamp(0.0, 255.0) as u8,
        (b * 255.0).clamp(0.0, 255.0) as u8,
        255,
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_constants_are_ordered() {
        assert!(TRANSITION_START > 0.0);
        assert!(TRANSITION_END > TRANSITION_START);
    }

    #[test]
    fn test_zone_satellite_color_produces_valid_rgba() {
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            for level in 1..=5 {
                let color = zone_satellite_color(zone, level);
                assert_eq!(color[3], 255, "Alpha should be fully opaque");
            }
        }
    }

    #[test]
    fn test_road_satellite_width_increases_with_road_type() {
        use simulation::grid::RoadType;
        let path_w = road_satellite_width(RoadType::Path);
        let local_w = road_satellite_width(RoadType::Local);
        let avenue_w = road_satellite_width(RoadType::Avenue);
        let highway_w = road_satellite_width(RoadType::Highway);
        assert!(path_w < local_w);
        assert!(local_w < avenue_w);
        assert!(avenue_w < highway_w);
    }

    #[test]
    fn test_create_blank_image_dimensions() {
        let img = create_blank_image();
        assert_eq!(img.width(), TEX_SIZE as u32);
        assert_eq!(img.height(), TEX_SIZE as u32);
    }

    #[test]
    fn test_to_rgba8_clamping() {
        let c = to_rgba8(1.5, -0.1, 0.5);
        assert_eq!(c[0], 255);
        assert_eq!(c[1], 0);
        assert_eq!(c[2], 127);
        assert_eq!(c[3], 255);
    }

    #[test]
    fn test_satellite_terrain_color_water() {
        let cell = simulation::grid::Cell {
            cell_type: CellType::Water,
            elevation: 0.2,
            zone: ZoneType::None,
            ..Default::default()
        };
        let weather = Weather::default();
        let color = satellite_terrain_color(&cell, &weather);
        assert!(color[2] > color[0], "Water blue channel should exceed red");
    }

    #[test]
    fn test_satellite_terrain_color_grass() {
        let cell = simulation::grid::Cell {
            cell_type: CellType::Grass,
            elevation: 0.5,
            zone: ZoneType::None,
            ..Default::default()
        };
        let weather = Weather::default();
        let color = satellite_terrain_color(&cell, &weather);
        assert!(color[1] > color[0], "Grass green channel should exceed red");
    }

    #[test]
    fn test_satellite_terrain_color_road() {
        let cell = simulation::grid::Cell {
            cell_type: CellType::Road,
            elevation: 0.5,
            zone: ZoneType::None,
            ..Default::default()
        };
        let weather = Weather::default();
        let color = satellite_terrain_color(&cell, &weather);
        assert!(color[0] < 128 && color[1] < 128 && color[2] < 128);
    }

    #[test]
    fn test_paint_grid_cell_within_bounds() {
        let size = 16;
        let mut pixels = vec![[0u8; 4]; size * size];
        let scale_x = 256.0 / size as f32;
        let scale_y = 256.0 / size as f32;
        paint_grid_cell(&mut pixels, size, scale_x, scale_y, 0, 0, [255, 0, 0, 255]);
        assert_eq!(pixels[0], [255, 0, 0, 255]);
    }

    #[test]
    fn test_paint_circle_center_pixel() {
        let size = 16;
        let mut pixels = vec![[0u8; 4]; size * size];
        paint_circle(&mut pixels, size, 8.0, 8.0, 1.0, [0, 255, 0, 255]);
        // Center pixel should be painted
        assert_eq!(pixels[8 * size + 8], [0, 255, 0, 255]);
    }

    #[test]
    fn test_road_colors_are_opaque() {
        use simulation::grid::RoadType;
        let types = [
            RoadType::Path,
            RoadType::OneWay,
            RoadType::Local,
            RoadType::Avenue,
            RoadType::Boulevard,
            RoadType::Highway,
        ];
        for rt in types {
            let c = road_satellite_color(rt);
            assert_eq!(c[3], 255);
        }
    }
}
