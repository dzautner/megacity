//! Customizable keybindings resource (UX-035).
//!
//! Provides a `KeyBindings` resource containing all configurable keyboard
//! shortcuts. Systems read from this resource instead of hardcoding `KeyCode`
//! values. A settings UI allows rebinding, with conflict detection and
//! "Reset to Defaults".

use bevy::prelude::*;

use crate::Saveable;

// =============================================================================
// Bindable Action enum
// =============================================================================

/// Every action that can be bound to a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindableAction {
    // Camera
    CameraPanUp,
    CameraPanDown,
    CameraPanLeft,
    CameraPanRight,
    CameraRotateLeft,
    CameraRotateRight,
    CameraZoomIn,
    CameraZoomOut,

    // Simulation speed
    TogglePause,
    SpeedNormal,
    SpeedFast,
    SpeedFastest,

    // Tools
    ToolRoad,
    ToolZoneRes,
    ToolZoneCom,
    ToolBulldoze,
    ToolInspect,
    ToggleGridSnap,
    DeleteBuilding,
    Escape,

    // Overlays
    OverlayCycleNext,

    // Panels
    ToggleJournal,
    ToggleCharts,
    ToggleAdvisor,
    TogglePolicies,
    ToggleSettings,
    ToggleSearch,

    // Save/Load (ctrl-modified)
    QuickSave,
    QuickLoad,
    NewGame,

    // Screenshot
    Screenshot,
}

impl BindableAction {
    /// Human-readable label for display in the settings UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::CameraPanUp => "Camera Pan Up",
            Self::CameraPanDown => "Camera Pan Down",
            Self::CameraPanLeft => "Camera Pan Left",
            Self::CameraPanRight => "Camera Pan Right",
            Self::CameraRotateLeft => "Camera Rotate Left",
            Self::CameraRotateRight => "Camera Rotate Right",
            Self::CameraZoomIn => "Camera Zoom In",
            Self::CameraZoomOut => "Camera Zoom Out",
            Self::TogglePause => "Toggle Pause",
            Self::SpeedNormal => "Speed: Normal (1x)",
            Self::SpeedFast => "Speed: Fast (2x)",
            Self::SpeedFastest => "Speed: Fastest (4x)",
            Self::ToolRoad => "Tool: Road",
            Self::ToolZoneRes => "Tool: Zone Residential",
            Self::ToolZoneCom => "Tool: Zone Commercial",
            Self::ToolBulldoze => "Tool: Bulldoze",
            Self::ToolInspect => "Tool: Inspect",
            Self::ToggleGridSnap => "Toggle Grid Snap",
            Self::DeleteBuilding => "Delete Building",
            Self::Escape => "Cancel / Deselect",
            Self::OverlayCycleNext => "Cycle Overlay",
            Self::ToggleJournal => "Toggle Journal",
            Self::ToggleCharts => "Toggle Charts",
            Self::ToggleAdvisor => "Toggle Advisor",
            Self::TogglePolicies => "Toggle Policies",
            Self::ToggleSettings => "Toggle Settings",
            Self::ToggleSearch => "Toggle Search",
            Self::QuickSave => "Quick Save",
            Self::QuickLoad => "Quick Load",
            Self::NewGame => "New Game",
            Self::Screenshot => "Screenshot",
        }
    }

    /// Category for grouping in the settings UI.
    pub fn category(self) -> &'static str {
        match self {
            Self::CameraPanUp
            | Self::CameraPanDown
            | Self::CameraPanLeft
            | Self::CameraPanRight
            | Self::CameraRotateLeft
            | Self::CameraRotateRight
            | Self::CameraZoomIn
            | Self::CameraZoomOut => "Camera",

            Self::TogglePause | Self::SpeedNormal | Self::SpeedFast | Self::SpeedFastest => "Speed",

            Self::ToolRoad
            | Self::ToolZoneRes
            | Self::ToolZoneCom
            | Self::ToolBulldoze
            | Self::ToolInspect
            | Self::ToggleGridSnap
            | Self::DeleteBuilding
            | Self::Escape => "Tools",

            Self::OverlayCycleNext => "Overlays",

            Self::ToggleJournal
            | Self::ToggleCharts
            | Self::ToggleAdvisor
            | Self::TogglePolicies
            | Self::ToggleSettings
            | Self::ToggleSearch => "Panels",

            Self::QuickSave | Self::QuickLoad | Self::NewGame | Self::Screenshot => "System",
        }
    }

    /// All bindable actions in display order.
    pub const ALL: &'static [BindableAction] = &[
        Self::CameraPanUp,
        Self::CameraPanDown,
        Self::CameraPanLeft,
        Self::CameraPanRight,
        Self::CameraRotateLeft,
        Self::CameraRotateRight,
        Self::CameraZoomIn,
        Self::CameraZoomOut,
        Self::TogglePause,
        Self::SpeedNormal,
        Self::SpeedFast,
        Self::SpeedFastest,
        Self::ToolRoad,
        Self::ToolZoneRes,
        Self::ToolZoneCom,
        Self::ToolBulldoze,
        Self::ToolInspect,
        Self::ToggleGridSnap,
        Self::DeleteBuilding,
        Self::Escape,
        Self::OverlayCycleNext,
        Self::ToggleJournal,
        Self::ToggleCharts,
        Self::ToggleAdvisor,
        Self::TogglePolicies,
        Self::ToggleSettings,
        Self::ToggleSearch,
        Self::QuickSave,
        Self::QuickLoad,
        Self::NewGame,
        Self::Screenshot,
    ];
}

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

