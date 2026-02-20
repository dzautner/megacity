//! Colorblind accessibility modes.
//!
//! Provides a `ColorblindMode` resource that indicates which color vision
//! deficiency adaptation is active. Rendering and UI systems read this resource
//! to select appropriate color palettes.

use bevy::prelude::*;

use crate::{decode_or_warn, Saveable};

/// The active colorblind accessibility mode.
///
/// Affects overlay color ramps, zone ground colors, traffic LOS indicators,
/// and status icon colors throughout the game.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
    bitcode::Encode,
    bitcode::Decode,
)]
pub enum ColorblindMode {
    /// Normal color vision (no adaptation).
    #[default]
    Normal,
    /// Red-blind (protanopia): reds appear dark/absent.
    /// Uses blue-orange-yellow palette, avoids red-green.
    Protanopia,
    /// Green-blind (deuteranopia): greens appear brownish.
    /// Uses blue-orange-yellow palette, avoids red-green.
    Deuteranopia,
    /// Blue-blind (tritanopia): blues appear greenish, yellows pinkish.
    /// Uses red-cyan palette, avoids blue-yellow.
    Tritanopia,
}

impl ColorblindMode {
    /// Human-readable label for display in settings UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Protanopia => "Protanopia (Red-blind)",
            Self::Deuteranopia => "Deuteranopia (Green-blind)",
            Self::Tritanopia => "Tritanopia (Blue-blind)",
        }
    }

    /// All available modes for UI iteration.
    pub const ALL: [ColorblindMode; 4] = [
        Self::Normal,
        Self::Protanopia,
        Self::Deuteranopia,
        Self::Tritanopia,
    ];
}

/// Resource holding the active colorblind mode.
#[derive(Resource, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColorblindSettings {
    pub mode: ColorblindMode,
}

impl Saveable for ColorblindSettings {
    const SAVE_KEY: &'static str = "colorblind_settings";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.mode == ColorblindMode::Normal {
            None // skip saving default state
        } else {
            Some(bitcode::encode(&self.mode))
        }
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let mode: ColorblindMode = decode_or_warn(Self::SAVE_KEY, bytes);
        Self { mode }
    }
}

pub struct ColorblindPlugin;

impl Plugin for ColorblindPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColorblindSettings>();
        // Register for save/load via extension map
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<ColorblindSettings>();
    }
}
