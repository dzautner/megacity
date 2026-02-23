//! Key binding types, the `KeyBindings` resource, serialization, plugin, and rebind system.

use bevy::prelude::*;

use super::actions::BindableAction;
use super::key_helpers::{keycode_label, keycode_to_u16, u16_to_keycode};
use crate::Saveable;

// =============================================================================
// Key binding definition
// =============================================================================

/// A single key binding: a key code plus optional modifier flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub ctrl: bool,
    pub shift: bool,
}

impl KeyBinding {
    /// Create a simple binding with no modifiers.
    pub const fn simple(key: KeyCode) -> Self {
        Self {
            key,
            ctrl: false,
            shift: false,
        }
    }

    /// Create a binding that requires Ctrl.
    pub const fn ctrl(key: KeyCode) -> Self {
        Self {
            key,
            ctrl: true,
            shift: false,
        }
    }

    /// Check if this binding is currently pressed (just_pressed for the key,
    /// modifiers must be held).
    pub fn just_pressed(self, keys: &ButtonInput<KeyCode>) -> bool {
        if !keys.just_pressed(self.key) {
            return false;
        }
        let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        ctrl_held == self.ctrl && shift_held == self.shift
    }

    /// Check if this binding's key is currently held (for continuous actions
    /// like camera pan). Modifier state is checked as well.
    pub fn pressed(self, keys: &ButtonInput<KeyCode>) -> bool {
        if !keys.pressed(self.key) {
            return false;
        }
        let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        ctrl_held == self.ctrl && shift_held == self.shift
    }

    /// Human-readable label (e.g. "Ctrl+S", "Shift+Tab", "F12").
    pub fn display_label(self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        parts.push(keycode_label(self.key));
        parts.join("+")
    }
}

// =============================================================================
// Serializable binding for persistence
// =============================================================================

#[derive(bitcode::Encode, bitcode::Decode, Clone, Copy, PartialEq)]
struct SerBinding {
    key_disc: u16,
    ctrl: bool,
    shift: bool,
}

impl SerBinding {
    fn from_binding(b: KeyBinding) -> Self {
        Self {
            key_disc: keycode_to_u16(b.key),
            ctrl: b.ctrl,
            shift: b.shift,
        }
    }

    fn to_binding(self) -> KeyBinding {
        KeyBinding {
            key: u16_to_keycode(self.key_disc),
            ctrl: self.ctrl,
            shift: self.shift,
        }
    }
}

#[derive(bitcode::Encode, bitcode::Decode, Clone, Default, PartialEq)]
struct SerKeyBindings {
    bindings: Vec<(u8, SerBinding)>,
}

// =============================================================================
// KeyBindings resource
// =============================================================================

/// Central resource holding all configurable keybindings.
/// Systems should read from this instead of hardcoding `KeyCode` values.
#[derive(Resource, Clone)]
pub struct KeyBindings {
    // Camera
    pub camera_pan_up: KeyBinding,
    pub camera_pan_up_alt: KeyBinding,
    pub camera_pan_down: KeyBinding,
    pub camera_pan_down_alt: KeyBinding,
    pub camera_pan_left: KeyBinding,
    pub camera_pan_left_alt: KeyBinding,
    pub camera_pan_right: KeyBinding,
    pub camera_pan_right_alt: KeyBinding,
    pub camera_rotate_left: KeyBinding,
    pub camera_rotate_right: KeyBinding,
    pub camera_zoom_in: KeyBinding,
    pub camera_zoom_out: KeyBinding,

    // Speed
    pub toggle_pause: KeyBinding,
    pub speed_normal: KeyBinding,
    pub speed_fast: KeyBinding,
    pub speed_fastest: KeyBinding,