// =============================================================================
// Helper: human-readable key labels
// =============================================================================

pub fn keycode_label(key: KeyCode) -> &'static str {
    match key {
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        KeyCode::Escape => "Esc",
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Tab => "Tab",
        KeyCode::Backspace => "Backspace",
        KeyCode::Delete => "Delete",
        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",
        KeyCode::NumpadAdd => "Num+",
        KeyCode::NumpadSubtract => "Num-",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PgUp",
        KeyCode::PageDown => "PgDn",
        _ => "???",
    }
}

fn keycode_to_u16(key: KeyCode) -> u16 {
    match key {
        KeyCode::KeyA => 0,
        KeyCode::KeyB => 1,
        KeyCode::KeyC => 2,
        KeyCode::KeyD => 3,
        KeyCode::KeyE => 4,
        KeyCode::KeyF => 5,
        KeyCode::KeyG => 6,
        KeyCode::KeyH => 7,
        KeyCode::KeyI => 8,
        KeyCode::KeyJ => 9,
        KeyCode::KeyK => 10,
        KeyCode::KeyL => 11,
        KeyCode::KeyM => 12,
        KeyCode::KeyN => 13,
        KeyCode::KeyO => 14,
        KeyCode::KeyP => 15,
        KeyCode::KeyQ => 16,
        KeyCode::KeyR => 17,
        KeyCode::KeyS => 18,
        KeyCode::KeyT => 19,
        KeyCode::KeyU => 20,
        KeyCode::KeyV => 21,
        KeyCode::KeyW => 22,
        KeyCode::KeyX => 23,
        KeyCode::KeyY => 24,
        KeyCode::KeyZ => 25,
        KeyCode::Digit0 => 26,
        KeyCode::Digit1 => 27,
        KeyCode::Digit2 => 28,
        KeyCode::Digit3 => 29,
        KeyCode::Digit4 => 30,
        KeyCode::Digit5 => 31,
        KeyCode::Digit6 => 32,
        KeyCode::Digit7 => 33,
        KeyCode::Digit8 => 34,
        KeyCode::Digit9 => 35,
        KeyCode::F1 => 36,
        KeyCode::F2 => 37,
        KeyCode::F3 => 38,
        KeyCode::F4 => 39,
        KeyCode::F5 => 40,
        KeyCode::F6 => 41,
        KeyCode::F7 => 42,
        KeyCode::F8 => 43,
        KeyCode::F9 => 44,
        KeyCode::F10 => 45,
        KeyCode::F11 => 46,
        KeyCode::F12 => 47,
        KeyCode::Escape => 48,
        KeyCode::Space => 49,
        KeyCode::Enter => 50,
        KeyCode::Tab => 51,
        KeyCode::Backspace => 52,
        KeyCode::Delete => 53,
        KeyCode::ArrowUp => 54,
        KeyCode::ArrowDown => 55,
        KeyCode::ArrowLeft => 56,
        KeyCode::ArrowRight => 57,
        KeyCode::NumpadAdd => 58,
        KeyCode::NumpadSubtract => 59,
        KeyCode::Home => 60,
        KeyCode::End => 61,
        KeyCode::PageUp => 62,
        KeyCode::PageDown => 63,
        _ => 999,
    }
}

