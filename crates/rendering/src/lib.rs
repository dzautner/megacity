use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub mod angle_snap;
pub mod box_selection;
pub mod building_mesh_variants;
pub mod building_meshes;
pub mod building_render;
pub mod building_status_enhanced;
pub mod camera;
pub mod citizen_render;
pub mod color_ramps;
pub mod colorblind_palette;
pub mod construction_anim;
pub mod cursor_preview;
pub mod day_night;
pub mod input;
pub mod lane_markings;
pub mod network_viz;
pub mod oneway_arrows;
pub mod overlay;
pub mod props;
pub mod satellite_view;
pub mod terrain_render;
pub mod tree_props;

pub mod road_grade;
pub mod road_render;
pub mod selection_highlight;
pub mod status_icons;
pub mod traffic_arrows;
pub mod traffic_los_render;
pub mod wind_streamlines;

pub mod grid_align;
pub mod parallel_snap;
pub mod screenshot;
pub mod zone_brush_preview;

use angle_snap::AngleSnapState;
use camera::{CameraDrag, LeftClickDrag, RightClickDrag};
use input::{
    ActiveTool, CursorGridPos, GridSnap, IntersectionSnap, RoadDrawState, SelectedBuilding,
    StatusMessage,
};
use overlay::OverlayState;
use props::PropsSpawned;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraDrag>()
            .init_resource::<LeftClickDrag>()
            .init_resource::<RightClickDrag>()
            .init_resource::<CursorGridPos>()
            .init_resource::<ActiveTool>()
            .init_resource::<OverlayState>()
            .init_resource::<StatusMessage>()
            .init_resource::<SelectedBuilding>()
            .init_resource::<PropsSpawned>()
            .init_resource::<RoadDrawState>()
            .init_resource::<GridSnap>()
            .init_resource::<AngleSnapState>()
            .init_resource::<IntersectionSnap>()
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
                    camera::camera_rotate_keyboard,
                    camera::apply_orbit_camera,
                ),
            )
            .add_systems(
                Update,
                (
                    input::update_cursor_grid_pos,
                    angle_snap::update_angle_snap,
                    input::update_intersection_snap,
                    grid_align::align_cursor_to_grid
                        .after(angle_snap::update_angle_snap)
                        .after(parallel_snap::apply_parallel_snap_to_cursor)
                        .before(input::handle_tool_input),
                    grid_align::align_angle_snap_to_grid
                        .after(angle_snap::update_angle_snap)
                        .before(input::handle_tool_input),
                    grid_align::align_intersection_snap_to_grid
                        .after(input::update_intersection_snap)
                        .before(input::handle_tool_input),
                    input::handle_tool_input,
                    input::handle_tree_tool,
                    input::keyboard_tool_switch,
                    input::toggle_grid_snap,
                    input::handle_escape_key,
                    input::delete_selected_building,
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
                    angle_snap::draw_angle_snap_indicator,
                    cursor_preview::draw_intersection_snap_indicator,
                    road_render::sync_road_segment_meshes,
                    lane_markings::sync_lane_marking_meshes,
                    road_grade::draw_road_grade_indicators,
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
                    props::spawn_tree_props,
                    props::spawn_road_props,
                    props::spawn_parked_cars,
                ),
            )
            .add_systems(
                Update,
                (
                    citizen_render::spawn_citizen_sprites,
                    citizen_render::trigger_lod_fade,
                    citizen_render::update_citizen_sprites,
                    citizen_render::update_lod_fade,
                    citizen_render::despawn_abstract_sprites,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    building_render::spawn_planted_tree_meshes,
                    building_render::cleanup_planted_tree_meshes
                        .run_if(on_timer(std::time::Duration::from_secs(1))),
                    status_icons::update_building_status_icons
                        .run_if(on_timer(std::time::Duration::from_secs(2))),
                    building_status_enhanced::update_enhanced_status_icons
                        .run_if(on_timer(std::time::Duration::from_secs(2))),
                    building_status_enhanced::lod_enhanced_status_icons,
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
            .add_systems(Update, oneway_arrows::draw_oneway_arrows)
            .add_plugins(traffic_los_render::TrafficLosRenderPlugin)
            .add_plugins(traffic_arrows::TrafficArrowsPlugin)
            .add_plugins(wind_streamlines::WindStreamlinesPlugin)
            .add_plugins(tree_props::TreePropsPlugin)
            .add_plugins(network_viz::NetworkVizPlugin);

        // Screenshot plugin (F12 to capture)
        app.add_plugins(screenshot::ScreenshotPlugin);

        // Building mesh variant plugin (level-aware model selection)
        app.add_plugins(building_mesh_variants::BuildingMeshVariantsPlugin);

        // Satellite view at maximum zoom-out
        app.add_plugins(satellite_view::SatelliteViewPlugin);

        // Parallel road snapping (UX-026)
        app.add_plugins(parallel_snap::ParallelSnapPlugin);

        // Box selection (UX-011)
        app.add_plugins(box_selection::BoxSelectionPlugin);

        // Zone brush preview (UX-018)
        app.add_plugins(zone_brush_preview::ZoneBrushPreviewPlugin);
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