    // Tools
    pub tool_road: KeyBinding,
    pub tool_zone_res: KeyBinding,
    pub tool_zone_com: KeyBinding,
    pub tool_bulldoze: KeyBinding,
    pub tool_inspect: KeyBinding,
    pub toggle_grid_snap: KeyBinding,
    pub toggle_curve_draw: KeyBinding,
    pub delete_building: KeyBinding,
    pub delete_building_alt: KeyBinding,
    pub escape: KeyBinding,

    // Overlays
    pub overlay_cycle_next: KeyBinding,

    // Panels
    pub toggle_journal: KeyBinding,
    pub toggle_charts: KeyBinding,
    pub toggle_advisor: KeyBinding,
    pub toggle_policies: KeyBinding,
    pub toggle_settings: KeyBinding,
    pub toggle_search: KeyBinding,

    // System
    pub quick_save: KeyBinding,
    pub quick_load: KeyBinding,
    pub new_game: KeyBinding,
    pub screenshot: KeyBinding,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            camera_pan_up: KeyBinding::simple(KeyCode::KeyW),
            camera_pan_up_alt: KeyBinding::simple(KeyCode::ArrowUp),
            camera_pan_down: KeyBinding::simple(KeyCode::KeyS),
            camera_pan_down_alt: KeyBinding::simple(KeyCode::ArrowDown),
            camera_pan_left: KeyBinding::simple(KeyCode::KeyA),
            camera_pan_left_alt: KeyBinding::simple(KeyCode::ArrowLeft),
            camera_pan_right: KeyBinding::simple(KeyCode::KeyD),
            camera_pan_right_alt: KeyBinding::simple(KeyCode::ArrowRight),
            camera_rotate_left: KeyBinding::simple(KeyCode::KeyQ),
            camera_rotate_right: KeyBinding::simple(KeyCode::KeyE),
            camera_zoom_in: KeyBinding::simple(KeyCode::NumpadAdd),
            camera_zoom_out: KeyBinding::simple(KeyCode::NumpadSubtract),
            toggle_pause: KeyBinding::simple(KeyCode::Space),
            speed_normal: KeyBinding::simple(KeyCode::Digit1),
            speed_fast: KeyBinding::simple(KeyCode::Digit2),
            speed_fastest: KeyBinding::simple(KeyCode::Digit3),
            tool_road: KeyBinding::simple(KeyCode::KeyR),
            tool_zone_res: KeyBinding::simple(KeyCode::KeyZ),
            tool_zone_com: KeyBinding::simple(KeyCode::KeyV),
            tool_bulldoze: KeyBinding::simple(KeyCode::KeyB),
            tool_inspect: KeyBinding::simple(KeyCode::KeyI),
            toggle_grid_snap: KeyBinding::simple(KeyCode::KeyF),
            toggle_curve_draw: KeyBinding::simple(KeyCode::KeyG),
            delete_building: KeyBinding::simple(KeyCode::Delete),
            delete_building_alt: KeyBinding::simple(KeyCode::Backspace),
            escape: KeyBinding::simple(KeyCode::Escape),
            overlay_cycle_next: KeyBinding::simple(KeyCode::Tab),
            toggle_journal: KeyBinding::simple(KeyCode::KeyJ),
            toggle_charts: KeyBinding::simple(KeyCode::KeyC),
            toggle_advisor: KeyBinding::simple(KeyCode::KeyA),
            toggle_policies: KeyBinding::simple(KeyCode::KeyP),
            toggle_settings: KeyBinding::simple(KeyCode::F9),
            toggle_search: KeyBinding::ctrl(KeyCode::KeyF),
            quick_save: KeyBinding::ctrl(KeyCode::KeyS),
            quick_load: KeyBinding::ctrl(KeyCode::KeyL),
            new_game: KeyBinding::ctrl(KeyCode::KeyN),
            screenshot: KeyBinding::simple(KeyCode::F12),
        }
    }
}

