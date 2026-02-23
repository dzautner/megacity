use bevy::prelude::*;

// Auto-discover all public modules from src/ directory.
// plugin_registration is declared manually below because it is private.
automod_dir::dir!(pub "src" exclude "plugin_registration");

mod plugin_registration;

use angle_snap::AngleSnapState;
use camera::{CameraDrag, LeftClickDrag, RightClickDrag};
use camera_smoothing::{CameraSmoothingConfig, CameraTarget, LastSmoothedState};
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
            .init_resource::<CameraTarget>()
            .init_resource::<CameraSmoothingConfig>()
            .init_resource::<LastSmoothedState>()
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
