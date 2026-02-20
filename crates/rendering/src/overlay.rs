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
    GroundwaterLevel,
    GroundwaterQuality,
    Wind,
}

/// Ordered list of all overlay modes for Tab/Shift+Tab cycling.
const ALL_OVERLAYS: [OverlayMode; 13] = [
    OverlayMode::None,
    OverlayMode::Power,
    OverlayMode::Water,
    OverlayMode::Traffic,
    OverlayMode::Pollution,
    OverlayMode::LandValue,
    OverlayMode::Education,
    OverlayMode::Garbage,
    OverlayMode::Noise,
    OverlayMode::WaterPollution,
    OverlayMode::GroundwaterLevel,
    OverlayMode::GroundwaterQuality,
    OverlayMode::Wind,
];

impl OverlayMode {
    /// Returns the next overlay mode in the cycle (wraps around).
    pub fn next(self) -> Self {
        let idx = ALL_OVERLAYS.iter().position(|&m| m == self).unwrap_or(0);
        ALL_OVERLAYS[(idx + 1) % ALL_OVERLAYS.len()]
    }

    /// Returns the previous overlay mode in the cycle (wraps around).
    pub fn prev(self) -> Self {
        let idx = ALL_OVERLAYS.iter().position(|&m| m == self).unwrap_or(0);
        ALL_OVERLAYS[(idx + ALL_OVERLAYS.len() - 1) % ALL_OVERLAYS.len()]
    }

    /// Human-readable label for display in status bar.
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Power => "Power",
            Self::Water => "Water",
            Self::Traffic => "Traffic",
            Self::Pollution => "Pollution",
            Self::LandValue => "Land Value",
            Self::Education => "Education",
            Self::Garbage => "Garbage",
            Self::Noise => "Noise",
            Self::WaterPollution => "Water Pollution",
            Self::GroundwaterLevel => "Groundwater Level",
            Self::GroundwaterQuality => "Groundwater Quality",
            Self::Wind => "Wind",
        }
    }
}

#[derive(Resource, Default)]
pub struct OverlayState {
    pub mode: OverlayMode,
}

/// Cycle overlays with Tab (forward) / Shift+Tab (backward).
/// Individual letter-key shortcuts have been removed to resolve keybinding
/// conflicts (see issue #905).  All overlays are reachable via Tab cycling.
pub fn toggle_overlay_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay: ResMut<OverlayState>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if keys.just_pressed(bindings.overlay_cycle_next.key) {
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        overlay.mode = if shift {
            overlay.mode.prev()
        } else {
            overlay.mode.next()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_cycles_forward_through_all_overlays() {
        let mut mode = OverlayMode::None;
        let expected = [
            OverlayMode::Power,
            OverlayMode::Water,
            OverlayMode::Traffic,
            OverlayMode::Pollution,
            OverlayMode::LandValue,
            OverlayMode::Education,
            OverlayMode::Garbage,
            OverlayMode::Noise,
            OverlayMode::WaterPollution,
            OverlayMode::GroundwaterLevel,
            OverlayMode::GroundwaterQuality,
            OverlayMode::Wind,
            OverlayMode::None, // wraps back
        ];
        for &exp in &expected {
            mode = mode.next();
            assert_eq!(mode, exp);
        }
    }

    #[test]
    fn prev_cycles_backward_through_all_overlays() {
        let mut mode = OverlayMode::None;
        let expected = [
            OverlayMode::Wind,
            OverlayMode::GroundwaterQuality,
            OverlayMode::GroundwaterLevel,
            OverlayMode::WaterPollution,
            OverlayMode::Noise,
            OverlayMode::Garbage,
            OverlayMode::Education,
            OverlayMode::LandValue,
            OverlayMode::Pollution,
            OverlayMode::Traffic,
            OverlayMode::Water,
            OverlayMode::Power,
            OverlayMode::None, // wraps back
        ];
        for &exp in &expected {
            mode = mode.prev();
            assert_eq!(mode, exp);
        }
    }

    #[test]
    fn next_then_prev_returns_to_original() {
        for &start in &ALL_OVERLAYS {
            assert_eq!(start.next().prev(), start);
            assert_eq!(start.prev().next(), start);
        }
    }

    #[test]
    fn label_returns_non_empty_for_all_variants() {
        for &mode in &ALL_OVERLAYS {
            assert!(!mode.label().is_empty());
        }
    }
}