impl KeyBindings {
    /// Get the binding for a specific action.
    pub fn get(&self, action: BindableAction) -> KeyBinding {
        match action {
            BindableAction::CameraPanUp => self.camera_pan_up,
            BindableAction::CameraPanDown => self.camera_pan_down,
            BindableAction::CameraPanLeft => self.camera_pan_left,
            BindableAction::CameraPanRight => self.camera_pan_right,
            BindableAction::CameraRotateLeft => self.camera_rotate_left,
            BindableAction::CameraRotateRight => self.camera_rotate_right,
            BindableAction::CameraZoomIn => self.camera_zoom_in,
            BindableAction::CameraZoomOut => self.camera_zoom_out,
            BindableAction::TogglePause => self.toggle_pause,
            BindableAction::SpeedNormal => self.speed_normal,
            BindableAction::SpeedFast => self.speed_fast,
            BindableAction::SpeedFastest => self.speed_fastest,
            BindableAction::ToolRoad => self.tool_road,
            BindableAction::ToolZoneRes => self.tool_zone_res,
            BindableAction::ToolZoneCom => self.tool_zone_com,
            BindableAction::ToolBulldoze => self.tool_bulldoze,
            BindableAction::ToolInspect => self.tool_inspect,
            BindableAction::ToggleGridSnap => self.toggle_grid_snap,
            BindableAction::ToggleCurveDraw => self.toggle_curve_draw,
            BindableAction::DeleteBuilding => self.delete_building,
            BindableAction::Escape => self.escape,
            BindableAction::OverlayCycleNext => self.overlay_cycle_next,
            BindableAction::ToggleJournal => self.toggle_journal,
            BindableAction::ToggleCharts => self.toggle_charts,
            BindableAction::ToggleAdvisor => self.toggle_advisor,
            BindableAction::TogglePolicies => self.toggle_policies,
            BindableAction::ToggleSettings => self.toggle_settings,
            BindableAction::ToggleSearch => self.toggle_search,
            BindableAction::QuickSave => self.quick_save,
            BindableAction::QuickLoad => self.quick_load,
            BindableAction::NewGame => self.new_game,
            BindableAction::Screenshot => self.screenshot,
        }
    }

    /// Set the binding for a specific action.
    pub fn set(&mut self, action: BindableAction, binding: KeyBinding) {
        match action {
            BindableAction::CameraPanUp => self.camera_pan_up = binding,
            BindableAction::CameraPanDown => self.camera_pan_down = binding,
            BindableAction::CameraPanLeft => self.camera_pan_left = binding,
            BindableAction::CameraPanRight => self.camera_pan_right = binding,
            BindableAction::CameraRotateLeft => self.camera_rotate_left = binding,
            BindableAction::CameraRotateRight => self.camera_rotate_right = binding,
            BindableAction::CameraZoomIn => self.camera_zoom_in = binding,
            BindableAction::CameraZoomOut => self.camera_zoom_out = binding,
            BindableAction::TogglePause => self.toggle_pause = binding,
            BindableAction::SpeedNormal => self.speed_normal = binding,
            BindableAction::SpeedFast => self.speed_fast = binding,
            BindableAction::SpeedFastest => self.speed_fastest = binding,
            BindableAction::ToolRoad => self.tool_road = binding,
            BindableAction::ToolZoneRes => self.tool_zone_res = binding,
            BindableAction::ToolZoneCom => self.tool_zone_com = binding,
            BindableAction::ToolBulldoze => self.tool_bulldoze = binding,
            BindableAction::ToolInspect => self.tool_inspect = binding,
            BindableAction::ToggleGridSnap => self.toggle_grid_snap = binding,
            BindableAction::ToggleCurveDraw => self.toggle_curve_draw = binding,
            BindableAction::DeleteBuilding => self.delete_building = binding,
            BindableAction::Escape => self.escape = binding,
            BindableAction::OverlayCycleNext => self.overlay_cycle_next = binding,
            BindableAction::ToggleJournal => self.toggle_journal = binding,
            BindableAction::ToggleCharts => self.toggle_charts = binding,
            BindableAction::ToggleAdvisor => self.toggle_advisor = binding,
            BindableAction::TogglePolicies => self.toggle_policies = binding,
            BindableAction::ToggleSettings => self.toggle_settings = binding,
            BindableAction::ToggleSearch => self.toggle_search = binding,
            BindableAction::QuickSave => self.quick_save = binding,
            BindableAction::QuickLoad => self.quick_load = binding,
            BindableAction::NewGame => self.new_game = binding,
            BindableAction::Screenshot => self.screenshot = binding,
        }
    }

