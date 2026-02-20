use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use simulation::config::{WORLD_HEIGHT, WORLD_WIDTH};

const PAN_SPEED: f32 = 500.0;
const ZOOM_SPEED: f32 = 0.15;
const MIN_DISTANCE: f32 = 20.0;
const MAX_DISTANCE: f32 = 4000.0;
const MIN_PITCH: f32 = 5.0 * std::f32::consts::PI / 180.0; // 5 degrees (near street level)
const MAX_PITCH: f32 = 80.0 * std::f32::consts::PI / 180.0; // 80 degrees
const ORBIT_SENSITIVITY: f32 = 0.005;
const KEYBOARD_ROTATE_SPEED: f32 = 2.0;

/// Movement threshold in pixels: right-click release below this triggers context menu.
const RIGHT_DRAG_THRESHOLD: f32 = 5.0;

/// Orbital camera model: camera orbits around a focus point on the ground.
#[derive(Resource)]
pub struct OrbitCamera {
    /// Ground point the camera looks at
    pub focus: Vec3,
    /// Horizontal rotation in radians
    pub yaw: f32,
    /// Elevation angle in radians (clamped between MIN_PITCH and MAX_PITCH)
    pub pitch: f32,
    /// Distance from focus point
    pub distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::new(WORLD_WIDTH / 2.0, 0.0, WORLD_HEIGHT / 2.0),
            yaw: 0.0,
            pitch: 45.0_f32.to_radians(),
            distance: 2000.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraDrag {
    pub dragging: bool,
    pub last_pos: Vec2,
}

#[derive(Resource, Default)]
pub struct CameraOrbitDrag {
    pub dragging: bool,
    pub last_pos: Vec2,
}

/// Tracks right-click drag state: differentiates click from drag.
/// When the mouse moves beyond `RIGHT_DRAG_THRESHOLD` pixels from the initial press,
/// it becomes a camera orbit. Otherwise, on release, it triggers the context menu.
#[derive(Resource, Default)]
pub struct RightClickDrag {
    pub pressed: bool,
    pub start_pos: Vec2,
    /// True once mouse has moved beyond threshold — this is an orbit drag, not a context-menu click.
    pub is_dragging: bool,
    /// Set to true for exactly one frame when right-click is released without exceeding the drag threshold.
    pub just_released_click: bool,
}

/// Tracks left-click drag state: differentiates click from drag.
/// When the mouse moves beyond `DRAG_THRESHOLD` pixels from the initial press,
/// it becomes a camera pan and suppresses tool input.
#[derive(Resource, Default)]
pub struct LeftClickDrag {
    pub pressed: bool,
    pub start_pos: Vec2,
    pub last_pos: Vec2,
    /// True once mouse has moved beyond threshold — this is a camera drag, not a tool click.
    pub is_dragging: bool,
}

const LEFT_DRAG_THRESHOLD: f32 = 5.0;

pub fn setup_camera(mut commands: Commands) {
    let orbit = OrbitCamera::default();
    let (pos, look_at) = orbit_to_transform(&orbit);

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(pos).looking_at(look_at, Vec3::Y),
    ));
    commands.insert_resource(orbit);
    commands.init_resource::<CameraOrbitDrag>();
}

fn clamp_focus(focus: &mut Vec3) {
    let margin = 500.0;
    focus.x = focus.x.clamp(-margin, WORLD_WIDTH + margin);
    focus.z = focus.z.clamp(-margin, WORLD_HEIGHT + margin);
}

fn orbit_to_transform(orbit: &OrbitCamera) -> (Vec3, Vec3) {
    // Spherical to cartesian offset from focus
    let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
    let y = orbit.distance * orbit.pitch.sin();
    let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();
    let pos = orbit.focus + Vec3::new(x, y, z);
    (pos, orbit.focus)
}

