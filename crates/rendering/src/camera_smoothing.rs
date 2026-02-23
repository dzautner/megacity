//! Camera smoothing via exponential interpolation (lerp).
//!
//! Input systems write to `CameraTarget` (the desired camera state).
//! Each frame, `smooth_camera_to_target` lerps `OrbitCamera` toward `CameraTarget`
//! using frame-rate independent exponential interpolation:
//!
//!   `value += (target - value) * (1 - exp(-speed * dt))`
//!
//! This produces smooth, responsive camera motion that converges quickly
//! without drift or frame-rate dependency.
//!
//! External systems (minimap, follow-citizen, notifications) that write directly
//! to `OrbitCamera` are detected via Bevy change detection, and `CameraTarget`
//! is synced to match, treating those writes as instant teleports.

use bevy::prelude::*;

use crate::camera::OrbitCamera;

/// Configurable smoothing parameters.
#[derive(Resource)]
pub struct CameraSmoothingConfig {
    /// Smoothing speed for position (higher = snappier). Default: 8.0.
    pub position_speed: f32,
    /// Smoothing speed for zoom (distance). Default: 8.0.
    pub zoom_speed: f32,
    /// Smoothing speed for rotation (yaw/pitch). Default: 8.0.
    pub rotation_speed: f32,
    /// Convergence threshold — stop interpolating when difference is below this.
    pub epsilon: f32,
}

impl Default for CameraSmoothingConfig {
    fn default() -> Self {
        Self {
            position_speed: 8.0,
            zoom_speed: 8.0,
            rotation_speed: 8.0,
            epsilon: 0.001,
        }
    }
}

/// The desired camera state that input systems write to.
///
/// `OrbitCamera` is the *actual* state applied to the camera transform.
/// `CameraTarget` is where input systems direct their intent.
/// The `smooth_camera_to_target` system bridges the gap each frame.
#[derive(Resource)]
pub struct CameraTarget {
    /// Desired ground focus point.
    pub focus: Vec3,
    /// Desired horizontal rotation in radians.
    pub yaw: f32,
    /// Desired elevation angle in radians.
    pub pitch: f32,
    /// Desired distance from focus.
    pub distance: f32,
}

impl Default for CameraTarget {
    fn default() -> Self {
        let orbit = OrbitCamera::default();
        Self {
            focus: orbit.focus,
            yaw: orbit.yaw,
            pitch: orbit.pitch,
            distance: orbit.distance,
        }
    }
}

/// Tracks the last values this system wrote to `OrbitCamera`, so we can detect
/// external modifications (by other systems that don't know about smoothing).
#[derive(Resource, Default)]
pub struct LastSmoothedState {
    pub focus: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

/// Exponential interpolation factor for a given speed and delta time.
///
/// Returns a value in `[0, 1]` representing how much to move toward the target.
/// At `speed = 8.0` and `dt = 1/60`, this gives ~0.125 per frame, which feels
/// snappy but smooth.
#[inline]
fn exp_lerp_factor(speed: f32, dt: f32) -> f32 {
    1.0 - (-speed * dt).exp()
}

/// System: detect external writes to `OrbitCamera` and sync `CameraTarget`.
///
/// If another system (minimap click, follow citizen, search jump, etc.) modifies
/// `OrbitCamera` directly, we detect the discrepancy and update `CameraTarget`
/// to match, treating it as an instant teleport.
///
/// This system must run BEFORE `smooth_camera_to_target` and AFTER any external
/// systems that might write to `OrbitCamera`.
pub fn sync_target_from_external_changes(
    orbit: Res<OrbitCamera>,
    mut target: ResMut<CameraTarget>,
    mut last: ResMut<LastSmoothedState>,
) {
    // Check if OrbitCamera was changed externally (values differ from what we last wrote)
    let focus_changed = (orbit.focus - last.focus).length_squared() > 0.0001;
    let yaw_changed = (orbit.yaw - last.yaw).abs() > 0.0001;
    let pitch_changed = (orbit.pitch - last.pitch).abs() > 0.0001;
    let dist_changed = (orbit.distance - last.distance).abs() > 0.01;

    if focus_changed {
        target.focus = orbit.focus;
        last.focus = orbit.focus;
    }
    if yaw_changed {
        target.yaw = orbit.yaw;
        last.yaw = orbit.yaw;
    }
    if pitch_changed {
        target.pitch = orbit.pitch;
        last.pitch = orbit.pitch;
    }
    if dist_changed {
        target.distance = orbit.distance;
        last.distance = orbit.distance;
    }
}

/// System: lerp `OrbitCamera` toward `CameraTarget` each frame.
///
/// Uses exponential interpolation for frame-rate independence.
/// When the camera is already at the target (within epsilon), no changes are made
/// to avoid unnecessary change detection triggers.
pub fn smooth_camera_to_target(
    target: Res<CameraTarget>,
    config: Res<CameraSmoothingConfig>,
    time: Res<Time>,
    mut orbit: ResMut<OrbitCamera>,
    mut last: ResMut<LastSmoothedState>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let pos_factor = exp_lerp_factor(config.position_speed, dt);
    let zoom_factor = exp_lerp_factor(config.zoom_speed, dt);
    let rot_factor = exp_lerp_factor(config.rotation_speed, dt);

    let eps = config.epsilon;

    // Smooth focus position
    let focus_delta = target.focus - orbit.focus;
    if focus_delta.length_squared() > eps * eps {
        orbit.focus += focus_delta * pos_factor;
    } else if focus_delta.length_squared() > 0.0 {
        orbit.focus = target.focus;
    }

    // Smooth distance (zoom)
    let dist_delta = target.distance - orbit.distance;
    if dist_delta.abs() > eps {
        orbit.distance += dist_delta * zoom_factor;
    } else if dist_delta != 0.0 {
        orbit.distance = target.distance;
    }

    // Smooth yaw
    let yaw_delta = target.yaw - orbit.yaw;
    if yaw_delta.abs() > eps {
        orbit.yaw += yaw_delta * rot_factor;
    } else if yaw_delta != 0.0 {
        orbit.yaw = target.yaw;
    }

    // Smooth pitch
    let pitch_delta = target.pitch - orbit.pitch;
    if pitch_delta.abs() > eps {
        orbit.pitch += pitch_delta * rot_factor;
    } else if pitch_delta != 0.0 {
        orbit.pitch = target.pitch;
    }

    // Record what we wrote so we can detect external changes next frame
    last.focus = orbit.focus;
    last.yaw = orbit.yaw;
    last.pitch = orbit.pitch;
    last.distance = orbit.distance;
}

/// System: sync `CameraTarget` and `LastSmoothedState` from `OrbitCamera` on startup.
///
/// Run once at startup, after `setup_camera` has initialized `OrbitCamera`.
pub fn init_camera_target(
    orbit: Res<OrbitCamera>,
    mut target: ResMut<CameraTarget>,
    mut last: ResMut<LastSmoothedState>,
) {
    target.focus = orbit.focus;
    target.yaw = orbit.yaw;
    target.pitch = orbit.pitch;
    target.distance = orbit.distance;

    last.focus = orbit.focus;
    last.yaw = orbit.yaw;
    last.pitch = orbit.pitch;
    last.distance = orbit.distance;
}
