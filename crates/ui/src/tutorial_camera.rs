use bevy::prelude::*;
use simulation::app_state::AppState;

use rendering::camera::OrbitCamera;
use simulation::tutorial::TutorialState;
use simulation::tutorial_hints::TutorialUiHint;

/// Tracks whether the tutorial camera has already been focused.
///
/// Prevents repeated camera movement every frame — we only auto-focus once
/// per camera target change.
#[derive(Resource, Default)]
pub struct TutorialCameraState {
    /// The last camera target we focused on, if any.
    last_target: Option<(f32, f32)>,
}

/// Moves the camera to the tutorial hint's camera target once per step change.
pub fn tutorial_camera_focus(
    tutorial: Res<TutorialState>,
    hint: Res<TutorialUiHint>,
    mut orbit: ResMut<OrbitCamera>,
    mut state: ResMut<TutorialCameraState>,
) {
    if !tutorial.active {
        state.last_target = None;
        return;
    }

    if let Some((x, z)) = hint.camera_target {
        if state.last_target != Some((x, z)) {
            orbit.focus.x = x;
            orbit.focus.z = z;
            // Set a comfortable overview distance for the tutorial
            orbit.distance = 1500.0;
            orbit.pitch = 50.0_f32.to_radians();
            state.last_target = Some((x, z));
        }
    }
}

/// Plugin that adds the tutorial camera focus system.
pub struct TutorialCameraPlugin;

impl Plugin for TutorialCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialCameraState>()
            .add_systems(Update, tutorial_camera_focus.run_if(in_state(AppState::Playing)));
    }
}
