use bevy::prelude::*;

pub mod angle_snap;
pub mod aqi_colors;
pub mod auto_grid_draw;
pub mod box_selection;
pub mod building_mesh_variants;
pub mod building_meshes;
pub mod building_preview_mesh;
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

pub mod enhanced_select;
pub mod freehand_draw;
pub mod grid_align;
pub mod intersection_preview;
pub mod parallel_draw;
pub mod parallel_snap;
pub mod screenshot;
pub mod zone_brush_preview;

mod plugin_registration;

use angle_snap::AngleSnapState;
use camera::{CameraDrag, LeftClickDrag, RightClickDrag};
use input::{
    ActiveTool, CursorGridPos, GridSnap, IntersectionSnap, RoadDrawState, SelectedBuilding,
    StatusMessage,
};
use overlay::{DualOverlayState, OverlayState};
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
            .init_resource::<DualOverlayState>()
            .init_resource::<StatusMessage>()
            .init_resource::<SelectedBuilding>()
            .init_resource::<PropsSpawned>()
            .init_resource::<RoadDrawState>()
            .init_resource::<GridSnap>()
            .init_resource::<AngleSnapState>()
            .init_resource::<IntersectionSnap>();

        // Register all rendering systems and plugins (extracted for conflict-free additions)
        plugin_registration::register_rendering_systems(app);
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
