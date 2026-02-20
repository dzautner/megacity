use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub mod building_mesh_variants;
pub mod building_meshes;
pub mod building_render;
pub mod camera;
pub mod citizen_render;
pub mod color_ramps;
pub mod construction_anim;
pub mod cursor_preview;
pub mod day_night;
pub mod input;
pub mod lane_markings;
pub mod overlay;
pub mod props;
pub mod terrain_render;

pub mod road_render;
pub mod selection_highlight;
pub mod status_icons;
pub mod traffic_los_render;

use camera::{CameraDrag, LeftClickDrag};
use input::{ActiveTool, CursorGridPos, GridSnap, RoadDrawState, SelectedBuilding, StatusMessage};
use overlay::OverlayState;
use props::PropsSpawned;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraDrag>()
            .init_resource::<LeftClickDrag>()
            .init_resource::<CursorGridPos>()
            .init_resource::<ActiveTool>()
            .init_resource::<OverlayState>()
            .init_resource::<StatusMessage>()
            .init_resource::<SelectedBuilding>()
            .init_resource::<PropsSpawned>()
            .init_resource::<RoadDrawState>()
            .init_resource::<GridSnap>()
            .add_systems(
                Startup,
                (
                    camera::setup_camera,
                    setup_lighting,
                    terrain_render::spawn_terrain_chunks,
                    cursor_preview::spawn_cursor_preview,
                    building_meshes::load_building_models,
                )
                    .chain()
                    .after(simulation::world_init::init_world),
            )
            .add_systems(
                Update,
                (
                    camera::camera_pan_keyboard,
                    camera::camera_pan_drag,
                    camera::camera_left_drag,
                    camera::camera_orbit_drag,
                    camera::camera_zoom,
                    camera::camera_zoom_keyboard,
                    camera::apply_orbit_camera,
                ),
            )
            .add_systems(
                Update,
                (
                    input::update_cursor_grid_pos,
                    input::handle_tool_input,
                    input::handle_tree_tool,
                    input::keyboard_tool_switch,
                    input::toggle_grid_snap,
                    input::tick_status_message,
                    overlay::toggle_overlay_keys,
                ),
            )
            .add_systems(
                Update,
                (
                    terrain_render::dirty_chunks_on_overlay_change,
                    terrain_render::rebuild_dirty_chunks,
                    cursor_preview::update_cursor_preview,
                    cursor_preview::draw_bezier_preview,
                    road_render::sync_road_segment_meshes,
                    lane_markings::sync_lane_marking_meshes,
                ),
            )
            .add_systems(Update, day_night::update_day_night_cycle)
            .add_systems(Update, day_night::update_fog_rendering)
            .add_systems(
                Update,
                (
                    building_render::spawn_building_meshes,
                    building_render::update_building_meshes,
                    building_render::update_construction_visuals,
                    building_render::cleanup_orphan_building_meshes
                        .run_if(on_timer(std::time::Duration::from_secs(1))),
                    citizen_render::spawn_citizen_sprites,
                    citizen_render::update_citizen_sprites,
                    citizen_render::despawn_abstract_sprites,
                    props::spawn_tree_props,
                    props::spawn_road_props,
                    props::spawn_parked_cars,
                ),
            )
            .add_systems(
                Update,
                (
                    building_render::spawn_planted_tree_meshes,
                    building_render::cleanup_planted_tree_meshes
                        .run_if(on_timer(std::time::Duration::from_secs(1))),
                    status_icons::update_building_status_icons
                        .run_if(on_timer(std::time::Duration::from_secs(2))),
                ),
            )
            .add_systems(
                Update,
                (
                    construction_anim::spawn_construction_props,
                    construction_anim::update_construction_anim,
                    construction_anim::animate_crane_rotation,
                    construction_anim::cleanup_construction_props,
                    construction_anim::cleanup_orphan_construction_props
                        .run_if(on_timer(std::time::Duration::from_secs(1))),
                ),
            )
            .add_systems(
                Update,
                (
                    selection_highlight::manage_selection_highlights,
                    selection_highlight::animate_selection_highlights,
                    selection_highlight::draw_connected_highlights,
                ),
            )
            .add_plugins(traffic_los_render::TrafficLosRenderPlugin);

        // Building mesh variant plugin (level-aware model selection)
        app.add_plugins(building_mesh_variants::BuildingMeshVariantsPlugin);
    }
}

fn setup_lighting(mut commands: Commands) {
    // Ambient light for baseline illumination
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.9, 0.9, 1.0),
        brightness: 300.0,
    });

    // Directional light (sun) angled from above
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4, // 45 degrees down
            std::f32::consts::FRAC_PI_6,  // slight rotation
            0.0,
        )),
    ));
}