/// System: apply OrbitCamera state to the actual camera Transform each frame.
pub fn apply_orbit_camera(
    orbit: Res<OrbitCamera>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    if !orbit.is_changed() {
        return;
    }
    let (pos, look_at) = orbit_to_transform(&orbit);
    let Ok(mut transform) = query.get_single_mut() else {
        return;
    };
    *transform = Transform::from_translation(pos).looking_at(look_at, Vec3::Y);
}

/// WASD/Arrow keys: pan focus along ground plane (direction relative to current yaw).
pub fn camera_pan_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut orbit: ResMut<OrbitCamera>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    let scale = orbit.distance / 1000.0;

    let mut dir = Vec2::ZERO;
    if bindings.camera_pan_up.pressed(&keys) || bindings.camera_pan_up_alt.pressed(&keys) {
        dir.y -= 1.0;
    }
    if bindings.camera_pan_down.pressed(&keys) || bindings.camera_pan_down_alt.pressed(&keys) {
        dir.y += 1.0;
    }
    if bindings.camera_pan_left.pressed(&keys) || bindings.camera_pan_left_alt.pressed(&keys) {
        dir.x -= 1.0;
    }
    if bindings.camera_pan_right.pressed(&keys) || bindings.camera_pan_right_alt.pressed(&keys) {
        dir.x += 1.0;
    }

    if dir != Vec2::ZERO {
        let dir = dir.normalize();
        let delta = PAN_SPEED * scale * time.delta_secs();
        // Rotate movement direction by current yaw
        let cos_yaw = orbit.yaw.cos();
        let sin_yaw = orbit.yaw.sin();
        let world_x = dir.x * cos_yaw + dir.y * sin_yaw;
        let world_z = -dir.x * sin_yaw + dir.y * cos_yaw;
        orbit.focus.x += world_x * delta;
        orbit.focus.z += world_z * delta;
        clamp_focus(&mut orbit.focus);
    }
}

/// Middle-mouse drag: pan focus.
pub fn camera_pan_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut drag: ResMut<CameraDrag>,
    mut orbit: ResMut<OrbitCamera>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let scale = orbit.distance / 1000.0;

    if buttons.just_pressed(MouseButton::Middle) {
        if let Some(pos) = window.cursor_position() {
            drag.dragging = true;
            drag.last_pos = pos;
        }
    }

    if buttons.just_released(MouseButton::Middle) {
        drag.dragging = false;
    }

    if drag.dragging {
        if let Some(pos) = window.cursor_position() {
            let delta = pos - drag.last_pos;
            // Rotate pan direction by current yaw
            let cos_yaw = orbit.yaw.cos();
            let sin_yaw = orbit.yaw.sin();
            let world_x = -delta.x * cos_yaw - delta.y * sin_yaw;
            let world_z = delta.x * sin_yaw - delta.y * cos_yaw;
            orbit.focus.x += world_x * scale;
            orbit.focus.z += world_z * scale;
            clamp_focus(&mut orbit.focus);
            drag.last_pos = pos;
        }
    }
}

/// Right-mouse drag: orbit (horizontal = yaw, vertical = pitch).
/// Only orbits after the mouse moves beyond RIGHT_DRAG_THRESHOLD pixels.
/// If released before that, sets `RightClickDrag::just_released_click` for one frame.
pub fn camera_orbit_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut drag: ResMut<CameraOrbitDrag>,
    mut right_click: ResMut<RightClickDrag>,
    mut orbit: ResMut<OrbitCamera>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    // Clear the one-frame flag from the previous frame
    right_click.just_released_click = false;

    if buttons.just_pressed(MouseButton::Right) {
        if let Some(pos) = window.cursor_position() {
            drag.dragging = true;
            drag.last_pos = pos;
            right_click.pressed = true;
            right_click.start_pos = pos;
            right_click.is_dragging = false;
        }
    }

    if buttons.just_released(MouseButton::Right) {
        drag.dragging = false;
        if right_click.pressed && !right_click.is_dragging {
            right_click.just_released_click = true;
        }
        right_click.pressed = false;
        right_click.is_dragging = false;
    }

    if drag.dragging {
        if let Some(pos) = window.cursor_position() {
            // Check if we've exceeded the drag threshold
            if !right_click.is_dragging {
                let dist = (pos - right_click.start_pos).length();
                if dist > RIGHT_DRAG_THRESHOLD {
                    right_click.is_dragging = true;
                    drag.last_pos = pos;
                }
            }

            // Only orbit once we've exceeded the threshold
            if right_click.is_dragging {
                let delta = pos - drag.last_pos;
                orbit.yaw += delta.x * ORBIT_SENSITIVITY;
                orbit.pitch =
                    (orbit.pitch - delta.y * ORBIT_SENSITIVITY).clamp(MIN_PITCH, MAX_PITCH);
                drag.last_pos = pos;
            }
        }
    }
}

