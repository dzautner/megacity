//! ECS systems for the satellite view overlay.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;

use simulation::buildings::Building;
use simulation::config::{WORLD_HEIGHT, WORLD_WIDTH};
use simulation::grid::WorldGrid;
use simulation::road_segments::RoadSegmentStore;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;
use simulation::weather::Weather;

use crate::building_render::BuildingMesh3d;
use crate::citizen_render::CitizenSprite;
use crate::lane_markings::LaneMarkingMesh;
use crate::props::PropEntity;
use crate::road_render::{RoadIntersectionMesh, RoadSegmentMesh};

use super::image_gen::{create_blank_image, generate_satellite_image};
use super::types::{SatelliteQuad, SatelliteView, SATELLITE_Y, TRANSITION_END, TRANSITION_START};

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
#[allow(clippy::type_complexity)]
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
