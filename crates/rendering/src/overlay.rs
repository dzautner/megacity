use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverlayMode {
    #[default]
    None,
    Power,
    Water,
    Traffic,
    Pollution,
    LandValue,
    Education,
    Garbage,
    Noise,
    WaterPollution,
}

#[derive(Resource, Default)]
pub struct OverlayState {
    pub mode: OverlayMode,
}

pub fn toggle_overlay_keys(keys: Res<ButtonInput<KeyCode>>, mut overlay: ResMut<OverlayState>) {
    if keys.just_pressed(KeyCode::KeyP) {
        overlay.mode = if overlay.mode == OverlayMode::Power {
            OverlayMode::None
        } else {
            OverlayMode::Power
        };
    }
    if keys.just_pressed(KeyCode::KeyO) {
        overlay.mode = if overlay.mode == OverlayMode::Water {
            OverlayMode::None
        } else {
            OverlayMode::Water
        };
    }
    if keys.just_pressed(KeyCode::KeyT) {
        overlay.mode = if overlay.mode == OverlayMode::Traffic {
            OverlayMode::None
        } else {
            OverlayMode::Traffic
        };
    }
    if keys.just_pressed(KeyCode::KeyN) {
        overlay.mode = if overlay.mode == OverlayMode::Pollution {
            OverlayMode::None
        } else {
            OverlayMode::Pollution
        };
    }
    if keys.just_pressed(KeyCode::KeyL) {
        overlay.mode = if overlay.mode == OverlayMode::LandValue {
            OverlayMode::None
        } else {
            OverlayMode::LandValue
        };
    }
    if keys.just_pressed(KeyCode::KeyE) {
        overlay.mode = if overlay.mode == OverlayMode::Education {
            OverlayMode::None
        } else {
            OverlayMode::Education
        };
    }
    if keys.just_pressed(KeyCode::KeyG) {
        overlay.mode = if overlay.mode == OverlayMode::Garbage {
            OverlayMode::None
        } else {
            OverlayMode::Garbage
        };
    }
    if keys.just_pressed(KeyCode::KeyM) {
        overlay.mode = if overlay.mode == OverlayMode::Noise {
            OverlayMode::None
        } else {
            OverlayMode::Noise
        };
    }
    if keys.just_pressed(KeyCode::KeyU) {
        overlay.mode = if overlay.mode == OverlayMode::WaterPollution {
            OverlayMode::None
        } else {
            OverlayMode::WaterPollution
        };
    }
}