/// Left-mouse drag: pan focus (with threshold to distinguish from clicks).
pub fn camera_left_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut left_drag: ResMut<LeftClickDrag>,
    mut orbit: ResMut<OrbitCamera>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let scale = orbit.distance / 1000.0;

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(pos) = window.cursor_position() {
            left_drag.pressed = true;
            left_drag.start_pos = pos;
            left_drag.last_pos = pos;
            left_drag.is_dragging = false;
        }
    }

    if buttons.just_released(MouseButton::Left) {
        left_drag.pressed = false;
        left_drag.is_dragging = false;
    }

    if left_drag.pressed {
        if let Some(pos) = window.cursor_position() {
            if !left_drag.is_dragging {
                let dist = (pos - left_drag.start_pos).length();
                if dist > LEFT_DRAG_THRESHOLD {
                    left_drag.is_dragging = true;
                    left_drag.last_pos = pos;
                }
            }

            if left_drag.is_dragging {
                let delta = pos - left_drag.last_pos;
                let cos_yaw = orbit.yaw.cos();
                let sin_yaw = orbit.yaw.sin();
                let world_x = -delta.x * cos_yaw - delta.y * sin_yaw;
                let world_z = delta.x * sin_yaw - delta.y * cos_yaw;
                orbit.focus.x += world_x * scale;
                orbit.focus.z += world_z * scale;
                clamp_focus(&mut orbit.focus);
                left_drag.last_pos = pos;
            }
        }
    }
}

/// Q/E keys: rotate camera left/right around focus point.
pub fn camera_rotate_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut orbit: ResMut<OrbitCamera>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    let mut yaw_delta = 0.0;
    if bindings.camera_rotate_left.pressed(&keys) {
        yaw_delta -= 1.0;
    }
    if bindings.camera_rotate_right.pressed(&keys) {
        yaw_delta += 1.0;
    }
    if yaw_delta != 0.0 {
        orbit.yaw += yaw_delta * KEYBOARD_ROTATE_SPEED * time.delta_secs();
    }
}

/// Scroll wheel: zoom (change distance).
pub fn camera_zoom(mut scroll_evts: EventReader<MouseWheel>, mut orbit: ResMut<OrbitCamera>) {
    for evt in scroll_evts.read() {
        let dy = match evt.unit {
            MouseScrollUnit::Line => evt.y,
            MouseScrollUnit::Pixel => evt.y / 100.0,
        };
        let factor = 1.0 - dy * ZOOM_SPEED;
        orbit.distance = (orbit.distance * factor).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }
}

/// Numpad +/-: zoom in/out (same step as one scroll line).
pub fn camera_zoom_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut orbit: ResMut<OrbitCamera>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    let mut dy = 0.0;
    if bindings.camera_zoom_in.pressed(&keys) {
        dy += 1.0;
    }
    if bindings.camera_zoom_out.pressed(&keys) {
        dy -= 1.0;
    }
    if dy != 0.0 {
        let factor = 1.0 - dy * ZOOM_SPEED;
        orbit.distance = (orbit.distance * factor).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }
}