fn u16_to_keycode(disc: u16) -> KeyCode {
    match disc {
        0 => KeyCode::KeyA,
        1 => KeyCode::KeyB,
        2 => KeyCode::KeyC,
        3 => KeyCode::KeyD,
        4 => KeyCode::KeyE,
        5 => KeyCode::KeyF,
        6 => KeyCode::KeyG,
        7 => KeyCode::KeyH,
        8 => KeyCode::KeyI,
        9 => KeyCode::KeyJ,
        10 => KeyCode::KeyK,
        11 => KeyCode::KeyL,
        12 => KeyCode::KeyM,
        13 => KeyCode::KeyN,
        14 => KeyCode::KeyO,
        15 => KeyCode::KeyP,
        16 => KeyCode::KeyQ,
        17 => KeyCode::KeyR,
        18 => KeyCode::KeyS,
        19 => KeyCode::KeyT,
        20 => KeyCode::KeyU,
        21 => KeyCode::KeyV,
        22 => KeyCode::KeyW,
        23 => KeyCode::KeyX,
        24 => KeyCode::KeyY,
        25 => KeyCode::KeyZ,
        26 => KeyCode::Digit0,
        27 => KeyCode::Digit1,
        28 => KeyCode::Digit2,
        29 => KeyCode::Digit3,
        30 => KeyCode::Digit4,
        31 => KeyCode::Digit5,
        32 => KeyCode::Digit6,
        33 => KeyCode::Digit7,
        34 => KeyCode::Digit8,
        35 => KeyCode::Digit9,
        36 => KeyCode::F1,
        37 => KeyCode::F2,
        38 => KeyCode::F3,
        39 => KeyCode::F4,
        40 => KeyCode::F5,
        41 => KeyCode::F6,
        42 => KeyCode::F7,
        43 => KeyCode::F8,
        44 => KeyCode::F9,
        45 => KeyCode::F10,
        46 => KeyCode::F11,
        47 => KeyCode::F12,
        48 => KeyCode::Escape,
        49 => KeyCode::Space,
        50 => KeyCode::Enter,
        51 => KeyCode::Tab,
        52 => KeyCode::Backspace,
        53 => KeyCode::Delete,
        54 => KeyCode::ArrowUp,
        55 => KeyCode::ArrowDown,
        56 => KeyCode::ArrowLeft,
        57 => KeyCode::ArrowRight,
        58 => KeyCode::NumpadAdd,
        59 => KeyCode::NumpadSubtract,
        60 => KeyCode::Home,
        61 => KeyCode::End,
        62 => KeyCode::PageUp,
        63 => KeyCode::PageDown,
        _ => KeyCode::Escape,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keybindings_no_unexpected_conflicts() {
        let kb = KeyBindings::default();
        let conflicts = kb.find_conflicts();
        assert!(
            conflicts.is_empty(),
            "unexpected conflicts: {:?}",
            conflicts
        );
    }

    #[test]
    fn test_keybinding_display_label() {
        assert_eq!(KeyBinding::simple(KeyCode::KeyA).display_label(), "A");
        assert_eq!(KeyBinding::ctrl(KeyCode::KeyS).display_label(), "Ctrl+S");
        assert_eq!(
            KeyBinding {
                key: KeyCode::Tab,
                ctrl: false,
                shift: true
            }
            .display_label(),
            "Shift+Tab"
        );
    }

    #[test]
    fn test_set_and_get_binding() {
        let mut kb = KeyBindings::default();
        let new = KeyBinding::simple(KeyCode::KeyX);
        kb.set(BindableAction::TogglePause, new);
        assert_eq!(kb.get(BindableAction::TogglePause), new);
    }

    #[test]
    fn test_conflict_detection() {
        let mut kb = KeyBindings::default();
        let same = KeyBinding::simple(KeyCode::KeyX);
        kb.set(BindableAction::ToolRoad, same);
        kb.set(BindableAction::ToolBulldoze, same);
        let conflicts = kb.find_conflicts();
        assert!(conflicts
            .iter()
            .any(
                |(a, b)| (*a == BindableAction::ToolRoad && *b == BindableAction::ToolBulldoze)
                    || (*a == BindableAction::ToolBulldoze && *b == BindableAction::ToolRoad)
            ));
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut kb = KeyBindings::default();
        kb.set(
            BindableAction::TogglePause,
            KeyBinding::simple(KeyCode::KeyX),
        );
        let bytes = kb.save_to_bytes().expect("should save");
        let loaded = KeyBindings::load_from_bytes(&bytes);
        assert_eq!(
            loaded.get(BindableAction::TogglePause),
            KeyBinding::simple(KeyCode::KeyX)
        );
    }

    #[test]
    fn test_saveable_skip_default() {
        assert!(KeyBindings::default().save_to_bytes().is_none());
    }

    #[test]
    fn test_keycode_roundtrip() {
        for code in [
            KeyCode::KeyA,
            KeyCode::KeyZ,
            KeyCode::Digit0,
            KeyCode::F12,
            KeyCode::Escape,
            KeyCode::Space,
            KeyCode::Tab,
            KeyCode::Delete,
            KeyCode::ArrowUp,
            KeyCode::NumpadAdd,
        ] {
            assert_eq!(u16_to_keycode(keycode_to_u16(code)), code);
        }
    }
}
