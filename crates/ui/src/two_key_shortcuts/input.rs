//! Input handling system for two-key shortcuts.
//!
//! Manages the state machine that listens for category key presses followed
//! by digit-key sub-tool selection, with timeout and escape cancellation.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use rendering::input::ActiveTool;
use rendering::overlay::{OverlayMode, OverlayState};

use crate::toolbar::OpenCategory;

use super::categories::build_shortcut_categories;

// ---------------------------------------------------------------------------
// State resource
// ---------------------------------------------------------------------------

/// Tracks the two-key shortcut state machine.
///
/// When the user presses a category key (e.g. R), `pending_category` is set to
/// the index into `build_shortcut_categories()`.  A timer starts counting down
/// from `TIMEOUT_SECS`.  If a digit key is pressed before the timer expires,
/// the corresponding sub-tool is activated.  If the timer expires or Escape is
/// pressed the pending state is cleared.
#[derive(Resource, Default)]
pub struct TwoKeyShortcutState {
    /// Index into the categories vec, or `None` if no category is pending.
    pub pending_category: Option<usize>,
    /// Remaining seconds before the popup auto-closes.
    pub timer: f32,
}

pub(crate) const TIMEOUT_SECS: f32 = 2.0;

// ---------------------------------------------------------------------------
// Input system
// ---------------------------------------------------------------------------

/// Maps digit key-codes to a 0-based sub-tool index (1->0, 2->1, ..., 9->8, 0->9).
fn digit_key_to_index(key: KeyCode) -> Option<usize> {
    match key {
        KeyCode::Digit1 => Some(0),
        KeyCode::Digit2 => Some(1),
        KeyCode::Digit3 => Some(2),
        KeyCode::Digit4 => Some(3),
        KeyCode::Digit5 => Some(4),
        KeyCode::Digit6 => Some(5),
        KeyCode::Digit7 => Some(6),
        KeyCode::Digit8 => Some(7),
        KeyCode::Digit9 => Some(8),
        KeyCode::Digit0 => Some(9),
        _ => None,
    }
}

/// Handles category-key and digit-key presses, plus timeout and Escape.
#[allow(clippy::too_many_arguments)]
pub(crate) fn two_key_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut state: ResMut<TwoKeyShortcutState>,
    mut tool: ResMut<ActiveTool>,
    mut overlay: ResMut<OverlayState>,
    mut open_cat: ResMut<OpenCategory>,
    mut contexts: EguiContexts,
) {
    // Skip when egui wants keyboard (text fields, etc.)
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    let categories = build_shortcut_categories();

    // --- If a category is already pending, handle sub-key / timeout / escape ---
    if let Some(cat_idx) = state.pending_category {
        // Escape cancels
        if keyboard.just_pressed(KeyCode::Escape) {
            state.pending_category = None;
            state.timer = 0.0;
            return;
        }

        // Check digit keys
        let digit_keys = [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
            KeyCode::Digit0,
        ];

        for &dk in &digit_keys {
            if keyboard.just_pressed(dk) {
                if let Some(sub_idx) = digit_key_to_index(dk) {
                    if cat_idx < categories.len() {
                        let cat = &categories[cat_idx];
                        if sub_idx < cat.items.len() {
                            let item = &cat.items[sub_idx];
                            if let Some(t) = item.tool {
                                *tool = t;
                            } else if let Some(ov) = item.overlay {
                                overlay.mode = if overlay.mode == ov {
                                    OverlayMode::None
                                } else {
                                    ov
                                };
                            }
                        }
                    }
                }
                // Close popup after selection (or invalid digit)
                state.pending_category = None;
                state.timer = 0.0;
                return;
            }
        }

        // Pressing a different category key switches category
        for (idx, cat) in categories.iter().enumerate() {
            if keyboard.just_pressed(cat.key) && idx != cat_idx {
                state.pending_category = Some(idx);
                state.timer = TIMEOUT_SECS;
                // Also open the toolbar category popup for consistency
                open_cat.0 = Some(idx);
                return;
            }
        }

        // Timer countdown
        state.timer -= time.delta_secs();
        if state.timer <= 0.0 {
            state.pending_category = None;
            state.timer = 0.0;
        }

        return;
    }

    // --- No pending category: check for category key presses ---
    // Skip if modifier keys are held — those are separate bindings (e.g. Shift+G for curve draw).
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if shift_held || ctrl_held {
        return;
    }
    for (idx, cat) in categories.iter().enumerate() {
        if keyboard.just_pressed(cat.key) {
            state.pending_category = Some(idx);
            state.timer = TIMEOUT_SECS;
            // Also open the matching toolbar category popup for consistency
            open_cat.0 = Some(idx);
            return;
        }
    }
}
