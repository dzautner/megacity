//! Helper functions for key-code labels and serialization mapping.

use bevy::prelude::*;

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

pub(crate) fn keycode_to_u16(key: KeyCode) -> u16 {
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

pub(crate) fn u16_to_keycode(disc: u16) -> KeyCode {
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
    use super::super::{BindableAction, KeyBinding, KeyBindings};
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
        use crate::Saveable;
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
        use crate::Saveable;
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