    /// Detect conflicts: returns pairs of actions sharing the same binding
    /// within the same category.
    pub fn find_conflicts(&self) -> Vec<(BindableAction, BindableAction)> {
        let mut conflicts = Vec::new();
        let all = BindableAction::ALL;
        for (i, &a) in all.iter().enumerate() {
            for &b in &all[i + 1..] {
                if a.category() != b.category() {
                    continue;
                }
                if self.get(a) == self.get(b) {
                    conflicts.push((a, b));
                }
            }
        }
        conflicts
    }

    fn to_ser(&self) -> SerKeyBindings {
        let mut bindings = Vec::new();
        for (i, &action) in BindableAction::ALL.iter().enumerate() {
            bindings.push((i as u8, SerBinding::from_binding(self.get(action))));
        }
        SerKeyBindings { bindings }
    }

    fn from_ser(ser: &SerKeyBindings) -> Self {
        let mut kb = KeyBindings::default();
        for &(idx, sb) in &ser.bindings {
            if let Some(&action) = BindableAction::ALL.get(idx as usize) {
                kb.set(action, sb.to_binding());
            }
        }
        kb
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl Saveable for KeyBindings {
    const SAVE_KEY: &'static str = "keybindings";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        let default = KeyBindings::default();
        let self_ser = self.to_ser();
        let default_ser = default.to_ser();
        if self_ser == default_ser {
            return None;
        }
        Some(bitcode::encode(&self_ser))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        match bitcode::decode::<SerKeyBindings>(bytes) {
            Ok(ser) => Self::from_ser(&ser),
            Err(e) => {
                warn!(
                    "KeyBindings: failed to decode {} bytes, falling back to default: {}",
                    bytes.len(),
                    e
                );
                Self::default()
            }
        }
    }
}

// =============================================================================
// State resource for the rebind UI
// =============================================================================

/// Tracks which action (if any) is currently awaiting a new key assignment.
#[derive(Resource, Default)]
pub struct RebindState {
    pub awaiting: Option<BindableAction>,
}

// =============================================================================
// Plugin
// =============================================================================

pub struct KeyBindingsPlugin;

impl Plugin for KeyBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindings>()
            .init_resource::<RebindState>()
            .add_systems(
                Update,
                capture_rebind_input.in_set(crate::SimulationUpdateSet::Input),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<KeyBindings>();
    }
}

/// System: when a rebind is in progress, capture the next key press and assign it.
/// Uses Option<Res> so the system gracefully no-ops in headless/test contexts
/// where Bevy's InputPlugin (and thus ButtonInput<KeyCode>) is not present.
fn capture_rebind_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut bindings: ResMut<KeyBindings>,
    mut rebind: ResMut<RebindState>,
) {
    let Some(keys) = keys else {
        return;
    };
    let Some(action) = rebind.awaiting else {
        return;
    };

    for key in keys.get_just_pressed() {
        if matches!(
            key,
            KeyCode::ControlLeft
                | KeyCode::ControlRight
                | KeyCode::ShiftLeft
                | KeyCode::ShiftRight
                | KeyCode::AltLeft
                | KeyCode::AltRight
                | KeyCode::SuperLeft
                | KeyCode::SuperRight
        ) {
            continue;
        }

        let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

        bindings.set(
            action,
            KeyBinding {
                key: *key,
                ctrl,
                shift,
            },
        );
        rebind.awaiting = None;
        return;
    }
}
